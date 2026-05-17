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
use crate::llm::LlmClient;
use crate::ui::UICommand;

/// Shared dependencies passed to every action.
#[derive(Clone)]
pub struct ActionContext {
    pub config: Arc<Config>,
    pub llm: Arc<LlmClient>,
    pub ui_tx: tokio::sync::mpsc::UnboundedSender<UICommand>,
    pub active_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}
