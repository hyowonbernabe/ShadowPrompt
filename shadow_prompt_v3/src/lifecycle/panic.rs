// Panic-kill hotkey: wipe clipboard, exit immediately, no confirmation. Ported as-is from v2.

use crate::capture::clipboard;

pub fn execute() -> ! {
    let _ = clipboard::clear();
    std::process::exit(0);
}
