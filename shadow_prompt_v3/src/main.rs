#![cfg_attr(not(feature = "debug"), windows_subsystem = "windows")]

// ShadowPrompt v3 — thin entry point. All real logic lives in lib.rs and the module tree. See
// docs/SHADOWPROMPT_V3_DESIGN.md.

fn main() -> anyhow::Result<()> {
    shadowprompt::run()
}
