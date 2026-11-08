// Action handlers: one function per hotkey. Design doc §7.3: Forms runs on its own independent
// task-exclusivity slot, separate from Clipboard/Screenshot Query's shared slot — firing one
// family doesn't cancel the other. Clipboard and Screenshot Query still share one slot with each
// other (cancel-and-replace, same as v2).

/// The running conversation as model messages, plus whether it holds any screenshot (so the
/// model chain is kept vision-capable). Rule: everything sent to the model carries this.
pub fn conversation_context(ctx: &ActionContext) -> (Vec<crate::llm::messages::Message>, bool) {
    let c = ctx.conversation.lock().unwrap();
    (c.to_messages(), c.has_image())
}

pub mod abort_active;
pub mod add_context;
pub mod clipboard_query;
pub mod debug_open_tab;
pub mod forms_run;
pub mod forms_run_legacy;
pub mod help_toggle;
pub mod hide_toggle;
pub mod launch_debugger;
pub mod reset_conversation;
pub mod screen_answer;
pub mod screenshot_query;
pub mod switch_model;
pub mod test_model;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::config::Config;
use crate::input::{InputEvent, RegionPurpose};
use crate::knowledge::KnowledgeStore;
use crate::llm::conversation::Conversation;
use crate::llm::LlmClient;
use crate::ui::commands::{IndicatorState, UICommand};

pub type TaskSlot = Arc<Mutex<Option<JoinHandle<()>>>>;

/// Shared dependencies passed to every action.
#[derive(Clone)]
pub struct ActionContext {
    pub config: Arc<Config>,
    pub llm: Arc<LlmClient>,
    /// Sandboxed `knowledge/` folder access for the Glob/Grep/Read doc tools — design doc §4.
    /// Constructed once at daemon startup (`lib.rs::run_daemon`), shared by every action.
    pub knowledge: Arc<KnowledgeStore>,
    pub ui_tx: tokio::sync::mpsc::UnboundedSender<UICommand>,
    /// Clipboard Query + Screenshot Query share this slot (cancel-and-replace).
    pub answer_task: TaskSlot,
    /// Forms runs get their own independent slot — design doc §7.3.
    pub forms_task: TaskSlot,
    /// Running conversation shared by Clipboard/Screenshot Query and add-context. Forms doesn't
    /// use it. std (not tokio) Mutex: only ever held for a quick copy/push, never across an await.
    pub conversation: Arc<std::sync::Mutex<Conversation>>,
    /// Model calls in flight across every slot — drives the indicator so one job finishing
    /// doesn't show Ready while another is still waiting on the model. See `busy()`.
    pub busy: Arc<AtomicUsize>,
    /// Screen Answer's own slot: holds the answer job and then its dot-tracking loop.
    pub screen_task: TaskSlot,
}

pub async fn dispatch(ctx: ActionContext, event: InputEvent) {
    // Drag updates fire on every mouse move; the finished region is tracked instead.
    if !matches!(event, InputEvent::ScreenshotDragUpdate { .. }) {
        crate::analytics::track("hotkey", serde_json::json!({ "action": event.name() }));
    }
    let name = event.name();
    match event {
        InputEvent::ClipboardQuery => {
            spawn_exclusive(name, ctx.answer_task.clone(), ctx.clone(), |c| async move { clipboard_query::execute(c).await }).await;
        }
        InputEvent::ScreenshotQuery | InputEvent::AddContext => {
            let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Processing));
        }
        InputEvent::ResetConversation => reset_conversation::execute(&ctx).await,
        // Real, confirmed gap fixed here: this is the only place a selection rectangle has ever
        // been shown, in any app — nothing sent UICommand::ShowDebugRect before this existed.
        InputEvent::ScreenshotDragUpdate { x, y, w, h } => {
            let _ = ctx.ui_tx.send(UICommand::ShowDebugRect { x, y, w, h });
        }
        InputEvent::ScreenshotRegion { x, y, w, h, purpose } => {
            let _ = ctx.ui_tx.send(UICommand::HideDebugRect);
            match purpose {
                RegionPurpose::Query => {
                    spawn_exclusive(name, ctx.answer_task.clone(), ctx.clone(), move |c| async move { screenshot_query::execute(c, x, y, w, h).await }).await;
                }
                // Not in `answer_task`: adding context must never abort a question in flight.
                RegionPurpose::Context => {
                    let c = ctx.clone();
                    tokio::spawn(async move {
                        let run = crate::analytics::run(name);
                        add_context::execute(c, x, y, w, h).await;
                        run.finish(&Ok::<(), anyhow::Error>(()));
                    });
                }
            }
        }
        InputEvent::ScreenshotCancel => {
            let _ = ctx.ui_tx.send(UICommand::HideDebugRect);
            let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(idle_indicator(&ctx)));
        }
        InputEvent::TestModel => {
            spawn_exclusive(name, ctx.answer_task.clone(), ctx.clone(), |c| async move { test_model::execute(c).await }).await;
        }
        InputEvent::SwitchModel => switch_model::execute(&ctx),
        InputEvent::ScreenAnswer => {
            spawn_exclusive(name, ctx.screen_task.clone(), ctx.clone(), |c| async move { screen_answer::execute(c).await }).await;
        }
        InputEvent::ClearDots => {
            clear_dots(&ctx).await;
            let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(idle_indicator(&ctx)));
        }
        // Own thread: the account window runs its own message loop and blocks until closed.
        InputEvent::Account => crate::ui::account_dialog::open_account_screen(),
        // Design doc §7.4: v3 legacy is the default/primary engine as of 2026-09-20 — it takes
        // over these two main binds. v3 new (AX-tree/fill_page loop, §7.3) moved to the
        // `*Axtree` binds below, parked but reachable, not deleted.
        InputEvent::FormsAnswerPage => {
            spawn_exclusive(name, ctx.forms_task.clone(), ctx.clone(), |c| async move { forms_run_legacy::execute(c, forms_run_legacy::Mode::AnswerPage).await }).await;
        }
        InputEvent::FormsAnswerAll => {
            spawn_exclusive(name, ctx.forms_task.clone(), ctx.clone(), |c| async move { forms_run_legacy::execute(c, forms_run_legacy::Mode::AnswerAll).await }).await;
        }
        // v3 new (AX-tree engine) is untested/not working as of 2026-09-21 — hidden from real
        // usage by gating it behind the `debug` feature, same convention as `debug_open_tab`.
        // A release build never has this feature on, so these binds are dead keys in production
        // until v3 new is actually finished and re-enabled.
        InputEvent::FormsAnswerPageAxtree => {
            #[cfg(feature = "debug")]
            spawn_exclusive(name, ctx.forms_task.clone(), ctx.clone(), |c| async move { forms_run::execute(c, forms_run::Mode::AnswerPage).await }).await;
        }
        InputEvent::FormsAnswerAllAxtree => {
            #[cfg(feature = "debug")]
            spawn_exclusive(name, ctx.forms_task.clone(), ctx.clone(), |c| async move { forms_run::execute(c, forms_run::Mode::AnswerAll).await }).await;
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
            let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(idle_indicator(&ctx)));
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
        InputEvent::SebUnpatch => {
            #[cfg(feature = "seb")]
            match crate::seb::reset() {
                Ok(msg) => {
                    log::info!("seb_unpatch: {msg}");
                    let _ = ctx.ui_tx.send(UICommand::FlashNotice(msg));
                }
                Err(e) => {
                    log::error!("seb_unpatch: {e}");
                    let _ = ctx.ui_tx.send(UICommand::FlashNotice(format!("SEB reset failed: {e}")));
                }
            }
            #[cfg(not(feature = "seb"))]
            {
                log::warn!("seb_unpatch: SEB support not compiled in");
                let _ = ctx.ui_tx.send(UICommand::FlashNotice("SEB support not available in this build".into()));
            }
        }
        InputEvent::Quit => std::process::exit(0),
    }
}

/// The indicator state to show when a job finishes: still `Processing` if any other model call
/// is in flight (any slot), else `Ready`.
pub fn idle_indicator(ctx: &ActionContext) -> IndicatorState {
    if ctx.busy.load(Ordering::SeqCst) > 0 { IndicatorState::Processing } else { IndicatorState::Ready }
}

/// Shows a failed job to the user: Error light plus a short reason on the overlay. The full
/// error is logged by the caller (`spawn_exclusive`).
pub fn report_failure(ctx: &ActionContext, e: &anyhow::Error) {
    let reason = crate::llm::user_facing_error(e);
    crate::analytics::track("error_shown", serde_json::json!({ "reason": reason }));
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Error));
    let _ = ctx.ui_tx.send(UICommand::FlashNotice(reason.to_string()));
}

/// Shows a finished answer: overlay first, then the clipboard copy. A clipboard locked by
/// another app is logged, never allowed to cost the user the answer or stick the indicator.
pub async fn deliver_answer(ctx: &ActionContext, answer: String) {
    let _ = ctx.ui_tx.send(UICommand::SetOverlayText(answer.clone()));
    match tokio::task::spawn_blocking(move || crate::capture::clipboard::write_text(&answer)).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => log::warn!("clipboard write failed: {e}"),
        Err(e) => log::warn!("clipboard write task failed: {e}"),
    }
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(idle_indicator(ctx)));
}

/// Counts one in-flight model call until dropped — including when the task is aborted, since
/// an abort drops the future and with it this guard.
pub struct BusyGuard(Arc<AtomicUsize>);

impl Drop for BusyGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

pub fn busy(ctx: &ActionContext) -> BusyGuard {
    ctx.busy.fetch_add(1, Ordering::SeqCst);
    BusyGuard(ctx.busy.clone())
}

/// Sends `RestoreOverlaysAfterCapture` when dropped, so an abort mid-capture can't leave the
/// answer/help overlays (and dots) hidden.
pub struct RestoreOverlaysOnDrop(pub tokio::sync::mpsc::UnboundedSender<UICommand>);

impl Drop for RestoreOverlaysOnDrop {
    fn drop(&mut self) {
        let _ = self.0.send(UICommand::RestoreOverlaysAfterCapture);
    }
}

/// Aborts whatever runs in `slot` and waits until it is actually gone, so nothing it does can
/// land after the caller's next UI update (e.g. a tracking tick re-showing cleared dots).
pub async fn abort_slot(slot: &TaskSlot) {
    let handle = slot.lock().await.take();
    if let Some(h) = handle {
        h.abort();
        let _ = h.await;
    }
}

/// Stops Screen Answer (answer + dot tracking) and removes its dots.
pub async fn clear_dots(ctx: &ActionContext) {
    abort_slot(&ctx.screen_task).await;
    let _ = ctx.ui_tx.send(UICommand::SetDots(Vec::new()));
}

async fn spawn_exclusive<F, Fut>(name: &'static str, slot: TaskSlot, ctx: ActionContext, action: F)
where
    F: FnOnce(ActionContext) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
    let mut guard = slot.lock().await;
    if let Some(h) = guard.take() {
        h.abort();
    }
    let handle = tokio::spawn(async move {
        // Dropped unfinished (aborted / replaced) = reported as cancelled.
        let run = crate::analytics::run(name);
        let r = action(ctx).await;
        run.finish(&r);
        if let Err(e) = r {
            log::error!("action error: {e}");
        }
    });
    *guard = Some(handle);
}
