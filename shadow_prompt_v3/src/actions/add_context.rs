// Add context — stores a screen region in the running conversation as reference material for
// later questions (e.g. a picture on page 1, questions about it on later pages). No model call:
// the entry rides along with the next question, so this is instant and can't time out.

use super::screenshot_query::capture_image_part;
use super::{idle_indicator, ActionContext};
use crate::llm::messages::ContentPart;
use crate::ui::commands::UICommand;

const LABEL: &str = "Reference material for the questions that follow. Read it and keep it in mind; \
don't answer anything in it — just wait for the next question.";

pub async fn execute(ctx: ActionContext, x: i32, y: i32, w: i32, h: i32) {
    let notice = match capture_image_part(&ctx, x, y, w, h).await {
        Ok(image_part) => {
            ctx.conversation.lock().unwrap().push_context(vec![ContentPart::text(LABEL), image_part]);
            "Context saved"
        }
        Err(e) => {
            log::error!("add_context: {e}");
            "Context failed"
        }
    };
    let _ = ctx.ui_tx.send(UICommand::FlashNotice(notice.to_string()));
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(idle_indicator(&ctx)));
}
