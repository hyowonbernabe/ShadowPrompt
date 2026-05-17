// Stateless clipboard query: read clipboard → LLM → write back + overlay.

use super::ActionContext;
use crate::capture::clipboard;
use crate::llm::system_prompts::ANSWER_MODE_GENERAL;
use crate::ui::commands::{IndicatorState, UICommand};

pub async fn execute(ctx: ActionContext) -> anyhow::Result<()> {
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Processing));

    let question = tokio::task::spawn_blocking(clipboard::read_text).await??;
    if question.trim().is_empty() {
        let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Error));
        anyhow::bail!("clipboard is empty");
    }

    log::info!("clipboard_query: {} chars", question.len());
    let answer = match ctx.llm.answer_text(ANSWER_MODE_GENERAL, &question).await {
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
