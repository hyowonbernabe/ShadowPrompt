// Reset conversation — forgets all context and earlier questions (e.g. switching from a test
// about whales to one about sharks). Single press on purpose: it's a frequent, user-driven
// action, and undoing a mistaken reset only costs re-capturing the reference material.

use super::ActionContext;
use crate::ui::commands::{IndicatorState, UICommand};

pub async fn execute(ctx: &ActionContext) {
    // A question still in flight belongs to the old conversation — stop it, or its answer would
    // land as the first turn of the new one.
    if let Some(handle) = ctx.answer_task.lock().await.take() {
        handle.abort();
    }
    ctx.conversation.lock().unwrap().clear();
    ctx.llm.new_session();
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Ready));
    let _ = ctx.ui_tx.send(UICommand::FlashNotice("Conversation reset".to_string()));
}
