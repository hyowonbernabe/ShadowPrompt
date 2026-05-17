// Panic key: wipe clipboard, exit immediately. No prompts.

pub fn execute() -> ! {
    // TODO: clear clipboard via capture::clipboard.
    std::process::exit(0);
}
