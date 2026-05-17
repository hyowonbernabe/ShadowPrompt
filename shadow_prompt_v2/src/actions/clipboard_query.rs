// Stateless clipboard query: read clipboard → LLM → write back + overlay.

use super::ActionContext;

pub async fn execute(_ctx: ActionContext) -> anyhow::Result<()> {
    // TODO: clipboard read, build stateless request, send, write back, update UI.
    Ok(())
}
