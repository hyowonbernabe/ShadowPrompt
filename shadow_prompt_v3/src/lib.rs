// ShadowPrompt v3 — public library surface. Module structure mirrors
// docs/SHADOWPROMPT_V3_DESIGN.md.

pub mod account;
pub mod actions;
pub mod analytics;
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
#[cfg(feature = "seb")]
pub mod seb;
pub mod ui;

use crate::cli::Cli;

/// Daemon entry point. Called by main.rs.
pub fn run() -> anyhow::Result<()> {
    // Without this, an unaware process gets DPI-virtualized GetSystemMetrics/BitBlt screen
    // capture on any monitor with scaling != 100%, while rdev's low-level mouse hook still
    // reports real physical pixels — Screenshot Query then grabs the wrong region on scaled
    // displays. Must run before any window or screen-metrics call.
    let dpi_mode = set_dpi_awareness();

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

    // First run: write the template; the launch dialog then offers log in / sign up, so nobody
    // has to hand-edit a key in.
    if let Ok(path) = config::paths::config_path() {
        if !path.exists() {
            config::load::init_template()?;
        }
    }

    let mut cfg = config::load::load()?;
    log::info!("shadowprompt v{} ready", env!("CARGO_PKG_VERSION"));
    // Positions (selection, capture, dots) are only right if this says per-monitor; anything
    // else on a 125%/150% display means regions and dots land in the wrong place.
    log::info!("dpi awareness: {dpi_mode}, system dpi {}", unsafe { windows::Win32::UI::HiDpi::GetDpiForSystem() });

    if args.probe {
        if cfg.account.logged_in() {
            cfg.openrouter.api_key = cfg.account.api_key.clone();
        }
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
        return rt.block_on(probe::run(cfg));
    }

    let resumed = std::env::var_os(lifecycle::self_restart::RESUME_ENV).is_some();
    analytics::track("app_start", serde_json::json!({ "resumed": resumed, "dpi": dpi_mode, "models": cfg.openrouter.models }));

    if !choose_key(&mut cfg)? {
        log::info!("launch dialog closed; exiting");
        analytics::track("launch_quit", serde_json::json!({}));
        // Quiet exit only (nothing is waiting on us): a short, bounded send of what's queued.
        analytics::flush_before_exit(std::time::Duration::from_secs(3));
        return Ok(());
    }

    run_daemon(cfg)
}

fn run_daemon(cfg: config::Config) -> anyhow::Result<()> {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(async move {
        let ui_tx = ui::start(cfg.visuals.clone())?;
        let mut input_rx = input::start(cfg.hotkeys.clone())?;

        let llm = llm::LlmClient::new(
            cfg.openrouter.api_key.clone(),
            cfg.openrouter.models.clone(),
            cfg.http.connect_timeout_secs,
            cfg.http.read_timeout_secs,
        )?;

        // Sandboxed knowledge/ folder — design doc §4. Exe-relative, never hardcoded absolute
        // (matches the USB-portability convention every other data dir already follows).
        let knowledge = Arc::new(knowledge::KnowledgeStore::new(
            config::paths::knowledge_dir()?,
            cfg.knowledge.max_doc_bytes,
            cfg.knowledge.cloud_url.clone(),
        )?);
        if cfg.knowledge.enabled {
            knowledge.refresh_in_background();
        }

        let action_ctx = actions::ActionContext {
            config: Arc::new(cfg),
            llm: Arc::new(llm),
            knowledge,
            ui_tx: ui_tx.clone(),
            answer_task: Arc::new(Mutex::new(None)),
            forms_task: Arc::new(Mutex::new(None)),
            conversation: Default::default(),
            busy: Default::default(),
            screen_task: Arc::new(Mutex::new(None)),
        };

        log::info!("daemon loop running; awaiting input events");
        while let Some(event) = input_rx.recv().await {
            log::debug!("event: {:?}", event);
            actions::dispatch(action_ctx.clone(), event).await;
        }
        Ok::<(), anyhow::Error>(())
    })
}

/// Picks the API key for this run (design: docs/superpowers/specs/2026-11-06-access-code-login-design.md).
/// A launch the user starts shows the dialog every time; a restart the app performs itself
/// (`self_restart::RESUME_ENV`) skips it and keeps the earlier choice. Returns false if the user
/// closed the dialog (exit without starting).
fn choose_key(cfg: &mut config::Config) -> anyhow::Result<bool> {
    use lifecycle::self_restart::{KEY_SOURCE_ENV, RESUME_ENV};
    let resumed = std::env::var_os(RESUME_ENV).is_some();
    // Read once, then drop it, so nothing this instance starts (e.g. Chrome) inherits it and a
    // copy launched from there by the user still shows the dialog.
    std::env::remove_var(RESUME_ENV);
    let from_account = if resumed {
        let wants_account = std::env::var(KEY_SOURCE_ENV).map(|v| v != "config").unwrap_or(true);
        let use_account = wants_account && cfg.account.logged_in();
        if use_account {
            cfg.openrouter.api_key = cfg.account.api_key.clone();
        }
        use_account
    } else {
        match ui::account_dialog::run_launch(&cfg.account, &cfg.openrouter.api_key) {
            ui::account_dialog::LaunchChoice::Quit => return Ok(false),
            ui::account_dialog::LaunchChoice::Use { key, from_account } => {
                // A log in / sign up just wrote [account] to the file, and a log in may have pulled
                // synced settings into it too; reload all of it so this run uses them.
                if let Ok(fresh) = config::load::load() {
                    *cfg = fresh;
                }
                cfg.openrouter.api_key = key;
                from_account
            }
        }
    };
    std::env::set_var(KEY_SOURCE_ENV, if from_account { "account" } else { "config" });
    if from_account {
        analytics::set_account(&cfg.account.code);
    }
    analytics::track("launch_choice", serde_json::json!({ "key_source": if from_account { "account" } else { "config" }, "resumed": resumed }));
    log::info!("api key: {}", if from_account { "account login" } else { "config.toml" });
    Ok(true)
}

/// Makes the process per-monitor DPI aware, falling back to older APIs (Windows 8.1, then
/// Vista) if the newest one fails, and reports which mode actually took effect.
fn set_dpi_awareness() -> &'static str {
    use windows::Win32::UI::HiDpi::{
        SetProcessDpiAwareness, SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        PROCESS_PER_MONITOR_DPI_AWARE,
    };
    // SAFETY: plain Win32 calls with constant arguments, made before any window exists.
    unsafe {
        if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).is_ok() {
            return "per-monitor v2";
        }
        if SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE).is_ok() {
            return "per-monitor v1 (fallback)";
        }
        if windows::Win32::UI::WindowsAndMessaging::SetProcessDPIAware().as_bool() {
            return "system-aware only (fallback; per-monitor unavailable)";
        }
    }
    "UNAWARE (all DPI calls failed; positions will be off on scaled displays)"
}
