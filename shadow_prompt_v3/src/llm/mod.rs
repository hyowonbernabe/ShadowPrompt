// LLM transport (OpenRouter) + the core agentic tool-loop. Design doc §2: one `run_turn`
// function used by all three entry points (Clipboard, Screenshot, Forms) — they differ only in
// system prompt, initial content, and which tools are attached.
//
// The event-stream/loop/tool separation this module should have (OPENCODE.md §1's "one idea
// worth stealing") is not yet built — this scaffold pass implements the loop directly against
// `Response`/`Message` types rather than through a canonical internal event enum first. That
// restructuring is real, valuable work for the phased build plan, not something to retrofit
// hastily here.
//
// M11: real SSE streaming. `request::build` now sends `stream: true` and this module reads the
// response body as a byte stream (`reqwest::Response::bytes_stream`, already available via the
// `stream` feature) rather than buffering it whole with `.text()`. `StreamAssembler` below folds
// a whole SSE stream — content-delta fragments, tool-call-argument fragments keyed by index,
// the `: OPENROUTER PROCESSING` keep-alive comment lines, the terminal `data: [DONE]` — down
// into the same `response::Response` shape the old non-streaming path used to get directly from
// `serde_json::from_str`, so `run_turn`/`simple_call`'s consumption logic below didn't need to
// change at all. See docs/OPENROUTER.md §5 for the wire format this is built against.

pub mod capabilities;
pub mod messages;
pub mod request;
pub mod response;
pub mod retry;
pub mod system_prompts;

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use messages::{ContentPart, Message, ToolCall, ToolCallFunction};
use request::{build, ServerOrFunctionTool};
use response::{AssistantMessage, Choice, Response};

/// A sink for partial assistant text as it streams in, one content-delta fragment at a time —
/// design doc §2/§16's streaming commitment. `run_turn`'s existing callers (`clipboard_query`,
/// `screenshot_query`, `browser/forms/mod.rs`) all pass `None` today (a no-op) so their behavior
/// is unchanged; a future UI-overlay caller passes a real closure (e.g. one that forwards each
/// fragment onto the `UICommand` channel) to actually paint tokens in as they arrive.
pub type DeltaSink<'a> = &'a (dyn Fn(&str) + Send + Sync);

const ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Clone)]
pub struct LlmClient {
    pub http: Arc<reqwest::Client>,
    pub api_key: Arc<str>,
    /// Default fallback chain from config (design doc §8). `switch_model` (design doc §8/§10)
    /// mutates which of the first two entries leads — see `active_models()`.
    pub default_models: Arc<Vec<String>>,
    /// Runtime toggle for `switch_model` — true once Sonnet 5 has been promoted to position #1.
    /// Pure in-memory state, resets to default (false) on every daemon restart.
    pub sonnet_primary: Arc<std::sync::atomic::AtomicBool>,
    /// One per daemon launch — design doc §8, improves OpenRouter's cache-hit routing.
    pub session_id: Arc<str>,
    /// The chat/completions URL to POST to. Always `ENDPOINT` in production — `new()` is the
    /// only production constructor and it always sets this to `ENDPOINT`. Private (not part of
    /// the public constructor signature) so nothing outside this module can accidentally point
    /// a real daemon at a different URL; tests in this module's own `mod tests` build an
    /// `LlmClient` via a direct struct literal (allowed — privacy is scoped per module tree, and
    /// `tests` is a descendant of this module) to point at a local `wiremock` server instead.
    endpoint: Arc<str>,
}

/// Result of `LlmClient::simple_call` — the answer text plus the ground-truth `model` field
/// from the API response, for callers that need both (currently just `test_model`, design doc
/// §3) without going through the full tool-calling loop in `run_turn`.
#[derive(Debug, Clone)]
pub struct SimpleCallResult {
    pub answer: String,
    pub model: String,
}

/// A tool the model can call during a `run_turn`. Deliberately narrow set only — design doc
/// §5/§7.3: no filesystem/shell access, nothing that makes this look like a general coding
/// agent. Concrete tools (`list_docs`, `read_doc`, `fill_page`, `screenshot`) live in their own
/// modules (`knowledge/`, `browser/forms/`, `capture/`) and implement this trait there.
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn definition(&self) -> messages::ToolDef;
    async fn execute(&self, args_json: &str) -> anyhow::Result<String>;
}

impl LlmClient {
    pub fn new(api_key: String, models: Vec<String>, connect_secs: u64, read_secs: u64) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(connect_secs))
            .timeout(Duration::from_secs(read_secs))
            .build()?;
        Ok(Self {
            http: Arc::new(http),
            api_key: Arc::from(api_key),
            default_models: Arc::new(models),
            sonnet_primary: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            session_id: Arc::from(uuid_like_session_id()),
            endpoint: Arc::from(ENDPOINT),
        })
    }

    /// The `switch_model` hotkey's action — design doc §8/§10. Toggles which of the two paid
    /// models leads; both stay in the chain either way, only their order changes. Returns the
    /// name of the model now in front, for the overlay confirmation.
    pub fn toggle_primary_model(&self) -> &'static str {
        let was = self.sonnet_primary.fetch_xor(true, std::sync::atomic::Ordering::SeqCst);
        if was { "Gemini 3.5 Flash Lite" } else { "Claude Sonnet 5" }
    }

    /// The effective fallback chain right now, honoring `switch_model`'s toggle. Callers that
    /// know the initial content contains an image MUST pass the result through
    /// `capabilities::filter_for_vision` before use — see that module for why.
    fn active_models(&self) -> Vec<String> {
        let mut chain = (*self.default_models).clone();
        if self.sonnet_primary.load(std::sync::atomic::Ordering::SeqCst) && chain.len() >= 2 {
            chain.swap(0, 1);
        }
        chain
    }

    /// The core agentic loop (design doc §2). One call handles all three entry points; callers
    /// pass in the mode-appropriate system prompt, the initial user content (text, or text +
    /// image), and the tool set available this turn.
    ///
    /// No round cap at all (design doc §2 already rejected a stricter budget; the bug-safety
    /// backstop that remained after that was removed too — user report: a genuinely long Forms
    /// answer-all run legitimately needs more than 25 tool-calling rounds, and hitting the cap
    /// forced a tools-stripped final call that itself then failed outright). Speed comes from
    /// the system prompt's own "answer directly, don't over-search" instruction, not from a
    /// code-enforced limit; a model that never stops calling tools now runs until it does,
    /// which is the explicit trade-off asked for here.
    pub async fn run_turn(
        &self,
        system_prompt: &str,
        initial_content: Vec<ContentPart>,
        tools: &[Arc<dyn Tool>],
        has_image: bool,
        on_delta: Option<DeltaSink<'_>>,
    ) -> anyhow::Result<String> {
        let mut messages = vec![Message::system_text(system_prompt), Message::User { content: initial_content }];

        let models = if has_image {
            capabilities::filter_for_vision(&self.active_models())
        } else {
            self.active_models()
        };
        if models.is_empty() {
            anyhow::bail!("no vision-capable model left in the fallback chain");
        }

        let mut tool_defs: Vec<ServerOrFunctionTool> = tools
            .iter()
            .map(|t| ServerOrFunctionTool::Function(t.definition()))
            .collect();
        tool_defs.push(request::web_search_server_tool());

        let mut round: u32 = 0;
        loop {
            let resp = self.call(&models, &messages, tool_defs.clone(), on_delta).await?;
            let choice = resp
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("no choices in response"))?;

            if choice.message.tool_calls.is_empty() {
                let answer = choice.message.content.unwrap_or_default();
                log::debug!("round {round}: final answer: {}", truncate_for_log(&answer));
                return Ok(answer);
            }

            // Real diagnostic gap found live: with no round cap left, a genuinely stuck loop
            // (same tool, never succeeding) is indistinguishable in the log from a genuinely
            // long one — both just say "1 tool call(s)" forever. Logging which tool and a
            // truncated view of its arguments/result turns "round 198, still going, no idea
            // why" into something actually diagnosable.
            let call_summary: Vec<String> = choice
                .message
                .tool_calls
                .iter()
                .map(|c| format!("{}({})", c.function.name, truncate_for_log(&c.function.arguments)))
                .collect();
            // The model can (and sometimes does) send visible reasoning/commentary text in the
            // same message as a tool call, not just a bare tool call with no explanation — that
            // text was being silently dropped from the log entirely, the exact thing that made a
            // stuck loop unreadable ("just tool calls and creating a connection," per the user).
            match choice.message.content.as_deref() {
                Some(text) if !text.is_empty() => {
                    log::debug!("round {round}: says: {}", truncate_for_log(text));
                }
                _ => {}
            }
            log::debug!("round {round}: calls: {}", call_summary.join(", "));
            messages.push(Message::Assistant {
                content: choice.message.content.clone(),
                tool_calls: choice.message.tool_calls.clone(),
            });

            // Design doc §2: execute multiple tool calls in one round concurrently, not
            // sequentially.
            let results = futures::future::join_all(choice.message.tool_calls.iter().map(|call| {
                let tools = tools.to_vec();
                async move {
                    let tool = tools.iter().find(|t| t.name() == call.function.name);
                    let output = match tool {
                        Some(t) => t
                            .execute(&call.function.arguments)
                            .await
                            .unwrap_or_else(|e| format!("error: {e}")),
                        None => format!("error: unknown tool '{}'", call.function.name),
                    };
                    (call.id.clone(), output)
                }
            }))
            .await;

            for (tool_call_id, content) in &results {
                log::debug!("round {round}: {tool_call_id} -> {}", truncate_for_log(content));
            }
            for (tool_call_id, content) in results {
                messages.push(Message::Tool { content, tool_call_id });
            }
            round += 1;
        }
    }

    /// A small dedicated non-tool-loop single-call path (build plan M3) for callers that need
    /// the ground-truth `model` field from the API response alongside the answer text —
    /// `run_turn` deliberately doesn't surface that (its return type is just the final answer
    /// text, and changing it would ripple into every one of its callers). Currently only
    /// `test_model` needs this: a literal "what model are you" health check that never needs
    /// tools, so a single non-looping call is all it requires. Does not touch `run_turn` at all.
    pub async fn simple_call(&self, system_prompt: &str, user_text: &str) -> anyhow::Result<SimpleCallResult> {
        let messages =
            vec![Message::system_text(system_prompt), Message::User { content: vec![ContentPart::text(user_text)] }];

        let models = self.active_models();
        if models.is_empty() {
            anyhow::bail!("no model left in the fallback chain");
        }

        let resp = self.call(&models, &messages, Vec::new(), None).await?;
        let model = resp.model;
        let answer = resp.choices.into_iter().next().and_then(|c| c.message.content).unwrap_or_default();
        Ok(SimpleCallResult { answer, model })
    }

    /// Splits `models` into chunks of at most `MAX_MODELS_PER_REQUEST` and tries each chunk as
    /// its own request, in order, moving to the next chunk only if the current one's request
    /// fails outright. This exists because of a real, undocumented server-enforced limit found
    /// by actually running this against the live API (not caught by any mocked test, since the
    /// mock never enforced it): OpenRouter rejects a `models` array with more than 3 entries
    /// with a 400 ("'models' array must have 3 items or fewer"). Design doc §8's 5-model chain
    /// predates this discovery. Chunking preserves the original 5-model reliability design
    /// (still 5 total attempts across at most 2 real requests) without OpenRouter's own native
    /// fallback-array mechanism doing the cross-chunk part — that part genuinely is our own
    /// retry logic now, unlike everything within one chunk, which still fully delegates to
    /// OpenRouter's server-side fallback exactly as designed.
    const MAX_MODELS_PER_REQUEST: usize = 3;

    async fn call(
        &self,
        models: &[String],
        messages: &[Message],
        tools: Vec<ServerOrFunctionTool>,
        on_delta: Option<DeltaSink<'_>>,
    ) -> anyhow::Result<Response> {
        let chunks: Vec<&[String]> = models.chunks(Self::MAX_MODELS_PER_REQUEST).collect();
        let mut last_err: Option<anyhow::Error> = None;
        for (i, chunk) in chunks.iter().enumerate() {
            match self.call_one_chunk(chunk, messages, tools.clone(), on_delta).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    log::warn!("model chunk {}/{} ({:?}) failed: {e}", i + 1, chunks.len(), chunk);
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("no models configured")))
    }

    async fn call_one_chunk(
        &self,
        models: &[String],
        messages: &[Message],
        tools: Vec<ServerOrFunctionTool>,
        on_delta: Option<DeltaSink<'_>>,
    ) -> anyhow::Result<Response> {
        let body = build(models, messages, tools, &self.session_id);
        let body = serde_json::to_string(&body)?;
        let http = self.http.clone();
        let api_key = self.api_key.clone();

        let endpoint = self.endpoint.clone();
        retry::with_retry(|| {
            let http = http.clone();
            let api_key = api_key.clone();
            let body = body.clone();
            let endpoint = endpoint.clone();
            async move {
                // No HTTP response at all yet — a connect/send failure. This is the one thing
                // this retry layer exists for (see retry.rs's module doc comment).
                let resp = http
                    .post(&*endpoint)
                    .bearer_auth(&*api_key)
                    .header("Content-Type", "application/json")
                    .header("HTTP-Referer", "https://github.com/hyowonbernabe/ShadowPrompt")
                    .header("X-Title", "ShadowPrompt")
                    // Design doc §8: keep out of OpenRouter's public app rankings.
                    .header("X-OpenRouter-App-Visibility", "hidden")
                    .body(body)
                    .send()
                    .await
                    .map_err(|e| retry::CallError::Transport(e.into()))?;
                let status = resp.status();
                if !status.is_success() {
                    // A real HTTP response came back — OpenRouter's own `models`-array fallback
                    // already owns retrying across error/429/downtime server-side *within this
                    // chunk* (design doc §8). Retrying the identical request client-side here
                    // would just duplicate or race that, for no benefit on a deterministic
                    // failure. Fatal (not retried within this chunk), but `call()` above still
                    // moves on to the next chunk, if any.
                    let text = resp.text().await.unwrap_or_default();
                    return Err(retry::CallError::Fatal(anyhow::anyhow!("openrouter {status}: {text}")));
                }
                // A real 2xx came back, so from here on a body-read/parse failure is judged the
                // same way the old non-streaming path judged it: a dropped connection mid-read is
                // `Transport` (worth retrying), a malformed SSE frame from a server that did
                // respond is `Fatal` (retrying identical bytes won't fix malformed JSON).
                read_sse_stream(resp, on_delta).await
            }
        })
        .await
    }
}

/// Caps a log line at a fixed length — tool arguments/results can be arbitrarily large (a
/// screenshot tool call's result, a large page-content blob), and a debug log needs to stay
/// scannable across a long-running loop, not dump megabytes per round.
fn truncate_for_log(s: &str) -> String {
    const MAX_LOG_CHARS: usize = 200;
    let flattened: String = s.chars().map(|c| if c == '\n' { ' ' } else { c }).collect();
    if flattened.chars().count() > MAX_LOG_CHARS {
        let head: String = flattened.chars().take(MAX_LOG_CHARS).collect();
        format!("{head}…")
    } else {
        flattened
    }
}

/// Reads `resp`'s body as a live byte stream and folds OpenRouter's SSE framing
/// (docs/OPENROUTER.md §5) into the same `Response` shape the non-streaming API used to return
/// directly. Frames are `data: {json}\n\n`; `: OPENROUTER PROCESSING` keep-alive comment lines
/// (any line starting with `:` — that's the general SSE comment syntax, not a magic string) are
/// discarded; `data: [DONE]` ends the stream. `on_delta`, when present, is invoked once per
/// non-empty `content` fragment *as it arrives* — not after the whole body is buffered — so a
/// caller can paint partial text incrementally.
async fn read_sse_stream(resp: reqwest::Response, on_delta: Option<DeltaSink<'_>>) -> Result<Response, retry::CallError> {
    let mut assembler = StreamAssembler::new(on_delta);
    let mut buf = String::new();
    let mut byte_stream = resp.bytes_stream();

    'outer: while let Some(next) = byte_stream.next().await {
        let bytes = next.map_err(|e| retry::CallError::Transport(e.into()))?;
        // Normalize CRLF to LF up front so the `\n\n` frame delimiter check below is a single
        // pattern regardless of which line ending the server used.
        buf.push_str(&String::from_utf8_lossy(&bytes).replace("\r\n", "\n"));

        while let Some(pos) = buf.find("\n\n") {
            let frame: String = buf.drain(..pos + 2).collect();
            if process_sse_frame(frame[..frame.len() - 2].trim_end_matches('\n'), &mut assembler)? {
                break 'outer; // saw `data: [DONE]`
            }
        }
    }
    // Lenient: a stream that ends (connection close) without a final blank line still has its
    // last frame's data sitting in `buf` — process it rather than silently dropping it.
    let remainder = buf.trim();
    if !remainder.is_empty() {
        let _ = process_sse_frame(remainder, &mut assembler)?;
    }

    assembler.finish()
}

/// Processes one already-delimited SSE frame (no trailing blank line). Returns `Ok(true)` when
/// the frame was the terminal `data: [DONE]` sentinel, telling the caller to stop reading.
fn process_sse_frame(frame: &str, assembler: &mut StreamAssembler) -> Result<bool, retry::CallError> {
    for line in frame.lines() {
        // SSE spec: a line starting with `:` is a comment, never data — this is exactly how
        // OpenRouter's `: OPENROUTER PROCESSING` keep-alive lines are meant to be recognized
        // (docs/OPENROUTER.md §5), not by matching that literal string.
        if line.is_empty() || line.starts_with(':') {
            continue;
        }
        let Some(data) = line.strip_prefix("data:") else { continue };
        let data = data.trim_start();
        if data == "[DONE]" {
            return Ok(true);
        }
        let chunk: response::StreamChunk = serde_json::from_str(data)
            .map_err(|e| retry::CallError::Fatal(anyhow::anyhow!("parse SSE chunk: {e}; data={data}")))?;
        assembler.ingest(chunk);
    }
    Ok(false)
}

/// One in-progress tool call, keyed by its stream index — see `response::ToolCallDelta`'s doc
/// comment for why `arguments` has to be concatenated rather than replaced.
#[derive(Default)]
struct ToolCallAccum {
    id: String,
    name: String,
    arguments: String,
}

/// Folds a whole SSE stream's chunks into one final `Response`. This is the tool-call
/// accumulation logic build plan M11 calls out as the riskiest part of this milestone: a tool
/// call's `id`/`name` typically arrive on the chunk that opens it, then only
/// `function.arguments` *string fragments* arrive on every chunk after — concatenated in
/// arrival order, the full argument list is only guaranteed to be valid JSON once the stream
/// itself ends (terminal `[DONE]`, or the underlying response finishing). There is no
/// end-of-this-specific-tool-call marker to wait for beyond that; a new `index` appearing is
/// just the next tool call starting, not a completion signal for the previous one, so
/// completion is judged by "the stream is over," never by anything per-call.
struct StreamAssembler<'a> {
    model: Option<String>,
    content: String,
    tool_calls: Vec<Option<ToolCallAccum>>,
    finish_reason: Option<String>,
    usage: Option<response::Usage>,
    saw_error: bool,
    on_delta: Option<DeltaSink<'a>>,
}

impl<'a> StreamAssembler<'a> {
    fn new(on_delta: Option<DeltaSink<'a>>) -> Self {
        Self {
            model: None,
            content: String::new(),
            tool_calls: Vec::new(),
            finish_reason: None,
            usage: None,
            saw_error: false,
            on_delta,
        }
    }

    fn ingest(&mut self, chunk: response::StreamChunk) {
        if let Some(model) = chunk.model {
            self.model = Some(model);
        }
        if let Some(usage) = chunk.usage {
            // The usage-carrying chunk sent just before `[DONE]` (docs/OPENROUTER.md §5).
            self.usage = Some(usage);
        }
        for choice in chunk.choices {
            if let Some(reason) = choice.finish_reason {
                // Mid-stream errors arrive as `finish_reason: "error"` rather than an HTTP
                // status (docs/OPENROUTER.md §5) — surfaced as a real error in `finish()`, not
                // silently folded into whatever partial content/tool-calls arrived before it.
                // `finish_reason` can legitimately appear twice (once on the real final chunk,
                // again on the usage-carrying chunk after it) — last one wins, and both are
                // equally valid since only the round-trip's control-flow decision in `run_turn`
                // (has a tool call vs. doesn't) reads this, never the specific reason string.
                if reason == "error" {
                    self.saw_error = true;
                }
                self.finish_reason = Some(reason);
            }

            if let Some(text) = choice.delta.content {
                if !text.is_empty() {
                    if let Some(sink) = self.on_delta {
                        sink(&text);
                    }
                    self.content.push_str(&text);
                }
            }

            for delta in choice.delta.tool_calls {
                if self.tool_calls.len() <= delta.index {
                    self.tool_calls.resize_with(delta.index + 1, || None);
                }
                let entry = self.tool_calls[delta.index].get_or_insert_with(ToolCallAccum::default);
                if let Some(id) = delta.id {
                    entry.id = id;
                }
                if let Some(function) = delta.function {
                    if let Some(name) = function.name {
                        entry.name = name;
                    }
                    if let Some(arguments) = function.arguments {
                        // The accumulation step: each chunk contributes a raw JSON-string
                        // fragment, not a value — only the fully concatenated string is
                        // guaranteed to parse.
                        entry.arguments.push_str(&arguments);
                    }
                }
            }
        }
    }

    fn finish(self) -> Result<Response, retry::CallError> {
        if self.saw_error {
            return Err(retry::CallError::Fatal(anyhow::anyhow!(
                "openrouter stream ended with finish_reason=error"
            )));
        }
        let tool_calls: Vec<ToolCall> = self
            .tool_calls
            .into_iter()
            .flatten()
            .map(|call| ToolCall {
                id: call.id,
                kind: "function".to_string(),
                function: ToolCallFunction { name: call.name, arguments: call.arguments },
            })
            .collect();
        let content = if self.content.is_empty() { None } else { Some(self.content) };
        Ok(Response {
            model: self.model.unwrap_or_default(),
            choices: vec![Choice { message: AssistantMessage { content, tool_calls }, finish_reason: self.finish_reason }],
            usage: self.usage,
        })
    }
}

fn uuid_like_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    format!("shadowprompt-{nanos:x}")
}

// M1/M11: exercise `run_turn`/`call` against a mocked HTTP layer (`wiremock`) instead of the
// real OpenRouter API — no real API key exists in this environment, and design doc §13 doesn't
// mandate live-network testing anyway. Every mock response body here is hand-constructed SSE
// (`data: {json}\n\n` frames, terminal `data: [DONE]`), matching docs/OPENROUTER.md §5's
// documented wire format exactly, since M11 flips the request builder to `stream: true`.
// Covers: a plain-text response with content split across several chunks plus a keep-alive
// comment line thrown in (reassembles correctly, callback fires per fragment), a tool call
// whose arguments arrive fragmented across several chunks (reassembles into valid JSON before
// the tool ever executes), a mixed round (visible text *and* a tool call in the same turn), a
// tool-call round trip whose result is actually fed back to the model, the loop running well
// past the old (now-removed) 25-round cap without cutting off early, and the retry/backoff path
// engaging end-to-end on a genuine transport failure (retry.rs has its own narrower unit tests
// for the pure backoff logic).
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Builds an `LlmClient` pointed at `endpoint` (a local mock server in every test here)
    /// instead of the real OpenRouter URL. Direct struct literal, not `new()` — deliberate:
    /// `new()` is the only production constructor and it always hardcodes `ENDPOINT`; tests
    /// reach into the private `endpoint` field directly because `mod tests` is a descendant of
    /// this module and Rust scopes privacy per module tree, not per file.
    fn test_client(endpoint: String, models: Vec<String>) -> LlmClient {
        LlmClient {
            http: Arc::new(reqwest::Client::new()),
            api_key: Arc::from("test-key"),
            default_models: Arc::new(models),
            sonnet_primary: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            session_id: Arc::from("test-session"),
            endpoint: Arc::from(endpoint),
        }
    }

    /// Builds an SSE response body out of already-JSON-encoded chunk values, in order, followed
    /// by the terminal `data: [DONE]` sentinel — docs/OPENROUTER.md §5's exact documented shape.
    fn sse_body(chunks: &[serde_json::Value]) -> String {
        let mut out = String::new();
        for chunk in chunks {
            out.push_str("data: ");
            out.push_str(&chunk.to_string());
            out.push_str("\n\n");
        }
        out.push_str("data: [DONE]\n\n");
        out
    }

    /// Same as `sse_body`, but interleaves an `: OPENROUTER PROCESSING` keep-alive comment line
    /// (docs/OPENROUTER.md §5) before every chunk, proving those lines get discarded rather than
    /// tripping up the JSON parse.
    fn sse_body_with_keepalives(chunks: &[serde_json::Value]) -> String {
        let mut out = String::new();
        for chunk in chunks {
            out.push_str(": OPENROUTER PROCESSING\n\n");
            out.push_str("data: ");
            out.push_str(&chunk.to_string());
            out.push_str("\n\n");
        }
        out.push_str("data: [DONE]\n\n");
        out
    }

    fn respond_sse(body: String) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_raw(body, "text/event-stream")
    }

    /// A trivial tool that records how many times it was called and returns a fixed string —
    /// just enough to prove the round-trip (called, result captured, result sent back).
    struct EchoTool {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl Tool for EchoTool {
        fn name(&self) -> &'static str {
            "echo_tool"
        }

        fn definition(&self) -> messages::ToolDef {
            messages::ToolDef {
                kind: "function",
                function: messages::ToolFunctionDef {
                    name: "echo_tool".to_string(),
                    description: "test-only tool".to_string(),
                    parameters: serde_json::json!({"type": "object", "properties": {}}),
                },
            }
        }

        async fn execute(&self, _args_json: &str) -> anyhow::Result<String> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok("echoed-result".to_string())
        }
    }

    /// Records the exact `args_json` string it was executed with — used to prove a fragmented
    /// streamed tool call reassembled into valid JSON *before* reaching the tool.
    struct CapturingTool {
        received_args: Arc<Mutex<Option<String>>>,
    }

    #[async_trait::async_trait]
    impl Tool for CapturingTool {
        fn name(&self) -> &'static str {
            "capturing_tool"
        }

        fn definition(&self) -> messages::ToolDef {
            messages::ToolDef {
                kind: "function",
                function: messages::ToolFunctionDef {
                    name: "capturing_tool".to_string(),
                    description: "test-only tool".to_string(),
                    parameters: serde_json::json!({"type": "object", "properties": {}}),
                },
            }
        }

        async fn execute(&self, args_json: &str) -> anyhow::Result<String> {
            *self.received_args.lock().unwrap() = Some(args_json.to_string());
            Ok("captured".to_string())
        }
    }

    #[tokio::test]
    async fn run_turn_reassembles_streamed_text_and_invokes_delta_callback() {
        let server = MockServer::start().await;
        // Content split across 3 chunks, plus a keep-alive comment line before each one — both
        // the reassembly and the keep-alive discard get exercised by one test.
        let body = sse_body_with_keepalives(&[
            serde_json::json!({"model": "google/gemini-3.5-flash-lite", "choices": [{"delta": {"content": "4"}}]}),
            serde_json::json!({"choices": [{"delta": {"content": "2"}}]}),
            serde_json::json!({"choices": [{"delta": {"content": ""}, "finish_reason": "stop"}]}),
            // The extra usage-carrying chunk sent just before [DONE] (docs/OPENROUTER.md §5) —
            // empty delta.content, finish_reason repeated a second time.
            serde_json::json!({
                "choices": [{"delta": {}, "finish_reason": "stop"}],
                "usage": {"prompt_tokens": 10, "completion_tokens": 2}
            }),
        ]);
        Mock::given(method("POST")).and(path("/")).respond_with(respond_sse(body)).expect(1).mount(&server).await;

        let client = test_client(server.uri(), vec!["google/gemini-3.5-flash-lite".to_string()]);
        let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let seen_for_cb = seen.clone();
        let on_delta = move |fragment: &str| seen_for_cb.lock().unwrap().push(fragment.to_string());

        let answer = client
            .run_turn("system prompt", vec![ContentPart::text("What is 6*7?")], &[], false, Some(&on_delta))
            .await
            .expect("run_turn should succeed against a streamed plain-text mocked response");

        assert_eq!(answer, "42", "the two content fragments must concatenate in arrival order");
        assert_eq!(
            *seen.lock().unwrap(),
            vec!["4".to_string(), "2".to_string()],
            "the callback must fire once per non-empty content fragment, as it arrives"
        );
    }

    #[tokio::test]
    async fn run_turn_executes_tool_calls_and_feeds_results_back() {
        let server = MockServer::start().await;

        let round1 = sse_body(&[serde_json::json!({
            "model": "google/gemini-3.5-flash-lite",
            "choices": [{
                "delta": {
                    "tool_calls": [{
                        "index": 0,
                        "id": "call_1",
                        "type": "function",
                        "function": {"name": "echo_tool", "arguments": "{}"}
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        })]);
        let round2 = sse_body(&[serde_json::json!({
            "model": "google/gemini-3.5-flash-lite",
            "choices": [{"delta": {"content": "done"}, "finish_reason": "stop"}]
        })]);

        // First matching mock (mounted first, equal priority) serves round 1 exactly once, then
        // it stops matching and the second mock (mounted after, unlimited) takes over for round 2
        // onward — the standard wiremock idiom for "sequential responses from the same endpoint."
        Mock::given(method("POST")).and(path("/")).respond_with(respond_sse(round1)).up_to_n_times(1).mount(&server).await;
        Mock::given(method("POST")).and(path("/")).respond_with(respond_sse(round2)).mount(&server).await;

        let calls = Arc::new(AtomicUsize::new(0));
        let tool: Arc<dyn Tool> = Arc::new(EchoTool { calls: calls.clone() });

        let client = test_client(server.uri(), vec!["google/gemini-3.5-flash-lite".to_string()]);
        let answer = client
            .run_turn("system prompt", vec![ContentPart::text("use the tool")], &[tool], false, None)
            .await
            .expect("run_turn should succeed after one tool round-trip");

        assert_eq!(answer, "done");
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "the tool should have been executed exactly once"
        );

        // Confirm the tool's result was actually fed back to the model as a `role: tool`
        // message in round 2's request — not just executed and discarded.
        let requests = server.received_requests().await.expect("request recording is on by default");
        assert_eq!(requests.len(), 2, "expected exactly 2 rounds: the tool call, then the final answer");
        let second_body: serde_json::Value =
            requests[1].body_json().expect("round 2's request body should be valid JSON");
        let sent_messages = second_body["messages"].as_array().expect("messages should be an array");
        let tool_msg = sent_messages
            .iter()
            .find(|m| m["role"] == "tool")
            .expect("round 2's request should carry the tool's result back as a role:tool message");
        assert_eq!(tool_msg["tool_call_id"], "call_1");
        assert_eq!(tool_msg["content"], "echoed-result");
    }

    /// The riskiest part of M11 (build plan's own words): a tool call's `function.arguments`
    /// arrives as raw JSON-string fragments spread across several chunks — id/name in the
    /// opening chunk, then 3+ chunks that each append a piece of the arguments string. No
    /// individual fragment is valid JSON; only the full concatenation is. This asserts both that
    /// the concatenation is byte-for-byte correct AND that it's valid, parseable JSON before the
    /// tool ever sees it.
    #[tokio::test]
    async fn run_turn_reassembles_fragmented_tool_call_arguments_before_executing() {
        let server = MockServer::start().await;

        let full_args = r#"{"query":"capital of France","limit":5}"#;
        let (frag1, frag2, frag3) = (r#"{"query":"#, r#""capital of France","#, r#""limit":5}"#);
        assert_eq!(format!("{frag1}{frag2}{frag3}"), full_args, "fixture fragments must reassemble to the fixture");

        let round1 = sse_body(&[
            // Opening chunk: id + name, first argument fragment.
            serde_json::json!({
                "model": "google/gemini-3.5-flash-lite",
                "choices": [{"delta": {"tool_calls": [{
                    "index": 0, "id": "call_frag", "type": "function",
                    "function": {"name": "capturing_tool", "arguments": frag1}
                }]}}]
            }),
            // Two more argument-only fragments — no id/name repeated.
            serde_json::json!({"choices": [{"delta": {"tool_calls": [{"index": 0, "function": {"arguments": frag2}}]}}]}),
            serde_json::json!({"choices": [{"delta": {"tool_calls": [{"index": 0, "function": {"arguments": frag3}}]}}]}),
            serde_json::json!({"choices": [{"delta": {}, "finish_reason": "tool_calls"}]}),
        ]);
        let round2 = sse_body(&[serde_json::json!({
            "model": "google/gemini-3.5-flash-lite",
            "choices": [{"delta": {"content": "done"}, "finish_reason": "stop"}]
        })]);

        Mock::given(method("POST")).and(path("/")).respond_with(respond_sse(round1)).up_to_n_times(1).mount(&server).await;
        Mock::given(method("POST")).and(path("/")).respond_with(respond_sse(round2)).mount(&server).await;

        let received_args: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let tool: Arc<dyn Tool> = Arc::new(CapturingTool { received_args: received_args.clone() });

        let client = test_client(server.uri(), vec!["google/gemini-3.5-flash-lite".to_string()]);
        let answer = client
            .run_turn("system prompt", vec![ContentPart::text("search something")], &[tool], false, None)
            .await
            .expect("run_turn should succeed once the fragmented tool call reassembles");

        assert_eq!(answer, "done");
        let captured = received_args.lock().unwrap().clone().expect("the tool must have been called");
        assert_eq!(captured, full_args, "fragments must concatenate in arrival order into the exact original JSON");
        serde_json::from_str::<serde_json::Value>(&captured)
            .expect("the reassembled arguments must be valid JSON before the tool ever executes");
    }

    /// `run_turn` must handle a round that mixes visible text and a tool call in the same
    /// turn (build plan M11: "a mix of both"), not just the pure-text or pure-tool-call cases.
    #[tokio::test]
    async fn run_turn_handles_mixed_text_and_tool_call_in_same_round() {
        let server = MockServer::start().await;

        let round1 = sse_body(&[
            serde_json::json!({
                "model": "google/gemini-3.5-flash-lite",
                "choices": [{"delta": {"content": "Let me check that. "}}]
            }),
            serde_json::json!({"choices": [{"delta": {"tool_calls": [{
                "index": 0, "id": "call_mixed", "type": "function",
                "function": {"name": "echo_tool", "arguments": "{}"}
            }]}, "finish_reason": "tool_calls"}]}),
        ]);
        let round2 = sse_body(&[serde_json::json!({
            "model": "google/gemini-3.5-flash-lite",
            "choices": [{"delta": {"content": "the answer is 42"}, "finish_reason": "stop"}]
        })]);

        Mock::given(method("POST")).and(path("/")).respond_with(respond_sse(round1)).up_to_n_times(1).mount(&server).await;
        Mock::given(method("POST")).and(path("/")).respond_with(respond_sse(round2)).mount(&server).await;

        let calls = Arc::new(AtomicUsize::new(0));
        let tool: Arc<dyn Tool> = Arc::new(EchoTool { calls: calls.clone() });

        let client = test_client(server.uri(), vec!["google/gemini-3.5-flash-lite".to_string()]);
        let answer = client
            .run_turn("system prompt", vec![ContentPart::text("mixed round")], &[tool], false, None)
            .await
            .expect("run_turn should handle a round with both text and a tool call");

        assert_eq!(answer, "the answer is 42");
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1, "the tool call present alongside text must still execute");
    }

    #[tokio::test]
    async fn run_turn_keeps_looping_past_the_old_25_round_cap_when_the_model_keeps_calling_tools() {
        let server = MockServer::start().await;

        // The old MAX_LOOP_ROUNDS backstop was 25 — user report: a genuinely long Forms
        // answer-all run legitimately needed more rounds than that, and hitting the cap forced
        // a tools-stripped final call that then failed outright. This proves there's no cap at
        // all anymore: the model keeps calling tools well past the old limit, and `run_turn`
        // keeps servicing every one of them rather than cutting off early.
        const ROUNDS_PAST_OLD_CAP: usize = 30;

        let looping_body = sse_body(&[serde_json::json!({
            "model": "google/gemini-3.5-flash-lite",
            "choices": [{
                "delta": {
                    "tool_calls": [{
                        "index": 0, "id": "call_loop", "type": "function",
                        "function": {"name": "echo_tool", "arguments": "{}"}
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        })]);
        for _ in 0..ROUNDS_PAST_OLD_CAP {
            Mock::given(method("POST"))
                .and(path("/"))
                .respond_with(respond_sse(looping_body.clone()))
                .up_to_n_times(1)
                .mount(&server)
                .await;
        }

        let final_body = sse_body(&[serde_json::json!({
            "model": "google/gemini-3.5-flash-lite",
            "choices": [{
                "delta": {"content": "done after a genuinely long run"},
                "finish_reason": "stop"
            }]
        })]);
        Mock::given(method("POST")).and(path("/")).respond_with(respond_sse(final_body)).mount(&server).await;

        let calls = Arc::new(AtomicUsize::new(0));
        let tool: Arc<dyn Tool> = Arc::new(EchoTool { calls: calls.clone() });

        let client = test_client(server.uri(), vec!["google/gemini-3.5-flash-lite".to_string()]);
        let answer = client
            .run_turn("system prompt", vec![ContentPart::text("keep going")], &[tool], false, None)
            .await
            .expect("run_turn should keep looping and return the model's real final answer, uncapped");

        assert_eq!(answer, "done after a genuinely long run");
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), ROUNDS_PAST_OLD_CAP);

        let requests = server.received_requests().await.expect("request recording is on by default");
        assert_eq!(
            requests.len(),
            ROUNDS_PAST_OLD_CAP + 1,
            "expected every tool-calling round plus exactly one real final round — no forced cutoff"
        );
    }

    #[tokio::test]
    async fn call_retries_on_transport_failure_with_backoff_before_giving_up() {
        // Deliberately no wiremock server here: port 1 on loopback has nothing listening, so the
        // very first thing that happens on every attempt is a genuine transport-level failure
        // (connection refused) — never an HTTP response. This exercises retry.rs's backoff
        // end-to-end through the real `LlmClient::call` path, not just retry.rs's own isolated
        // unit tests of `with_retry` in the abstract.
        let client = test_client("http://127.0.0.1:1".to_string(), vec!["m".to_string()]);
        let started = std::time::Instant::now();

        let result = client.run_turn("system prompt", vec![ContentPart::text("hi")], &[], false, None).await;

        let elapsed = started.elapsed();
        assert!(result.is_err(), "expected a transport error against an unreachable port");
        // 3 attempts, 1s then 2s backoff between them -> at least ~3s of sleeping before the
        // final failure. A loose 2s floor keeps this robust against scheduling jitter while
        // still proving more than one attempt actually happened (a single attempt with no
        // retry would return near-instantly).
        assert!(
            elapsed >= Duration::from_secs(2),
            "expected at least ~2 backoff sleeps' worth of elapsed time before giving up, got {elapsed:?}"
        );
    }

    #[test]
    fn model_chunks_split_five_models_into_three_then_two() {
        // Pure logic check, no network: a real, live 400 ("'models' array must have 3 items or
        // fewer") found by actually running --probe against the real API — not caught by any
        // mocked test beforehand, since nothing mocked that constraint. `MAX_MODELS_PER_REQUEST`
        // and the chunking in `LlmClient::call` exist specifically because of this.
        let models = ["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "e".to_string()];
        let chunks: Vec<Vec<String>> = models
            .chunks(LlmClient::MAX_MODELS_PER_REQUEST)
            .map(|c| c.to_vec())
            .collect();
        assert_eq!(chunks, vec![vec!["a", "b", "c"], vec!["d", "e"]]);
    }

    #[tokio::test]
    async fn call_falls_through_to_the_next_chunk_when_the_first_is_rejected() {
        let server = MockServer::start().await;
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_responder = calls.clone();

        // One mock, matching every request to this endpoint: the first hit simulates the real
        // "too many models" 400, every hit after that succeeds — proving `call()` actually
        // issues a second real HTTP request for the remaining models instead of giving up after
        // the first chunk's request is rejected.
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(move |_: &wiremock::Request| {
                let n = calls_for_responder.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if n == 0 {
                    ResponseTemplate::new(400).set_body_json(serde_json::json!({
                        "error": {"message": "'models' array must have 3 items or fewer.", "code": 400}
                    }))
                } else {
                    respond_sse(sse_body(&[serde_json::json!({
                        "model": "d",
                        "choices": [{"delta": {"content": "ok"}, "finish_reason": "stop"}]
                    })]))
                }
            })
            .expect(2)
            .mount(&server)
            .await;

        let client = test_client(
            server.uri(),
            vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "e".to_string()],
        );
        let answer = client
            .run_turn("system prompt", vec![ContentPart::text("hi")], &[], false, None)
            .await
            .expect("should succeed once the second chunk is tried");
        assert_eq!(answer, "ok");
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2, "expected exactly one request per chunk");
    }
}
