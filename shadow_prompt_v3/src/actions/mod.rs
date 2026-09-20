// Action handlers: one function per hotkey. Design doc §7.3: Forms runs on its own independent
// task-exclusivity slot, separate from Clipboard/Screenshot Query's shared slot — firing one
// family doesn't cancel the other. Clipboard and Screenshot Query still share one slot with each
// other (cancel-and-replace, same as v2).

pub mod abort_active;
pub mod clipboard_query;
pub mod debug_open_tab;
pub mod forms_run;
pub mod forms_run_legacy;
pub mod help_toggle;
pub mod hide_toggle;
pub mod launch_debugger;
pub mod screenshot_query;
pub mod switch_model;
pub mod test_model;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::config::Config;
use crate::input::InputEvent;
use crate::knowledge::KnowledgeStore;
use crate::llm::LlmClient;
use crate::ui::commands::{IndicatorState, UICommand};

pub type TaskSlot = Arc<Mutex<Option<JoinHandle<()>>>>;

/// Shared dependencies passed to every action.
#[derive(Clone)]
pub struct ActionContext {
    pub config: Arc<Config>,
    pub llm: Arc<LlmClient>,
    /// Sandboxed `knowledge/` folder access for `list_docs`/`read_doc` — design doc §4.
    /// Constructed once at daemon startup (`lib.rs::run_daemon`), shared by every action.
    pub knowledge: Arc<KnowledgeStore>,
    pub ui_tx: tokio::sync::mpsc::UnboundedSender<UICommand>,
    /// Clipboard Query + Screenshot Query share this slot (cancel-and-replace).
    pub answer_task: TaskSlot,
    /// Forms runs get their own independent slot — design doc §7.3.
    pub forms_task: TaskSlot,
}

pub async fn dispatch(ctx: ActionContext, event: InputEvent) {
    match event {
        InputEvent::ClipboardQuery => {
            spawn_exclusive(ctx.answer_task.clone(), ctx.clone(), |c| async move { clipboard_query::execute(c).await }).await;
        }
        InputEvent::ScreenshotQuery => {
            let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Processing));
        }
        // Real, confirmed gap fixed here: this is the only place a selection rectangle has ever
        // been shown, in any app — nothing sent UICommand::ShowDebugRect before this existed.
        InputEvent::ScreenshotDragUpdate { x, y, w, h } => {
            let _ = ctx.ui_tx.send(UICommand::ShowDebugRect { x, y, w, h });
        }
        InputEvent::ScreenshotRegion { x, y, w, h } => {
            let _ = ctx.ui_tx.send(UICommand::HideDebugRect);
            spawn_exclusive(ctx.answer_task.clone(), ctx.clone(), move |c| async move { screenshot_query::execute(c, x, y, w, h).await }).await;
        }
        InputEvent::ScreenshotCancel => {
            let _ = ctx.ui_tx.send(UICommand::HideDebugRect);
            let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Ready));
        }
        InputEvent::TestModel => {
            spawn_exclusive(ctx.answer_task.clone(), ctx.clone(), |c| async move { test_model::execute(c).await }).await;
        }
        InputEvent::SwitchModel => switch_model::execute(&ctx),
        // Design doc §7.4: v3 legacy is the default/primary engine as of 2026-09-20 — it takes
        // over these two main binds. v3 new (AX-tree/fill_page loop, §7.3) moved to the
        // `*Axtree` binds below, parked but reachable, not deleted.
        InputEvent::FormsAnswerPage => {
            spawn_exclusive(ctx.forms_task.clone(), ctx.clone(), |c| async move { forms_run_legacy::execute(c, forms_run_legacy::Mode::AnswerPage).await }).await;
        }
        InputEvent::FormsAnswerAll => {
            spawn_exclusive(ctx.forms_task.clone(), ctx.clone(), |c| async move { forms_run_legacy::execute(c, forms_run_legacy::Mode::AnswerAll).await }).await;
        }
        // v3 new (AX-tree engine) is untested/not working as of 2026-09-21 — hidden from real
        // usage by gating it behind the `debug` feature, same convention as `debug_open_tab`.
        // A release build never has this feature on, so these binds are dead keys in production
        // until v3 new is actually finished and re-enabled.
        InputEvent::FormsAnswerPageAxtree => {
            #[cfg(feature = "debug")]
            spawn_exclusive(ctx.forms_task.clone(), ctx.clone(), |c| async move { forms_run::execute(c, forms_run::Mode::AnswerPage).await }).await;
        }
        InputEvent::FormsAnswerAllAxtree => {
            #[cfg(feature = "debug")]
            spawn_exclusive(ctx.forms_task.clone(), ctx.clone(), |c| async move { forms_run::execute(c, forms_run::Mode::AnswerAll).await }).await;
        }
        InputEvent::LaunchDebugger => {
            if let Err(e) = launch_debugger::execute(&ctx) {
                log::error!("launch_debugger: {e}");
            }
        }
        InputEvent::Abort => {
            abort_active::execute(&ctx).await;
        }
        InputEvent::HideToggle => {
            let _ = ctx.ui_tx.send(UICommand::ToggleHide);
        }
        InputEvent::HelpToggle => help_toggle::execute(&ctx).await,
        InputEvent::RestartDaemon => {
            if let Err(e) = crate::lifecycle::self_restart::execute() {
                log::error!("restart: {e}");
            }
        }
        InputEvent::InstaDeleteArmed => {
            let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::InstaDeleteArmed));
        }
        InputEvent::InstaDeleteDisarmed => {
            let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Ready));
        }
        InputEvent::InstaDeleteConfirmed => {
            log::warn!("insta-delete confirmed");
            if let Err(e) = crate::lifecycle::self_delete::execute() {
                log::error!("self_delete: {e}");
            }
        }
        InputEvent::PanicKill => crate::lifecycle::panic::execute(),
        InputEvent::DebugOpenTab => {
            if let Err(e) = debug_open_tab::execute().await {
                log::error!("debug_open_tab: {e}");
            }
        }
    }
}

async fn spawn_exclusive<F, Fut>(slot: TaskSlot, ctx: ActionContext, action: F)
where
    F: FnOnce(ActionContext) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
    let mut guard = slot.lock().await;
    if let Some(h) = guard.take() {
        h.abort();
    }
    let handle = tokio::spawn(async move {
        if let Err(e) = action(ctx).await {
            log::error!("action error: {e}");
        }
    });
    *guard = Some(handle);
}
