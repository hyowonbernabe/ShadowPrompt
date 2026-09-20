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

    // Cross-page memory in `AnswerAll` (design doc §7.4): same flattened-text-prepend approach
    // §7.3/M8 already used — `run_turn` still has no multi-turn history parameter, an inherited
    // fidelity gap, not a new one introduced here.
    let mut prior_pages_context = String::new();

    loop {
        let extracted = extract_page(&page).await?;
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
    if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(stripped) {
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
                        if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(candidate) {
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
async fn advance_to_next_page(page: &Page) -> anyhow::Result<NextPageOutcome> {
    let submit_present = page.find_element(r#"div[role="button"][aria-label*="Submit"]"#).await.is_ok();

    match page.find_element(r#"div[role="button"][aria-label*="Next"]"#).await {
        Ok(btn) => {
            btn.click().await.map_err(|e| anyhow::anyhow!("clicking Next: {e}"))?;
            tokio::time::sleep(std::time::Duration::from_millis(900)).await;
            Ok(NextPageOutcome::Advanced)
        }
        Err(_) if submit_present => Ok(NextPageOutcome::SubmitOnly),
        Err(_) => {
            anyhow::bail!("advance_to_next_page (legacy): found neither a Next nor a Submit control on this page")
        }
    }
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
