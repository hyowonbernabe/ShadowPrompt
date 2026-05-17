// Vision query: capture screen region → vision LLM → clipboard + overlay.

use super::ActionContext;
use crate::capture::{clipboard, image, screen};
use crate::llm::messages::{ContentPart, Message};
use crate::llm::system_prompts::ANSWER_MODE_GENERAL;
use crate::ui::commands::{IndicatorState, UICommand};

pub async fn execute(ctx: ActionContext, x: i32, y: i32, w: i32, h: i32) -> anyhow::Result<()> {
    execute_inner(ctx, x, y, w, h, false).await
}

pub async fn execute_online(ctx: ActionContext, x: i32, y: i32, w: i32, h: i32) -> anyhow::Result<()> {
    execute_inner(ctx, x, y, w, h, true).await
}

async fn execute_inner(ctx: ActionContext, x: i32, y: i32, w: i32, h: i32, online: bool) -> anyhow::Result<()> {
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Processing));

    let png = tokio::task::spawn_blocking(move || screen::capture_region(x, y, w, h)).await??;
    let part = tokio::task::spawn_blocking(move || image::prepare_for_request(&png)).await??;

    let messages = vec![
        ctx.llm.system_message(ANSWER_MODE_GENERAL),
        Message::User { content: vec![
            ContentPart::text("Answer the question(s) visible in this screenshot."),
            part,
        ]},
    ];

    let call = if online { ctx.llm.call_online(messages).await } else { ctx.llm.call(messages).await };
    let answer = match call {
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
