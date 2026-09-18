// Tab open/close safety — design doc §7.3, added directly in response to two scenarios the
// user raised:
//   1. Never close the tab if it's the only one left in the window (would kill the whole
//      browser/CDP session, not just a tab).
//   2. Never close until the save is confirmed, not just assumed — default to leaving it open
//      if nothing confirms the save, rather than risk losing answers.
//
// M8: implemented for real. Guard 1 (`enough_tabs_open`) is a plain, fully verifiable count
// check. Guard 2 (`save_confirmed`/`text_signals_saved`) is a best-effort implementation of an
// inherently-unverifiable-here requirement — see `safe_to_close`'s doc comment for the honest
// limitation.

use chromiumoxide::{Browser, Page};

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

/// Returns true if it's safe to close `page` — false if it's the only tab left in the browser,
/// or if the save-confirmation signal can't be found.
///
/// ## Guard 1 — never close the only tab left
///
/// `Browser::pages()` (same call `mod.rs`'s `find_active_forms_url` already uses to enumerate
/// every open target) is checked before closing; if this is the only open tab, closing it would
/// take the whole debug-attached Chrome window (and CDP session) down with it, not just this
/// one tab — refused unconditionally, regardless of the save signal below.
///
/// ## Guard 2 — save-confirmation signal — honest limitation, unverified against a live form
///
/// Google Forms was directly observed, during this project's design-phase research (design doc
/// §7.2), to show a live "Draft saved" indicator after interacting with fields — but the exact
/// DOM/accessibility signal to poll for was never pinned down there, only confirmed to exist
/// visually. It cannot be pinned down here either: there is no live browser available in this
/// environment (the same constraint M6/M7's own honest-limitation notes already flagged for
/// their AX-role/name assumptions). This is a best-effort implementation of an
/// inherently-unverifiable-here requirement, not a confirmed one.
///
/// What it actually does: fetches a fresh accessibility-tree snapshot of `page` (reusing
/// `fill::fetch_ax_snapshot`/`fill::name_of` — the exact same AX-tree bridge `fill_page` and
/// `advance_to_next_page` already use, rather than a third independent one), and looks for any
/// non-ignored node whose accessible name contains "saved" (case-insensitive, deliberately
/// fuzzy — same spirit as `fill.rs`'s fuzzy label matching, for the same reason: Google's actual
/// wording could just as easily be "Saved", "All changes saved", or something not observed
/// during the design-phase research at all). If nothing matches, this returns `Ok(false)` —
/// exactly as designed: default to "leave it open" whenever the signal can't be confirmed, never
/// assume success just because nothing errored.
pub async fn safe_to_close(browser: &Browser, page: &Page) -> anyhow::Result<bool> {
    let pages = browser
        .pages()
        .await
        .map_err(|e| anyhow::anyhow!("listing open tabs for safe_to_close: {e}"))?;

    if !enough_tabs_open(pages.len()) {
        log::info!("forms: safe_to_close — this is the only tab left in the browser, refusing to close");
        return Ok(false);
    }

    match save_confirmed(page).await {
        Ok(true) => Ok(true),
        Ok(false) => {
            log::info!("forms: safe_to_close — no save-confirmation signal found on the page, leaving tab open");
            Ok(false)
        }
        Err(e) => {
            log::warn!("forms: safe_to_close — error reading page state for save signal ({e}), leaving tab open");
            Ok(false)
        }
    }
}

/// Pure "would closing this tab leave zero open" check — isolated so guard 1 has a real unit
/// test without needing a live `Browser` (design doc §7.3's first tab-safety guard).
fn enough_tabs_open(open_tab_count: usize) -> bool {
    open_tab_count > 1
}

async fn save_confirmed(page: &Page) -> anyhow::Result<bool> {
    let nodes = super::fill::fetch_ax_snapshot(page).await?;
    Ok(nodes.iter().filter(|n| !n.ignored).filter_map(super::fill::name_of).any(|name| text_signals_saved(&name)))
}

/// Pure fuzzy match against a node's accessible name — isolated so guard 2's matching logic has
/// a real unit test independent of the (unverifiable-here) question of whether this is actually
/// the right text to look for at all. See `safe_to_close`'s doc comment.
fn text_signals_saved(name: &str) -> bool {
    name.to_ascii_lowercase().contains("saved")
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

    #[test]
    fn text_signals_saved_matches_plausible_variants_case_insensitively() {
        assert!(text_signals_saved("Draft saved"));
        assert!(text_signals_saved("All changes saved"));
        assert!(text_signals_saved("SAVED"));
    }

    #[test]
    fn text_signals_saved_false_for_unrelated_text() {
        assert!(!text_signals_saved("Submit"));
        assert!(!text_signals_saved("Question 3 of 5"));
        assert!(!text_signals_saved(""));
    }
}
