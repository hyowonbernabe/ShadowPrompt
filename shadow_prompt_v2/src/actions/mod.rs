// Action handlers: one free async function per hotkey-driven action.
// See docs/architecture.md "Layered Module Boundaries".

pub mod abort_active;
pub mod clipboard_query;
pub mod forms_run;
pub mod hide_toggle;
pub mod launch_debugger;
pub mod ocr_query;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::config::Config;
use crate::input::InputEvent;
use crate::llm::LlmClient;
use crate::ui::commands::{IndicatorState, UICommand};

/// Shared dependencies passed to every action.
#[derive(Clone)]
pub struct ActionContext {
    pub config: Arc<Config>,
    pub llm: Arc<LlmClient>,
    pub ui_tx: tokio::sync::mpsc::UnboundedSender<UICommand>,
    pub active_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

/// Dispatch an InputEvent to the appropriate action handler.
pub async fn dispatch(ctx: ActionContext, event: InputEvent) {
    match event {
        InputEvent::ClipboardQuery => spawn_exclusive(ctx, |c| async move {
            clipboard_query::execute(c).await
        }).await,
        InputEvent::OcrQuery => spawn_exclusive(ctx, |c| async move {
            ocr_query::execute(c).await
        }).await,
        InputEvent::FormsAuto => spawn_exclusive(ctx, |c| async move {
            forms_run::execute(c, true).await
        }).await,
        InputEvent::FormsSingle => spawn_exclusive(ctx, |c| async move {
            forms_run::execute(c, false).await
        }).await,
        InputEvent::Abort => abort_active::execute(ctx).await,
        InputEvent::LaunchDebugger => {
            if let Err(e) = launch_debugger::execute(ctx.clone()).await {
                log::error!("launch_debugger: {e}");
            }
        }
        InputEvent::HideToggle => {
            let _ = ctx.ui_tx.send(UICommand::ToggleHide);
        }
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
        InputEvent::OcrRegionPoint { .. } => {
            // Mouse-driven region capture: handled inside ocr_query session.
        }
    }
}

async fn spawn_exclusive<F, Fut>(ctx: ActionContext, action: F)
where
    F: FnOnce(ActionContext) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
    let mut slot = ctx.active_task.lock().await;
    if let Some(h) = slot.take() {
        h.abort();
    }
    let ctx2 = ctx.clone();
    let handle = tokio::spawn(async move {
        if let Err(e) = action(ctx2).await {
            log::error!("action error: {e}");
        }
    });
    *slot = Some(handle);
}
