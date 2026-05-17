// ShadowPrompt v2 — public library surface.
// Module structure mirrors docs/architecture.md.

pub mod actions;
pub mod browser;
pub mod capture;
pub mod config;
pub mod input;
pub mod lifecycle;
pub mod llm;
pub mod logger;
pub mod ui;

/// Daemon entry point. Called by `main.rs` and by integration tests.
pub fn run() -> anyhow::Result<()> {
    // TODO: parse CLI, load config, init logger, build runtime, wire channels, start threads.
    anyhow::bail!("not yet implemented")
}
