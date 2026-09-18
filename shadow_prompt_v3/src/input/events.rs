/// Events produced by the input layer and consumed by the orchestrator. One variant per hotkey
/// in the final list — design doc §10.
#[derive(Debug, Clone)]
pub enum InputEvent {
    // Answer
    ClipboardQuery,
    ScreenshotQuery,
    /// Live drag-select feedback — sent on every mouse move once the first corner is placed, so
    /// the UI can actually show a growing rectangle instead of giving zero visual feedback
    /// during selection (a real, confirmed gap: nothing ever sent `UICommand::ShowDebugRect`
    /// before this, in any app, ever — not something specific to any one target application).
    ScreenshotDragUpdate { x: i32, y: i32, w: i32, h: i32 },
    ScreenshotRegion { x: i32, y: i32, w: i32, h: i32 },
    ScreenshotCancel,
    TestModel,
    SwitchModel,
    // Google Forms
    LaunchDebugger,
    FormsAnswerPage,
    FormsAnswerAll,
    // Utility
    Abort,
    HideToggle,
    HelpToggle,
    // Lifecycle
    RestartDaemon,
    InstaDeleteArmed,
    InstaDeleteConfirmed,
    InstaDeleteDisarmed,
    PanicKill,
    /// Dev-only test hotkey — opens a new tab in the debug Chrome window via the exact same code
    /// path Forms uses, without the LLM loop around it. Only does anything under the `debug`
    /// Cargo feature (see `actions::debug_open_tab`).
    DebugOpenTab,
}
