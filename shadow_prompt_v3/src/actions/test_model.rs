// test_model — design doc §3. "What model are you?" health check. Overlay only, no clipboard
// write, no tools. Shows both the model's own (possibly wrong) self-report and the ground-truth
// model id from the API response.

use super::ActionContext;
use crate::ui::commands::{IndicatorState, UICommand};

pub async fn execute(ctx: ActionContext) -> anyhow::Result<()> {
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Processing));

    // No tool-calling loop needed here — a literal "what model are you" query never needs
    // tools — so `LlmClient::simple_call` (build plan M3) is used instead of `run_turn`, which
    // only returns the final answer text and discards the ground-truth `model` field.
    let call = ctx
        .llm
        .simple_call("Reply with exactly one sentence naming what model you are.", "What model are you?")
        .await;

    let result = match call {
        Ok(r) => r,
        Err(e) => {
            let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Error));
            return Err(e);
        }
    };

    let text = format!("Self-report: {}\nGround truth: {}", result.answer, result.model);

    let _ = ctx.ui_tx.send(UICommand::SetOverlayText(text));
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Ready));
    Ok(())
}
