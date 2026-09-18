// launch_debugger — opens incognito Chrome/Edge with the debug port on. Ported as-is from v2.

use super::ActionContext;
use crate::browser::debugger;

pub fn execute(_ctx: &ActionContext) -> anyhow::Result<()> {
    debugger::launch_incognito()
}
