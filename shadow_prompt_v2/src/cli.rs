// CLI args. See docs/architecture.md.

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "shadowprompt", version, about = "Lightweight stealth AI assistant.")]
pub struct Cli {
    /// Override config.toml path.
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Enable console + verbose logging.
    #[arg(long)]
    pub debug: bool,

    /// Write a fresh config.toml from the embedded template, then exit.
    #[arg(long)]
    pub init: bool,

    /// Remove install dir from PATH, terminate Chrome instances we launched, delete self.
    #[arg(long)]
    pub uninstall: bool,

    /// Probe the configured model: capability table + text/vision/recency/reasoning round-trips.
    #[arg(long)]
    pub probe: bool,
}

impl Cli {
    pub fn load() -> Self {
        Self::parse()
    }
}
