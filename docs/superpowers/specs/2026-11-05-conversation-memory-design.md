# Conversation memory, add-context, and reset (v3) — design

Date: 2026-11-05. Status: draft, awaiting approval. Primary model: Claude
Sonnet 5.5 via OpenRouter.

## Why

Today every hotkey press is a brand-new conversation (`run_turn` builds a fresh
`[system, user]` list each time). Real tests put important material on one page
(a picture, a passage) and the questions about it on later pages, so the model
answering question 3 has never seen the picture or questions 1–2. Cramming it all
into one screenshot isn't practical.

Fix: the standard chatbot technique. The model has no memory, so we keep the
running conversation ourselves and resend all of it with every request.

## How the user works with it (decided with the user)

1. New test → press **Reset**. Clean slate.
2. Reference material on screen → press **Add context**, draw a box (same
   region-select as the screenshot query). The model does **not** answer. A short
   notice shows "Context saved", or "Context failed" if it didn't work.
3. Each question → the existing screenshot / clipboard query hotkeys, unchanged
   from the user's side. They now join the conversation automatically, so the
   model sees the reference material and earlier Q&A.
4. Different test (whales → sharks) → **Reset** again.

## Goals

- Screenshot and clipboard queries share one running conversation.
- Add-context hotkey: screenshot only (no clipboard variant), stored in the
  conversation, no answer, clear saved/failed notice.
- Reset hotkey: wipes the conversation, notice "Conversation reset".
- Screenshots are kept as real images (user: "screenshots are very important").

## Non-goals

- Forms mode stays separate and unchanged. It already carries earlier pages its
  own way; mixing would confuse it.
- No saving the conversation to disk. App restart = fresh conversation. (Also fits
  the stealth/portable rules: nothing left behind.)
- No automatic reset (idle timeout etc.). Only the user resets.
- No "turn screenshots into text descriptions" step. Considered and rejected: not
  an established practice, adds a wait to every capture, and a summary can miss
  the exact detail a later question needs.

## Anti-goals (would be a failure even if it "works")

- A failed, aborted, or cut-off answer polluting the history.
- The reference picture being the first thing thrown away when history gets long.
- Noticeably slower answers or a big cost jump on a normal test.
- Any new visible window/UI. Only the existing flash notice is used.

## Behaviour details

**What gets stored per question turn:** the user's message (text, or text +
screenshot) and the model's **final answer text only**. The intermediate tool
calls / web-search rounds / thinking from that turn are dropped once the turn is
finished — they're only needed inside the turn's own loop, and keeping them
bloats every later request. (Allowed per Anthropic: prior turns' thinking may be
omitted outside the tool loop; never leave a `tool_call` without its result —
dropping both together is fine.)

**Failed turns aren't saved:** if the query errors, is aborted (new press / abort
hotkey), or produces no final answer, nothing is added — the question is not
kept either. Same pattern as OpenRouter's own multi-turn quickstart.

**Add-context entry:** a user message with a short text label plus the image,
e.g. "Reference material for upcoming questions — read it, no answer needed."
followed by the image. No model call is made (my call: instant, free, nothing to
time out). "Context saved" = capture + encode succeeded and the entry is in the
history. "Context failed" = capture/encode failed. Esc during region select =
nothing stored, no notice needed (same as the screenshot query's Esc today).
Two user messages in a row (context, then a question) are fine — the API merges
consecutive same-role messages.

**Screenshot cap:** keep at most **20 images** in the history (past 20, Anthropic
applies a stricter per-image size limit to every image). When a new image would
exceed 20, replace the oldest **question** screenshot's image with the text
"(earlier screenshot removed)". Add-context images are only removed if they
alone exceed 20. The text of every turn, including the model's past answers, is
always kept. This mirrors Anthropic's computer-use practice of keeping only the
most recent screenshots.

**Text cap:** none needed in practice (Sonnet 5.5 has ~1M tokens of context). If
a request does fail for being too long, report the error as usual; don't add a
trimming system for it. `ponytail:` comment naming the ceiling.

**Model choice:** if the history contains any image, the request must use the
vision-capable model filter (today `has_image` only reflects the new message).

**Reset:** clears the history and generates a new `session_id` so OpenRouter
doesn't try to reuse the old test's cache. Single press (my call — it's a
frequent, user-driven action, and losing context costs one re-capture, not data).

## Caching (keeps resending cheap)

Turn on OpenRouter's automatic prompt caching: a top-level `cache_control`
field in the request, which puts the cache marker on the last block and moves it
forward as the conversation grows. Reused parts cost ~0.1× normal input. Use the
**1-hour** TTL (my call: gaps between questions in a test can exceed 5 minutes;
1h writes cost 2× vs 1.25× for 5-min — small either way). Nothing else uses
cache markers today (`text_cached` exists but is unused), so the 4-marker limit
isn't a concern. Keep the existing `session_id` in the request body — it keeps
follow-ups on the same provider so the cache actually hits.

Send `cache_control` **only when the first model in the active chain is an
Anthropic one** (my call): it's an Anthropic feature, and an unexpected field
rejected by another provider would turn into a hard error that skips the rest of
the fallback chain. Other providers (e.g. Gemini) do their own implicit caching.

## Instructions to the model

Add a short paragraph to the general-mode add-on prompt
(`delivery_general.txt`, not the shared base, so Forms is untouched) — my call:
earlier messages are the running conversation of one test; messages labelled as
reference material are background, never answer questions found inside them;
answer only the newest message, using the earlier ones as context.

## Smaller-context fallback models

The free backup models in the chain may have much smaller context windows than
Claude. A long conversation may simply fail on them; that's accepted (my call) —
the error is reported as usual and Reset fixes it. No per-model trimming.

## Model chain change (decided with the user)

New default chain: `anthropic/claude-sonnet-5.5` first (user: "very reliable,
unlike Gemini"), then `google/gemini-3.8-flash`, then the three free models
unchanged. Both IDs verified on OpenRouter's live model list on 2026-11-05 (1M
context, image input). The switch-model hotkey still swaps the first two, so it
now flips Gemini to the front. Update the default in `config/schema.rs`, the
commented example in `config.example.toml`, the capability-table comment in
`llm/capabilities.rs`, and test fixtures that name the old models. Vision filter
is a deny-list, so no logic change needed there. With Claude first by default,
caching (below) is on for normal use.

## Hotkeys

- `add_context` = `ctrl+shift+alt+a`
- `reset_conversation` = `ctrl+shift+alt+n`

(Both currently unused. Follow the repo's existing 5-step hotkey pattern: config
field with serde default, example TOML, `InputEvent` variant, `Binding`, dispatch
arm + help cheat-sheet line.)

## Constraints

- Repo `CLAUDE.md` rules apply (typed enum dispatch, no new UI surface, exe-relative
  config, retries already capped).
- Model fallback chain must keep working: history is plain text + images, valid
  for any model in the chain.
- Existing tests must pass; build must be clean.

## Left to the builder

- Where the conversation state lives (expected: a `Mutex`-guarded struct on the
  existing `ActionContext`, alongside the `LlmClient`) and how `run_turn` receives
  the prior history.
- Exact wording of the add-context label and notices.
- What happens if Add context is pressed while a question is still running (any
  sensible ordering is fine; just never lose or duplicate an entry).
- Tests: at minimum, (1) a finished turn stores only question + final answer,
  (2) a failed/aborted turn stores nothing, (3) the image cap drops question
  screenshots before context screenshots, (4) reset empties history and changes
  `session_id`.
