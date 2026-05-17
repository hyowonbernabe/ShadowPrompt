// Input layer: global keyboard + mouse hook (rdev) → InputEvent on channel.

pub mod events;
pub mod parser;
pub mod state_machine;

pub use events::InputEvent;

/// Spawn the input listener OS thread. Returns a receiver for InputEvent.
pub fn start() -> anyhow::Result<tokio::sync::mpsc::UnboundedReceiver<InputEvent>> {
    let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
    // TODO: spawn std::thread, call rdev::listen, translate to InputEvent, send on tx.
    Ok(rx)
}
