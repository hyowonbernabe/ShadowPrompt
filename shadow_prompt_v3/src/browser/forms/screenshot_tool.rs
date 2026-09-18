// The `screenshot(region?)` fallback tool for Forms — design doc §7.3: available inside the
// Forms flow itself (not just the standalone Screenshot Query hotkey), for anything the
// accessibility-tree read can't resolve cleanly (a picture-based question, an unusual custom
// widget). Reuses the exact same real capture pipeline Screenshot Query already uses
// (`capture::screen::capture_region` + `capture::image::prepare_for_request` —
// `actions/screenshot_query.rs` is the reference for this pattern) rather than inventing a
// second one.
//
// Feeds back into the model's own tool-call context for that turn, never the shared overlay
// (`ui::commands::UICommand`) — this tool never sends a `UICommand`, unlike Screenshot Query's
// own action handler, which does.
//
// ## Honest limitation, not glossed over
//
// This codebase's OpenAI-compatible wire format (`llm::messages::Message::Tool`) carries only a
// plain `String` for a tool result — there is no multimodal/image content-part slot on a
// tool-role message, only on a `User` message (see `messages.rs`'s `Message` enum: `Tool` has
// `content: String`, `User`/`System` have `content: Vec<ContentPart>`). So this tool cannot
// literally attach an image the way `build_page_message` attaches one to the *initial* page
// content — doing that for real would mean changing the shared `Message`/`run_turn` wire types
// in `llm/mod.rs`, which is out of this milestone's owned-file scope (`browser/forms/mod.rs` and
// `browser/forms/tab_lifecycle.rs` only) and, as observed directly while this milestone was
// being worked, is already under active concurrent change elsewhere for M11's streaming work —
// a second unrelated edit to that same shared loop right now would be exactly the kind of
// footprint this build plan's phasing rules warn against.
//
// What this tool does instead, matching the instruction to return the image "as the tool's
// text/structured result the way `fill_page`'s tool result already works": capture the real
// screen region, encode it as a `data:` PNG URL, and return that (plus width/height) as a
// structured JSON string. A model that only reads tool-role text sees the data URL as an opaque
// string, not actual pixels — genuine visual inspection through this specific channel is
// unverified, and, given the wire-format constraint above, likely not achievable without
// further work on `llm/mod.rs` itself. Flagged here rather than silently implied to work.

use serde::{Deserialize, Serialize};

use crate::capture::{image, screen};
use crate::llm::messages::{ContentPart, ToolDef, ToolFunctionDef};
use crate::llm::Tool;

const MAX_IMAGE_LONG_EDGE_PX: u32 = 1600;

pub struct ScreenshotTool;

#[derive(Deserialize, Default)]
struct Args {
    x: Option<i32>,
    y: Option<i32>,
    width: Option<i32>,
    height: Option<i32>,
}

#[derive(Serialize)]
struct ScreenshotResult {
    width: i32,
    height: i32,
    data_url: String,
    note: &'static str,
}

#[async_trait::async_trait]
impl Tool for ScreenshotTool {
    fn name(&self) -> &'static str {
        "screenshot"
    }

    fn definition(&self) -> ToolDef {
        ToolDef {
            kind: "function",
            function: ToolFunctionDef {
                name: self.name().to_string(),
                description: "Capture a screenshot of the current screen — a specific region \
                    (x, y, width, height, in screen pixels) if given, otherwise the whole \
                    primary screen. Use this only for anything the page's structural field read \
                    can't resolve cleanly: a picture-based question, or an unusual custom widget."
                    .to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "x": {"type": "integer", "description": "Region left, in screen pixels."},
                        "y": {"type": "integer", "description": "Region top, in screen pixels."},
                        "width": {"type": "integer", "description": "Region width, in pixels."},
                        "height": {"type": "integer", "description": "Region height, in pixels."}
                    }
                }),
            },
        }
    }

    async fn execute(&self, args_json: &str) -> anyhow::Result<String> {
        let args: Args = if args_json.trim().is_empty() {
            Args::default()
        } else {
            serde_json::from_str(args_json).unwrap_or_default()
        };

        let (x, y, w, h) = region_or_full_screen(args.x, args.y, args.width, args.height, primary_screen_size);

        let png = tokio::task::spawn_blocking(move || screen::capture_region(x, y, w, h)).await??;
        let content_part =
            tokio::task::spawn_blocking(move || image::prepare_for_request(&png, MAX_IMAGE_LONG_EDGE_PX)).await??;

        let data_url = match content_part {
            ContentPart::Image { image_url } => image_url.url,
            ContentPart::Text { .. } => anyhow::bail!("prepare_for_request unexpectedly returned a text part"),
        };

        let result = ScreenshotResult {
            width: w,
            height: h,
            data_url,
            note: "Captured as a data: PNG URL. This tool's result is plain text, so this URL \
                cannot be rendered back to you as an actual image through this channel in the \
                current build — treat this as a best-effort fallback, not a substitute for the \
                page's own structural field read.",
        };
        Ok(serde_json::to_string(&result)?)
    }
}

/// Resolves the requested capture region, defaulting to the whole primary screen whenever the
/// caller didn't give a complete, positive-size (x, y, width, height) — matching
/// `screenshot(region?)`'s optional-region design (design doc §7.3). `full_screen` is injected
/// (rather than calling `GetSystemMetrics` directly here) so this pure branching logic has a
/// real unit test that doesn't depend on a real Win32 display session being available (CI runs
/// headless — a real `GetSystemMetrics` call there is a separate, environment-dependent
/// concern, not part of what this function itself decides).
fn region_or_full_screen(
    x: Option<i32>,
    y: Option<i32>,
    width: Option<i32>,
    height: Option<i32>,
    full_screen: impl Fn() -> (i32, i32),
) -> (i32, i32, i32, i32) {
    match (x, y, width, height) {
        (Some(x), Some(y), Some(w), Some(h)) if w > 0 && h > 0 => (x, y, w, h),
        _ => {
            let (sw, sh) = full_screen();
            (0, 0, sw, sh)
        }
    }
}

fn primary_screen_size() -> (i32, i32) {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    // `unsafe` confined to this one Win32 metrics call, per project convention — the same API
    // `ui/manager.rs` already calls the same way elsewhere in this codebase.
    unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_or_full_screen_uses_given_region_when_complete_and_positive() {
        let got = region_or_full_screen(Some(10), Some(20), Some(100), Some(50), || panic!("must not be called"));
        assert_eq!(got, (10, 20, 100, 50));
    }

    #[test]
    fn region_or_full_screen_falls_back_when_any_part_missing() {
        let got = region_or_full_screen(Some(10), None, Some(100), Some(50), || (1920, 1080));
        assert_eq!(got, (0, 0, 1920, 1080));
    }

    #[test]
    fn region_or_full_screen_falls_back_when_dimensions_non_positive() {
        let got = region_or_full_screen(Some(10), Some(20), Some(0), Some(50), || (1920, 1080));
        assert_eq!(got, (0, 0, 1920, 1080));
    }
}
