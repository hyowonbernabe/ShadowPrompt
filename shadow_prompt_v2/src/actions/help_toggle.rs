// Help-toggle action: builds a static hotkey cheat sheet from the current
// config and toggles its display in the bottom-right overlay.

use std::sync::Mutex;

use super::ActionContext;
use crate::ui::UICommand;

static VISIBLE: Mutex<bool> = Mutex::new(false);

pub async fn execute(ctx: ActionContext) {
    let mut v = VISIBLE.lock().unwrap();
    if *v {
        let _ = ctx.ui_tx.send(UICommand::HideHelp);
        *v = false;
    } else {
        let text = build_help_text(&ctx);
        let _ = ctx.ui_tx.send(UICommand::ShowHelp(text));
        *v = true;
    }
}

fn build_help_text(ctx: &ActionContext) -> String {
    let h = &ctx.config.hotkeys;
    let rows: [(&str, &str); 13] = [
        (&h.clipboard_query,        "Answer (clipboard)"),
        (&h.clipboard_query_search, "Answer + web search"),
        (&h.ocr_query,              "Answer (screen region)"),
        (&h.ocr_query_search,       "Vision + web search"),
        (&h.forms_auto,             "Forms: auto-paginate"),
        (&h.forms_single,           "Forms: single page"),
        (&h.launch_debugger,        "Launch Chrome on :9222"),
        (&h.abort,                  "Abort current task"),
        (&h.hide_toggle,            "Hide / show UI"),
        (&h.help_toggle,            "Toggle this help"),
        (&h.restart_daemon,         "Restart daemon"),
        (&h.insta_delete,           "Insta-delete (tap twice)"),
        (&h.panic_kill,             "Panic: wipe clipboard + exit"),
    ];

    let max_key_len = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let mut out = String::new();
    for (k, label) in rows {
        out.push_str(&format!("{:<width$}  {}\n", k, label, width = max_key_len));
    }
    out.pop(); // trailing newline
    out
}
