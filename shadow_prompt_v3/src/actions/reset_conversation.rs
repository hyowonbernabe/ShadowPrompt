// Reset conversation — forgets all context and earlier questions (e.g. switching from a test
// about whales to one about sharks). Single press on purpose: it's a frequent, user-driven
// action, and undoing a mistaken reset only costs re-capturing the reference material.

use super::{abort_slot, clear_dots, idle_indicator, ActionContext};
use crate::ui::commands::UICommand;

pub async fn execute(ctx: &ActionContext) {
    // A question still in flight belongs to the old conversation — stop it, or its answer would
    // land as the first turn of the new one. Screen Answer dots belong to the old test too.
    abort_slot(&ctx.answer_task).await;
    clear_dots(ctx).await;
    ctx.conversation.lock().unwrap().clear();
    ctx.llm.new_session();
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(idle_indicator(ctx)));
    let _ = ctx.ui_tx.send(UICommand::FlashNotice("Conversation reset".to_string()));
}
