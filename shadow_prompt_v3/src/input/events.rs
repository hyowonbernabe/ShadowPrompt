/// Events produced by the input layer and consumed by the orchestrator. One variant per hotkey
/// in the final list — design doc §10.
/// What a finished region selection is for — one selection mechanism serves both hotkeys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RegionPurpose {
    /// Screenshot Query: answer what's in the region.
    #[default]
    Query,
    /// Add context: store the region in the conversation as reference material, no answer.
    Context,
}

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
    ScreenshotRegion { x: i32, y: i32, w: i32, h: i32, purpose: RegionPurpose },
    ScreenshotCancel,
    TestModel,
    SwitchModel,
    // Conversation memory
    /// Arms a region selection whose result is stored as reference material, not answered.
    AddContext,
    ResetConversation,
    /// Screen Answer: all monitors -> Claude -> tracked dots on click targets.
    ScreenAnswer,
    ClearDots,
    /// Opens the account screen (access-code login).
    Account,
    // Google Forms
    LaunchDebugger,
    /// v3 legacy (design doc §7.4) — default/primary as of 2026-09-20.
    FormsAnswerPage,
    FormsAnswerAll,
    /// v3 new (design doc §7.3) — AX-tree read + `fill_page` agentic loop, parked/secondary,
    /// moved off the main binds but kept reachable.
    FormsAnswerPageAxtree,
    FormsAnswerAllAxtree,
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
    /// Reset/unpatch SEB during a running session.
    SebUnpatch,
    /// Clean exit — no clipboard wipe, no self-delete.
    Quit,
}

impl InputEvent {
    /// Stable name for analytics. Exhaustive on purpose: a new hotkey won't compile until it's named.
    pub fn name(&self) -> &'static str {
        match self {
            InputEvent::ClipboardQuery => "clipboard_query",
            InputEvent::ScreenshotQuery => "screenshot_query_armed",
            InputEvent::ScreenshotDragUpdate { .. } => "screenshot_drag",
            InputEvent::ScreenshotRegion { purpose: RegionPurpose::Query, .. } => "screenshot_query",
            InputEvent::ScreenshotRegion { purpose: RegionPurpose::Context, .. } => "add_context",
            InputEvent::ScreenshotCancel => "screenshot_cancel",
            InputEvent::TestModel => "test_model",
            InputEvent::SwitchModel => "switch_model",
            InputEvent::AddContext => "add_context_armed",
            InputEvent::ResetConversation => "reset_conversation",
            InputEvent::ScreenAnswer => "screen_answer",
            InputEvent::ClearDots => "clear_dots",
            InputEvent::Account => "account_screen",
            InputEvent::LaunchDebugger => "launch_debugger",
            InputEvent::FormsAnswerPage => "forms_answer_page",
            InputEvent::FormsAnswerAll => "forms_answer_all",
            InputEvent::FormsAnswerPageAxtree => "forms_answer_page_axtree",
            InputEvent::FormsAnswerAllAxtree => "forms_answer_all_axtree",
            InputEvent::Abort => "abort",
            InputEvent::HideToggle => "hide_toggle",
            InputEvent::HelpToggle => "help_toggle",
            InputEvent::RestartDaemon => "restart",
            InputEvent::InstaDeleteArmed => "insta_delete_armed",
            InputEvent::InstaDeleteConfirmed => "insta_delete_confirmed",
            InputEvent::InstaDeleteDisarmed => "insta_delete_disarmed",
            InputEvent::PanicKill => "panic_kill",
            InputEvent::DebugOpenTab => "debug_open_tab",
            InputEvent::SebUnpatch => "seb_unpatch",
            InputEvent::Quit => "quit",
        }
    }
}
