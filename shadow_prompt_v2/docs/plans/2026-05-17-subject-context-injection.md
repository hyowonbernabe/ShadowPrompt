# Subject context injection (Path A: caching)

Date: 2026-05-17

## Goal

User drops markdown notes into a `knowledge/` folder. Daemon loads them, prepends to every clipboard / vision / forms request as reference material. Prompt caching keeps the per-call cost manageable.

## Research findings

OpenRouter docs confirm:

- **Format**: `cache_control: { "type": "ephemeral" }` attached to a content block.
- **Two modes**:
  - *Automatic*: one `cache_control` at top level of request. System auto-advances breakpoint as conversation grows. Best for multi-turn.
  - *Explicit*: per-content-block `cache_control`. Max **4 breakpoints**. Best for static large blocks.
- **We use explicit.** Our knowledge block is static; we want full control.
- **TTL**: 5 min default; `"ttl": "1h"` extends to 1 hour. 1-hour works across all Claude providers (Anthropic, Bedrock, Vertex).
- **Pricing (Anthropic Claude)**:
  - Cache write 5-min TTL: 1.25x normal input cost
  - Cache write 1-hour TTL: 2x normal input cost
  - Cache read: 0.1x normal input cost
- **Provider compatibility**:
  - Automatic caching forces routing to Anthropic direct (excludes Bedrock/Vertex)
  - Explicit `cache_control` works on Anthropic + Bedrock + Vertex
  - We're already pinning to Anthropic via provider preferences, so either works for us
- **Sticky routing**: after first cached request, OpenRouter pins the same model to the same provider so the cache stays hot
- **Verification**: response `usage` includes `prompt_tokens_details.cached_tokens` (and Anthropic-style `cache_creation_input_tokens` / `cache_read_input_tokens`) so we can log hit rate
- **Recommended use**: "large bodies of text such as character cards, CSV data, RAG data, book chapters" — fits us exactly

## Decision: explicit breakpoint, 1-hour TTL

- Exam sessions typically last 1-3 hours. 5-min TTL would re-write the cache every gap between questions. 1-hour TTL writes once per session.
- Write fee for 100k tokens: 2.0x at $3/1M = $0.60 once per hour
- Read fee within hour: 0.1x at $3/1M = $0.03/query
- Expected cost for a 50-query session: 0.60 + 50 * 0.03 = ~$2.10 (vs $15 without caching)

## Architecture

### Files on disk

```
<install_dir>/
  shadowprompt.exe
  config/
    config.toml
  knowledge/
    _default/             # always-on (optional)
      common.md
    english/
      grammar.md
      poetry.md
    biology/
      cells.md
      genetics.md
```

- Dirs are subject names. Files are markdown.
- `_default/` is the only special name; auto-loaded if present.
- Empty `knowledge/` = feature inert.

### Config

```toml
[knowledge]
enabled = true
active_subjects = ["english"]   # in addition to _default
cache_ttl = "1h"                # "5m" | "1h"
max_chars = 800000              # safety cap (~200k tokens)
```

### Loader (`src/knowledge/`)

- `loader.rs::load(install_dir, cfg) -> KnowledgeBundle`
  - Walks `_default/` + each active subject dir
  - Reads `*.md` files, sorts alphabetically
  - Concatenates with headers: `# <subject>/<filename>\n<content>\n\n`
  - Enforces `max_chars` cap; warns + truncates if exceeded
- Returns `{ text: String, source_count: usize, char_count: usize }`

### Message format upgrade

Current `Message::System { content: String }` cannot carry `cache_control`. Anthropic format requires the system to be a content array when any block is cached.

Two options:
1. **Promote system to a content array unconditionally.** OpenRouter accepts this for Anthropic and OpenAI-compat. Cleanest.
2. **Keep string when no cache needed, switch to array when caching.** More variants, more code paths.

Choose option 1. New `Message::System { content: Vec<ContentPart> }`.

`ContentPart::Text` gains an optional `cache_control` field:
```rust
pub enum ContentPart {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    Image { image_url: ImageUrl },
}

pub struct CacheControl {
    #[serde(rename = "type")]
    pub kind: &'static str, // "ephemeral"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<&'static str>, // "5m" | "1h"
}
```

### Per-call system structure

```json
{
  "role": "system",
  "content": [
    { "type": "text", "text": "<answer mode rules>" },
    {
      "type": "text",
      "text": "<reference material concatenated>",
      "cache_control": { "type": "ephemeral", "ttl": "1h" }
    }
  ]
}
```

Knowledge block is the second item. The cache marker on it tells Anthropic to cache everything up to and including this block. So the prompt prefix (rules + knowledge) becomes a cached prefix.

### Where it gets injected

- `LlmClient` owns `Arc<KnowledgeBundle>`. Constructed from config at startup.
- `answer_text` and `call` build the system message as a content array. If knowledge bundle is empty, single block (no cache marker — sub-1024 tokens of rules wouldn't cache anyway). If non-empty, two blocks with the second marked.
- Probe action skips knowledge injection (test the bare model).
- Forms answer_mode_forms swaps the rules block but keeps the knowledge block. Same cache key.

### Cache hit logging

- Parse `usage` field from response
- Log `cache_write_tokens`, `cache_read_tokens` per call
- After first call you'd see `cache_write: N, cache_read: 0`
- On subsequent calls: `cache_write: 0, cache_read: N`

### Subject switching

- v1: config-pinned only. Edit `active_subjects`, restart daemon.
- v2 (optional later): hotkey to cycle. Out of scope for first cut.

### Pre-installed notes

- Commit your `.md` files to `shadow_prompt_v2/knowledge/<subject>/`
- Release workflow copies the whole `knowledge/` tree into the release zip
- `install.ps1` extracts as-is alongside the exe
- Users override or add their own files post-install

If you want yours private: don't commit to the public repo. Instead place files manually in `<install_dir>/knowledge/` post-install. Or set up a private bundle URL the installer fetches; deferred.

## Implementation order

1. Schema: add `KnowledgeConfig` section
2. `src/knowledge/loader.rs`: walk dirs, concat files, char cap
3. Message types: promote system to content array; add `CacheControl`
4. Request builder: unchanged signature; serializer carries the new fields
5. `LlmClient::new` accepts knowledge bundle; system-prompt assembly uses it
6. `clipboard_query`, `ocr_query`, `forms/mod.rs`: build system as `Vec<ContentPart>` with cached knowledge block
7. Log cache usage from response
8. Config example: add `[knowledge]` section
9. `knowledge/` dir created with empty `_default/` placeholder
10. Release workflow: copy `knowledge/` into zip

Time estimate: 1 day. No new external deps.

## Open questions

- **TTL**: 1h fixed, or expose as config? Config, default `"1h"`.
- **Forms scope**: inject knowledge into Forms too? Yes — exam contexts benefit most there.
- **Probe**: skip knowledge? Yes — probe tests bare model capability.
- **Token estimation**: cheap char/4 heuristic enough, or use a real tokenizer? Heuristic enough for cap; rely on server-side usage for true counts.
- **Multiple subjects active**: do they share one cache block or get separate blocks? One block. Simpler, single cache key.

## Risk / failure modes

- **Cache miss on every call**: maybe content varies subtly (whitespace, file order). Mitigation: normalize content at load (strip trailing whitespace, sort files deterministically).
- **Block under 1024 tokens**: caching silently disabled. Mitigation: log expected/actual token count; warn if too small to cache.
- **Provider routing breaks cache**: OpenRouter routes to a different provider that doesn't have the cache. Mitigation: our existing provider pinning to `anthropic` for `anthropic/*` models handles this.
- **Knowledge bundle grows past context window**: 1M token window on Sonnet 4.6 is generous, but big notes could still bust. Mitigation: `max_chars` cap with refusal + log.

## Out of scope (next iteration)

- Hotkey to cycle active subject
- Per-feature subject overrides (e.g. forms uses biology, clipboard uses english)
- Automatic subject detection
- RAG retrieval (if/when Path A hurts)
- Cache pre-warming on startup (send a no-op query to write cache before first user query)
