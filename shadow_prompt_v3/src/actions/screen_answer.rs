// Screen Answer: one hotkey -> capture every monitor -> Claude answers what's on screen and calls
// `mark` on each click/fill target -> tiny pulsing dots appear there and follow the page as it
// scrolls (capture::track), for up to TRACK_LIFETIME.
//
// Runs in its own task slot (`ActionContext::screen_task`): a new press cancels the old job and
// its dots; the same task stays alive as the tracking loop after the answer, so `abort` /
// `clear_dots` stopping the slot stops tracking too.
//
// OpenRouter + Claude only, on its own model list (`[screen_answer].models`) — not the main
// fallback chain, and `switch_model` doesn't touch it. Screenshots go in the first message, not
// via a screenshot tool: saves a round-trip, and tool results are text-only on this wire format.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::clipboard_query::{knowledge_context, memory_tools};
use super::{busy, deliver_answer, report_failure, ActionContext, RestoreOverlaysOnDrop};
use crate::capture::screen::{self, MonitorShot};
use crate::capture::track::{self, Gray, COARSE_FACTOR};
use crate::capture::image;
use crate::llm::messages::{ContentPart, ToolDef, ToolFunctionDef};
use crate::llm::system_prompts::BASE;
use crate::llm::{LlmClient, Tool};
use crate::ui::commands::{IndicatorState, UICommand};

/// Anthropic downsizes images past ~1568px long edge / ~1.15MP itself, which would shift the
/// coordinates the model reports. Staying under both keeps image pixels == model pixels.
const MAX_LONG_EDGE: f64 = 1568.0;
const MAX_PIXELS: f64 = 1_150_000.0;
const TRACK_INTERVAL: Duration = Duration::from_millis(500);
/// Tracking (and its captures) stop after this; dots disappear with it.
const TRACK_LIFETIME: Duration = Duration::from_secs(600);
/// Same settle as Screenshot Query: let the UI thread actually hide our overlays before capture.
const OVERLAY_HIDE_SETTLE: Duration = Duration::from_millis(60);

const DELIVERY: &str = "DELIVERY (screen answer mode)
You are shown one screenshot per monitor of the user's screen, labelled with its monitor number and image size. Find the question(s) visible on screen and answer them. Check every monitor: questions can be on any of them, and answer the ones on every monitor, not just the first.

For every control the user should click or fill, call the `mark` tool once: the monitor number and the x,y pixel position (in that monitor's image) of the exact spot to click — the centre of the radio button, checkbox, grid cell, or the input box. Also give anchor_x, anchor_y: a point on nearby text that will NOT change when the user clicks or types (the option's label text for a choice; the question text just above an input box). Multiple correct options, checkbox sets, and grid/matrix rows each get their own mark. Be precise; a wrong mark is worse than none.

After marking, reply with the short answer text per the ANSWER FORMAT rules (for typed answers, the text to type). Do not mark anything for a question you are not answering.

If you cannot answer — the screen is black or blank (some apps block screenshots), no question is visible, or the question is cut off — do not call `mark`. Instead reply in one short plain sentence saying what you see and why you can't answer, e.g. \"I can't see a question, the screen is black (this app likely blocks screenshots).\" This overrides the never-abstain rule above.";

#[derive(Debug, Clone, Copy, PartialEq)]
struct Mark {
    /// 0-based index into the captured monitors.
    monitor: usize,
    target: (f64, f64),
    anchor: (f64, f64),
}

struct MarkTool {
    /// Size of each image actually sent, so marks can be bounds-checked.
    image_sizes: Vec<(u32, u32)>,
    marks: Arc<Mutex<Vec<Mark>>>,
}

#[derive(serde::Deserialize)]
struct MarkArgs {
    monitor: usize,
    x: f64,
    y: f64,
    anchor_x: Option<f64>,
    anchor_y: Option<f64>,
}

impl MarkTool {
    fn parse(&self, args_json: &str) -> anyhow::Result<Mark> {
        let a: MarkArgs = serde_json::from_str(args_json)?;
        let idx = a.monitor.checked_sub(1).filter(|i| *i < self.image_sizes.len());
        let Some(monitor) = idx else {
            anyhow::bail!("monitor must be 1..={}", self.image_sizes.len());
        };
        let (w, h) = self.image_sizes[monitor];
        let inside = |x: f64, y: f64| x >= 0.0 && y >= 0.0 && x < w as f64 && y < h as f64;
        let anchor = (a.anchor_x.unwrap_or(a.x), a.anchor_y.unwrap_or(a.y));
        if !inside(a.x, a.y) || !inside(anchor.0, anchor.1) {
            anyhow::bail!("point outside monitor {}'s {w}x{h} image", a.monitor);
        }
        Ok(Mark { monitor, target: (a.x, a.y), anchor })
    }
}

#[async_trait::async_trait]
impl Tool for MarkTool {
    fn name(&self) -> &'static str {
        "mark"
    }

    fn definition(&self) -> ToolDef {
        ToolDef {
            kind: "function",
            function: ToolFunctionDef {
                name: self.name().to_string(),
                description: "Put a small dot on the user's screen at a spot they should click \
                    or fill in. Call once per spot. Coordinates are pixels in the given \
                    monitor's screenshot."
                    .to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "monitor": {"type": "integer", "description": "Monitor number as labelled (1-based)."},
                        "x": {"type": "integer", "description": "Click target x, image pixels."},
                        "y": {"type": "integer", "description": "Click target y, image pixels."},
                        "anchor_x": {"type": "integer", "description": "x of nearby text that won't change when clicked/typed."},
                        "anchor_y": {"type": "integer", "description": "y of that text."}
                    },
                    "required": ["monitor", "x", "y", "anchor_x", "anchor_y"]
                }),
            },
        }
    }

    async fn execute(&self, args_json: &str) -> anyhow::Result<String> {
        let m = self.parse(args_json)?;
        self.marks.lock().unwrap().push(m);
        Ok("marked".to_string())
    }
}

/// Scale factor (<= 1) that fits a monitor under both Anthropic image limits.
fn image_scale(w: u32, h: u32) -> f64 {
    let (w, h) = (w as f64, h as f64);
    (MAX_LONG_EDGE / w.max(h)).min((MAX_PIXELS / (w * h)).sqrt()).min(1.0)
}

/// Image-space point -> monitor-local physical pixel.
fn to_monitor(p: (f64, f64), scale: f64) -> (i32, i32) {
    ((p.0 / scale).round() as i32, (p.1 / scale).round() as i32)
}

struct Tracked {
    monitor: usize,
    anchor: track::Anchor,
}

/// Clears the dots when dropped: every exit of `execute` (finished, error, abort) leaves no
/// frozen dot behind on a page that may have scrolled.
struct ClearDotsOnDrop(tokio::sync::mpsc::UnboundedSender<UICommand>);

impl Drop for ClearDotsOnDrop {
    fn drop(&mut self) {
        let _ = self.0.send(UICommand::SetDots(Vec::new()));
    }
}

pub async fn execute(ctx: ActionContext) -> anyhow::Result<()> {
    let _ = ctx.ui_tx.send(UICommand::SetDots(Vec::new()));
    let _clear = ClearDotsOnDrop(ctx.ui_tx.clone());
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Processing));
    let result = run(&ctx).await;
    if let Err(e) = &result {
        report_failure(&ctx, e);
    }
    result
}

async fn run(ctx: &ActionContext) -> anyhow::Result<()> {
    let _ = ctx.ui_tx.send(UICommand::HideOverlaysForCapture);
    let restore = RestoreOverlaysOnDrop(ctx.ui_tx.clone());
    tokio::time::sleep(OVERLAY_HIDE_SETTLE).await;
    let shots = tokio::task::spawn_blocking(screen::capture_monitors).await;
    drop(restore);
    let shots = Arc::new(shots?.map_err(|e| anyhow::anyhow!("screen capture: {e}"))?);
    if shots.is_empty() {
        anyhow::bail!("screen_answer: no monitors found");
    }

    let shots_for_encode = shots.clone();
    let (content, scales, image_sizes) = tokio::task::spawn_blocking(move || build_content(&shots_for_encode)).await??;

    let marks = Arc::new(Mutex::new(Vec::new()));
    let mut tools = memory_tools(ctx);
    tools.push(Arc::new(MarkTool { image_sizes, marks: marks.clone() }));

    let cfg = &ctx.config;
    let llm = LlmClient::new(
        cfg.openrouter.api_key.clone(),
        cfg.screen_answer.models.clone(),
        cfg.http.connect_timeout_secs,
        cfg.http.read_timeout_secs,
    )?;
    let prompt = format!("{BASE}\n\n{DELIVERY}{}", knowledge_context(ctx).await);
    let answer = {
        let _busy = busy(ctx);
        llm.run_turn(&prompt, content, &tools, true, None).await?
    };

    deliver_answer(ctx, answer).await;

    let mut marks: Vec<Mark> = marks.lock().unwrap().clone();
    // Only MAX_DOTS can be drawn; tracking more just burns CPU every tick.
    marks.truncate(crate::ui::manager::MAX_DOTS);
    if marks.is_empty() {
        return Ok(());
    }
    let threshold = cfg.screen_answer.match_threshold;
    let t0 = Instant::now();
    let mut tracked = tokio::task::spawn_blocking(move || make_tracked(&shots, &scales, &marks)).await?;
    log::debug!("screen_answer: {} anchor(s) ready in {:?}", tracked.len(), t0.elapsed());
    if tracked.is_empty() {
        log::warn!("screen_answer: every mark landed on a blank patch; nothing to track");
        return Ok(());
    }

    let started = Instant::now();
    while started.elapsed() < TRACK_LIFETIME {
        let tick = Instant::now();
        let (pts, back) = tokio::task::spawn_blocking(move || {
            let pts = screen::capture_monitors().map(|s| locate_all(&mut tracked, &s, threshold));
            (pts, tracked)
        })
        .await?;
        tracked = back;
        if tick.elapsed() > Duration::from_secs(1) {
            log::warn!("screen_answer: tracking tick took {:?}", tick.elapsed());
        }
        match pts {
            Ok(p) => {
                let _ = ctx.ui_tx.send(UICommand::SetDots(p));
            }
            Err(e) => log::warn!("screen_answer: tracking capture failed: {e}"),
        }
        tokio::time::sleep(TRACK_INTERVAL).await;
    }
    Ok(())
}

type Encoded = (Vec<ContentPart>, Vec<f64>, Vec<(u32, u32)>);

fn build_content(shots: &[MonitorShot]) -> anyhow::Result<Encoded> {
    let mut content = vec![ContentPart::text(format!(
        "{} monitor(s). Answer the question(s) on screen and mark where to click/fill.",
        shots.len()
    ))];
    let (mut scales, mut sizes) = (Vec::new(), Vec::new());
    for (i, s) in shots.iter().enumerate() {
        let scale = image_scale(s.width as u32, s.height as u32);
        let (iw, ih) = ((s.width as f64 * scale).round() as u32, (s.height as f64 * scale).round() as u32);
        let png = image::encode_png(s.width as u32, s.height as u32, &s.rgba)?;
        content.push(ContentPart::text(format!("Monitor {}: image {iw}x{ih} px.", i + 1)));
        content.push(image::prepare_for_request(&png, iw.max(ih))?);
        scales.push(scale);
        sizes.push((iw, ih));
    }
    Ok((content, scales, sizes))
}

fn grays(s: &MonitorShot) -> (Gray, Gray) {
    let (w, h) = (s.width as usize, s.height as usize);
    (Gray::from_rgba(w, h, &s.rgba, 1), Gray::from_rgba(w, h, &s.rgba, COARSE_FACTOR))
}

fn make_tracked(shots: &[MonitorShot], scales: &[f64], marks: &[Mark]) -> Vec<Tracked> {
    let g: Vec<(Gray, Gray)> = shots.iter().map(grays).collect();
    marks
        .iter()
        .filter_map(|m| {
            let (full, coarse) = &g[m.monitor];
            let anchor = track::make_anchor(full, coarse, to_monitor(m.anchor, scales[m.monitor]), to_monitor(m.target, scales[m.monitor]))?;
            Some(Tracked { monitor: m.monitor, anchor })
        })
        .collect()
}

/// Dot positions in virtual-screen coordinates, for every anchor confidently found right now.
/// A monitor that disappeared (unplugged) just yields no dots for its marks.
fn locate_all(tracked: &mut [Tracked], shots: &[MonitorShot], threshold: f32) -> Vec<(i32, i32)> {
    // Grayscale only the monitors that actually have dots — on a 4K/8K second screen without
    // dots this conversion was most of the per-tick cost.
    let g: Vec<Option<(Gray, Gray)>> = shots
        .iter()
        .enumerate()
        .map(|(i, s)| tracked.iter().any(|t| t.monitor == i).then(|| grays(s)))
        .collect();
    tracked
        .iter_mut()
        .filter_map(|t| {
            let (full, coarse) = g.get(t.monitor)?.as_ref()?;
            let (x, y) = track::locate(&mut t.anchor, full, coarse, threshold)?;
            let s = &shots[t.monitor];
            Some((s.left + x, s.top + y))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool() -> MarkTool {
        MarkTool { image_sizes: vec![(1456, 819), (1000, 800)], marks: Arc::new(Mutex::new(Vec::new())) }
    }

    #[test]
    fn scale_respects_both_limits() {
        let s = image_scale(2560, 1440);
        assert!(2560.0 * s <= MAX_LONG_EDGE + 0.5);
        assert!(2560.0 * s * 1440.0 * s <= MAX_PIXELS + 1.0);
        assert_eq!(image_scale(800, 600), 1.0);
    }

    #[test]
    fn image_point_maps_back_to_physical_pixels() {
        let s = image_scale(2560, 1440);
        let img = (1280.0 * s, 720.0 * s);
        assert_eq!(to_monitor(img, s), (1280, 720));
    }

    #[test]
    fn mark_is_one_based_and_bounds_checked() {
        let t = tool();
        let m = t.parse(r#"{"monitor":2,"x":10,"y":20,"anchor_x":30,"anchor_y":20}"#).unwrap();
        assert_eq!(m, Mark { monitor: 1, target: (10.0, 20.0), anchor: (30.0, 20.0) });
        assert!(t.parse(r#"{"monitor":0,"x":1,"y":1}"#).is_err());
        assert!(t.parse(r#"{"monitor":3,"x":1,"y":1}"#).is_err());
        assert!(t.parse(r#"{"monitor":2,"x":1000,"y":1}"#).is_err());
        assert!(t.parse(r#"{"monitor":1,"x":5,"y":5,"anchor_x":-1,"anchor_y":5}"#).is_err());
    }

    #[test]
    fn missing_anchor_defaults_to_target() {
        let m = tool().parse(r#"{"monitor":1,"x":5,"y":6}"#).unwrap();
        assert_eq!(m.anchor, (5.0, 6.0));
    }
}
