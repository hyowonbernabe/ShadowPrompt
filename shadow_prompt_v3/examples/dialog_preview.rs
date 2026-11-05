// Shows the login / account dialog on its own, for working on its look.
//   cargo run --example dialog_preview --features debug -- launch logged-in has-key
//   cargo run --example dialog_preview --features debug -- account logged-in
// Set SHADOWPROMPT_CAPTURABLE=1 to make it visible to screenshots. Doesn't start the app, so it
// never closes a running ShadowPrompt and never touches hotkeys or the clipboard.

fn main() {
    // Same DPI mode as the app, so sizes match what users see.
    unsafe {
        let _ = windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
            windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        );
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |a: &str| args.iter().any(|x| x == a);
    shadowprompt::ui::account_dialog::preview(has("account"), has("logged-in"), has("has-key"));
}
