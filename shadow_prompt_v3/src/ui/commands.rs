/// Commands sent from orchestrator/actions to the UI thread.
#[derive(Debug, Clone)]
pub enum UICommand {
    SetIndicatorState(IndicatorState),
    SetFormIndicator(FormIndicatorState),
    /// Sets the overlay's full text. Long text (more than `crawl_trigger_lines` wrapped lines,
    /// design doc §6) engages the gradient-crawl display instead of a static paint.
    SetOverlayText(String),
    ClearOverlay,
    ShowDebugRect { x: i32, y: i32, w: i32, h: i32 },
    HideDebugRect,
    /// Belt-and-suspenders for Screenshot Query's own capture (design doc §6/§13): regardless of
    /// whether `WDA_EXCLUDEFROMCAPTURE` is actually honored by whatever's reading the screen, our
    /// own region capture must never bleed a leftover answer/help panel from a previous query
    /// into a brand new one. Sent right before `capture_region`, paired with
    /// `RestoreOverlaysAfterCapture` right after.
    HideOverlaysForCapture,
    RestoreOverlaysAfterCapture,
    ToggleHide,
    ShowHelp(String),
    HideHelp,
    /// Brief overlay flash for switch_model — design doc §8/§10, e.g. "Switched to: Claude Sonnet 5".
    /// Transient: the UI thread auto-clears it a few seconds after it's set (see `manager.rs`'s
    /// `FLASH_NOTICE_MS`/`FLASH_TIMER_ID`), reverting the overlay to whatever it was showing
    /// before (the standing answer text, or empty) rather than sticking around indefinitely.
    FlashNotice(String),
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
    /// A run just finished successfully — shown briefly (green) before auto-hiding, so a
    /// successful run has a visible end state instead of jumping straight back to `Hidden`.
    Done,
    Failed,
    Aborted,
}
