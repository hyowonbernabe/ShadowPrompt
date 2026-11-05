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
        Binding { combo: parse(&cfg.forms_answer_page_axtree)?, make_event: || InputEvent::FormsAnswerPageAxtree },
        Binding { combo: parse(&cfg.forms_answer_all_axtree)?, make_event: || InputEvent::FormsAnswerAllAxtree },
        Binding { combo: parse(&cfg.abort)?, make_event: || InputEvent::Abort },
        Binding { combo: parse(&cfg.hide_toggle)?, make_event: || InputEvent::HideToggle },
        Binding { combo: parse(&cfg.help_toggle)?, make_event: || InputEvent::HelpToggle },
        Binding { combo: parse(&cfg.restart_daemon)?, make_event: || InputEvent::RestartDaemon },
        // insta_delete is matched separately below; it drives the double-tap state machine, not
        // a direct one-shot event.
        Binding { combo: parse(&cfg.panic_kill)?, make_event: || InputEvent::PanicKill },
        Binding { combo: parse(&cfg.debug_open_tab)?, make_event: || InputEvent::DebugOpenTab },
        // Newer bindings go last: the listener fires the first match, so if a user's existing
        // config already maps an older action to one of these combos, their mapping keeps
        // working (and `duplicate_warnings` logs the clash).
        Binding { combo: parse(&cfg.add_context)?, make_event: || InputEvent::AddContext },
        Binding { combo: parse(&cfg.reset_conversation)?, make_event: || InputEvent::ResetConversation },
        Binding { combo: parse(&cfg.screen_answer)?, make_event: || InputEvent::ScreenAnswer },
        Binding { combo: parse(&cfg.clear_dots)?, make_event: || InputEvent::ClearDots },
    ])
}

/// One warning per pair of actions bound to the same combo (insta_delete included). Logged,
/// not shown — a visible prompt would break stealth.
pub fn duplicate_warnings(bindings: &[Binding], insta: &KeyCombo) -> Vec<String> {
    let mut out = Vec::new();
    for (i, a) in bindings.iter().enumerate() {
        if a.combo == *insta {
            out.push(format!("{:?} shares its hotkey with insta_delete; insta_delete wins", (a.make_event)()));
        }
        for b in &bindings[i + 1..] {
            if a.combo == b.combo {
                out.push(format!(
                    "{:?} and {:?} share a hotkey; only {:?} will fire",
                    (a.make_event)(),
                    (b.make_event)(),
                    (a.make_event)()
                ));
            }
        }
    }
    out
}

pub fn insta_delete_combo(cfg: &HotkeysConfig) -> anyhow::Result<KeyCombo> {
    parse(&cfg.insta_delete)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_hotkeys_have_no_duplicates() {
        let cfg: crate::config::schema::Config =
            toml::from_str(include_str!("../../config/config.example.toml")).expect("example config parses");
        let bindings = build(&cfg.hotkeys).unwrap();
        let insta = insta_delete_combo(&cfg.hotkeys).unwrap();
        assert_eq!(duplicate_warnings(&bindings, &insta), Vec::<String>::new());
    }

    #[test]
    fn clash_is_reported_and_older_binding_wins() {
        let mut cfg: crate::config::schema::Config =
            toml::from_str(include_str!("../../config/config.example.toml")).unwrap();
        cfg.hotkeys.abort = cfg.hotkeys.add_context.clone();
        let bindings = build(&cfg.hotkeys).unwrap();
        let insta = insta_delete_combo(&cfg.hotkeys).unwrap();
        let w = duplicate_warnings(&bindings, &insta);
        assert_eq!(w.len(), 1);
        assert!(w[0].starts_with("Abort and AddContext"), "{}", w[0]);
    }
}
