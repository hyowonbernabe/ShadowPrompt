// Chrome/Edge automation via chromiumoxide (pure Rust CDP client — design doc §7.3). No Node,
// no Playwright dependency, no bundled Chromium.

pub mod debugger;
pub mod forms;

/// Marker string written into the incognito profile's temp dir, so any future
/// process-cleanup logic can identify Chrome instances this daemon spawned.
pub const PROFILE_MARKER: &str = "shadowprompt-v3-debug-profile";
