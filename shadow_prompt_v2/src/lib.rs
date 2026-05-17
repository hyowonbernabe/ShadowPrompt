// ShadowPrompt v2 — public library surface.
// Module structure mirrors docs/architecture.md.

pub mod actions;
pub mod browser;
pub mod capture;
pub mod cli;
pub mod config;
pub mod input;
pub mod lifecycle;
pub mod llm;
pub mod logger;
pub mod ui;

use crate::cli::Cli;

/// Daemon entry point. Called by `main.rs` and by integration tests.
pub fn run() -> anyhow::Result<()> {
    let args = Cli::load();

    if args.init {
        let path = config::load::init_template()?;
        println!("config written to {}", path.display());
        return Ok(());
    }

    if args.uninstall {
        lifecycle::self_delete::execute()?;
        return Ok(());
    }

    logger::init(args.debug)?;
    lifecycle::startup::install_panic_hook();
    let _lock = lifecycle::startup::acquire_single_instance()?;

    if let Ok(path) = config::paths::config_path() {
        if !path.exists() {
            config::load::init_template()?;
            anyhow::bail!(
                "first run: edit {} (set openrouter.api_key) then relaunch",
                path.display()
            );
        }
    }

    let cfg = config::load::load()?;
    log::info!(
        "shadowprompt v{} ready; model={}",
        env!("CARGO_PKG_VERSION"),
        cfg.openrouter.model_id
    );

    run_daemon(cfg, args)
}

fn run_daemon(cfg: config::Config, _args: Cli) -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let mut input_rx = input::start(cfg.hotkeys.clone())?;
        log::info!("daemon loop running; awaiting input events");
        let _ = &cfg;
        while let Some(event) = input_rx.recv().await {
            log::info!("event: {:?}", event);
        }
        Ok::<(), anyhow::Error>(())
    })
}
