// Map parsed KeyCombos to InputEvent factories driven by config.

use super::events::InputEvent;
use super::parser::{parse, KeyCombo};
use crate::config::schema::HotkeysConfig;

pub struct Binding {
    pub combo: KeyCombo,
    pub make_event: fn() -> InputEvent,
}

pub fn build(cfg: &HotkeysConfig) -> anyhow::Result<Vec<Binding>> {
    Ok(vec![
        // Order matters: more-specific (4-modifier) combos must come BEFORE
        // the 3-modifier base combos so the listener matches them first.
        Binding { combo: parse(&cfg.clipboard_query_search)?, make_event: || InputEvent::ClipboardQuerySearch },
        Binding { combo: parse(&cfg.clipboard_query)?, make_event: || InputEvent::ClipboardQuery },
        Binding { combo: parse(&cfg.ocr_query_search)?, make_event: || InputEvent::OcrQuerySearch },
        Binding { combo: parse(&cfg.ocr_query)?, make_event: || InputEvent::OcrQuery },
        Binding { combo: parse(&cfg.forms_auto)?, make_event: || InputEvent::FormsAuto },
        Binding { combo: parse(&cfg.forms_single)?, make_event: || InputEvent::FormsSingle },
        Binding { combo: parse(&cfg.abort)?, make_event: || InputEvent::Abort },
        Binding { combo: parse(&cfg.launch_debugger)?, make_event: || InputEvent::LaunchDebugger },
        Binding { combo: parse(&cfg.hide_toggle)?, make_event: || InputEvent::HideToggle },
        Binding { combo: parse(&cfg.restart_daemon)?, make_event: || InputEvent::RestartDaemon },
        // insta_delete is matched separately; it drives the state machine, not a direct event.
        Binding { combo: parse(&cfg.panic_kill)?, make_event: || InputEvent::PanicKill },
    ])
}

pub fn insta_delete_combo(cfg: &HotkeysConfig) -> anyhow::Result<KeyCombo> {
    parse(&cfg.insta_delete)
}
