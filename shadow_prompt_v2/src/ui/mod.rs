// UI layer: Win32 windows + message loop. Sync, runs on its own OS thread.

pub mod commands;
pub mod debug_rect;
pub mod form_indicator;
pub mod indicator;
pub mod overlay;
pub mod state;

pub use commands::UICommand;

/// Spawn UI thread. Returns sender for UICommand.
pub fn start() -> anyhow::Result<tokio::sync::mpsc::UnboundedSender<UICommand>> {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    // TODO: spawn std::thread, create Win32 windows, run GetMessage loop,
    // poll channel via PeekMessage between dispatches.
    Ok(tx)
}
