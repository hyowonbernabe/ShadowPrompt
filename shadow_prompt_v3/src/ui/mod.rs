// UI layer: Win32 windows + message loop. Sync, runs on its own OS thread.

pub mod colors;
pub mod commands;
pub mod manager;

pub use commands::UICommand;

use crate::config::schema::VisualsConfig;

/// Spawn UI thread. Returns sender for UICommand.
pub fn start(visuals: VisualsConfig) -> anyhow::Result<tokio::sync::mpsc::UnboundedSender<UICommand>> {
    manager::start(visuals)
}
