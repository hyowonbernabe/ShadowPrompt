// Google Forms orchestration — design doc §7.3. Both hotkeys always open a new background tab
// (not the user's own current tab): find the Forms URL from whichever tab is currently active
// in the debug-attached browser, then open a fresh tab on that same URL to do the actual work,
// invisibly, while the user's own tab is left completely alone.
//
// Because only page 1 has a distinct URL (`.../viewform`) and every page after that shares one
// identical URL (`.../formResponse` — confirmed empirically, design doc §7.2), there is no way to
// link directly to "page 3." Both modes walk forward from page 1, skip any page that's already
// fully answered (autosave carries prior progress into the new tab), and stop the walk at the
// first page with something still blank — that's the page you're actually stuck on, found
// without tracking an explicit page number.

pub mod fill;
pub mod read;
pub mod screenshot_tool;
pub mod tab_lifecycle;

use std::sync::Arc;

use chromiumoxide::cdp::browser_protocol::accessibility::AxNode;
use fill::FillPageTool;
use read::{all_fields_answered, read_page, PageContent};

use crate::browser::debugger::DEBUG_PORT;
use crate::llm::messages::{ContentPart, Message};
use crate::llm::system_prompts::forms_prompt;
use crate::llm::LlmClient;

pub enum FormsMode {
    AnswerPage,
    AnswerAll,
}

pub async fn execute_form_flow(llm: Arc<LlmClient>, mode: FormsMode, max_pages: u32) -> anyhow::Result<()> {
    let browser = crate::browser::debugger::connect().await?;
    let (form_url, source_page) = find_active_forms_url(&browser).await?;
    let page = tab_lifecycle::open_forms_tab(&browser, &source_page, &form_url).await?;

    let mut history: Vec<Message> = vec![Message::system_text(forms_prompt())];

    match mode {
        FormsMode::AnswerPage => {
            // Walk forward, skipping any page that's already fully answered (autosave carries
            // prior progress into this fresh tab — design doc §7.2/§7.3), until either a page
            // with something blank is found, or the walk runs into Submit-only (nothing left to
            // do) or the page-count safety cap. Once a blank page is found, fill exactly that one
            // page and stop — deliberately no further `advance_to_next_page` call afterward,
            // unlike `AnswerAll` below.
            let mut target: Option<PageContent> = None;
            for page_idx in 0..max_pages {
                let content: PageContent = read_page(&page).await?;
                log::info!("forms: page {} — {} field(s)", page_idx + 1, content.fields.len());

                if all_fields_answered(&content) {
                    log::info!("forms: page {} already fully answered, skipping ahead", page_idx + 1);
                    match advance_to_next_page(&page).await? {
                        NextPageOutcome::Advanced => continue,
                        NextPageOutcome::SubmitOnly => {
                            log::info!(
                                "forms: reached Submit-only page with nothing left blank; nothing to fill"
                            );
                            break;
                        }
                    }
                }

                target = Some(content);
                break;
            }

            if let Some(content) = target {
                let fill_tool: Arc<dyn crate::llm::Tool> = Arc::new(FillPageTool { page: page.clone() });
                // TODO(design doc §7.3): also attach a `screenshot(region?)` fallback tool here
                // for anything the accessibility read can't resolve cleanly (a picture question,
                // an unusual widget) — M8.
                let tools = vec![fill_tool];

                let user_content = build_page_message(&content);
                history.push(Message::User { content: user_content.clone() });

                let answer = llm
                    .run_turn(&forms_prompt(), user_content, &tools, !content_images(&content).is_empty(), None)
                    .await?;
                history.push(Message::assistant_text(answer));
            }
        }
        FormsMode::AnswerAll => {
            // Cross-page memory (design doc §7.3): `run_turn` (llm/mod.rs) always starts a fresh
            // [system, user] pair per call — it has no multi-turn history parameter to hand
            // prior pages' messages into, and that function's own signature is under active
            // concurrent change elsewhere in this build (M11's streaming work just landed a new
            // `on_delta` parameter on it) — piling a second, unrelated change onto that same
            // shared loop right now is exactly the kind of footprint the build plan's "one
            // milestone at a time" rule warns against. Instead, prior pages' context and this
            // run's own answers for them are carried forward as extra text prepended to each
            // subsequent page's own user content: the model still sees everything it already
            // did earlier in this run before answering the next page, which is the actual
            // consistency goal — just via one accumulating text block instead of literal
            // role-tagged turns.
            let mut prior_pages_context = String::new();

            for page_idx in 0..max_pages {
                let content: PageContent = read_page(&page).await?;
                log::info!("forms: page {} — {} field(s)", page_idx + 1, content.fields.len());

                if !all_fields_answered(&content) {
                    let fill_tool: Arc<dyn crate::llm::Tool> = Arc::new(FillPageTool { page: page.clone() });
                    // M8: `screenshot(region?)` fallback tool, for anything the accessibility
                    // read can't resolve cleanly (a picture question, an unusual widget) —
                    // design doc §7.3. Feeds back as this tool's own text result, never the
                    // shared overlay; see `screenshot_tool.rs`'s module doc for the honest
                    // limitation on how far "feeds back" actually goes given this codebase's
                    // text-only tool-result wire format.
                    let screenshot_tool: Arc<dyn crate::llm::Tool> = Arc::new(screenshot_tool::ScreenshotTool);
                    let tools = vec![fill_tool, screenshot_tool];

                    let mut user_content = build_page_message(&content);
                    if !prior_pages_context.is_empty() {
                        user_content.insert(0, ContentPart::text(prior_pages_context.clone()));
                    }
                    history.push(Message::User { content: user_content.clone() });

                    // The screenshot fallback tool is always attached this turn, so keep the
                    // model chain vision-capable throughout even when this page's own fields
                    // carry no image of their own (design doc §7.3/§8's vision-capability
                    // filter) — not just `!content_images(&content).is_empty()`, which only
                    // reflects the page's own extracted images.
                    let answer = llm.run_turn(&forms_prompt(), user_content, &tools, true, None).await?;
                    history.push(Message::assistant_text(answer.clone()));

                    prior_pages_context.push_str(&format!(
                        "--- Page {} (already handled earlier in this run) ---\n{}\nYour answer(s) for that page:\n{}\n\n",
                        page_idx + 1,
                        content.page_context,
                        answer,
                    ));
                } else {
                    log::info!("forms: page {} already fully answered, skipping", page_idx + 1);
                }

                let outcome = advance_to_next_page(&page).await?;
                if let Some(reason) = should_stop_walk(page_idx, max_pages, &outcome) {
                    log::info!("forms: {reason}");
                    break;
                }
            }
        }
    }

    if tab_lifecycle::safe_to_close(&browser, &page).await.unwrap_or(false) {
        if let Err(e) = page.close().await {
            log::warn!("forms: failed to close Forms tab after run: {e}");
        }
    }

    Ok(())
}

fn content_images(content: &PageContent) -> Vec<String> {
    content.fields.iter().flat_map(|f| f.image_urls.clone()).collect()
}

fn build_page_message(content: &PageContent) -> Vec<ContentPart> {
    let fields_json = serde_json::to_string_pretty(&content.fields).unwrap_or_default();
    log::debug!("forms: page content sent to model — context: {:?}, fields: {}", content.page_context, fields_json);
    let mut parts = vec![ContentPart::text(format!("{}\n\n{}", content.page_context, fields_json))];
    for url in content_images(content) {
        parts.push(ContentPart::Image { image_url: crate::llm::messages::ImageUrl { url } });
    }
    parts
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NextPageOutcome {
    Advanced,
    SubmitOnly,
}

/// Pure walk-forward stop decision for `AnswerAll`'s per-page loop (design doc §7.3's ~20-page
/// safety cap, `forms.max_pages`) — isolated from `execute_form_flow`'s async body so the bound
/// itself has a real unit test without needing a live `chromiumoxide::Page`. Everything else in
/// that loop (`read_page`, `advance_to_next_page`) needs a real CDP connection to exercise, the
/// same honest constraint M6/M7 already flagged for their own pure-logic seams — this function
/// is the one piece of the loop's control flow that doesn't.
///
/// Returns `Some(reason)` (for logging) the moment the walk should stop: either the hard
/// non-negotiable (a Submit-only page — never advanced past, never clicked) or the bug-safety
/// page-count cap. Returns `None` to keep going.
fn should_stop_walk(page_idx: u32, max_pages: u32, outcome: &NextPageOutcome) -> Option<&'static str> {
    match outcome {
        NextPageOutcome::SubmitOnly => Some("reached Submit-only page; stopping, never clicking it"),
        NextPageOutcome::Advanced if page_idx + 1 >= max_pages => {
            Some("reached the max_pages safety cap; stopping")
        }
        NextPageOutcome::Advanced => None,
    }
}

/// Finds a "Next"-labeled clickable control via a fresh accessibility-tree snapshot and clicks
/// it if present; if only a "Submit"-labeled control remains (no Next), returns `SubmitOnly`
/// without ever touching it.
///
/// This is the one hard, structural non-negotiable of the whole Forms feature (design doc §7.3):
/// there is no submit tool or submit-clicking code anywhere in this codebase, and this function
/// cannot become one — it only ever *detects* a Submit-shaped control to know when to stop. The
/// `next.is_some()` branch below is the only branch that clicks anything, and it only clicks a
/// control whose accessible name matched "next", never "submit". If neither is found, this
/// returns an error rather than silently guessing — an unrecognized page state must not be
/// treated as either "keep going" or "safe to stop."
///
/// Reuses `fill::fetch_ax_snapshot`/`fill::click_node`/`fill::is_role`/`fill::name_of` (the same
/// AX-node -> DOM.resolveNode -> Runtime.callFunctionOn bridge `fill_page` uses for every other
/// element interaction — see `fill.rs`'s module doc for why that bridge was chosen) rather than
/// re-deriving an independent one here.
///
/// Genuine open uncertainty, not verified against a live browser (no Chrome available in this
/// environment — the milestone's own stated constraint, same as `read.rs`/`fill.rs`): whether
/// Google Forms' actual Next/Submit controls are really exposed with AX role `"button"`, and
/// whether their accessible names are literally "Next"/"Submit" (vs. e.g. an icon-only control
/// whose name comes from an `aria-label`). The substring match below is case-insensitive and
/// deliberately loose for exactly that reason.
async fn advance_to_next_page(page: &chromiumoxide::Page) -> anyhow::Result<NextPageOutcome> {
    let nodes = fill::fetch_ax_snapshot(page).await?;

    let mut next: Option<&AxNode> = None;
    let mut submit_seen = false;
    for node in &nodes {
        if node.ignored || !fill::is_role(node, &["button"]) {
            continue;
        }
        let Some(name) = fill::name_of(node) else { continue };
        let lower = name.to_ascii_lowercase();
        if lower.contains("next") {
            next = Some(node);
        } else if lower.contains("submit") {
            submit_seen = true;
        }
    }

    if let Some(next_node) = next {
        fill::click_node(page, next_node).await?;
        return Ok(NextPageOutcome::Advanced);
    }

    if submit_seen {
        return Ok(NextPageOutcome::SubmitOnly);
    }

    anyhow::bail!("advance_to_next_page: found neither a Next nor a Submit control on this page")
}

/// `Browser::pages()` right after a fresh `Browser::connect()` (which `execute_form_flow` does
/// on every single Forms hotkey press, not once at daemon startup) can race the handler's
/// `Target.setDiscoverTargets` handshake — CDP has to round-trip that call and then deliver a
/// `targetCreated` event per existing tab before `pages()` knows about tabs that were already
/// open before this connection existed, which is the normal case here (the user navigates in the
/// debug window, *then* fires the hotkey). A handful of short retries absorbs that instead of
/// failing on a target list that just hadn't populated yet.
const PAGE_DISCOVERY_RETRIES: u32 = 5;
const PAGE_DISCOVERY_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(100);

/// `Browser::pages()` right after a fresh `Browser::connect()` (which every caller here does on
/// every single hotkey press, not once at daemon startup) can race the handler's
/// `Target.setDiscoverTargets` handshake — CDP has to round-trip that call and then deliver a
/// `targetCreated` event per existing tab before `pages()` knows about tabs that were already
/// open before this connection existed, which is the normal case here (the user navigates in the
/// debug window, *then* fires the hotkey). A handful of short retries absorbs that instead of
/// failing on a target list that just hadn't populated yet.
///
/// Real gap found by thinking through multi-tab use (user report: Brave + a second Chrome + this
/// debug Chrome all open, plus multiple form tabs inside the debug window itself): the old version
/// took "the first open tab whose URL matches Google Forms' known shapes," which ignored which tab
/// the user was actually looking at entirely. If a stale Forms tab from an earlier test was still
/// open in the background, a hotkey fired while looking at an unrelated tab (e.g. YouTube) would
/// silently answer the *wrong, stale* form instead of failing. Other browsers (Brave, a second
/// Chrome) were never in scope either way — `browser.pages()` only ever sees targets belonging to
/// this one CDP-attached debug Chrome process.
///
/// `document.hasFocus()` is true for exactly the one document that is both the frontmost tab of
/// its window *and* in a window that currently has OS input focus — unlike `TargetInfo`, which
/// carries no such flag, this reliably answers "which tab is the user actually on" and naturally
/// handles a second debug-Chrome *window* too (at most one tab across all of them should ever
/// report true).
pub(crate) async fn find_focused_page(browser: &chromiumoxide::Browser) -> anyhow::Result<chromiumoxide::Page> {
    let mut pages = Vec::new();
    for attempt in 0..PAGE_DISCOVERY_RETRIES {
        pages = browser
            .pages()
            .await
            .map_err(|e| anyhow::anyhow!("listing open tabs on debug Chrome: {e}"))?;
        if !pages.is_empty() || attempt + 1 == PAGE_DISCOVERY_RETRIES {
            break;
        }
        tokio::time::sleep(PAGE_DISCOVERY_RETRY_DELAY).await;
    }

    for page in pages {
        let has_focus = page
            .evaluate("document.hasFocus()")
            .await
            .ok()
            .and_then(|r| r.into_value::<bool>().ok())
            .unwrap_or(false);
        if has_focus {
            return Ok(page);
        }
    }

    anyhow::bail!("couldn't find a focused tab in debug Chrome on :{DEBUG_PORT} — click into the tab with your form first")
}

async fn find_active_forms_url(
    browser: &chromiumoxide::Browser,
) -> anyhow::Result<(String, chromiumoxide::Page)> {
    let page = find_focused_page(browser).await?;
    let url = page
        .url()
        .await
        .ok()
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("focused tab in debug Chrome on :{DEBUG_PORT} has no URL"))?;

    if is_forms_url(&url) {
        Ok((url, page))
    } else {
        anyhow::bail!(
            "the focused tab in debug Chrome on :{DEBUG_PORT} isn't a Google Form ({url}) — click into the form's tab first"
        )
    }
}

fn is_forms_url(url: &str) -> bool {
    url.contains("docs.google.com/forms") || url.contains("forms.gle")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_stop_walk_stops_at_submit_only_well_before_the_cap() {
        assert!(should_stop_walk(2, 20, &NextPageOutcome::SubmitOnly).is_some());
    }

    #[test]
    fn should_stop_walk_keeps_going_when_advanced_and_under_the_cap() {
        assert!(should_stop_walk(2, 20, &NextPageOutcome::Advanced).is_none());
    }

    #[test]
    fn should_stop_walk_respects_max_pages_even_when_never_submit_only() {
        // max_pages = 5 (0-indexed page_idx 0..=4): having just visited the 5th page
        // (page_idx == 4), the walk must stop even on an `Advanced` outcome — this is the actual
        // bug-insurance cap (design doc §7.3), proven here without driving 5 real pages through
        // a live browser.
        assert!(should_stop_walk(4, 5, &NextPageOutcome::Advanced).is_some());
        assert!(should_stop_walk(3, 5, &NextPageOutcome::Advanced).is_none());
    }

    #[test]
    fn should_stop_walk_submit_only_wins_even_at_the_last_possible_page() {
        assert!(should_stop_walk(4, 5, &NextPageOutcome::SubmitOnly).is_some());
    }
}
