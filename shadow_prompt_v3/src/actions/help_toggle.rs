// help_toggle — auto-generated hotkey cheat sheet from live config. Ported from v2, updated for
// the final 13-hotkey list (design doc §10).

use std::sync::Mutex;

use super::ActionContext;
use crate::ui::UICommand;

static VISIBLE: Mutex<bool> = Mutex::new(false);

pub async fn execute(ctx: &ActionContext) {
    let mut v = VISIBLE.lock().unwrap();
    if *v {
        let _ = ctx.ui_tx.send(UICommand::HideHelp);
        *v = false;
    } else {
        let text = build_help_text(ctx);
        let _ = ctx.ui_tx.send(UICommand::ShowHelp(text));
        *v = true;
    }
}

fn build_help_text(ctx: &ActionContext) -> String {
    let h = &ctx.config.hotkeys;
    let rows: [(&str, &str); 13] = [
        (&h.clipboard_query, "Answer (clipboard)"),
        (&h.screenshot_query, "Answer (screen region)"),
        (&h.test_model, "Test which model is answering"),
        (&h.switch_model, "Switch primary paid model"),
        (&h.launch_debugger, "Launch Chrome on :9222"),
        (&h.forms_answer_page, "Forms: answer this page"),
        (&h.forms_answer_all, "Forms: answer all pages"),
        (&h.abort, "Abort current task"),
        (&h.hide_toggle, "Hide / show UI"),
        (&h.help_toggle, "Toggle this help"),
        (&h.restart_daemon, "Restart daemon"),
        (&h.insta_delete, "Insta-delete (tap twice)"),
        (&h.panic_kill, "Panic: wipe clipboard + exit"),
    ];

    // Not a configurable binding — Escape always cancels an in-progress screenshot selection
    // (input/mod.rs), so it's worth listing even though it has no `HotkeysConfig` field.
    const FIXED_ESCAPE_ROW: (&str, &str) = ("esc", "Cancel screenshot selection (while dragging)");

    let mut all_rows: Vec<(&str, &str)> = rows.into_iter().collect();
    all_rows.push(FIXED_ESCAPE_ROW);
    // Dev-only test hotkey (actions::debug_open_tab) — does nothing in a production build, so it
    // doesn't belong in the cheat sheet a real build shows either.
    #[cfg(feature = "debug")]
    all_rows.push((&h.debug_open_tab, "[debug] Open new tab in debug Chrome"));

    let max_key_len = all_rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let mut out = String::new();
    for (k, label) in all_rows {
        out.push_str(&format!("{:<width$}  {}\n", k, label, width = max_key_len));
    }
    out.pop();
    out
}
