// Reset conversation — forgets all context and earlier questions (e.g. switching from a test
// about whales to one about sharks). Single press on purpose: it's a frequent, user-driven
// action, and undoing a mistaken reset only costs re-capturing the reference material.

use super::ActionContext;
use crate::ui::commands::UICommand;

pub fn execute(ctx: &ActionContext) {
    ctx.conversation.lock().unwrap().clear();
    ctx.llm.new_session();
    let _ = ctx.ui_tx.send(UICommand::FlashNotice("Conversation reset".to_string()));
}
