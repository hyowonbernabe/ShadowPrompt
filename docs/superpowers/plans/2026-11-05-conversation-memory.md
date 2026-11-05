# Conversation Memory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Screenshot/clipboard queries share one running conversation; add-context and reset hotkeys; Sonnet 5.5 first; prompt caching on for Claude.

**Architecture:** A `Conversation` (list of finished turns) lives on `ActionContext` behind a `std::sync::Mutex`. Queries snapshot it into messages, call a new `LlmClient::run_turn_with_history`, and append the turn only on success. Region-select carries a `RegionPurpose` so one selection mechanism serves both query and add-context.

**Tech Stack:** Rust 2021, tokio, serde, existing OpenRouter client.

**Spec:** `docs/superpowers/specs/2026-11-05-conversation-memory-design.md`

## Global Constraints

- Default chain: `anthropic/claude-sonnet-5.5`, `google/gemini-3.8-flash`, then the 3 free models unchanged.
- Image cap 20; replacement text `(earlier screenshot removed)`; question images drop before context images.
- `cache_control` top-level `{"type":"ephemeral","ttl":"1h"}` only when `models[0]` starts with `anthropic/`.
- Hotkeys `add_context = "ctrl+shift+alt+a"`, `reset_conversation = "ctrl+shift+alt+n"`, both `#[serde(default)]`.
- Forms paths unchanged (they keep calling `run_turn`, no history).
- No new UI surface; notices via `UICommand::FlashNotice`.

## Review Focus

1. Abort/new press mid-answer → that turn not stored (future dropped before append).
2. Add context pressed while a question is in flight → context not lost, not aborting the question (runs outside `answer_task` slot).
3. History has an image but the new message is text-only → vision filter still applied.
4. 21st image → oldest *question* image replaced, context image kept.
5. Reset → history empty and next request uses a new `session_id`.

---

### Task 1: Model chain swap
**Files:** `src/config/schema.rs` (default chain + doc comment), `config/config.example.toml`, `src/llm/capabilities.rs` (comment + test IDs), `src/llm/mod.rs` (`toggle_primary_model` → `String`, returns `active_models()[0]`), `src/actions/switch_model.rs`.
- [ ] Update; `cargo test` passes.

### Task 2: `Conversation` type
**Files:** Create `src/llm/conversation.rs`; `pub mod conversation;` in `src/llm/mod.rs`.
**Produces:**
- `pub enum TurnKind { Context, Question }`
- `pub struct Conversation` with `fn push_context(&mut self, content: Vec<ContentPart>)`, `fn push_question(&mut self, question: Vec<ContentPart>, answer: String)`, `fn to_messages(&self) -> Vec<Message>`, `fn has_image(&self) -> bool`, `fn clear(&mut self)`, `fn len(&self) -> usize`.
- Pushes enforce the 20-image cap.
- [ ] Tests: `question_turn_stores_question_and_answer_only`, `cap_drops_question_images_before_context_images`, `clear_empties_history`, `has_image_reflects_history`.

### Task 3: History-aware turn + session reset + caching
**Files:** `src/llm/mod.rs`, `src/llm/request.rs`.
- `pub async fn run_turn_with_history(&self, system_prompt: &str, history: Vec<Message>, initial_content: Vec<ContentPart>, tools, has_image, on_delta) -> anyhow::Result<String>`; `run_turn` delegates with empty history.
- `session_id: Arc<std::sync::Mutex<Arc<str>>>`; `pub fn new_session(&self)`.
- `Request.cache_control: Option<CacheControl>`, set in `build` per Global Constraints.
- [ ] Tests: request JSON has `cache_control` for an anthropic-first chain and lacks it for a gemini-first one; `new_session` changes the id; history messages appear before the new user message in the sent body.

### Task 4: Queries join the conversation
**Files:** `src/actions/mod.rs` (`conversation: Arc<std::sync::Mutex<Conversation>>` on `ActionContext`), `src/lib.rs`, `src/actions/clipboard_query.rs` (+ test ctx), `src/actions/screenshot_query.rs`.
- Snapshot `to_messages()` + `has_image()` before the call; `push_question` only after `Ok`.

### Task 5: Add-context + reset hotkeys
**Files:** `src/input/events.rs` (`RegionPurpose { Query, Context }`; `ScreenshotRegion` gains `purpose`; new `AddContext`, `ResetConversation`), `src/input/mod.rs` (store purpose when arming), `src/input/bindings.rs`, `src/config/schema.rs`, `config/config.example.toml`, `src/actions/mod.rs`, create `src/actions/add_context.rs`, `src/actions/help_toggle.rs`.
- Add-context: capture like screenshot_query (shared helper extracted from it), `push_context([text label, image])`, `FlashNotice("Context saved")` / `("Context failed")`. Spawned as a plain task, not in `answer_task`.
- Reset: `conversation.clear()`, `llm.new_session()`, `FlashNotice("Conversation reset")`.

### Task 6: Prompt + verify
**Files:** `src/llm/system_prompts/delivery_general.txt`.
- Add the conversation/reference-material paragraph from the spec.
- [ ] `cargo build --release`, `cargo test`, `cargo clippy` clean. Commit.
