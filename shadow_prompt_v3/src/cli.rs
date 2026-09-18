// CLI args.

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "shadowprompt", version, about = "Lightweight stealth AI assistant.")]
pub struct Cli {
    /// Enable console + verbose logging. Design doc §13: logging is dev-flag only, silent in
    /// prod/release builds.
    #[arg(long)]
    pub debug: bool,

    /// Write a fresh config.toml from the embedded template, then exit.
    #[arg(long)]
    pub init: bool,

    /// Remove install dir from PATH, terminate Chrome instances we launched, delete self.
    #[arg(long)]
    pub uninstall: bool,

    /// Probe the configured model chain: capability/text/vision round-trips. Complements the
    /// live `test_model` hotkey (design doc §3) with an out-of-daemon check, e.g. before first
    /// launch.
    #[arg(long)]
    pub probe: bool,
}

impl Cli {
    pub fn load() -> Self {
        Self::parse()
    }
}
