/// Commands sent from orchestrator/actions to the UI thread.
#[derive(Debug, Clone)]
pub enum UICommand {
    SetIndicatorState(IndicatorState),
    SetFormIndicator(FormIndicatorState),
    SetOverlayText(String),
    ClearOverlay,
    ShowDebugRect { x: i32, y: i32, w: i32, h: i32 },
    HideDebugRect,
    ToggleHide,
    ShowHelp(String),
    HideHelp,
    Shutdown,
}

#[derive(Debug, Clone, Copy)]
pub enum IndicatorState {
    Ready,
    Processing,
    Error,
    InstaDeleteArmed,
}

#[derive(Debug, Clone, Copy)]
pub enum FormIndicatorState {
    Hidden,
    Running,
    Failed,
    Aborted,
}
