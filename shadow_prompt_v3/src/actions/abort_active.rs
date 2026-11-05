// Abort: stops whatever's running in ALL task slots, unconditionally — design doc §7.3.
// Simplest predictable behavior: "get me out of whatever's happening right now."

use super::{abort_slot, clear_dots, idle_indicator, ActionContext};
use crate::ui::commands::UICommand;

pub async fn execute(ctx: &ActionContext) {
    abort_slot(&ctx.answer_task).await;
    abort_slot(&ctx.forms_task).await;
    // Screen Answer's tracking loop dies with its slot; drop its dots too.
    clear_dots(ctx).await;
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(idle_indicator(ctx)));
}
