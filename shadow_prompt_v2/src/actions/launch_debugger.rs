// Launch incognito Chrome with --remote-debugging-port=9222. Detached.

use super::ActionContext;
use crate::browser::debugger;

pub async fn execute(_ctx: ActionContext) -> anyhow::Result<()> {
    debugger::launch_incognito()
}
