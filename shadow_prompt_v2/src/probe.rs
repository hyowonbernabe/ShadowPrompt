// Capability probe: run a few targeted round-trips against the configured
// model and print pass/fail per category. Invoked via `--probe`.

use std::time::Instant;

use crate::config::Config;
use crate::llm::capabilities;
use crate::llm::messages::{ContentPart, ImageUrl, Message};
use crate::llm::LlmClient;

/// Generate a 64x64 solid-red PNG at runtime. Some providers reject sub-32px
/// inputs, so we don't rely on a tiny hardcoded blob.
fn red_square_data_url() -> anyhow::Result<String> {
    use base64::Engine;
    use image::{ImageBuffer, Rgba};
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(64, 64, Rgba([255, 0, 0, 255]));
    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    Ok(format!("data:image/png;base64,{b64}"))
}

pub async fn run(cfg: Config) -> anyhow::Result<()> {
    let model = cfg.openrouter.model_id.clone();
    println!("=== ShadowPrompt model probe ===");
    println!("model: {model}");

    let caps = capabilities::for_model(&model);
    println!(
        "advertised capabilities: vision={} reasoning={} prompt_caching={} context_window={}",
        caps.vision, caps.reasoning, caps.prompt_caching, caps.context_window
    );
    println!();

    let llm = LlmClient::new(
        cfg.openrouter.api_key.clone(),
        model,
        cfg.http.connect_timeout_secs,
        cfg.http.read_timeout_secs,
    )?;

    probe_text(&llm).await;
    probe_vision(&llm).await;
    probe_recency(&llm).await;
    probe_reasoning(&llm).await;
    probe_search(&llm, &cfg.openrouter.model_id).await;

    Ok(())
}

async fn probe_text(llm: &LlmClient) {
    section("TEXT");
    let t = Instant::now();
    match llm
        .answer_text(
            "Reply with exactly one word.",
            "What is the capital of France?",
        )
        .await
    {
        Ok(r) => {
            let ok = r.to_lowercase().contains("paris");
            verdict(ok, &format!("'{}' ({}ms)", r.trim(), t.elapsed().as_millis()));
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
    let msgs = vec![
        Message::System {
            content: "Reply with exactly one color name.".to_string(),
        },
        Message::User {
            content: vec![
                ContentPart::Text {
                    text: "What single color fills this image?".to_string(),
                },
                ContentPart::Image {
                    image_url: ImageUrl { url },
                },
            ],
        },
    ];
    let t = Instant::now();
    match llm.call(msgs).await {
        Ok(r) => {
            let ok = r.to_lowercase().contains("red");
            verdict(ok, &format!("'{}' ({}ms)", r.trim(), t.elapsed().as_millis()));
        }
        Err(e) => verdict(false, &format!("{e}")),
    }
}

async fn probe_recency(llm: &LlmClient) {
    section("RECENCY (knowledge cutoff)");
    let t = Instant::now();
    match llm
        .answer_text(
            "Reply with a single year (4 digits) only.",
            "What is the most recent year you have reliable knowledge of?",
        )
        .await
    {
        Ok(r) => {
            verdict(true, &format!("'{}' ({}ms) — interpret manually", r.trim(), t.elapsed().as_millis()));
        }
        Err(e) => verdict(false, &format!("{e}")),
    }
}

async fn probe_reasoning(llm: &LlmClient) {
    section("REASONING (multi-step)");
    let t = Instant::now();
    match llm
        .answer_text(
            "Reply with one number only.",
            "A train leaves station A at 9:00 going 60 km/h. Another leaves station B (180 km east of A) at 9:30 going 40 km/h west. At what hour and minute do they meet? Answer as HH:MM (24-hour).",
        )
        .await
    {
        Ok(r) => {
            // Correct answer is 11:12 — but accept anything reasonable; user judges.
            verdict(true, &format!("'{}' (expected 11:00; {}ms)", r.trim(), t.elapsed().as_millis()));
        }
        Err(e) => verdict(false, &format!("{e}")),
    }
}

async fn probe_search(_llm: &LlmClient, model_id: &str) {
    section("WEB SEARCH");
    let online = model_id.ends_with(":online");
    if online {
        println!(
            "  model id ends with :online → OpenRouter Web Search plugin is active. \
             Recency answers will reflect live web results."
        );
        verdict(true, "enabled via :online suffix");
    } else {
        println!(
            "  model id does NOT end with :online → no live web search. \
             To enable, set model_id to '{model_id}:online' in config.toml. \
             Adds ~$4/1000 results to per-query cost."
        );
        verdict(false, "disabled (default)");
    }
}

fn section(name: &str) {
    println!("[{name}]");
}

fn verdict(ok: bool, detail: &str) {
    let mark = if ok { "PASS" } else { "FAIL" };
    println!("  {mark}: {detail}");
    println!();
}
