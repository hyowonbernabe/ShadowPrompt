// switch_model — design doc §8/§10. Toggles which paid model leads the fallback chain; the
// other stays in the chain as the next fallback, neither is ever removed.

use super::ActionContext;
use crate::ui::commands::UICommand;

pub fn execute(ctx: &ActionContext) {
    let now_leading = ctx.llm.toggle_primary_model();
    let _ = ctx.ui_tx.send(UICommand::FlashNotice(format!("Switched to: {now_leading}")));
}
