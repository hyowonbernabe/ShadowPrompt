// Capability probe: out-of-daemon round-trips against the configured model chain, printing
// pass/fail per category. Invoked via `--probe`. Design doc §12 / build plan M12 — a lighter,
// automated complement to the live `test_model` hotkey, updated for v3's 5-model fallback chain
// and tool-calling loop (v2's probe checked a single configured model; this checks whichever
// model in the chain actually answers, surfaced via the ground-truth `model` field, same as
// `test_model` does).

use std::time::Instant;

use base64::Engine;
use image::{ImageBuffer, Rgb};

use crate::config::Config;
use crate::llm::messages::{ContentPart, ImageUrl};
use crate::llm::LlmClient;

pub async fn run(cfg: Config) -> anyhow::Result<()> {
    println!("=== ShadowPrompt v3 model probe ===");
    println!("configured fallback chain:");
    for (i, m) in cfg.openrouter.models.iter().enumerate() {
        println!("  {}. {m}", i + 1);
    }
    println!();

    let llm = LlmClient::new(
        cfg.openrouter.api_key.clone(),
        cfg.openrouter.models.clone(),
        cfg.http.connect_timeout_secs,
        cfg.http.read_timeout_secs,
    )?;

    probe_text(&llm).await;
    probe_vision(&llm).await;
    probe_tools_and_search(&llm).await;

    Ok(())
}

async fn probe_text(llm: &LlmClient) {
    section("TEXT");
    let t = Instant::now();
    match llm
        .simple_call("Reply with exactly one word.", "What is the capital of France?")
        .await
    {
        Ok(r) => {
            let ok = r.answer.to_lowercase().contains("paris");
            verdict(ok, &format!("'{}' via {} ({}ms)", r.answer.trim(), r.model, t.elapsed().as_millis()));
        }
        Err(e) => verdict(false, &format!("{e}")),
    }
}

async fn probe_vision(llm: &LlmClient) {
    section("VISION");
    let url = match red_square_data_url() {
        Ok(u) => u,
        Err(e) => {
            verdict(false, &format!("could not build probe image: {e}"));
            return;
        }
    };
    let content = vec![
        ContentPart::text("What single color fills this image? Reply with exactly one word."),
        ContentPart::Image { image_url: ImageUrl { url } },
    ];
    let t = Instant::now();
    // Empty tool set, no delta callback — this is a diagnostic round trip, not a real turn.
    match llm.run_turn("Reply with exactly one color name.", content, &[], true, None).await {
        Ok(r) => {
            let ok = r.to_lowercase().contains("red");
            verdict(ok, &format!("'{}' ({}ms)", r.trim(), t.elapsed().as_millis()));
        }
        Err(e) => verdict(false, &format!("{e}")),
    }
}

/// Design doc §5/§8: `openrouter:web_search` and every function tool sit in the same `tools`
/// array on every real turn already, unconditionally (see `LlmClient::run_turn`) — there's no
/// separate "is search enabled" config check the way v2's `:online`-suffix probe had, since v3
/// doesn't gate search behind a suffix at all. What's actually worth probing here is whether the
/// model chain round-trips *with* tools attached without erroring (a bad tool schema or a model
/// that rejects the request shape would surface here), not whether search specifically fires —
/// that's inherently up to the model's own judgment per turn, not something a fixed probe
/// question can force deterministically.
async fn probe_tools_and_search(llm: &LlmClient) {
    section("TOOLS (schema round-trip; search/tool-use itself is the model's own judgment call, not forced here)");
    let t = Instant::now();
    let content = vec![ContentPart::text("Reply with exactly one word: \"ok\".")];
    match llm.run_turn("Reply with exactly one word.", content, &[], false, None).await {
        Ok(r) => verdict(true, &format!("'{}' ({}ms) — request with the full tool schema attached succeeded", r.trim(), t.elapsed().as_millis())),
        Err(e) => verdict(false, &format!("{e}")),
    }
}

/// 256x256 solid red, RGB (no alpha) — some Anthropic-via-Bedrock/Vertex routes reject RGBA or
/// sub-200px inputs (v2's own probe note, still true here since the underlying providers
/// haven't changed).
fn red_square_data_url() -> anyhow::Result<String> {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(256, 256, Rgb([255, 0, 0]));
    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    Ok(format!("data:image/png;base64,{b64}"))
}

fn section(name: &str) {
    println!("[{name}]");
}

fn verdict(ok: bool, detail: &str) {
    let mark = if ok { "PASS" } else { "FAIL" };
    println!("  {mark}: {detail}");
    println!();
}
