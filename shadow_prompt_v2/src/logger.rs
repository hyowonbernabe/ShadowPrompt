// Logger init. File sink always on; stdout sink in --debug.

use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::{create_dir_all, OpenOptions};

use crate::config::paths::log_path;

pub fn init(debug: bool) -> anyhow::Result<()> {
    let path = log_path()?;
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let file = OpenOptions::new().create(true).append(true).open(&path)?;

    let cfg = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .set_target_level(LevelFilter::Error)
        .build();

    let file_level = if debug { LevelFilter::Debug } else { LevelFilter::Info };
    let mut loggers: Vec<Box<dyn simplelog::SharedLogger>> =
        vec![WriteLogger::new(file_level, cfg.clone(), file)];

    if debug {
        loggers.push(TermLogger::new(
            LevelFilter::Debug,
            cfg,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ));
    }

    CombinedLogger::init(loggers).ok();
    Ok(())
}
