use simplelog::*;
use std::fs::File;
use std::path::Path;

pub fn init() -> anyhow::Result<()> {
    // Ensure data/logs directory exists
    let log_dir = Path::new("data/logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(log_dir)?;
    }

    let log_file = File::create(log_dir.join("error.log"))?;

    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        log_file,
    )?;

    Ok(())
}
