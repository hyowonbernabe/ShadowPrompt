// v3 legacy Forms engine — design doc §7.4. Built to replace v3 new (`super`'s AX-tree read +
// `fill_page` agentic loop) as the default, after extended live testing left that engine unable
// to reliably type or answer questions at all. Keeps `run_turn`'s real reasoning/search/
// `list_docs`/`read_doc` capability, but reads/fills the page with `extractor.rs`/`injector.rs`
// (v2's proven JS-extraction technique, patched) instead of chromiumoxide's AX-tree read, and
// answers with one structured JSON map instead of a `fill_page` tool call — the specific design
// choice that removes the entire class of bug that made v3 new unusable (§7.4's own writeup: no
// tool call driving the fill action means no schema for a model to call with empty/malformed
// arguments, and no tool-loop for it to get stuck retrying).
//
// No separate background tab, no tab-closing logic, no page-count cap — all deliberately dropped
// (design doc §7.4). Operates directly on whichever Forms tab is already open and focused,
// exactly like v2, reusing `super::find_focused_page`/`super::is_forms_url` since that part is
// generic Chrome-connection plumbing, not anything specific to opening a new tab.

pub mod extractor;
pub mod injector;

use std::collections::HashMap;
use std::sync::Arc;

use chromiumoxide::Page;

use crate::llm::messages::{ContentPart, ImageUrl};
use crate::llm::system_prompts::forms_legacy_prompt;
use crate::llm::{LlmClient, Tool};

use extractor::{ExtractedPage, Question, QuestionKind};

pub enum FormsMode {
    AnswerPage,
    AnswerAll,
}

pub async fn execute_legacy_form_flow(
    llm: Arc<LlmClient>,
    tools: Vec<Arc<dyn Tool>>,
    mode: FormsMode,
) -> anyhow::Result<()> {
    let browser = crate::browser::debugger::connect().await?;
    let page = super::find_focused_page(&browser).await?;
    let url = page.url().await.ok().flatten().unwrap_or_default();
    if !super::is_forms_url(&url) {
        anyhow::bail!("the focused tab isn't a Google Form ({url}) — click into the form's tab first");
    }

    // User-requested behavior: "answer all" covers the whole form start to finish no matter which
    // page the tab happens to be showing when the hotkey fires — not the narrower "walk forward
    // from wherever you already are" this engine originally shipped with (design doc §7.4, a
    // deliberate simplification at the time, now superseded by this request). Only page 1 has a
    // stable URL (`.../viewform`); every later page reuses one shared `.../formResponse` URL
    // (confirmed live), so navigating fresh to the `viewform` variant is what actually lands back
    // on page 1. Google's own autosave means anything already answered survives that navigation,
    // so the normal per-page `needs_attention` check below simply skips it rather than re-asking —
    // this doesn't erase progress, it just re-starts the walk from the top.
    // Known honest limitation: if the current page has a field edited but not yet committed by
    // Forms' own autosave, this navigation can trigger the browser's native "leave site?"
    // confirmation, which blocks until a human dismisses it — not something this codebase can
    // safely auto-accept without risking clicking through a dialog it didn't mean to.
    if matches!(mode, FormsMode::AnswerAll) {
        // `?hl=en` (Google's own "host language" URL parameter, confirmed live) pins the UI to
        // English deterministically. Root cause worth fixing here, not just working around:
        // Google Forms' own UI chrome (Next/Submit/Back text, required-question markers, etc.)
        // was observed switching to Filipino unpredictably between reloads with no action from
        // this app or the user — broke `advance_to_next_page`'s English-only text match outright.
        // Since this is a navigation this engine already forces itself, pinning the language it
        // lands on is free; `advance_to_next_page`'s multi-language word list stays as a fallback
        // for the (unforceable) case where the user's own already-open tab is in some other
        // language — that path can't have a URL parameter retroactively applied without a
        // disruptive reload of a tab this engine doesn't own.
        let base_url = url.replacen("/formResponse", "/viewform", 1);
        let start_url = if base_url.contains('?') { format!("{base_url}&hl=en") } else { format!("{base_url}?hl=en") };
        page.goto(start_url).await.map_err(|e| anyhow::anyhow!("navigating back to page 1: {e}"))?;
        tokio::time::sleep(std::time::Duration::from_millis(900)).await;
    }

    // Cross-page memory in `AnswerAll` (design doc §7.4): same flattened-text-prepend approach
    // §7.3/M8 already used — `run_turn` still has no multi-turn history parameter, an inherited
    // fidelity gap, not a new one introduced here.
    let mut prior_pages_context = String::new();

    loop {
        let extracted = extract_page_with_retry(&page).await?;
        log::info!("forms (legacy): page has {} question(s)", extracted.questions.len());

        let unanswered: Vec<&Question> = extracted.questions.iter().filter(|q| q.needs_attention()).collect();

        if !unanswered.is_empty() {
            let has_image = extracted.questions.iter().any(|q| !q.image_urls.is_empty());
            let mut user_content = build_user_message(&extracted.page_context, &extracted.questions);
            if !prior_pages_context.is_empty() {
                user_content.insert(0, ContentPart::text(prior_pages_context.clone()));
            }

            let answer = llm.run_turn(&forms_legacy_prompt(), user_content, &tools, has_image, None).await?;
            let answers = parse_answers(&answer);
            let inject_js = injector::build_injector_js(&answers);
            page.evaluate(inject_js).await.map_err(|e| anyhow::anyhow!("injecting Forms answers: {e}"))?;

            // Dropdowns can't be filled by the JS injector above (see its comment) — real,
            // OS-trusted clicks only. `id` is "q" + this question's position among
            // `[role="listitem"]` elements, the same indexing the extractor and JS injector both
            // already rely on.
            for q in extracted.questions.iter().filter(|q| q.kind == QuestionKind::Dropdown) {
                let Some(val) = answers.get(&q.id) else { continue };
                let Some(idx) = q.id.strip_prefix('q').and_then(|s| s.parse::<usize>().ok()) else {
                    continue;
                };
                if let Err(e) = injector::select_dropdown_option(&page, idx, val).await {
                    log::warn!("forms (legacy): {e}");
                }
            }

            log::info!("forms (legacy): answered {} question id(s)", answers.len());

            prior_pages_context.push_str(&format!(
                "--- Earlier page in this run ---\n{}\nYour answer(s) for it:\n{}\n\n",
                extracted.page_context, answer
            ));
        } else {
            log::info!("forms (legacy): page already fully answered, nothing to do");
        }

        if matches!(mode, FormsMode::AnswerPage) {
            break;
        }

        match advance_to_next_page(&page).await? {
            NextPageOutcome::Advanced => continue,
            NextPageOutcome::SubmitOnly => {
                log::info!("forms (legacy): reached Submit-only page; stopping, never clicking it");
                break;
            }
        }
    }

    Ok(())
}

async fn extract_page(page: &Page) -> anyhow::Result<ExtractedPage> {
    let raw: String = page
        .evaluate(extractor::EXTRACTOR_JS)
        .await
        .map_err(|e| anyhow::anyhow!("running Forms page extractor: {e}"))?
        .into_value()
        .map_err(|e| anyhow::anyhow!("extractor did not return a JSON string: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| anyhow::anyhow!("parsing extracted page JSON: {e}; body={raw}"))
}

const EXTRACT_RETRIES: u32 = 5;
const EXTRACT_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(300);

/// Confirmed live: clicking "Next" sometimes triggers a real browser navigation (page 1 → 2, a
/// genuine URL change) and sometimes just client-side routing with no navigation at all (later
/// pages all share one URL) — there's no way to tell which happened ahead of time, and calling
/// `page.wait_for_navigation()` unconditionally would hang forever on the second case. When it
/// is a real navigation, `extract_page` right after the click can race Chrome tearing down the
/// old JS execution context before the new one is ready ("Cannot find context with specified
/// id"), same class of CDP timing race `browser/forms/mod.rs`'s own `PAGE_DISCOVERY_RETRIES`
/// already works around for a different call. A short bounded retry handles both cases the same
/// way: the very first attempt just succeeds immediately when there was no real navigation.
async fn extract_page_with_retry(page: &Page) -> anyhow::Result<ExtractedPage> {
    let mut last_err = None;
    for attempt in 0..EXTRACT_RETRIES {
        match extract_page(page).await {
            Ok(extracted) => return Ok(extracted),
            Err(e) => {
                last_err = Some(e);
                if attempt + 1 < EXTRACT_RETRIES {
                    tokio::time::sleep(EXTRACT_RETRY_DELAY).await;
                }
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("extract_page_with_retry: exhausted retries with no error captured")))
}

/// Builds the model's turn content from *every* question on the page, answered or not — design
/// doc §7.4/§7.3's already-answered posture: the model sees everything as context (an answered
/// question can be a clue or setup for a later one), never just the blank ones.
fn build_user_message(page_context: &str, questions: &[Question]) -> Vec<ContentPart> {
    let mut text = String::new();
    if !page_context.trim().is_empty() {
        text.push_str(page_context.trim());
        text.push_str("\n\n");
    }
    for q in questions {
        text.push_str(&format!("[{}] ({:?}) {}\n", q.id, q.kind, q.text));
        if !q.options.is_empty() {
            text.push_str("  options: ");
            text.push_str(&q.options.join(" | "));
            text.push('\n');
        }
        if !q.rows.is_empty() {
            text.push_str("  rows: ");
            text.push_str(&q.rows.join(" | "));
            text.push('\n');
        }
        if let Some(cv) = q.current_value.as_deref().filter(|v| !v.trim().is_empty()) {
            text.push_str(&format!("  already answered: {cv}\n"));
        }
        if matches!(q.kind, QuestionKind::Grid | QuestionKind::CheckboxGrid) && !q.blank_rows.is_empty() {
            text.push_str(&format!("  still blank rows: {}\n", q.blank_rows.join(" | ")));
        }
        if !q.image_urls.is_empty() {
            text.push_str(&format!("  images: {} attached\n", q.image_urls.len()));
        }
    }
    let mut parts = vec![ContentPart::text(text)];
    for q in questions {
        for url in &q.image_urls {
            parts.push(ContentPart::Image { image_url: ImageUrl { url: url.clone() } });
        }
    }
    parts
}

/// Ported verbatim from shadow_prompt_v2's `parse_answers` — the balanced-brace text-repair
/// fallback design doc §2 already designates as the safety net under tool-call structured output,
/// used here as the *primary* mechanism since this delivery mode has no tool call at all.
fn parse_answers(raw: &str) -> HashMap<String, String> {
    let trimmed = raw.trim();
    let stripped = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim_end_matches("```")
        .trim();
    if let Some(map) = try_parse_answer_map(stripped) {
        return map;
    }

    if let Some(start) = stripped.find('{') {
        let bytes = stripped.as_bytes();
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escape = false;
        for (i, &b) in bytes.iter().enumerate().skip(start) {
            if escape {
                escape = false;
                continue;
            }
            match b {
                b'\\' if in_string => escape = true,
                b'"' => in_string = !in_string,
                b'{' if !in_string => depth += 1,
                b'}' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        let candidate = &stripped[start..=i];
                        if let Some(map) = try_parse_answer_map(candidate) {
                            return map;
                        }
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    log::warn!("forms (legacy): could not parse answer JSON; raw body was:\n{raw}");
    HashMap::new()
}

/// Confirmed live: the delivery prompt tells the model a Grid/CheckboxGrid answer is "a JSON
/// object mapping row label to chosen column" — and a model that follows that literally puts a
/// real nested object as the value, not a JSON-encoded string. Deserializing straight into
/// `HashMap<String, String>` rejects the *entire* top-level object the moment any one value isn't
/// a plain string, silently dropping every answer on the page, not just the Grid one. Values are
/// parsed as `serde_json::Value` first and non-string ones re-serialized back to a JSON string,
/// so the rest of the pipeline — and `injector.rs`'s own `JSON.parse(val)` for these two kinds —
/// keeps seeing the single string-valued shape it already expects.
fn try_parse_answer_map(candidate: &str) -> Option<HashMap<String, String>> {
    let raw_map: HashMap<String, serde_json::Value> = serde_json::from_str(candidate).ok()?;
    Some(
        raw_map
            .into_iter()
            .map(|(k, v)| {
                let s = match v {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };
                (k, s)
            })
            .collect(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NextPageOutcome {
    Advanced,
    SubmitOnly,
}

/// Finds a Next-labeled control and clicks it; if only a Submit-labeled control remains, returns
/// `SubmitOnly` without ever touching it — same hard non-negotiable as `super`'s
/// `advance_to_next_page` (design doc §7.3/§7.4), just via plain CSS-selector find/click
/// (`Page::find_element`) instead of the AX-tree bridge, ported from v2's `submit_guard.rs`
/// selectors. There is still no submit-clicking code anywhere in this codebase.
/// Confirmed live: Google Forms doesn't reliably put "Next"/"Submit" in the button's `aria-label`
/// at all — sometimes it's there, sometimes both buttons on the page have no `aria-label`
/// whatsoever, and the page's own UI language flips unpredictably between reloads (English and
/// Filipino both observed live in this same session). Checked against both the `aria-label` (when
/// present) and the button's own visible text, in both languages actually seen. This only ever
/// widens what counts as a *positive* match for "Next" or "Submit" — it never clicks by
/// elimination (e.g. "not recognized as Back, so it must be Next"), so an unrecognized future
/// locale fails safely (an error, not a wrong click) rather than risking Submit.
const NEXT_WORDS: &[&str] = &["next", "susunod"];
const SUBMIT_WORDS: &[&str] = &["submit", "isumite"];

async fn button_matches(btn: &chromiumoxide::Element, words: &[&str]) -> bool {
    let aria = btn.attribute("aria-label").await.ok().flatten().unwrap_or_default().to_lowercase();
    let text = btn.inner_text().await.ok().flatten().unwrap_or_default().to_lowercase();
    words.iter().any(|w| aria.contains(w) || text.contains(w))
}

async fn advance_to_next_page(page: &Page) -> anyhow::Result<NextPageOutcome> {
    let buttons = page
        .find_elements(r#"div[role="button"]"#)
        .await
        .map_err(|e| anyhow::anyhow!("listing page-bottom controls: {e}"))?;

    let mut next_btn = None;
    let mut submit_present = false;
    for btn in &buttons {
        if button_matches(btn, NEXT_WORDS).await {
            next_btn = Some(btn);
        } else if button_matches(btn, SUBMIT_WORDS).await {
            submit_present = true;
        }
    }

    if let Some(btn) = next_btn {
        btn.click().await.map_err(|e| anyhow::anyhow!("clicking Next: {e}"))?;
        tokio::time::sleep(std::time::Duration::from_millis(900)).await;
        return Ok(NextPageOutcome::Advanced);
    }
    if submit_present {
        return Ok(NextPageOutcome::SubmitOnly);
    }
    anyhow::bail!(
        "advance_to_next_page (legacy): found neither a Next nor a Submit control on this page \
         (checked aria-label and visible text, English/Filipino)"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use extractor::QuestionKind;

    fn question(id: &str, kind: QuestionKind, current_value: Option<&str>) -> Question {
        Question {
            id: id.to_string(),
            kind,
            text: "text".to_string(),
            options: vec![],
            rows: vec![],
            blank_rows: vec![],
            image_urls: vec![],
            current_value: current_value.map(str::to_string),
        }
    }

    #[test]
    fn parse_answers_reads_clean_json() {
        let map = parse_answers(r#"{"q0":"Blue","q1":"42"}"#);
        assert_eq!(map.get("q0"), Some(&"Blue".to_string()));
        assert_eq!(map.get("q1"), Some(&"42".to_string()));
    }

    #[test]
    fn parse_answers_accepts_nested_object_for_grid_answers() {
        // Real model output that broke this: q1 answered as a genuine nested object (following
        // the delivery prompt's own instructions), which used to make the ENTIRE reply fail to
        // parse — q0 and q3 were plain strings and got dropped too, not just q1/q2.
        let map = parse_answers(
            r#"{"q0":"Test fixture","q1":{"Algebra":"Very familiar","Geometry":"Very familiar"},"q3":"3"}"#,
        );
        assert_eq!(map.get("q0"), Some(&"Test fixture".to_string()));
        assert_eq!(map.get("q3"), Some(&"3".to_string()));
        let q1 = map.get("q1").expect("q1 must survive, not be dropped");
        let reparsed: HashMap<String, String> = serde_json::from_str(q1).unwrap();
        assert_eq!(reparsed.get("Algebra"), Some(&"Very familiar".to_string()));
    }

    #[test]
    fn parse_answers_extracts_json_wrapped_in_prose() {
        let map = parse_answers("Sure! Here are my answers: {\"q0\": \"Blue\"} Hope this helps!");
        assert_eq!(map.get("q0"), Some(&"Blue".to_string()));
    }

    #[test]
    fn parse_answers_strips_markdown_code_fence() {
        let map = parse_answers("```json\n{\"q0\":\"Blue\"}\n```");
        assert_eq!(map.get("q0"), Some(&"Blue".to_string()));
    }

    #[test]
    fn parse_answers_returns_empty_map_on_garbage() {
        assert!(parse_answers("I don't know the answer to that.").is_empty());
    }

    #[test]
    fn build_user_message_includes_already_answered_questions_as_context() {
        let questions = vec![
            question("q0", QuestionKind::Radio, Some("Blue")),
            question("q1", QuestionKind::ShortText, None),
        ];
        let parts = build_user_message("", &questions);
        let ContentPart::Text { text, .. } = &parts[0] else { panic!("expected text part") };
        assert!(text.contains("q0"), "already-answered q0 must still appear as context");
        assert!(text.contains("already answered: Blue"));
        assert!(text.contains("q1"));
    }

    #[test]
    fn build_user_message_surfaces_blank_grid_rows_not_the_filled_ones() {
        let mut q = question("q0", QuestionKind::Grid, Some(r#"{"Row 1":"Always"}"#));
        q.rows = vec!["Row 1".into(), "Row 2".into()];
        q.blank_rows = vec!["Row 2".into()];
        let parts = build_user_message("", &[q]);
        let ContentPart::Text { text, .. } = &parts[0] else { panic!("expected text part") };
        assert!(text.contains("still blank rows: Row 2"));
        assert!(!text.contains("still blank rows: Row 1"));
    }
}
