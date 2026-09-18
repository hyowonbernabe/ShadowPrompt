# OpenRouter — Reference for ShadowPrompt v3

Researched directly against openrouter.ai/docs (live as of 2026-09-18). Every claim below is cited to a
specific doc URL. Items with no documentation found are marked **NOT DOCUMENTED** — treat as "not found
in the pages examined," not as confirmed-absent.

Primary model target: `google/gemini-3.5-flash-lite`. Fallback models (chosen for throughput, not
smarts): `inclusionai/ling-3.0-flash-fin:free`, `nex-agi/nex-n2.5-mini:free`,
`inclusionai/ling-3.0-flash-sante:free`.

---

## 1. Web Search (`:online` suffix / `web` plugin)

Activation: append `:online` to a model slug, or use an explicit plugin block.

```json
{
  "model": "openai/gpt-5.2:online",
  "plugins": [{
    "id": "web",
    "engine": "native|exa|firecrawl|parallel|perplexity",
    "mode": "turbo|fast|auto|basic|advanced|deep-lite|deep|deep-reasoning|instant",
    "max_results": 5,
    "search_prompt": "Custom prompt text",
    "include_domains": ["example.com"],
    "exclude_domains": ["reddit.com"]
  }]
}
```
[Web Search plugin doc](https://openrouter.ai/docs/guides/features/plugins/web-search)

- **Result injection**: OpenRouter auto-injects a system instruction ("A web search was conducted on
  `date`. Incorporate the following web search results...cite them using markdown links named using the
  domain of the source") — overridable via `search_prompt`. Results return as OpenAI-style `annotations`
  on the message: `{"type": "url_citation", "url_citation": {"url", "title", "content", "start_index",
  "end_index"}}`.
- **Model compatibility**: works with any model when routed through Exa or another external engine;
  `native` engine requires the underlying provider to support search itself (Anthropic, OpenAI, Google,
  Perplexity, SpaceXAI have native search). Using the plugin "will incur extra costs, even with free
  models."
- **Result count/recency control**: `max_results` (default 5) is documented; a `recency`/`freshness`
  parameter is **NOT DOCUMENTED**. Domain include/exclude lists are the only scoping control found.
- **Engines confirmed, including Exa**: `native`, `exa` (OpenRouter's default fallback engine, hosted,
  billed via OpenRouter credits), `parallel`, `perplexity`, `firecrawl` (explicitly **bring-your-own-key**
  — you supply your own Firecrawl account). This is the closest thing to a "self-hosted" path; a fully
  custom/self-hosted search backend is **NOT DOCUMENTED**.
- **Pricing**: Exa $0.007/request (instant/fast/auto, 10 results incl., +$0.001/extra result), $0.012
  (deep modes), $0.015 (deep-reasoning); Parallel $0.001 (turbo/fast) or $0.005 (basic/advanced);
  Perplexity $0.005/request; Firecrawl 2 credits/10 results + 5 credits/result-with-highlights (10,000
  free credits included); native billed per provider's own search-context pricing (low/medium/high
  tiers).
- **Adjacent/separate feature**: "Web Search Server Tool" / "Web Fetch Server Tool" under
  `/docs/guides/features/server-tools/`, and a Responses-API-specific web search page — model-callable
  tool primitives, distinct from the `:online`/`web` plugin. Overlap not fully characterized. [Server
  tools listing](https://openrouter.ai/docs/guides/features/server-tools)

## 2. Prompt Caching

[Prompt Caching doc](https://openrouter.ai/docs/features/prompt-caching)

- **Automatic/implicit caching (no config needed)**: OpenAI, DeepSeek, Grok, Moonshot AI, Groq, Z.AI, and
  **Google Gemini 2.5-series and newer** — automatic, similar to OpenAI's.
- **Gemini caching CONFIRMED** — direct answer to the critical question for our primary model. Gemini
  supports both implicit (automatic, 2.5+) and explicit (`cache_control` breakpoints, "similar to
  Anthropic's approach") caching.
- **Explicit caching (`cache_control` breakpoints)**: Anthropic Claude (top-level or per-content-block),
  Google Gemini, Alibaba Qwen — syntax: `{"type": "text", "text": "...", "cache_control": {"type":
  "ephemeral"}}`.
- **TTL**: Anthropic 5 min default / 1 hr optional (`ttl: "1h"`, 2x base write price); **Gemini fixed 5
  min (non-extending on hit)**; Qwen 5 min; OpenAI minimum 30 min.
- **Minimum token thresholds**: Anthropic 4,096 (Opus 4.x, Haiku 4.5) / 2,048 (Haiku 3.5) / 1,024 (Sonnet
  4.x, Opus 4/4.1); **Gemini 2.5 Flash = 1,024, Pro = 4,096**; OpenAI = 1,024.
- **Cost multipliers**: Anthropic write 1.25x (5min)/2x (1hr), read 0.1x; OpenAI write 1.25x, read
  0.25–0.5x; **Gemini write = input price + storage fee, read 0.25x**; Grok/Moonshot no extra write cost,
  read 0.25x; Groq read 0.5x; Qwen write 1.25x, read 0.1x.
- **Usage accounting fields**: `usage.prompt_tokens_details.cached_tokens` (read) and
  `.cache_write_tokens` (write), plus `cache_discount` (savings applied).
- **Provider-sticky routing**: OpenRouter routes repeat requests to the same provider endpoint to
  maximize cache hits (10-min inactivity window, hashed from first system+user message by default),
  forceable via a `session_id` field or `x-session-id` header — relevant for our session design if we
  want reliable cache hits.

## 3. Multimodality / Vision

- **Image input**: `{"type": "image_url", "image_url": {"url": "..."}}` — hosted URLs or base64 data URLs
  (`data:image/jpeg;base64,...`). Types: png, jpeg, webp, gif. Optional `detail`:
  `"low"|"high"|"auto"`. [Image Inputs](https://openrouter.ai/docs/guides/overview/multimodal/image-understanding)
- **Gemini vision confirmed** — official worked example uses `google/gemini-3-flash-preview` for image
  input. Queryable per-model via `architecture.input_modalities` containing `"image"`. [Models
  API](https://openrouter.ai/docs/api/api-reference/models/get-models)
- **Size/count limits**: no universal numeric limit — "varies per provider and per model." A 20MB
  max-content-size figure appears only in the provider-registration schema, not as a guaranteed
  end-user limit. **NOT DOCUMENTED**: a concrete per-model max-image-size/count table.
- **Video input**: `"video_url"` (Chat Completions) or `"input_video"` (beta Responses API); base64 or
  link. `processing: "agentic"|"static"` is Gemini-specific. **"The Messages API does not support video
  inputs."** Gemini via AI Studio only accepts YouTube links (not Vertex); Gemini via Vertex AI requires
  base64 only. Formats: mp4, mpeg, mov, webm. No numeric max length/size documented. [Video
  Inputs](https://openrouter.ai/docs/guides/overview/multimodal/videos)
- **Audio input**: `"input_audio"`, **base64 required — direct URLs NOT supported**:
  `{"type": "input_audio", "input_audio": {"data": "...", "format": "wav"}}`. Formats: wav, mp3, aiff,
  aac, ogg, flac, m4a, pcm16, pcm24 (not all models support all formats). Separate dedicated TTS/STT
  endpoints also exist. [Audio](https://openrouter.ai/docs/guides/overview/multimodal/audio)
- **PDF/file input**: `{"type": "file", "file": {"filename": "...", "file_data": "url or
  data:application/pdf;base64,..."}}`. Works on any model — native-file-capable models get the PDF
  directly, others get pre-parsed content via a `file-parser` plugin: `mistral-ocr` ($2/1,000 pages,
  default, good for scanned PDFs), `cloudflare-ai` (free, markdown conversion), or `native`. Max 8 images
  extracted per PDF. Parsed content returns as `annotations`, resendable on follow-up turns to skip
  re-parsing charges. [PDFs](https://openrouter.ai/docs/guides/overview/multimodal/pdfs)

## 4. Structured/JSON Output & Tool Calling

- **`response_format` syntax**: JSON mode `{"type": "json_object"}`; JSON Schema/strict mode `{"type":
  "json_schema", "json_schema": {"name": "...", "strict": true, "schema": {...}}}`. The full OpenAPI
  schema also lists `text`, `grammar` (GBNF), `python` as additional `type` values — reference-only,
  verify against target provider. A separate top-level `structured_outputs` boolean also exists.
  [Structured Outputs](https://openrouter.ai/docs/guides/features/structured-outputs) / [Chat completion
  schema](https://openrouter.ai/docs/api/api-reference/chat/send-chat-completion-request)
- **Strict mode**: varies per provider — no static "which models support it" list in prose; filter the
  models catalog by `supported_parameters=structured_outputs` and/or set `provider.require_parameters:
  true`. Per our model research: confirmed present for `google/gemini-3.5-flash-lite` and
  `nex-agi/nex-n2.5-mini:free`; **NOT present** for either `inclusionai/ling-3.0-flash-fin:free` or
  `inclusionai/ling-3.0-flash-sante:free`.
- **Response Healing plugin** (adjacent robustness feature): `"plugins": [{"id": "response-healing"}]`
  auto-repairs malformed JSON (missing brackets, markdown fencing, trailing commas, unquoted keys) for
  `json_schema`/`json_object` responses — **non-streaming only**, can't fix `max_tokens` truncation.
  [Response Healing](https://openrouter.ai/docs/guides/features/plugins/response-healing)
- **Tool/function schema** (standard OpenAI-style, auto-transformed for providers lacking native
  `tools` support):
  ```json
  {"type": "function", "function": {"name": "...", "description": "...", "parameters": {"type": "object", "properties": {...}, "required": [...]}}}
  ```
- **`tool_choice`**: `"auto"` (default), `"none"`, `"required"` (support "varies by model" — test before
  depending on it), or forced: `{"type": "function", "function": {"name": "specific_tool"}}`.
- **Parallel tool calls**: `"parallel_tool_calls": false` (default `true`).
- **Wire format**: assistant message `"tool_calls": [{"id": "call_abc123", "type": "function",
  "function": {"name": "...", "arguments": "{...json string...}"}}]`; tool result: `{"role": "tool",
  "tool_call_id": "call_abc123", "content": "..."}`.
- **Server-side tools** (distinct concept): `openrouter:`-prefixed tool types (`openrouter:web_search`,
  `openrouter:datetime`, shell/bash execution, image generation, subagent delegation) OpenRouter executes
  server-side — mixable with our own function-defined tools in the same request. [Server
  Tools](https://openrouter.ai/docs/guides/features/server-tools)
- **NOT DOCUMENTED**: exact incremental tool-call-argument SSE delta streaming shape.

## 5. Latency & Performance / Provider Routing / Streaming

**Provider-routing object (`provider.*`)** — [Provider Selection](https://openrouter.ai/docs/guides/routing/provider-selection):

| Param | Behavior |
|---|---|
| `order` | ordered provider slug list to try |
| `allow_fallbacks` (default `true`) | permits backup providers beyond `order` |
| `require_parameters` | only route to providers supporting all requested params |
| `data_collection` (`"allow"`/`"deny"`) | restrict to non-data-retaining providers |
| `zdr` | restrict routing to Zero-Data-Retention endpoints only |
| `only` / `ignore` | allow-list / deny-list of provider slugs |
| `quantizations` | filter by quant level |
| `sort` | `"price"` \| `"throughput"` \| `"latency"` |
| `preferred_min_throughput` / `preferred_max_latency` | thresholds, percentile form e.g. `{"p90": 40}` |
| `max_price` | ceiling $/M tokens |

**Default routing algorithm** (no `sort`/`order`): (1) prioritize providers with no significant outage in
the last 30s, (2) weight by inverse-square of price among stable providers, (3) remainder become
fallbacks. Setting `sort`/`order` disables this.

**Multi-model + provider-sort interaction**: with both `models` (fallback list) and provider `sort`,
OpenRouter groups endpoints by model first — primary model's endpoints always tried before any fallback
model's. `partition: "none"` disables grouping for global cross-model sorting.

**Latency formula**: `Total Latency = TTFT (Network + Queue + Prefill) + (Output Tokens / Generation
TPS)`. Low overhead attributed to Cloudflare Workers edge routing. Tuning examples use **p90 only**
(`preferred_max_latency: {"p90": 2.5}` seconds, trailing 5-min window) — **no official p50/p99 platform
benchmark published**; [openrouter.ai/compare](https://openrouter.ai/compare) explicitly does not show
latency/throughput/uptime. Stable `session_id` in agent loops improves provider KV-cache hit rate,
cutting prefill delay "up to 80–90%." [Latency &
Performance](https://openrouter.ai/docs/guides/best-practices/latency-and-performance)

**Streaming (SSE)**: `"stream": true`. Format `data: {json}\n\n`, terminated by `data: [DONE]`;
keep-alive comments `: OPENROUTER PROCESSING` must be discarded (SSE spec: lines starting with `:` are
comments). One extra usage-carrying chunk sent just before `[DONE]` (empty `delta.content`) — so
`finish_reason` can appear twice. Mid-stream errors arrive as an SSE event with `finish_reason: "error"`
rather than an HTTP status. Recommended libs: `eventsource-parser`, OpenAI SDK, Vercel AI SDK. Stream
cancellation supported for OpenAI/Anthropic/Fireworks; **not** for AWS Bedrock or
Groq. [Streaming](https://openrouter.ai/docs/api_reference/streaming)

## 6. Reasoning

Use the **Chat Completions shape** (below) — matches typical OpenAI/Groq/Ollama-style
integration. [Reasoning Tokens](https://openrouter.ai/docs/guides/best-practices/reasoning-tokens)

```json
"reasoning": { "effort": "high", "max_tokens": 2000, "exclude": false, "enabled": true }
```

- `effort` enum: `max`, `xhigh`, `high`, `medium`, `low`, `minimal`, `none` — ≈95%/95%/80%/50%/20%/10%/off
  of `max_tokens`.
- `max_tokens` — alternative direct token-budget control (Anthropic-style providers).
- `exclude` — hide reasoning text from the response, **billing still applies**.
- `enabled` — turn reasoning on with provider defaults.
- `summary` sub-field (`auto`/`concise`/`detailed`/`null`).
- **Provider support**: OpenAI (`effort`, o1/o3/GPT-5), Anthropic (`max_tokens`), **Google/Gemini 3
  (`effort`, mapped internally to `thinkingLevel`)**, xAI (`effort`), DeepSeek (`effort`, R1), Alibaba
  Qwen (`max_tokens` → `thinking_budget`).
- **Billing**: reasoning tokens billed as ordinary output tokens (no separate discounted rate), count
  against `max_tokens` alongside visible output.
- **Reporting**: non-streaming → `choices[].message.reasoning_details`; streaming →
  `choices[].delta.reasoning_details`; usage → `usage.completion_tokens_details.reasoning_tokens`. Legacy
  aliases: `message.reasoning`, `reasoning_content`, request-side `include_reasoning` (superseded).
- Separate beta Responses API variant exists but has a narrower confirmed parameter set — treat the Chat
  Completions guide as authoritative.

## 7. Model Fallback / Routing (`models` array — different model IDs)

**Confirmed exactly what we need**: the `models` array **mixes completely different model
families/providers**, not just alternate providers of the same model. [Model
Fallbacks](https://openrouter.ai/docs/guides/routing/model-fallbacks):

```json
{
  "models": ["~anthropic/claude-sonnet-latest", "gryphe/mythomax-l2-13b"],
  "messages": [...]
}
```

"If the first model returns an error, OpenRouter will automatically try the next model in the list." —
directly validates our Gemini → Ling → Nex → Ling fallback chain of unrelated model IDs.

- **Auto-fallback triggers**: context-length validation errors, moderation flags, **rate-limiting
  (429)**, downtime. No special client handling needed for 429-triggered fallback — automatic.
- **Pricing**: billed using whichever model actually served the request (`model` field in response). If
  every fallback errors too, that final error is returned (no infinite retry).
- **Distinction from `provider.order`**: `provider.order`/`sort` keeps the same model, changes upstream
  *provider* (provider-level failover, on by default); `models` array changes the model itself. No single
  canonical page states this side-by-side — synthesized across [Model
  Fallbacks](https://openrouter.ai/docs/guides/routing/model-fallbacks) and [Provider
  Selection](https://openrouter.ai/docs/guides/routing/provider-selection).
- **Anthropic-shim caveat**: the `/messages` endpoint has its own separate `fallbacks` param (max 3, 400
  beyond that) that **cannot combine with `models`** — not relevant if using standard Chat Completions.
- **Auto Router** (separate feature): dynamic model-picker classifying prompts into ~30 task types,
  selecting by trailing-7-day community spend share within a cost-tier band
  (`low/medium/high/xhigh/max`) — distinct from manual `models` lists. Exact invocation slug **NOT
  DOCUMENTED** (only `openrouter/free` "Free Models Router" found).

## 8. Everything Else

**Authentication**: `Authorization: Bearer <OPENROUTER_API_KEY>`. Keys at openrouter.ai/keys, optional
per-key credit limit, OAuth PKCE flow available. [Authentication](https://openrouter.ai/docs/api_reference/authentication)

**Rate limits** — two categories: credit limits (balance) and rate limits (frequency/free-tier
caps/DDoS). [Limits](https://openrouter.ai/docs/api_reference/limits)

Free (`:free`) model limits, credit-gated:

| Lifetime credits purchased | Requests/min | Requests/day |
|---|---|---|
| < $10 | 20 | 50 |
| ≥ $10 | 20 | 1,000 |

Daily counter resets UTC midnight; 20/min cap fixed regardless of tier; ≥$10 one-time lifetime purchase
raises daily cap to 1,000. Multiple accounts/keys don't bypass this — global per-account. Cloudflare DDoS
protection can throttle regardless of tier. `402` = negative balance/exhausted key limit; `429` may be
OpenRouter's own limit or passed through from upstream (latter case → `models`-array fallback applies
automatically). New accounts get "a small free allowance" — exact figure **NOT DOCUMENTED**.

**Pricing/billing**: "OpenRouter never marks up the pricing of the underlying providers." Credits are
pre-purchased. Purchase fees: Stripe 5.5% ($0.80 min), Coinbase/crypto 5%. Refunds within 24h for unused
credits (platform fees non-refundable, crypto never refundable). Unused credits may expire after one
year. [FAQ](https://openrouter.ai/docs/faq)

**BYOK (Bring Your Own Key)**: route through your own provider account. **Fee: 5%** of normal
OpenRouter cost, deducted from OpenRouter credits — free allowance before 5% applies: **$25,000/month**
(Pay-as-you-go) / **$200,000/month** (Enterprise) of list-price inference spend. Supported: OpenAI, Azure
(AI Foundry + OpenAI), AWS Bedrock, Google Vertex AI. Keys are "Prioritized" (tried before OpenRouter's
own) vs "Fallback" (tried after); per-key "shared capacity fallback" toggle; filters by
model/API-key/member evaluated before routing. [BYOK](https://openrouter.ai/docs/guides/overview/auth/byok)

**Presets**: reusable named configs (model, provider prefs, system prompt, params, tools), referenced as
`"model": "@preset/slug"` or combined `"model": "openai/gpt-4@preset/email-copywriter"`. Manageable via UI
or API. Request-level params shallow-merge over preset values. [Presets](https://openrouter.ai/docs/guides/features/presets)

**Errors**:
```typescript
type ErrorResponse = { error: { code: number; message: string; metadata?: Record<string, unknown>; } };
```
HTTP status mirrors `error.code`. Documented: 400, 401, 402 (insufficient credits), 403 (moderation), 408
(timeout), 429 (rate limited, `Retry-After` header), 502 (model down/invalid upstream), 503 (no provider
meets routing requirements). Moderation errors carry `error.metadata.{reasons, flagged_input (≤100
chars), provider_name, model_slug}`. Provider errors carry `error_type` +
`provider_code`. [Errors](https://openrouter.ai/docs/api_reference/errors-and-debugging)

**Headers (app attribution)**: `HTTP-Referer` (app URL, needed for ranking/attribution),
`X-OpenRouter-Title`/`X-Title` (display name, must pair with `HTTP-Referer`),
`X-OpenRouter-Categories` (≤2/request, ≤10/app, ≤30 chars), `X-OpenRouter-App-Visibility: hidden`
(exclude from public rankings, personal analytics kept, cannot change after app creation). [App
Attribution](https://openrouter.ai/docs/app-attribution)

**Generation/usage stats**: `GET /api/v1/generation?id=<generation_id>` — `total_cost`,
`cache_discount`, `upstream_inference_cost`, `provider_name`, `latency`, `generation_time`,
`finish_reason`, `native_finish_reason`, token breakdowns, `native_tokens_cached`, `num_search_results`,
`is_byok`, `provider_responses` (fallback chain), `session_id`, `data_region`. Synchronous response
`usage` already includes native-tokenizer `prompt_tokens`/`completion_tokens` without a follow-up
call. [Get Generation](https://openrouter.ai/docs/api/api-reference/generations/get-generation)

**Privacy / Zero Data Retention** — directly relevant, exam-question content is privacy-sensitive:
- OpenRouter itself: does not use inputs/outputs for training; doesn't persist prompt/response content
  unless you opt into logging; image/audio/video not persisted beyond routing duration (exceptions:
  abuse/security/billing/legal). [Privacy Policy](https://openrouter.ai/privacy)
- **Model-provider-level logging is separate and varies** — some providers may train on your data;
  opt-out toggles are separate for paid vs free models at openrouter.ai/settings/privacy. [Provider
  Logging](https://openrouter.ai/docs/guides/privacy/provider-logging)
- Providers classified: "Retained for X days," "unknown period," "Zero retention," "Unknown retention
  policy" (live from `/api/frontend/v1/all-providers`).
- **ZDR is first-class**: account-level (per model-family toggle), org-guardrail level
  (`enforce_zdr_anthropic` etc.), or per-request `"provider": {"zdr": true}`. [ZDR](https://openrouter.ai/docs/guides/features/zdr)
- `"provider": {"data_collection": "deny"}` restricts to non-retaining providers without full ZDR
  semantics; `enforce_distillable_text` filters by distillation permission.
- **Critical limitation**: "ZDR enforcement only applies to provider routing for inference requests. It
  does not apply to plugins and tools you choose to enable, such as web search." Enabling the web-search
  plugin can route data through non-ZDR third parties (Exa/Perplexity/etc.) *even under strict ZDR* — **factor
  this in if enabling web search alongside sensitive exam content**.
- In-memory caching is explicitly **not** "retention" under OpenRouter's ZDR stance — caching remains
  usable under ZDR.
- Enterprise: in-region routing (`eu.openrouter.ai`/`us.openrouter.ai`), can route exclusively to ZDR
  providers.

**Terms of Service — automation/abuse**: no separate Acceptable Use Policy; conduct rules live in
[ToS](https://openrouter.ai/terms) §7: prohibits scraping/crawling of **OpenRouter's own site**
(§7.5/7.6), "Red Teaming" (prompt injection/jailbreaking without prior written approval, §7.11), network
interference (§7.12d). §3.2/§9 allow suspension "for any reason or no reason," unused credits forfeited
on violation. No documented escalation ladder tied to automated-usage detection beyond "sole discretion."
**Note**: since ShadowPrompt automates Google Forms (not OpenRouter itself), §7.5/7.6 don't apply to our
browser-automation subsystem — but §7.11 (Red Teaming ban) is worth keeping in mind generally, even
though adversarial prompting isn't our use case.

---

## Model-Specific Capability Data

### `google/gemini-3.5-flash-lite` (PRIMARY) — no free tier
- **Context**: 1,048,576 input / 65,536 max output, identical across all 8 endpoints.
- **Modalities**: input text+image+video+file+audio → output text.
- **Supported params**: `reasoning`, `include_reasoning`, `reasoning_effort`, `response_format`,
  `structured_outputs`, `tools`, `tool_choice` (all 4 modes), `max_tokens`, `seed`, `stop`.
- **8 provider endpoints, all Google-operated**: Vertex AI (global/EU/US × flex/standard/priority) and
  Google AI Studio (flex/standard/priority). Pricing $0.15–$0.54/M prompt, $1.25–$4.50/M completion
  depending on tier; standard tier ≈ $0.30/$2.50/M, image $0.30/M. Cache read $0.000015/M on standard.
- **Uptime**: 99.82–99.98% across endpoints.
- **Throughput/latency**: `null` on live API for all 8 endpoints at fetch time — **NOT DOCUMENTED** as a
  measured figure. A marketing claim of "150+ tok/s" exists for the launch (not a docs page).
- **Rate limits**: no model-specific cap found (paid) — general account credit-tier limiting only.
- **8x provider redundancy** — real OpenRouter-side failover if one Google backend degrades.

### `inclusionai/ling-3.0-flash-fin:free`
- Context: 262,144 in / 32,768 out. Text-only. Free.
- Supports: `reasoning`, `tools`, `tool_choice` (all 4 modes), sampling params. **`response_format`/
  `structured_outputs` NOT supported.**
- **Single provider endpoint: Novita only** — no OpenRouter-side failover.
- Uptime: 99.87% (1d), 100% (30m/5m).

### `nex-agi/nex-n2.5-mini:free`
- Context: 262,144 in / 235,929 max completion. Input text+image → text. Free.
- Supports `reasoning`, `tools`, `tool_choice` (all 4 modes), **and `response_format`/
  `structured_outputs`** — only one of the three free models confirmed for both tool-calling and
  structured output together.
- **Single provider endpoint: Nex AGI itself** (bf16) — no failover redundancy.
- Underlying architecture (third-party source): 35.1B-param Qwen3.5-based MoE.
- Uptime: 99.96% (30m) but only **95.86% (1d)** — notably lower daily uptime than the Ling models.

### `inclusionai/ling-3.0-flash-sante:free`
- Same profile as Ling Fin: context 262,144/32,768, text-only, `reasoning`/`tools`/`tool_choice`
  supported, **`response_format`/`structured_outputs` NOT supported**.
- **Single provider endpoint: Novita only.**
- Uptime: 100% (30m figure available).

### Free-tier rate limit policy (uniform across all three `:free` models)
20 req/min always; 50/day if <$10 lifetime credits, 1,000/day if ≥$10 (one-time unlock). Global per
account, not bypassable via multiple keys.

### Architecturally significant finding
Gemini 3.5 Flash Lite has 8x redundant provider endpoints (real failover). All three free fallback models
have exactly **one** provider endpoint each — if Novita (serving both Ling models) or Nex AGI's infra goes
down, there's no OpenRouter-side failover for that model, only our own `models`-array fallback to the
*next model in the list*. **Two of three free fallbacks share a single upstream (Novita)** — a Novita
outage takes out two of three fallback options simultaneously. Worth weighting fallback order accordingly
(put Nex first or accept the shared-upstream risk knowingly).

---

## Master List of NOT DOCUMENTED Items (flagged, not fabricated)

- Web search: `recency`/`freshness` control; a fully self-hosted (non-Firecrawl) custom search backend.
- Official p50/p99 platform latency benchmarks (only p90 as a request-parameter threshold).
- Exact incremental tool-call-argument SSE delta streaming payload shape.
- Exact invocation slug for "Auto Router" (only `openrouter/free` Free Models Router found).
- A single canonical page stating `models`-array vs `provider.order` distinction directly.
- Exact dollar value of new-account free allowance.
- Static list of exactly which models support `strict: true` (docs say query dynamically).
- Universal numeric image/video/audio size/count/duration limits ("varies per model/provider").
- Full Responses-API reasoning parameter parity with Chat Completions.
- Moderation/`is_moderated` flag in model-endpoints JSON for any of the four researched models.
- Measured throughput (tok/s) for any of the four target models via live API (`null` at fetch time).
- Content of trust.openrouter.ai (SafeBase trust center) — found via search only, not fetched.
