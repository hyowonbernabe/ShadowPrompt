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
        Binding { combo: parse(&cfg.clipboard_query)?, make_event: || InputEvent::ClipboardQuery },
        Binding { combo: parse(&cfg.screenshot_query)?, make_event: || InputEvent::ScreenshotQuery },
        Binding { combo: parse(&cfg.test_model)?, make_event: || InputEvent::TestModel },
        Binding { combo: parse(&cfg.switch_model)?, make_event: || InputEvent::SwitchModel },
        Binding { combo: parse(&cfg.launch_debugger)?, make_event: || InputEvent::LaunchDebugger },
        Binding { combo: parse(&cfg.forms_answer_page)?, make_event: || InputEvent::FormsAnswerPage },
        Binding { combo: parse(&cfg.forms_answer_all)?, make_event: || InputEvent::FormsAnswerAll },
        Binding { combo: parse(&cfg.abort)?, make_event: || InputEvent::Abort },
        Binding { combo: parse(&cfg.hide_toggle)?, make_event: || InputEvent::HideToggle },
        Binding { combo: parse(&cfg.help_toggle)?, make_event: || InputEvent::HelpToggle },
        Binding { combo: parse(&cfg.restart_daemon)?, make_event: || InputEvent::RestartDaemon },
        // insta_delete is matched separately below; it drives the double-tap state machine, not
        // a direct one-shot event.
        Binding { combo: parse(&cfg.panic_kill)?, make_event: || InputEvent::PanicKill },
        Binding { combo: parse(&cfg.debug_open_tab)?, make_event: || InputEvent::DebugOpenTab },
    ])
}

pub fn insta_delete_combo(cfg: &HotkeysConfig) -> anyhow::Result<KeyCombo> {
    parse(&cfg.insta_delete)
}
