// v3 legacy Forms hotkeys — design doc §7.4, the default/primary Forms engine as of 2026-09-20.
// Same FormIndicator wiring as the axtree engine (`forms_run.rs`), different flow underneath:
// no new background tab, operates directly on whatever Forms tab is already open and focused.

use super::ActionContext;
use crate::browser::forms::legacy::{execute_legacy_form_flow, FormsMode};
use crate::ui::commands::{FormIndicatorState, UICommand};

pub enum Mode {
    AnswerPage,
    AnswerAll,
}

pub async fn execute(ctx: ActionContext, mode: Mode) -> anyhow::Result<()> {
    let forms_mode = match mode {
        Mode::AnswerPage => FormsMode::AnswerPage,
        Mode::AnswerAll => FormsMode::AnswerAll,
    };

    // Design doc §7.4: real reasoning/search/docs capability, no fill/browser tool at all.
    let tools = super::clipboard_query::memory_tools(&ctx);

    let _ = ctx.ui_tx.send(UICommand::SetFormIndicator(FormIndicatorState::Running));
    let result = execute_legacy_form_flow(ctx.llm.clone(), tools, forms_mode).await;
    let _ = ctx.ui_tx.send(UICommand::SetFormIndicator(FormIndicatorState::Hidden));
    result
}
