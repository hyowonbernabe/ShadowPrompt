// Screenshot Query — design doc §3. Drag-select region -> vision query. No OCR anywhere in
// this path; the image goes straight to the model.

use super::{deliver_answer, report_failure, ActionContext, RestoreOverlaysOnDrop};
use super::clipboard_query::{ask_in_conversation, memory_tools};
use crate::capture::{image, screen};
use crate::llm::messages::ContentPart;
use crate::ui::commands::{IndicatorState, UICommand};

const MAX_IMAGE_LONG_EDGE_PX: u32 = 1600;

/// Gives the UI thread a moment to actually process `HideOverlaysForCapture` (a fire-and-forget
/// channel send, no ack) before the region capture reads the screen. Belt-and-suspenders on top
/// of `WDA_EXCLUDEFROMCAPTURE` (design doc §6) — this guarantees our *own* capture never bleeds
/// a leftover answer/help panel from a previous query into a brand new one, regardless of
/// whether the OS-level exclusion is actually honored by whatever's reading the screen.
const OVERLAY_HIDE_SETTLE: std::time::Duration = std::time::Duration::from_millis(60);

/// Captures a screen region (with our own overlays hidden) and encodes it as a request image
/// part. Shared by Screenshot Query and add-context.
pub async fn capture_image_part(ctx: &ActionContext, x: i32, y: i32, w: i32, h: i32) -> anyhow::Result<ContentPart> {
    let _ = ctx.ui_tx.send(UICommand::HideOverlaysForCapture);
    let restore = RestoreOverlaysOnDrop(ctx.ui_tx.clone());
    tokio::time::sleep(OVERLAY_HIDE_SETTLE).await;
    let capture_result = tokio::task::spawn_blocking(move || screen::capture_region(x, y, w, h)).await;
    drop(restore);
    let png = capture_result?.map_err(|e| anyhow::anyhow!("screen capture: {e}"))?;
    tokio::task::spawn_blocking(move || image::prepare_for_request(&png, MAX_IMAGE_LONG_EDGE_PX)).await?
}

pub async fn execute(ctx: ActionContext, x: i32, y: i32, w: i32, h: i32) -> anyhow::Result<()> {
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Processing));

    let result = async {
        let image_part = capture_image_part(&ctx, x, y, w, h).await?;
        let content = vec![ContentPart::text("Answer the question(s) visible in this screenshot."), image_part];
        ask_in_conversation(&ctx, content, true, &memory_tools(&ctx)).await
    }
    .await;
    match result {
        Ok(answer) => {
            deliver_answer(&ctx, answer).await;
            Ok(())
        }
        Err(e) => {
            report_failure(&ctx, &e);
            Err(e)
        }
    }
}
