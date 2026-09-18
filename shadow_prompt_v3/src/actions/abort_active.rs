// Abort: stops whatever's running in BOTH task slots, unconditionally — design doc §7.3.
// Simplest predictable behavior: "get me out of whatever's happening right now."

use super::ActionContext;

pub async fn execute(ctx: &ActionContext) {
    for slot in [&ctx.answer_task, &ctx.forms_task] {
        let mut guard = slot.lock().await;
        if let Some(handle) = guard.take() {
            handle.abort();
        }
    }
}
