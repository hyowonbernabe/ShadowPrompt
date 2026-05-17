/// Events produced by the input layer and consumed by the orchestrator.
#[derive(Debug, Clone)]
pub enum InputEvent {
    ClipboardQuery,
    OcrQuery,
    OcrRegion { x: i32, y: i32, w: i32, h: i32 },
    OcrCancel,
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
