// ShadowPrompt v2 — public library surface.
// Module structure mirrors docs/architecture.md.

pub mod actions;
pub mod browser;
pub mod capture;
pub mod cli;
pub mod config;
pub mod input;
pub mod knowledge;
pub mod lifecycle;
pub mod llm;
pub mod logger;
pub mod probe;
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

    if args.probe {
        let cfg = config::load::load()?;
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        return rt.block_on(probe::run(cfg));
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
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let ui_tx = ui::start(cfg.visuals.clone())?;
        let mut input_rx = input::start(cfg.hotkeys.clone())?;

        let install_dir = config::paths::exe_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let knowledge = knowledge::load(&install_dir, &cfg.knowledge);

        let llm = llm::LlmClient::new(
            cfg.openrouter.api_key.clone(),
            cfg.openrouter.model_id.clone(),
            cfg.http.connect_timeout_secs,
            cfg.http.read_timeout_secs,
            knowledge,
            cfg.knowledge.cache_ttl.clone(),
        )?;

        let action_ctx = actions::ActionContext {
            config: Arc::new(cfg),
            llm: Arc::new(llm),
            ui_tx: ui_tx.clone(),
            active_task: Arc::new(Mutex::new(None)),
        };

        log::info!("daemon loop running; awaiting input events");
        while let Some(event) = input_rx.recv().await {
            log::debug!("event: {:?}", event);
            actions::dispatch(action_ctx.clone(), event).await;
        }
        Ok::<(), anyhow::Error>(())
    })
}
