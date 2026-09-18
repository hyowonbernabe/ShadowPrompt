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

use chromiumoxide::cdp::browser_protocol::target::CreateTargetParams;
use chromiumoxide::{Browser, Page};

/// Same reasoning as `mod.rs`'s `PAGE_DISCOVERY_RETRIES` — chromiumoxide's internal target list
/// is only populated once it's processed the async `Target.targetCreated` event, which can lag
/// behind a command response by a moment. Short bounded retry, not a long fixed wait.
const GET_PAGE_RETRIES: u32 = 10;
const GET_PAGE_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(100);

pub async fn open_forms_tab(browser: &Browser, form_url: &str) -> anyhow::Result<Page> {
    // Three prior attempts at this, all confirmed by live testing (via the debug_open_tab
    // hotkey, exactly so this could be iterated on quickly without a full Forms/LLM run each
    // time):
    //   1. Bare URL (`new_window` left unset) — no error, but opens a brand new top-level
    //      window every time. Chrome's own doc says this field defaults to `false` (a tab); in
    //      practice, attached via `--remote-debugging-port` rather than CDP-launched, it
    //      behaves like `true`.
    //   2. Explicit `new_window(false)` — fails outright with CDP error -32000 "Failed to open
    //      new tab - no browser is open."
    //   3. `for_tab(true)` — this one actually opens a real tab in the existing window (user
    //      directly confirmed this fixes the original report), but then chromiumoxide's own
    //      handler panics internally: `Browser::new_page` waits on `self.targets.get_mut
    //      (&target_id)` being populated by the (separate, async) `Target.targetCreated` event
    //      before it'll hand back a `Page`, and hard-`panic!("Created target not present")`
    //      instead of retrying if that event hasn't arrived yet by the time the `createTarget`
    //      command's own response comes back — a genuine upstream race, not something wrong on
    //      this app's side (the crate's own source has a `// TODO can this even happen?` right
    //      next to that panic).
    // Fix: keep `for_tab(true)` (it's what actually gets the right window/tab behavior), but
    // stop going through `Browser::new_page` (`HandlerMessage::CreatePage`, the code path that
    // panics) — issue the raw `Target.createTarget` command via `Browser::execute`
    // (`HandlerMessage::Command`, a plain passthrough with none of `new_page`'s special-cased
    // post-processing), then separately poll `Browser::get_page` for the resulting target,
    // tolerating exactly the same "event hasn't landed yet" race by retrying instead of
    // asserting it can't happen.
    let params = CreateTargetParams::builder()
        .url(form_url)
        .for_tab(true)
        .build()
        .map_err(|e| anyhow::anyhow!("building new-tab params: {e}"))?;
    let created = browser
        .execute(params)
        .await
        .map_err(|e| anyhow::anyhow!("opening new Forms tab: {e}"))?;
    let target_id = created.result.target_id;

    for attempt in 0..GET_PAGE_RETRIES {
        match browser.get_page(target_id.clone()).await {
            Ok(page) => return Ok(page),
            Err(_) if attempt + 1 < GET_PAGE_RETRIES => {
                tokio::time::sleep(GET_PAGE_RETRY_DELAY).await;
            }
            Err(e) => return Err(anyhow::anyhow!("new Forms tab was created but never became attachable: {e}")),
        }
    }
    unreachable!("loop above always returns")
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
