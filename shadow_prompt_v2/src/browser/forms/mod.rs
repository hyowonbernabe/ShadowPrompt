// Google Forms multi-step flow.

pub mod answered;
pub mod extractor;
pub mod injector;
pub mod submit_guard;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use headless_chrome::Browser;

use crate::browser::debugger::DEBUG_PORT;
use crate::llm::messages::{ContentPart, ImageUrl, Message};
use crate::llm::system_prompts::ANSWER_MODE_FORMS;
use crate::llm::LlmClient;
use extractor::{Question, EXTRACTOR_JS};
use injector::build_injector_js;
use submit_guard::{next_button_selector, submit_button_selector};

pub enum FormsMode {
    AutoPaginate,
    SinglePage,
}

const MAX_PAGES: u32 = 10;

pub async fn execute_form_flow(llm: Arc<LlmClient>, mode: FormsMode) -> anyhow::Result<()> {
    let ws_url = fetch_debugger_ws().await?;
    let browser = Browser::connect(ws_url)?;
    let tabs = browser.get_tabs().lock().unwrap().clone();
    let tab = tabs
        .iter()
        .find(|t| {
            let u = t.get_url();
            u.contains("docs.google.com/forms") || u.contains("forms.gle") || u.contains("/forms/d/")
        })
        .or_else(|| tabs.iter().find(|t| t.get_url().contains("docs.google.com")))
        .ok_or_else(|| {
            let urls: Vec<String> = tabs.iter().map(|t| t.get_url()).collect();
            anyhow::anyhow!(
                "no Google Forms tab open in debug Chrome. Navigate to a form in the debug window first. Open tabs: {:?}",
                urls
            )
        })?
        .clone();

    let mut history: Vec<Message> = vec![llm.system_message(ANSWER_MODE_FORMS)];

    let max_pages = match mode {
        FormsMode::AutoPaginate => MAX_PAGES,
        FormsMode::SinglePage => 1,
    };

    for page_idx in 0..max_pages {
        tokio::time::sleep(Duration::from_millis(400)).await;

        let extracted: serde_json::Value = tab
            .evaluate(EXTRACTOR_JS, false)?
            .value
            .ok_or_else(|| anyhow::anyhow!("extractor returned no value"))?;

        let questions_json = extracted.as_str().unwrap_or("[]");
        let questions: Vec<Question> = serde_json::from_str(questions_json)
            .map_err(|e| anyhow::anyhow!("parse questions: {e}; body={questions_json}"))?;

        let unanswered: Vec<&Question> = questions
            .iter()
            .filter(|q| q.current_value.as_deref().unwrap_or("").trim().is_empty())
            .collect();

        if !unanswered.is_empty() {
            // In AutoPaginate we strip image parts from prior turns to keep
            // request size bounded across many image-heavy pages. SinglePage
            // already starts a fresh history per invocation, so no pruning
            // is needed there.
            if matches!(mode, FormsMode::AutoPaginate) {
                prune_images_from_history(&mut history);
            }

            let user_msg = build_user_message(&unanswered);
            history.push(user_msg);

            let answer = llm.call(history.clone()).await?;
            history.push(Message::Assistant { content: answer.clone() });

            let answers = parse_answers(&answer);
            let inj_js = build_injector_js(&answers);
            let _ = tab.evaluate(&inj_js, false)?;
            log::info!("page {}: answered {} questions", page_idx + 1, answers.len());
        } else {
            log::info!("page {}: all questions already answered", page_idx + 1);
        }

        if matches!(mode, FormsMode::SinglePage) {
            break;
        }

        let submit_present = tab.find_element(submit_button_selector()).is_ok();
        let next = tab.find_element(next_button_selector());
        match next {
            Ok(btn) => {
                btn.click()?;
            }
            Err(_) => {
                if submit_present {
                    log::info!("reached Submit page; halting (never auto-submit)");
                } else {
                    log::info!("no Next button; halting");
                }
                break;
            }
        }
    }

    Ok(())
}

fn build_user_message(unanswered: &[&Question]) -> Message {
    let mut parts: Vec<ContentPart> = Vec::new();
    let mut text = String::new();
    for q in unanswered {
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
        if !q.image_urls.is_empty() {
            text.push_str(&format!("  images: {} attached\n", q.image_urls.len()));
        }
    }
    parts.push(ContentPart::text(text));
    for q in unanswered {
        for url in &q.image_urls {
            parts.push(ContentPart::Image { image_url: ImageUrl { url: url.clone() } });
        }
    }
    Message::User { content: parts }
}

/// Replace image parts in earlier user messages with a text placeholder so
/// total request size stays bounded across many image-heavy pages.
fn prune_images_from_history(history: &mut [Message]) {
    for msg in history.iter_mut() {
        if let Message::User { content } = msg {
            let mut pruned = 0;
            content.retain(|p| {
                let keep = !matches!(p, ContentPart::Image { .. });
                if !keep {
                    pruned += 1;
                }
                keep
            });
            if pruned > 0 {
                content.push(ContentPart::text(format!(
                    "[{} earlier image(s) dropped from context]",
                    pruned
                )));
            }
        }
    }
}

fn parse_answers(raw: &str) -> HashMap<String, String> {
    let trimmed = raw.trim();
    let body = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed);
    let body = body.trim_end_matches("```").trim();
    serde_json::from_str(body).unwrap_or_default()
}

async fn fetch_debugger_ws() -> anyhow::Result<String> {
    #[derive(serde::Deserialize)]
    struct Version {
        #[serde(rename = "webSocketDebuggerUrl")]
        ws: String,
    }
    let url = format!("http://localhost:{}/json/version", DEBUG_PORT);
    let v: Version = reqwest::get(&url)
        .await
        .map_err(|e| anyhow::anyhow!("Chrome debugger not reachable on :{DEBUG_PORT}: {e}"))?
        .json()
        .await?;
    Ok(v.ws)
}
