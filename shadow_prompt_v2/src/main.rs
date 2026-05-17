#![cfg_attr(not(feature = "debug"), windows_subsystem = "windows")]

// ShadowPrompt v2 — thin entry point.
// All real logic lives in lib.rs and module tree. See docs/architecture.md.

fn main() -> anyhow::Result<()> {
    shadowprompt::run()
}
