/// Events produced by the input layer and consumed by the orchestrator.
#[derive(Debug, Clone)]
pub enum InputEvent {
    ClipboardQuery,
    ClipboardQuerySearch,
    OcrQuery,
    OcrQuerySearch,
    OcrRegion { x: i32, y: i32, w: i32, h: i32, online: bool },
    OcrCancel,
    FormsAuto,
    FormsSingle,
    Abort,
    LaunchDebugger,
    HideToggle,
    HelpToggle,
    RestartDaemon,
    InstaDeleteArmed,
    InstaDeleteConfirmed,
    InstaDeleteDisarmed,
    PanicKill,
}
