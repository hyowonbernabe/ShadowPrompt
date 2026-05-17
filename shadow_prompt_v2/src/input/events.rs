/// Events produced by the input layer and consumed by the orchestrator.
#[derive(Debug, Clone)]
pub enum InputEvent {
    ClipboardQuery,
    OcrQuery,
    OcrRegionPoint { x: i32, y: i32 },
    FormsAuto,
    FormsSingle,
    Abort,
    LaunchDebugger,
    HideToggle,
    RestartDaemon,
    InstaDeleteArmed,
    InstaDeleteConfirmed,
    InstaDeleteDisarmed,
    PanicKill,
}
