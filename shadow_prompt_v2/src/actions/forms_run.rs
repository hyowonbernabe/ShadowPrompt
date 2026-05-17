// Multi-step Forms automation. Owns ephemeral conversation on the stack.
// See docs/architecture.md "Multi-Step Reasoning Flows" in agents.md.

use super::ActionContext;

pub enum Mode {
    AutoPaginate,
    SinglePage,
}

pub async fn execute(_ctx: ActionContext, _auto_paginate: bool) -> anyhow::Result<()> {
    // TODO: attach Chrome, loop per page (extract → filter answered → LLM → inject),
    // stop at Submit, never click Submit.
    Ok(())
}
