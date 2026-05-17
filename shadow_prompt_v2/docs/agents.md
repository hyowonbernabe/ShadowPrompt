# ShadowPrompt v2 — Agents

> Source of truth for everything related to the LLM and how ShadowPrompt interacts with it.
> Covers transport, model selection, model agnosticism, system prompt design, request shape, multi-step reasoning flows, answer formatting, vision handling, streaming, extended thinking, and failure modes.
> Last updated: 2026-05-17

---

## Purpose

This document defines how ShadowPrompt talks to the model. If anything in the codebase touches the LLM — message construction, system prompt, request shape, retry behavior, image encoding, streaming, thinking — the design intent for that behavior lives here. Code may reference this document. When code and this document disagree, this document is authoritative until updated.

---

## Transport and Model Selection

- **Transport:** OpenRouter only. All requests go through `https://openrouter.ai/api/v1/chat/completions` using the OpenAI-compatible message format that OpenRouter exposes.
- **Default model:** the latest Claude Sonnet available on OpenRouter (Sonnet 4.6 or successor). Claude is the family ShadowPrompt is tuned for — system prompts, retry behavior, and feature toggles are validated against Claude first.
- **Model selection:** chosen via `config.toml` field `model_id` (e.g. `anthropic/claude-sonnet-4.6`, `google/gemini-2.5-pro`, `openai/gpt-4o`).
- **Hard requirement:** the chosen model must support image inputs natively. The daemon validates this on startup by checking OpenRouter's model metadata; if the model is text-only, the daemon exits with an error pointing to the config field.
- **No model cycling at runtime.** Changing models requires editing `config.toml` and restarting the daemon (the restart hotkey is appropriate here).

---

## Model Agnosticism

ShadowPrompt v2 is biased toward Claude but must work with any multimodal OpenRouter model. Implementation rules:

1. **Use the standard chat-completions schema.** Stick to the OpenAI-compatible request format that OpenRouter normalizes across providers. Avoid Anthropic-specific endpoints (no direct `/v1/messages` calls) unless the gain is overwhelming.
2. **Detect provider-specific capabilities and toggle accordingly.** Features that exist on Claude but not on Gemini (e.g., `cache_control`) must be applied conditionally. The daemon inspects the model id prefix (`anthropic/`, `google/`, `openai/`, etc.) and enables provider-specific request fields only when appropriate.
3. **Capability flags live in code, not config.** A small table mapping model id → supported features (vision, thinking, caching, etc.) is maintained in the codebase and updated when OpenRouter adds new models.
4. **Never assume a capability is present.** Extended thinking, prompt caching, structured output, and any other provider-specific feature are silently skipped if the current model does not support them. The daemon does not error out; it just sends a plainer request.
5. **Tune the system prompt for the lowest-common-denominator behavior.** The system prompt must produce correct answer-mode output on Claude, Gemini, and GPT-4o without per-provider variants.

---

## Request Shape (per query)

All queries — clipboard, OCR, and each step of a Forms run — produce a single chat completion request with the same general shape:

```json
{
  "model": "<configured model id>",
  "messages": [
    { "role": "system", "content": "<system prompt>" },
    { "role": "user", "content": [<question content parts>] }
  ],
  "stream": false,
  "max_tokens": 4096
}
```

Provider-specific extras layered on top when supported:

- **Anthropic (Claude):** include `cache_control: { type: "ephemeral", ttl: "1h" }` on the system message so the system prompt is cached for repeated use within an exam session. Also set `extra_headers` if needed for the 1M-context beta.
- **Extended thinking:** add `reasoning: { effort: "medium" }` (OpenRouter's normalized field) or the provider's native equivalent. See "Extended Thinking" below.
- **Anything else:** kept out unless a tangible win exists for the specific use case.

---

## Conversation Model

ShadowPrompt v2 is **stateless** for clipboard and OCR queries and **ephemerally stateful** for Google Forms runs.

### Clipboard and OCR queries — stateless

Each query is a single, independent request. The full message array is:

```
[
  { role: "system",    content: <system prompt> },
  { role: "user",      content: <this question, possibly with image> }
]
```

There is no history. There is no carry-over between queries. There is no token counter. There is no auto-wipe logic. The previous answer is not visible to the next query.

### Google Forms run — ephemeral context

A single Forms execution (one press of the auto-paginate or single-page hotkey through to flow completion or abort) maintains a short-lived in-memory conversation:

```
[
  { role: "system",    content: <forms-specific system prompt> },
  { role: "user",      content: <page 1 questions + images> },
  { role: "assistant", content: <page 1 answers as JSON> },
  { role: "user",      content: <page 2 questions + images> },
  { role: "assistant", content: <page 2 answers as JSON> },
  ...
]
```

Rules:

- The conversation lives only in memory for the duration of the form flow.
- It is **discarded the moment the flow ends** — successful completion, user abort, page cap reached, or fatal error.
- It is never shared with clipboard or OCR queries (those are stateless even while a Forms flow is in progress).
- Page cap (default 10 pages) bounds total tokens naturally; no need for trimming logic.

Rationale: forms commonly reference earlier pages ("using the data from the previous section"). Without ephemeral context, the model would give inconsistent or contradicting answers across pages. With it, consistency improves measurably while the token cost stays small (a 10-page form is typically under 30k tokens total).

---

## System Prompt

The system prompt establishes answer-mode behavior. It is the most important piece of model-facing configuration in v2.

### Core rules the system prompt must enforce

1. **Every user message is a question, not a chat message.** Treat every incoming user turn as an exam or quiz prompt. There is no conversation to participate in — only questions to answer.

2. **Output only the answer.** Do not greet, acknowledge, restate the question, narrate reasoning, or offer follow-up help. The answer is the entire response.

3. **No conversational references to prior turns.** When context exists (Forms ephemeral conversation), use it silently. Never say "as I mentioned earlier," "from the previous problem," "in our earlier discussion," or anything similar.

4. **Format the answer based on the question type.** The model decides the format dynamically:
   - **Multiple choice (A/B/C/D, or numbered options):** respond with the letter or number only. No explanation unless explicitly asked. Example: `B`.
   - **True / false:** respond with `True` or `False`. Nothing else.
   - **Short answer / fill in the blank:** respond with the minimal correct phrase. No surrounding sentence.
   - **Numeric:** respond with the number. Include units only if the question specifies units. No commas in numbers above 999 unless culturally required by the question's language.
   - **Definition / explanation question:** respond with a tight paragraph. No bullet lists unless the question explicitly asks for a list.
   - **Code:** respond with a single fenced code block. No prose around it unless the question requires explanation.

5. **Confidence handling.** If genuinely uncertain between two options on a multiple-choice question, the model still picks one. It does not output "B or C" or "I'm not sure." Pick the most likely answer.

6. **No refusals on academic content.** This is an assistant for exam-style content. Standard refusal patterns ("I cannot help with cheating") are explicitly suppressed for in-scope content. Refusals only apply to genuinely harmful content (weapons, malware, etc.).

7. **Language matching.** Respond in the same language as the question.

### Conceptual shape

A single short, declarative prompt — no examples, no roleplay framing. Approximate shape:

> You are answering exam questions. Every message is a question. Output only the answer in the appropriate format for the question type. Multiple choice: letter only. True/false: word only. Short answer: minimum correct phrase. Numeric: number only, units only if specified. Code: single fenced block. Never greet, restate, explain unprompted, or reference prior questions. Use prior questions as silent context only when present. If uncertain on multiple choice, pick the most likely option. Respond in the language of the question.

The actual prompt text lives in the codebase, embedded at compile time, and may be tuned over time. Substantive changes are reflected in this document.

### Forms variant

A separate Forms-specific system prompt extends the base rules with:

- Structured output instructions (answers returned as a JSON object keyed by question id).
- Instruction to use earlier pages' answers as silent consistency context.
- Reminder that the form may include images as primary context.
- Reminder to never reference pages by number ("page 1", "earlier") — just answer.

---

## Multi-Step Reasoning Flows

### Clipboard query (single-step)

```
User hotkey
  → read clipboard
  → build stateless request (system + this question)
  → request (streaming)
  → write final answer to clipboard + overlay
```

Single round trip. No tool use, no intermediate calls.

### OCR query (single-step, vision)

```
User hotkey
  → region selection (two-click)
  → capture PNG of region
  → encode as base64
  → build stateless request (system + user message with image part + optional OCR text part)
  → request (streaming)
  → write final answer to clipboard + overlay
```

The OCR text extracted from `Windows.Media.Ocr` is optional context. The primary signal is the image itself. Including OCR text as a parallel text block can help when image clarity is poor; this is a tuning knob, not a strict requirement.

### Google Forms (multi-step, agentic)

This is the most complex flow in v2.

```
Pre-flight:
  - Ensure Chrome debug session exists on port 9222
    (launch incognito if absent, attach cookies via rookie)
  - Initialize ephemeral Forms conversation (system prompt only)

Per page:
  1. Inject EXTRACTOR_JS into the current form page
  2. Read structured JSON describing every question on the page:
     - question id
     - question type (radio, checkbox, dropdown, text, date, grid, etc.)
     - question text
     - answer options (where applicable)
     - any image URLs embedded in or around the question
     - **current answer state** — is this question already filled in by the user?
  3. Filter out questions the user has already answered.
     If every question on the page is already answered, skip the model call
     entirely and move on to step 7.
  4. For each image URL on remaining (unanswered) questions, fetch image bytes
     from the page via DevTools.
  5. Build a user turn containing:
     - The list of unanswered questions for this page (structured text)
     - All images for those questions, attached as message content parts
     Append the turn to the ephemeral Forms conversation.
  6. Request the model. Append the assistant response to the conversation.
     The assistant returns answers as a JSON object keyed by question id.
  7. Inject answers back into the form DOM via DevTools
     (click radio, check checkbox, type into text field, select dropdown).
     Never touch already-answered questions.
  8. If auto-paginate mode and a "Next" button exists, click it and loop.
     If "Submit" button is the only forward control, STOP — do not click it.
     If single-page mode, STOP after step 7.
```

### Critical Forms behaviors

- **Skip already-answered questions.** Detection happens via DOM state — checked radios, filled inputs, selected dropdowns. The daemon never re-asks the model about an answered question and never overwrites the user's manual choices.
- **Never submit.** The daemon recognizes the Submit button and treats it as untouchable. Auto-paginate advances via Next; when only Submit remains, the flow stops cleanly. The user submits manually when they are ready. This is non-negotiable — submission is the user's act, not the daemon's.
- **Structured output.** The model is asked to return answers as JSON keyed by question id. This is the one place in v2 where structured output is requested, and it is done via the system prompt instructing the format, not via OpenRouter's structured-output API (model-portable, no per-provider variants).
- **Image dimensions.** Before sending, images are clamped to a maximum dimension to control token cost. Exact limit is a tuning knob, deferred.
- **Page cap.** Auto-pagination stops after 10 pages or when only Submit remains, whichever comes first. Configurable.
- **Abort.** The user can cancel at any time with the abort hotkey. The active tokio task handle is aborted; the browser is left in whatever state it was in (no rollback). The ephemeral Forms conversation is discarded.

---

## Vision and Image Handling

The image policy is model-agnostic — sized to work well on Claude, Gemini, and GPT-4o without per-provider tuning.

- **Transport:** images are sent as base64-encoded data URLs in the message content array, using the OpenAI-style multimodal message format that OpenRouter normalizes across providers.
- **Multiple images per turn:** supported. A Forms page with multiple diagrams produces a single user turn with all image parts plus the text part.
- **Resize policy:** every image is resized in-process so its longest edge is at most **1568 px**, preserving aspect ratio. This matches Claude's recommended size for fastest time-to-first-token and stays well within Gemini's and GPT-4o's optimal token-cost ranges. Smaller images pass through unchanged.
- **Format:** PNG. JPEG is not used because most screen captures contain text and benefit from lossless compression. All target providers support PNG.
- **Per-request image cap:** **20 images**. Above this, Claude forces an additional 2000×2000 downscale across all images; staying under the threshold preserves the chosen resize policy.
- **Total request size cap:** target under **18 MB** per request. Gemini's 20 MB ceiling is the smallest among supported providers; the 2 MB margin covers the non-image payload (messages, system prompt, headers).
- **Batching for >20-image pages:** if a single Forms page contains more than 20 images, split the page's questions across multiple sequential requests within the same ephemeral Forms run. The assistant's earlier-batch answers remain in the conversation, preserving consistency. This case is rare in practice.

---

## Adaptive Thinking

When the configured model supports model-side reasoning (Claude 4.x extended thinking, Gemini 2.5/3 Pro thinking, GPT-5 reasoning, etc.), the daemon enables it on every request. The model itself decides how deeply to reason.

### How adaptivity works

Modern reasoning models self-modulate their internal thinking depth based on the difficulty of the input. A trivial MCQ produces a short or near-zero reasoning trace; a multi-step math or logic problem produces a longer one. The daemon does not classify difficulty itself — it simply requests reasoning with a generous upper bound and lets the model decide how much of that budget to use.

### Request shape

Sent on every request to a reasoning-capable model:

```json
"reasoning": {
  "effort": "high",
  "exclude": true
}
```

- `effort: "high"` — OpenRouter's normalized field. Modern models interpret this as an upper bound; they self-cap internally and rarely use the full budget on easy questions.
- `exclude: true` — reasoning tokens are billed but not returned in the response payload. The daemon does not need them; only the final answer matters. This also saves bandwidth.

For Claude specifically, this maps to `thinking: { type: "enabled", budget_tokens: <high> }`. OpenRouter handles the translation.

For models that do not support extended thinking, the `reasoning` field is silently dropped before sending (capability detection happens at the daemon layer based on model id).

### No configuration knob

Reasoning effort is **not configurable in `config.toml`**. The daemon always requests high effort and relies on the model's internal adaptivity. Exposing a knob would either reduce accuracy (low effort) or waste tokens (medium effort with no real benefit) — neither matches the goal of "the model thinks as hard as the question deserves."

### Why prefer model-side adaptive thinking over daemon-side multi-step

- **One API call, not many.** Daemon-orchestrated chain-of-thought multiplies latency, cost, and failure surface.
- **Already adaptive.** Modern reasoning models think more on hard problems and less on easy ones — no need to reinvent this logic outside the model.
- **Model-portable.** OpenRouter's `reasoning` field abstracts across providers.
- **Higher accuracy.** Reasoning models reliably outperform single-shot calls on math, logic, and code.

---

## Response Delivery (Non-Streaming)

- **Transport:** OpenRouter chat completions endpoint, non-streaming (`stream: false`). The daemon waits for the full response before acting on it.
- **Clipboard write:** the full final answer is written to the clipboard in one shot once the response arrives.
- **Overlay:** the full final answer is rendered to the text overlay in one shot. No character-by-character streaming.
- **Concurrent queries:** if the user fires another query while a previous one is still in flight, the previous request future is dropped (cancelled). Since clipboard and OCR queries are stateless, no history bookkeeping is needed. For an in-flight Forms run, the abort hotkey is the supported way to cancel; firing a new clipboard or OCR query does not abort the Forms run.
- **Reasoning tokens:** when adaptive thinking is enabled, the reasoning trace is requested with `exclude: true` so it never reaches the daemon. The daemon receives only the final answer.

Streaming was considered for v2.0 and dropped as unnecessary complexity for the exam-use case. May be revisited in a later version.

---

## Failure Modes and Retry Policy

| Failure | Behavior |
|---|---|
| Network timeout | Retry up to 3 times with exponential backoff (1s, 2s, 4s). |
| HTTP 429 (rate limit) | Retry up to 3 times, honoring `Retry-After` header when present. |
| HTTP 500–504 | Retry up to 3 times with exponential backoff. |
| HTTP 401 / 403 | No retry. Show error on indicator (red), log "Auth failure — check API key" to the log file. |
| HTTP 400 (bad request) | No retry. Log the error. Indicates a bug in message construction. |
| Response truncated (finish_reason: length) | Use the partial content as the answer. Log warning. No automatic retry — the user can rerun if needed. |
| Model returns empty content | Treat as failed. Indicator turns red. No retry. |
| Invalid JSON in Forms structured response | Single retry asking the model to fix the JSON. If still invalid, abort the form flow with a visible error. |
| Model rejects request (e.g., refusal) | No retry. Show error. User can adjust the question or model. |

After three retries, the operation fails visibly (red indicator, error message on overlay). No silent fallback to another provider — there is no other provider in v2.

---

## Logging

- All LLM requests and responses are logged at `debug` level to `data/logs/app.log` (or the equivalent v2 location).
- API keys are **never** logged.
- Image payloads are logged as metadata only (dimensions, byte size), never the base64 bytes.
- Reasoning tokens are logged at `trace` level only (off by default in release).
- Logs rotate at a sensible size (e.g., 10MB per file, keep 3 files).

---

## Future Considerations (Not in v2.0)

- Tool use / function calling for richer Forms automation.
- Per-question retry-with-different-prompt on confidence failures.
- Local fallback model for offline use.
- Multi-model routing (cheap model for trivial questions, premium for hard).
- Caching of common answers across sessions.

These are out of scope for v2.0 and are documented here only so they are not forgotten if v2.1 is planned.
