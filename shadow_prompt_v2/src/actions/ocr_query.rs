// Vision query: capture fullscreen → vision LLM → clipboard + overlay.
// (Region selection is deferred; full screen with vision is sufficient for v2 MVP.)

use super::ActionContext;
use crate::capture::{clipboard, image, screen};
use crate::llm::messages::{ContentPart, Message};
use crate::llm::system_prompts::ANSWER_MODE_GENERAL;
use crate::ui::commands::{IndicatorState, UICommand};

pub async fn execute(ctx: ActionContext) -> anyhow::Result<()> {
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Processing));

    let png = tokio::task::spawn_blocking(screen::capture_fullscreen).await??;
    let part = tokio::task::spawn_blocking(move || image::prepare_for_request(&png)).await??;

    let messages = vec![
        Message::System { content: ANSWER_MODE_GENERAL.to_string() },
        Message::User { content: vec![
            ContentPart::Text { text: "Answer the question(s) visible in this screenshot.".to_string() },
            part,
        ]},
    ];

    let answer = match ctx.llm.call(messages).await {
        Ok(a) => a,
        Err(e) => {
            let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Error));
            return Err(e);
        }
    };

    let answer_for_write = answer.clone();
    tokio::task::spawn_blocking(move || clipboard::write_text(&answer_for_write)).await??;
    let preview: String = answer.chars().take(400).collect();
    let _ = ctx.ui_tx.send(UICommand::SetOverlayText(preview));
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Ready));
    Ok(())
}
