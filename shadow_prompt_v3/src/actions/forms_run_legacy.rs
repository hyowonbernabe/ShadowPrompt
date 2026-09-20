// v3 legacy Forms hotkeys — design doc §7.4, the default/primary Forms engine as of 2026-09-20.
// Same FormIndicator wiring as the axtree engine (`forms_run.rs`), different flow underneath:
// no new background tab, operates directly on whatever Forms tab is already open and focused.
// A same-browser background tab was tried and reverted (2026-09-20): opening it forced a
// user-visible tab switch, and Chrome throttles rendering on any tab that isn't the active one,
// which made "Next" clicks and dropdown clicks alike hang for tens of seconds on later pages.

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
    // `Failed` was defined and rendered (ui/manager.rs) but never actually sent — a failed run
    // used to clear to `Hidden` exactly like a successful one, giving no visible way to tell them
    // apart. Same convention as `clipboard_query.rs`'s `IndicatorState::Error`. `Done` (green,
    // ui/manager.rs auto-hides it after a few seconds) gives success its own visible end state too
    // instead of jumping straight back to invisible.
    let end_state = if result.is_ok() { FormIndicatorState::Done } else { FormIndicatorState::Failed };
    let _ = ctx.ui_tx.send(UICommand::SetFormIndicator(end_state));
    result
}
