// simplelog init -> data/logs/shadowprompt.log. Design doc §13: logging only happens when
// launched with --debug (dev use); a normal release-mode launch with no --debug flag installs
// no logger at all, so nothing sensitive (question content, API responses) ever touches disk in
// the shipped, day-to-day stealth path.

use crate::config::paths::logs_dir;

pub fn init(debug: bool) -> anyhow::Result<()> {
    if !debug {
        return Ok(());
    }
    let dir = logs_dir()?;
    std::fs::create_dir_all(&dir)?;
    let log_path = dir.join("shadowprompt.log");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(
            simplelog::LevelFilter::Debug,
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
        simplelog::WriteLogger::new(simplelog::LevelFilter::Debug, simplelog::Config::default(), file),
    ])?;
    Ok(())
}
