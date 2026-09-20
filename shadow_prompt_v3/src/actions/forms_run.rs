// Forms hotkey actions — design doc §7.3. Both modes always open a new background tab now
// (design decision changed mid-conversation from an earlier separate-hotkey variant).

use super::ActionContext;
use crate::browser::forms::{execute_form_flow, FormsMode};
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

    // Real gap found in live testing: `SetFormIndicator`/`FormIndicatorState` (top-right pixel,
    // stacked under the main indicator) were fully defined and handled in the UI layer but never
    // actually sent from anywhere — this scaffold's own header comment used to say so directly.
    // Wired here: on while the flow is actively running; `Done` (green) on success, briefly, then
    // ui/manager.rs auto-hides it; `Failed` (red) on error, same as `forms_run_legacy.rs`.
    let _ = ctx.ui_tx.send(UICommand::SetFormIndicator(FormIndicatorState::Running));
    let result = execute_form_flow(ctx.llm.clone(), forms_mode, ctx.config.forms.max_pages).await;
    let end_state = if result.is_ok() { FormIndicatorState::Done } else { FormIndicatorState::Failed };
    let _ = ctx.ui_tx.send(UICommand::SetFormIndicator(end_state));
    result
}
