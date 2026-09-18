// Screenshot Query — design doc §3. Drag-select region -> vision query. No OCR anywhere in
// this path; the image goes straight to the model.

use super::ActionContext;
use super::clipboard_query::memory_tools;
use crate::capture::{clipboard, image, screen};
use crate::llm::messages::ContentPart;
use crate::llm::system_prompts::general_prompt;
use crate::ui::commands::{IndicatorState, UICommand};

const MAX_IMAGE_LONG_EDGE_PX: u32 = 1600;

/// Gives the UI thread a moment to actually process `HideOverlaysForCapture` (a fire-and-forget
/// channel send, no ack) before the region capture reads the screen. Belt-and-suspenders on top
/// of `WDA_EXCLUDEFROMCAPTURE` (design doc §6) — this guarantees our *own* capture never bleeds
/// a leftover answer/help panel from a previous query into a brand new one, regardless of
/// whether the OS-level exclusion is actually honored by whatever's reading the screen.
const OVERLAY_HIDE_SETTLE: std::time::Duration = std::time::Duration::from_millis(60);

pub async fn execute(ctx: ActionContext, x: i32, y: i32, w: i32, h: i32) -> anyhow::Result<()> {
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Processing));

    let _ = ctx.ui_tx.send(UICommand::HideOverlaysForCapture);
    tokio::time::sleep(OVERLAY_HIDE_SETTLE).await;
    let capture_result = tokio::task::spawn_blocking(move || screen::capture_region(x, y, w, h)).await;
    let _ = ctx.ui_tx.send(UICommand::RestoreOverlaysAfterCapture);
    let png = capture_result??;
    let image_part = tokio::task::spawn_blocking(move || image::prepare_for_request(&png, MAX_IMAGE_LONG_EDGE_PX)).await??;

    let content = vec![ContentPart::text("Answer the question(s) visible in this screenshot."), image_part];
    let tools = memory_tools(&ctx);

    let call = ctx.llm.run_turn(&general_prompt(), content, &tools, true, None).await;
    let answer = match call {
        Ok(a) => a,
        Err(e) => {
            let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Error));
            return Err(e);
        }
    };

    let answer_for_write = answer.clone();
    tokio::task::spawn_blocking(move || clipboard::write_text(&answer_for_write)).await??;
    let _ = ctx.ui_tx.send(UICommand::SetOverlayText(answer));
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Ready));
    Ok(())
}
