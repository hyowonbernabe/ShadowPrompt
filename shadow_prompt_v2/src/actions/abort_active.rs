// Abort the currently-running task (Forms run, query, etc.).

use super::ActionContext;

pub async fn execute(ctx: ActionContext) {
    let mut slot = ctx.active_task.lock().await;
    if let Some(handle) = slot.take() {
        handle.abort();
    }
}
