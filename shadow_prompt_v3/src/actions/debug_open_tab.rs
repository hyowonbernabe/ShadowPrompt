// Debug-only test hotkey (Ctrl+Shift+Alt+Z) — opens a new tab in the debug Chrome window via
// the exact same code path `execute_form_flow` uses (`tab_lifecycle::open_forms_tab`), without
// the LLM loop around it. Lets "does this open a new tab or a new window" get tested instantly
// and repeatedly instead of waiting on a full Forms run each time. Gated behind the `debug`
// Cargo feature — same reasoning as the debug selection-rectangle box (`ui::commands`): this has
// no business existing in a production build.

#[cfg(feature = "debug")]
pub async fn execute() -> anyhow::Result<()> {
    use crate::browser::debugger;
    use crate::browser::forms::tab_lifecycle;

    let browser = debugger::connect().await?;
    tab_lifecycle::open_forms_tab(&browser, "about:blank").await?;
    Ok(())
}

#[cfg(not(feature = "debug"))]
pub async fn execute() -> anyhow::Result<()> {
    Ok(())
}
