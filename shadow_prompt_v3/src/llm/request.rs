// Build OpenRouter chat/completions requests.
//
// Design-doc decisions encoded here (docs/SHADOWPROMPT_V3_DESIGN.md §8):
//   - `models` (fallback array), not a single `model` field — OpenRouter tries each in order on
//     error/429/downtime, no custom Rust retry logic needed for that mechanism.
//   - `session_id`: one per daemon launch, improves provider-sticky cache-hit routing.
//   - Anthropic models pinned to the direct Anthropic route (`provider.order: ["anthropic"]`) —
//     a lesson already learned in v2 (Bedrock/Vertex routes for Claude have lagged on vision
//     support before). Generalized here to match on any `anthropic/` prefix in the *first*
//     model of the chain, same as v2 did for its single `model` field.
//   - Reasoning fixed to low/default effort always — no per-question reasoning-effort hotkey
//     (design doc §9: "answer fast" is a system-prompt concern, not a request-parameter one).
//   - Web search via OpenRouter's `openrouter:web_search` *server tool*, sitting in the same
//     `tools` array as our own function tools, so the model decides per-turn whether to call it
//     — NOT the always-on `{"id":"web"}` plugin, which runs a search on every request
//     regardless of need and would contradict "autonomous, only when needed" (corrected during
//     scaffolding; the design doc's plugin-based wording predates this correction).

use super::messages::{Message, ToolDef};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Request<'a> {
    pub models: &'a [String],
    pub messages: &'a [Message],
    pub max_tokens: u32,
    pub stream: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ServerOrFunctionTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderPrefs>,
    pub session_id: &'a str,
}

/// A request's `tools` array entries are either our own function-tool defs (`list_docs`,
/// `read_doc`, `fill_page`, `screenshot`) or OpenRouter's server-executed tools (currently just
/// `openrouter:web_search`) — same array, same wire shape from the model's point of view.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ServerOrFunctionTool {
    Function(ToolDef),
    /// A server tool reference — just `{"type": "openrouter:web_search"}`, no `function` block.
    Server { r#type: &'static str },
}

pub fn web_search_server_tool() -> ServerOrFunctionTool {
    ServerOrFunctionTool::Server { r#type: "openrouter:web_search" }
}

#[derive(Debug, Serialize)]
pub struct Reasoning {
    pub effort: &'static str,
    pub exclude: bool,
}

#[derive(Debug, Serialize)]
pub struct ProviderPrefs {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<&'static str>,
    pub allow_fallbacks: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn build<'a>(
    models: &'a [String],
    messages: &'a [Message],
    tools: Vec<ServerOrFunctionTool>,
    session_id: &'a str,
) -> Request<'a> {
    Request {
        models,
        messages,
        max_tokens: 4096,
        // Design doc §2 commits to streaming the answer into the overlay token-by-token.
        // M11: real SSE streaming is implemented on the response-handling side (see
        // `mod.rs`'s SSE frame parser + tool-call-argument accumulator), so this can now
        // actually be `true` — the body comes back as `data: {...}\n\n`-delimited frames
        // (docs/OPENROUTER.md §5) instead of one complete JSON object.
        stream: true,
        tools,
        tool_choice: None, // "auto" is the default; explicit forcing not needed anywhere yet
        parallel_tool_calls: Some(true), // design doc §2: execute multiple tool calls concurrently
        reasoning: Some(Reasoning { effort: "low", exclude: true }),
        // RESOLVED (M2, design doc §8/§16 open question): NOT applying the anthropic-only
        // provider.order pin here, unlike v2 — and this is now a documented conclusion, not an
        // unverified guess. v2 could pin safely because it only ever sent ONE model per request.
        // v3 sends a `models` array with up to 5 different models in a single request, and the
        // question was whether `provider.order: ["anthropic"]` scopes to just the Anthropic
        // entry of that array or applies to the whole request.
        //
        // Checked directly against OpenRouter's docs (no live API call needed for this, since
        // it's a schema/wire-shape question, not a runtime-behavior one):
        //   - Chat Completions request schema (openrouter.ai/docs/api-reference/overview):
        //     `provider` is a top-level field, a sibling of `models`/`route`/`user` — there is
        //     no per-entry provider block nested inside the `models` array anywhere in the
        //     schema. Structurally, a single request can only ever carry ONE `provider`
        //     preferences object, so it cannot be scoped to one model among several.
        //   - Provider Selection guide (openrouter.ai/docs/guides/routing/provider-selection),
        //     the "Advanced Sorting with Partition" section: "By default, when you specify
        //     multiple models (fallbacks), OpenRouter groups endpoints by model before sorting"
        //     and `partition: "none"` "allow[s] endpoints to be sorted globally across all
        //     models" — `provider.sort`/`partition` are explicitly described as operating
        //     *across* the whole `models` array, reinforcing that `provider.*` preferences are
        //     evaluated at the request level, not per model entry.
        //   - No page found states plainly what happens when `provider.order` names a provider
        //     that doesn't serve some of the other models in a mixed `models` array (this exact
        //     interaction stays genuinely undocumented — flagged as such in docs/OPENROUTER.md's
        //     "Master List of NOT DOCUMENTED Items"). But the schema fact above is sufficient on
        //     its own: since `provider.order` cannot be expressed as scoped to a single `models`
        //     entry at all, pinning it here would apply `["anthropic"]` to every model in the
        //     chain, including google/gemini and the free inclusionai/nex-agi models, none of
        //     which have an "anthropic" provider — a real risk of starving their routing, not a
        //     safe optimization. Left unpinned, matching the design doc's risk-averse default,
        //     now for a cited structural reason rather than an open question.
        provider: None,
        session_id,
    }
}
