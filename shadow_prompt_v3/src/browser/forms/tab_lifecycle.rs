// Tab open/close safety — design doc §7.3, added directly in response to two scenarios the
// user raised:
//   1. Never close the tab if it's the only one left in the window (would kill the whole
//      browser/CDP session, not just a tab).
//   2. Never close until the save is confirmed, not just assumed — default to leaving it open
//      if nothing confirms the save, rather than risk losing answers.
//
// M8's original guard 2 watched the accessibility tree for a node whose name contained "saved" —
// confirmed live (2026-09-20) to actually miss a real "Draft saved" indicator outright, the exact
// kind of fragility a UI-text/translation-dependent check was always going to have (this project
// already hit auto-translate flakiness elsewhere breaking plain English matching). Replaced with
// `SaveTracker`, which watches Google Forms' own autosave network request directly — confirmed
// live via the Network domain: every field edit fires `POST .../draftresponse`, completing with
// HTTP 200 on success. This is the actual save mechanism, not a downstream UI side effect of it:
// no UI language dependency at all, and no fixed timer either — it's fully event-driven, so a
// slow connection just takes as long as it takes rather than getting guessed at.

use std::collections::HashSet;
use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::network::{
    EnableParams as NetworkEnableParams, EventLoadingFailed, EventLoadingFinished, EventRequestWillBeSent, RequestId,
};
use chromiumoxide::{Browser, Page};
use futures::StreamExt;

/// Same reasoning as `mod.rs`'s `PAGE_DISCOVERY_RETRIES` — chromiumoxide's internal target list
/// is only populated once it's processed the async `Target.targetCreated` event, which can lag
/// behind a command response by a moment. Short bounded retry, not a long fixed wait.
const NEW_TAB_RETRIES: u32 = 10;
const NEW_TAB_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(100);

pub async fn open_forms_tab(browser: &Browser, source_page: &Page, form_url: &str) -> anyhow::Result<Page> {
    // Four prior attempts at this, all confirmed by live testing (via the debug_open_tab hotkey,
    // exactly so this could be iterated on quickly without a full Forms/LLM run each time):
    //   1. Bare URL (`new_window` left unset) — no error, but opens a brand new top-level
    //      window every time. Chrome's own doc says this field defaults to `false` (a tab); in
    //      practice, attached via `--remote-debugging-port` rather than CDP-launched, it
    //      behaves like `true`.
    //   2. Explicit `new_window(false)` — fails outright with CDP error -32000 "Failed to open
    //      new tab - no browser is open."
    //   3. `for_tab(true)` — this one actually opens a real tab in the existing window, but then
    //      chromiumoxide's own handler panics internally: `Browser::new_page` waits on
    //      `self.targets.get_mut(&target_id)` being populated by the (separate, async)
    //      `Target.targetCreated` event before it'll hand back a `Page`, and hard-
    //      `panic!("Created target not present")` instead of retrying if that event hasn't
    //      arrived yet.
    //   4. `for_tab(true)` again, worked around the panic by issuing the raw `Target.createTarget`
    //      via `Browser::execute` and separately polling `Browser::get_page` — no more panic, but
    //      `get_page` never resolves, ever, no matter how long you retry: confirmed by reading
    //      `chromiumoxide::handler::target::Target::poll`, which unconditionally
    //      `return`s `None` (never issuing the `Target.attachToTarget` command that would set
    //      `session_id`, which is what `get_or_create_page` actually waits on) whenever
    //      `!self.is_page()`. `for_tab(true)` creates a target whose CDP `type` is the literal
    //      string `"tab"` (Chrome's newer tab-strip target kind), and `TargetType::new` only maps
    //      `"page"/"background_page"/"service_worker"/"shared_worker"/"other"/"browser"/"webview"`
    //      to known variants — `"tab"` falls through to `Unknown`, so `is_page()` is false
    //      forever. This crate structurally cannot attach to a `for_tab(true)` target; no amount
    //      of retrying fixes it.
    // Fix: stop using CDP `Target.createTarget` for this at all. Instead, run `window.open(url,
    // '_blank')` as JS inside the already-attached source page — a page-initiated `window.open`
    // creates an ordinary CDP target of type `"page"`, fully compatible with chromiumoxide's
    // normal attach/poll path, while still landing as a real tab in the same window (this is
    // exactly what `window.open` does in a real browser). The new tab has no CDP target id
    // available from JS, so it's found by diffing `Browser::pages()` before/after for a target id
    // that wasn't there before, with the same short bounded retry as everywhere else in this file
    // that waits on the `Target.targetCreated` race.
    let before: std::collections::HashSet<_> = browser
        .pages()
        .await
        .map_err(|e| anyhow::anyhow!("listing open tabs before opening new Forms tab: {e}"))?
        .iter()
        .map(|p| p.target_id().clone())
        .collect();

    // `window.open(...)`'s own return value is a `Window` reference — deeply self-referential
    // (`window.window`, `window.self`, `window.top`, ...) — and CDP's `Runtime.evaluate` tries to
    // build a serializable preview of whatever the expression evaluates to, which fails walking a
    // `Window` object's reference graph with "-32000: Object reference chain is too long." The
    // trailing `void 0` discards it, so the expression's completion value is plain `undefined`.
    let open_js = format!(
        "window.open({}, '_blank'); void 0",
        serde_json::to_string(form_url).map_err(|e| anyhow::anyhow!("encoding Forms URL for window.open: {e}"))?
    );
    source_page
        .evaluate(open_js)
        .await
        .map_err(|e| anyhow::anyhow!("opening new Forms tab via window.open: {e}"))?;

    for attempt in 0..NEW_TAB_RETRIES {
        let pages = browser
            .pages()
            .await
            .map_err(|e| anyhow::anyhow!("listing open tabs after opening new Forms tab: {e}"))?;
        if let Some(page) = pages.into_iter().find(|p| !before.contains(p.target_id())) {
            wait_for_document_ready(&page).await;

            // Confirmed live: `window.open(url, '_blank')` switches the active tab to the new one
            // by default in this Chrome version — the opposite of what "left completely alone"
            // requires. Page-level JS has no way to control which tab is active in the same
            // window (that's browser-chrome territory, not exposed to scripts), so this restores
            // it via CDP directly, which does have that authority. Best-effort: if this fails, the
            // new tab still works, it's just visibly in front until the user clicks back.
            use chromiumoxide::cdp::browser_protocol::target::ActivateTargetParams;
            if let Err(e) = browser.execute(ActivateTargetParams::new(source_page.target_id().clone())).await {
                log::warn!("forms: couldn't restore focus to the source tab after opening the background tab: {e}");
            }

            return Ok(page);
        }
        if attempt + 1 < NEW_TAB_RETRIES {
            tokio::time::sleep(NEW_TAB_RETRY_DELAY).await;
        }
    }
    anyhow::bail!("new Forms tab was opened via window.open but never appeared in the target list")
}

/// The new tab's CDP target can be discovered via `Browser::pages()` well before Chrome has
/// actually finished navigating to `form_url` — confirmed live: `read_page` on the freshly-
/// returned page saw "0 field(s)", got treated as "already answered" (vacuously true on an empty
/// list), and `advance_to_next_page` then failed outright because the real form content just
/// hadn't loaded yet. Best-effort poll of `document.readyState`, same short-bounded-retry shape as
/// the rest of this file; if it never reports ready, callers proceed anyway and any real failure
/// surfaces downstream instead of hanging here.
const PAGE_READY_RETRIES: u32 = 30;
const PAGE_READY_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(200);

async fn wait_for_document_ready(page: &Page) {
    for attempt in 0..PAGE_READY_RETRIES {
        let ready = page
            .evaluate("document.readyState === 'complete'")
            .await
            .ok()
            .and_then(|r| r.into_value::<bool>().ok())
            .unwrap_or(false);
        if ready {
            return;
        }
        if attempt + 1 < PAGE_READY_RETRIES {
            tokio::time::sleep(PAGE_READY_RETRY_DELAY).await;
        }
    }
}

/// Returns true if it's safe to close the tab this `saved` verdict came from — false if it's the
/// only tab left in the browser, regardless of `saved`.
///
/// ## Guard 1 — never close the only tab left
///
/// `Browser::pages()` (same call `mod.rs`'s `find_active_forms_url` already uses to enumerate
/// every open target) is checked before closing; if this is the only open tab, closing it would
/// take the whole debug-attached Chrome window (and CDP session) down with it, not just this
/// one tab — refused unconditionally, regardless of the save signal.
///
/// ## Guard 2 — save confirmation
///
/// Computed by the caller via `SaveTracker` (attached *before* any fill actions run, so it
/// doesn't miss early autosave requests) and passed in here as `saved`, rather than this
/// function re-deriving it from `page` — the tracker has to observe the whole run's network
/// activity, not just a one-shot snapshot at close time.
pub async fn safe_to_close(browser: &Browser, saved: bool) -> anyhow::Result<bool> {
    let pages = browser
        .pages()
        .await
        .map_err(|e| anyhow::anyhow!("listing open tabs for safe_to_close: {e}"))?;

    if !enough_tabs_open(pages.len()) {
        log::info!("forms: safe_to_close — this is the only tab left in the browser, refusing to close");
        return Ok(false);
    }

    if !saved {
        log::info!("forms: safe_to_close — autosave not confirmed complete, leaving tab open");
    }
    Ok(saved)
}

/// Pure "would closing this tab leave zero open" check — isolated so guard 1 has a real unit
/// test without needing a live `Browser` (design doc §7.3's first tab-safety guard).
fn enough_tabs_open(open_tab_count: usize) -> bool {
    open_tab_count > 1
}

/// How long to keep waiting, after the last outstanding autosave request settles, for a further
/// one to show up before concluding nothing more is coming. Not the save-completion signal itself
/// (that's fully event-driven, see `SaveTracker::all_saved`'s doc) — just a small grace window so
/// a save request that's about to be sent but hasn't hit the wire yet isn't missed.
const SETTLE_GRACE: Duration = Duration::from_millis(700);

/// Upper bound on the whole wait, purely defensive against Chrome's Network domain misbehaving
/// (e.g. never delivering a completion event at all) — not a substitute for the event-driven
/// wait above it. A real autosave taking this long would be a genuinely broken connection, not
/// this project's problem to work around further.
const OVERALL_TIMEOUT: Duration = Duration::from_secs(25);

/// Watches Google Forms' own autosave network requests directly, instead of any DOM/UI signal —
/// see this module's doc comment for why. Must be attached *before* any fill actions run: it only
/// sees requests fired after subscription, and the very first field edit on a page can save
/// almost immediately.
pub struct SaveTracker {
    requests: chromiumoxide::listeners::EventStream<EventRequestWillBeSent>,
    finished: chromiumoxide::listeners::EventStream<EventLoadingFinished>,
    failed: chromiumoxide::listeners::EventStream<EventLoadingFailed>,
}

impl SaveTracker {
    pub async fn attach(page: &Page) -> anyhow::Result<Self> {
        page.execute(NetworkEnableParams::default())
            .await
            .map_err(|e| anyhow::anyhow!("enabling Network domain for save tracking: {e}"))?;
        let requests = page
            .event_listener::<EventRequestWillBeSent>()
            .await
            .map_err(|e| anyhow::anyhow!("subscribing to outgoing requests: {e}"))?;
        let finished = page
            .event_listener::<EventLoadingFinished>()
            .await
            .map_err(|e| anyhow::anyhow!("subscribing to request completions: {e}"))?;
        let failed = page
            .event_listener::<EventLoadingFailed>()
            .await
            .map_err(|e| anyhow::anyhow!("subscribing to request failures: {e}"))?;
        Ok(Self { requests, finished, failed })
    }

    /// True if every `.../draftresponse` autosave request seen since `attach` completed
    /// successfully, with none still outstanding and none failed, and at least one was seen at
    /// all (an empty page with nothing to save shouldn't vacuously read as "confirmed saved").
    /// Fully event-driven: a slow connection just makes this take longer, not fail — `attach`'s
    /// events already sat buffered in an unbounded channel for however long the run itself took,
    /// so this only needs to actually wait out whatever is still genuinely in flight right now.
    pub async fn all_saved(mut self) -> bool {
        let mut pending: HashSet<RequestId> = HashSet::new();
        let mut completed: HashSet<RequestId> = HashSet::new();
        let mut saw_any = false;
        let mut any_failed = false;
        let deadline = tokio::time::Instant::now() + OVERALL_TIMEOUT;

        loop {
            let now = tokio::time::Instant::now();
            if now >= deadline {
                break;
            }
            let settled = saw_any && pending.iter().all(|id| completed.contains(id));
            let wait = if settled { SETTLE_GRACE } else { deadline - now };

            tokio::select! {
                Some(ev) = self.requests.next() => {
                    if ev.request.url.contains("/draftresponse") {
                        pending.insert(ev.request_id.clone());
                        saw_any = true;
                    }
                }
                Some(ev) = self.finished.next() => {
                    completed.insert(ev.request_id.clone());
                }
                Some(ev) = self.failed.next() => {
                    if pending.contains(&ev.request_id) {
                        any_failed = true;
                    }
                    completed.insert(ev.request_id.clone());
                }
                _ = tokio::time::sleep(wait) => {
                    if settled {
                        break;
                    }
                }
            }
        }

        saw_any && !any_failed && pending.iter().all(|id| completed.contains(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enough_tabs_open_false_when_only_one_or_zero() {
        assert!(!enough_tabs_open(1));
        assert!(!enough_tabs_open(0));
    }

    #[test]
    fn enough_tabs_open_true_when_more_than_one() {
        assert!(enough_tabs_open(2));
        assert!(enough_tabs_open(5));
    }
}
