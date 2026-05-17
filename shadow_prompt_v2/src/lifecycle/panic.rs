// Panic key: wipe clipboard, exit immediately. No prompts.

use crate::capture::clipboard;

pub fn execute() -> ! {
    let _ = clipboard::clear();
    std::process::exit(0);
}
