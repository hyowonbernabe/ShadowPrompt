// Multi-step Forms automation. Owns ephemeral conversation on the stack.
// See docs/architecture.md "Multi-Step Reasoning Flows" in agents.md.

use super::ActionContext;
use crate::browser::forms::{execute_form_flow, FormsMode};
use crate::ui::commands::{FormIndicatorState, UICommand};

pub async fn execute(ctx: ActionContext, auto_paginate: bool) -> anyhow::Result<()> {
    let mode = if auto_paginate { FormsMode::AutoPaginate } else { FormsMode::SinglePage };
    let _ = ctx.ui_tx.send(UICommand::SetFormIndicator(FormIndicatorState::Running));

    let result = execute_form_flow(ctx.llm.clone(), mode).await;

    let final_state = match &result {
        Ok(()) => FormIndicatorState::Hidden,
        Err(e) => {
            log::error!("forms flow: {e}");
            FormIndicatorState::Failed
        }
    };
    let _ = ctx.ui_tx.send(UICommand::SetFormIndicator(final_state));
    result
}
