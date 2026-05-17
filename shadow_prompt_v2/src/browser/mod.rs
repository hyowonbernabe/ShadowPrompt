// Chrome session + Google Forms automation.

pub mod cookies;
pub mod debugger;
pub mod forms;

/// Marker string written into temp profile path so chrome_cleanup can identify
/// processes spawned by this daemon.
pub const PROFILE_MARKER: &str = "shadowprompt-debug-profile";
