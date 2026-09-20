# ShadowPrompt v3 — Phased Build Plan

> Companion to `docs/SHADOWPROMPT_V3_DESIGN.md` (the *what and why*) and `docs/SHADOWPROMPT_V3_DESIGN.md`
> §16 "Scaffold status" (the *what's actually real right now*). This doc is the *build order* —
> one milestone at a time, each independently shippable and testable before the next begins, so
> no single step is ever "rewrite half the crate." Every milestone below cites which design-doc
> section it implements and which scaffold-status stub it's replacing, so there's never a
> question of whether a task is inventing new scope or just finishing something already decided.

## Working rules

- **One milestone at a time.** Don't start M(n+1) while M(n)'s exit criteria haven't been run and
  confirmed. This is the whole point of phasing it — no milestone should ever require touching
  more than one or two of the "structurally wired but stubbed" areas from the scaffold-status
  list at once.
- **Every milestone ends with `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
  clean, plus its own stated manual exit criterion actually run** — not just "the code compiles."
  A milestone that only compiles is not done.
- **If a milestone's own scope grows while working it, split it, don't expand it.** Same rule
  the old v2 roadmap had, worth keeping.
- **Every open question the design doc flagged gets resolved inside the specific milestone that
  touches it**, not deferred silently. Three are flagged in design-doc §16: the `provider.order`
  multi-model scoping question (M2), whether `openrouter:web_search`'s engine selection matches
  the plugin's (M5), and what Forms' save-confirmation signal actually looks like (M8).
- **Commit per milestone**, not per file. A milestone is the unit of "this is a working,
  demonstrable increment."

---

## M0 — Prove the scaffold actually runs — REAL BUG FOUND HERE, FIXED

**Update from actually running it**: `config/paths.rs` only resolved config/knowledge/log
directories exe-relative, dropping the "exe-relative first, then CWD" fallback the original
v1/v2 design documented. This is invisible to every mocked/unit test (none of them go through
real path resolution against a real exe location) and only showed up by literally running
`cargo run -- --probe`, where `cargo run` puts the built exe under `target/debug/` — nowhere
near the crate root where `config/`/`knowledge/` actually live. Fixed: `resolve_dir()` now
checks exe-relative first, falls back to CWD if that doesn't exist yet; `--init`/first-run
template-writing still always targets the exe-relative path specifically (via a new
`config_write_path()`), so a real `--init` on a real install never accidentally writes into CWD
instead of the portable layout. This is exactly the kind of bug this milestone existed to catch
— confirms the milestone's own reasoning, not just its checklist.

**Goal**: everything that currently only has compile-time verification (§16: "cargo check/test/
clippy all clean") gets an actual runtime check. This is the cheapest possible milestone and it
comes first because if the daemon can't even boot cleanly, nothing built on top of it matters.

- `cargo run -- --init`, confirm `config/config.toml` gets written, edit in a real key.
- `cargo run` (no `--debug`): confirm no console window appears (design doc §13 — silent in
  prod), confirm the indicator pixel actually renders at the configured corner, confirm it's
  click-through/non-interactive (doesn't steal focus).
- Press every one of the 13 hotkeys from a config with harmless/no-op-safe bindings temporarily
  assigned, confirm each logs (`--debug` build) that its `InputEvent` fired and reached
  `dispatch()` — this doesn't require the LLM to work yet, just confirms the whole input → action
  wiring is alive end to end.
- `help_toggle`: confirm the cheat-sheet overlay renders with all 13 rows, correct labels.
- `panic_kill`: confirm clipboard clears and the process exits immediately.
- `insta_delete`: confirm the double-tap arm/confirm window actually behaves as designed (tap
  once, wait past 2s, confirm it disarms rather than firing on a third unrelated tap).

**Exit criteria**: every hotkey observably does *something* (even if that something is "logs and
does nothing yet" for the ones later milestones build out) and the process starts/stops cleanly
in both debug and release-profile builds.

**Depends on**: nothing — this is checking what's already scaffolded.

---

## M1 — The LLM core loop, for real — DONE

**Status**: verified independently (cargo check/clippy/test all clean, 21/21 tests). Real bug
found and fixed in `retry.rs`: it was retrying every error indiscriminately, including
deterministic HTTP 4xx failures the module's own doc comment says it should never retry (that's
OpenRouter's `models`-array job) — fixed via a `Transport`/`Fatal` error split, 3 new tests. 7 new
tests total using `wiremock` cover plain-text responses, a real tool-call round trip (asserting
the actual second request body includes the tool result), the `MAX_LOOP_ROUNDS` backstop firing
with an exact request-count assertion, and a real transport-failure-with-backoff case.

**Post-M1 fix (live testing, real Forms `answer_all` run)**: the "should never fire in normal
use" 25-round backstop fired in entirely normal use — a genuinely long form legitimately needed
more tool-calling rounds than that, and hitting the cap forced a tools-stripped final call that
itself then failed outright (`openrouter stream ended with finish_reason=error`). Per the user's
explicit instruction, `MAX_LOOP_ROUNDS` and the whole backstop path are **removed** — `run_turn`'s
loop is now unbounded, relying on the system prompt's existing "answer directly, don't
over-search" guidance and the manual Abort hotkey instead of a code-enforced cap. The backstop
test was replaced with one asserting the loop keeps servicing tool calls well past the old
25-round limit (30 rounds) before returning the model's real final answer, with no cutoff. See
design doc §2. Verified: `cargo check`/`clippy --all-targets -- -D warnings`/`test` clean, 54/54.


**Goal**: `run_turn` (design doc §2) actually talks to OpenRouter and gets a real answer back.
Replaces the "hasn't been exercised against a real OpenRouter response yet" stub status (§16).

- Wire a real API key, confirm `LlmClient::call` gets a real HTTP 200 and `Response` parses.
- Get `clipboard_query` working fully end to end with **no tools attached yet** (pass an empty
  tool slice) — copy a plain trivia question, hit the hotkey, confirm a real answer comes back
  and gets written to the clipboard + shown on the overlay.
- Confirm the retry/backoff path (`llm/retry.rs`) actually retries on a real transient failure —
  easiest way to test this without waiting for a real outage: temporarily point `ENDPOINT` at an
  unreachable host for one manual test run, confirm 3 attempts with backoff, then revert.
- Confirm the `MAX_LOOP_ROUNDS` backstop path (§2) is reachable in principle — doesn't need a
  real infinite loop, just confirm the code path that forces a tools-disabled final call compiles
  and would fire correctly (a quick unit test with a mocked always-tool-calling response is
  reasonable here, this is exactly the kind of pure-logic path worth a real test).

**Exit criteria**: `clipboard_query` on a real plain-text question works end to end against the
live OpenRouter API, with a real answer landing in the clipboard and on the overlay.

**Depends on**: M0.

---

## M2 — Model fallback + the vision filter, for real — DONE (real bug found + fixed during live testing)

**Post-verification update, found only by actually running `--probe` live**: OpenRouter caps a
`models` array at 3 entries (`400 — 'models' array must have 3 items or fewer`), undocumented
anywhere this project found beforehand. `LlmClient::call` now chunks the 5-model chain into
groups of ≤3 and falls through to the next chunk on a hard failure — see design doc §8 for the
full writeup and why it happens to align cleanly with the vision filter. Verified live against
the real API after the fix: text/vision/tools all PASS.

**Status**: verified. `provider.order` question resolved with a citable structural answer (not
a guess): OpenRouter's Chat Completions schema has `provider` as a top-level field, a sibling of
`models`, with no per-entry provider block anywhere in the schema — a single request can only
carry one `provider` object, so it structurally cannot be scoped to just the Anthropic entry of
a 5-model chain. Pinning it would apply `["anthropic"]` request-wide, starving routing for the
non-Anthropic models. Left unpinned, `provider: None`, now for a documented reason. Design doc
§8 updated to match.


**Goal**: prove the 5-model fallback chain and the vision-capability filter (design doc §8,
`llm/capabilities.rs` — already unit-tested for its pure logic, not yet proven against a real
multi-model request) actually behave as designed against the real API. Also resolves the first
open question from §16: whether `provider.order` pinning is safe on a multi-model request — this
milestone is where that gets answered, not guessed at.

- Confirm a plain-text request really does carry the full 5-model `models` array and OpenRouter
  accepts it (no 400 for the array shape itself).
- Force a fallback deliberately (e.g. temporarily set the first model in config to an invalid
  slug) and confirm OpenRouter actually falls through to the next one — check the response's
  `model` field to see which one actually answered.
- `switch_model`: confirm the toggle actually changes which model leads, confirmed via the same
  response `model` field, and confirm pressing it twice returns to the original order.
- Get `screenshot_query` working end to end with a real image — confirm the vision-filter
  actually drops the two Ling models from the chain used for that request (add a temporary log
  line showing the filtered vs. unfiltered chain length during this milestone's testing, remove
  it after).
- **Resolve the provider-pinning question**: test whether adding `provider.order: ["anthropic"]`
  to a request that also includes non-Anthropic models in its `models` array breaks routing for
  those other models, or is safely scoped to just the Anthropic entry. Do this as a deliberate,
  isolated test (a request with the full 5-model chain, first forced to Anthropic, then to
  Gemini, with the pin present in both) before deciding whether to add the pin back. Whatever the
  answer, write it into design-doc §8 as a resolved fact, not left as an open question anymore.

**Exit criteria**: `screenshot_query` works end to end on a real screenshot; a forced-bad primary
model still produces a real answer via fallback; the provider-pinning question has a documented,
tested answer either way.

**Depends on**: M1.

---

## M3 — `test_model` and `switch_model` polish — `test_model` half DONE

**Status**: `test_model`'s ground-truth surfacing is done and verified — a new additive
`LlmClient::simple_call` (single non-looping call, no tools, returns both the model's self-report
and the real `Response.model`) was added specifically to avoid touching `run_turn`'s signature
(which would have rippled into every other caller mid-parallel-build). `run_turn` confirmed
byte-for-byte untouched. `switch_model`'s `FlashNotice` auto-clear is folded into the M10 agent's
work instead (same file, sequenced there rather than run in parallel on `ui/manager.rs`).


**Goal**: close the two small gaps §16 flagged for these two already-scaffolded actions.

- `test_model`: thread the ground-truth `model` field out of `run_turn` (or add a small
  dedicated non-tool-loop single-call path just for this action, if that's cleaner than changing
  `run_turn`'s signature for one caller) so the overlay actually shows both the model's own
  self-report and the ground-truth model id, as designed (§3).
- `switch_model`'s `FlashNotice`: currently reuses `SetOverlayText` permanently (§16) — add
  actual auto-clear-after-a-few-seconds behavior so it behaves like a transient notice, not a
  sticky overlay message that lingers until the next real answer overwrites it.

**Exit criteria**: pressing `test_model` shows two distinguishable pieces of information (its own
answer to "what model are you," and the real one); pressing `switch_model` shows a brief
confirmation that clears on its own.

**Depends on**: M1 (needs real responses flowing through `run_turn`, not the M2-specific
fallback-chain/vision-filter work — M2 doesn't gate this one, corrected after an independent
check flagged the original "depends on M2" as overstated).

---

## M4 — Memory tool, wired for real — DONE

**Status**: verified. `KnowledgeStore` threaded into `ActionContext`, constructed once at daemon
startup, `memory_tools()` returns real tools gated on `cfg.knowledge.enabled` (previously dead
config, now actually wired). Tests use a real temp directory and exercise the actual sandbox
(including a real escape-attempt-rejected test), not mocks. 24/24 tests pass.


**Goal**: `list_docs`/`read_doc` (design doc §4) are already real, tested, sandboxed logic
(`knowledge/mod.rs`) — but `actions/clipboard_query.rs::memory_tools()` is still a stub returning
an empty `Vec` (§16). Thread the real `KnowledgeStore` through.

- Add a `KnowledgeStore` (or `Arc<KnowledgeStore>`) to `ActionContext`, constructed once at
  daemon startup from `cfg.knowledge` + the exe-relative `knowledge/` dir.
- Update `memory_tools()` to return real `ListDocsTool`/`ReadDocTool` instances instead of the
  empty stub.
- Put a real test doc in `knowledge/`, ask a clipboard question that requires it, confirm the
  model actually calls `list_docs` then `read_doc` and the answer reflects the doc's content.
- Also confirm the "answer directly by default" posture from the system prompt (§9) actually
  holds — ask a question that doesn't need the knowledge folder at all, confirm it answers
  without calling either tool (this is a real behavioral check on the prompt, not just plumbing).

**Exit criteria**: a clipboard/screenshot query correctly uses a real knowledge doc when relevant,
and correctly skips the tools entirely when not.

**Depends on**: M1 (tool-calling loop must already work).

---

## M5 — Web search tool, verified — DONE

**Status**: resolved with citation, not left open. `openrouter:web_search`'s own docs
(openrouter.ai/docs/guides/features/server-tools/web-search, cross-checked against the plugin's
page) document an identical `engine` parameter with the same default: `auto` = "native search if
the provider supports it, otherwise falls back to Exa" — the same semantics already verified for
the `{"id":"web"}` plugin, just relocated into the server tool's own `parameters` shape. Design
doc §5 updated to drop the hedge.


**Goal**: resolve the second open question from §16 — does `openrouter:web_search`'s server-side
engine selection (native-vs-Exa) actually behave the way the plugin-based mechanism was verified
to behave, or does it need separate confirmation?

- Ask a clipboard question that clearly needs current information the model wouldn't have
  memorized, confirm it actually calls `openrouter:web_search` (not just answers from stale
  training knowledge) and the result comes back usable.
- Confirm it does **not** fire on an ordinary question that doesn't need it — this is the actual
  point of using a server tool instead of the always-on plugin (design doc §5's correction), and
  it needs to be observed, not assumed.
- Check whatever's inspectable about which engine actually served the search (native vs. Exa) for
  a Gemini-primary vs. a Ling-fallback request if that's visible in the response/annotations, and
  write the answer into design-doc §5, resolving the flagged uncertainty either way.

**Exit criteria**: a current-events question gets a real, correct, current answer via the search
tool; an ordinary question provably does not trigger a search; the engine-selection question is
answered and documented, not left open.

**Depends on**: M1.

---

## M6 — Browser engine: real accessibility-tree page reading — DONE

**Status**: verified. `read_page` implemented for real against chromiumoxide 0.9.1's actual
generated CDP bindings — read directly from the crate's checked-in generated source
(`chromiumoxide_cdp-0.9.1/src/cdp.rs`), not guessed. All three design-doc §7.1 bugs are
structurally impossible in this implementation (no branch anywhere that could skip a
heading-less field or filter a `data:` URI), each proven by a dedicated unit test against a
hand-built AX-tree-shaped fixture. `find_active_forms_url` implemented (first open tab matching
Forms' URL shape, same pattern v2 used). Honestly flagged as unverified: the exact `AXRole`
strings a live Chrome actually emits for Google Forms (hedged with case-insensitive matching) —
cannot be checked without a live browser, exactly as this milestone anticipated. 21/21 tests
pass, zero clippy warnings.


**Goal**: the single riskiest, most novel milestone in this plan — design doc §7.3 already flags
this as needing hands-on verification, not a scaffold-pass guess. `browser/forms/read.rs::read_page`
is currently `anyhow::bail!("not yet implemented")` (§16). This milestone is scoped to *reading*
only — no filling, no page navigation yet, those are M7.

- Get `chromiumoxide` actually connecting to a `launch_debugger`-opened Chrome instance and
  finding the currently-active tab (this also implements `find_active_forms_url`, currently a
  stub).
- Implement `read_page` for real against CDP's Accessibility domain (or whatever chromiumoxide's
  actual API surface turns out to expose for it — verify this hands-on, the design doc explicitly
  didn't pin down the exact call shape).
- Test against a real multi-page Google Form (the same one used for the empirical URL/autosave
  research in design-doc §7.2 is a good target, since its actual structure is already known) and
  specifically verify the three bugs this whole rebuild exists to fix (design-doc §7.1) are
  actually fixed:
  1. A page-level title/description gets picked up as `page_context`, not silently dropped.
  2. A question with no heading text but an image still shows up as a field, not skipped.
  3. An inline (`data:`-URI) image, if the test form has one, doesn't get filtered out.
- `all_fields_answered` (already real logic, §16) should now be exercisable against real
  extracted data — confirm it correctly identifies a page with a still-blank field.

**Exit criteria**: `read_page` against a real, live Forms page returns a `PageContent` whose
`page_context` and `fields` are correct by inspection, including at least one case each covering
the three bugs above.

**Depends on**: M0 (needs `launch_debugger` proven to actually open a debug-attached Chrome).

---

## M7 — Fill, navigate, and the walk-forward logic — DONE

**Goal**: build the *acting* half of Forms automation on top of M6's now-real reading. Replaces
`fill_page`, `advance_to_next_page`, and the walk-forward loop in `browser/forms/mod.rs` — all
currently real control-flow with stubbed internals (§16).

- Implement `fill_page` for real: label-based fuzzy matching (not exact-string), the live-value
  re-check backstop before acting on any field (design doc §7.3, explicitly re-confirmed —
  grid/matrix rows checked individually, every other question type checked whole-question),
  per-field success/failure reporting back to the model.
- Implement `advance_to_next_page` for real: find a Next-labeled control and click it if present;
  detect a Submit-only state and stop without ever touching it (the one hard non-negotiable, and
  it should be structurally impossible to violate — there is still no submit tool anywhere in
  this codebase at this point, and this milestone must not add one).
- Get `forms_answer_page` working fully end to end on the real test form: confirm it walks
  forward correctly (skips already-answered pages, per the empirically-confirmed autosave
  behavior from design-doc §7.2), stops at the first page with something blank, fills it, and
  stops — without touching any page after it.
- Specifically test the already-answered enforcement: manually pre-fill a couple of fields
  (including at least one partially-filled grid) on the test form before running
  `forms_answer_page`, confirm those exact values are untouched afterward and only the blanks
  got filled.

**Exit criteria**: `forms_answer_page` works correctly end to end on a real form, including the
already-answered/grid-row/checkbox-whole-question distinctions, verified by inspection of the
live page afterward, not just by the tool's own reported success.

**Depends on**: M6.

**Status**: verified (cargo check/clippy/test all clean, 36/36). Element interaction goes AX
`backend_dom_node_id` → `DOM.resolveNode` → `Runtime.callFunctionOn` (chromiumoxide's own
`Element` type is unconstructable from an AX-tree-driven lookup — its constructor is
`pub(crate)` — so this reaches the same underlying mechanism through the public `Page::execute`
surface instead). The never-submit guard was checked at the exact call-site level, not just by
reading comments: `click_node` is invoked only on a Next-matched control; the Submit-matched
branch returns a stop signal with zero interaction, confirmed by reading the literal code path.
Backstop re-checks live state at execution time (grid rows individually, everything else
whole-field), reusing `read::read_page` for whole-field consistency with M6. Honestly flagged,
same as M6: whether Google Forms' actual custom widgets respond to a synthetic `.click()`/value-
set the way a real pointer event would, and whether its Next/Submit controls really carry AX role
`"button"` with literal "Next"/"Submit" names — both are this milestone's own stated,
un-avoidable limit without a live browser.

---

## M8 — `forms_answer_all`, tab lifecycle, and the in-Forms screenshot fallback — DONE

**Goal**: everything M7 built, extended to the multi-page case, plus the tab-safety guards and
the last piece of the Forms tool surface. Resolves the third open question from §16 — what
Forms' actual save-confirmation signal looks like.

- Extend the walk-forward loop to keep going instead of stopping after one page — confirm the
  same conversation/session carries across pages (cross-page memory for consistency, design doc
  §7.3), confirm it stops cleanly the moment only Submit remains.
- Confirm the page-count safety cap (`forms.max_pages`, default 20) is reachable in principle
  without needing an actual 20-page form to test — a unit test on the loop-bound logic in
  isolation is reasonable here.
- **Resolve the save-confirmation question empirically**: interact with the real test form,
  watch for whatever visual/DOM signal actually indicates "saved" (a "Draft saved" indicator was
  observed visually during design-phase research, design-doc §7.2 — pin down the actual
  selector/text to poll for), implement `tab_lifecycle::safe_to_close` for real against that
  signal, defaulting to "not safe" whenever it can't be found, exactly as designed.
- Implement "never close the only tab left": check `browser.pages().len()` (or chromiumoxide's
  equivalent) before closing, confirm behavior manually by closing the user's own tab mid-run and
  observing the background tab correctly refuses to close itself afterward.
- Add the `screenshot(region?)` fallback tool to the Forms tool set (currently just a TODO
  comment in `browser/forms/mod.rs`, §16) — reuses `capture::screen`/`capture::image` from M1's
  proven Screenshot Query path, feeds the result back as tool-call context, not to the shared
  overlay (confirm this doesn't visually collide with a concurrent Clipboard/Screenshot Query, a
  claim design-doc §7.3 flagged as reasoning that was never separately confirmed against a real
  concurrent-run test).

**Exit criteria**: `forms_answer_all` completes a real multi-page form correctly and stops at
Submit; the two tab-safety guards are each demonstrated by deliberately provoking the scenario
they guard against; the screenshot fallback tool is callable and its result reaches the model
without touching the shared overlay.

**Depends on**: M7.

**Status**: verified (cargo check/clippy/test all clean, 49/49). Never-submit path re-confirmed
at the exact `should_stop_walk`/`SubmitOnly` logic level, including an explicit test for
"Submit-only wins even at the last possible page." `safe_to_close`'s two guards both real: the
tab-count guard is fully verifiable and tested; the save-confirmation guard is an honest
best-effort (fuzzy-matches "saved" in the accessibility tree, defaults to "leave it open" if
nothing matches) — the exact real signal remains unverified without a live form, flagged
plainly rather than assumed. `screenshot_tool.rs` added as a real Forms tool.

**Known, flagged simplification — cross-page memory**: `run_turn` has no history parameter
(never did, and adding one now would mean yet another signature change while other work depends
on the current one), so `AnswerAll`'s "same conversation across pages" (design doc §7.3) is
implemented as a flattened text summary of prior pages prepended to each new page's content,
not real multi-turn message history the way v2 had it. This still achieves the functional goal
(later pages see everything already decided) and was a deliberate choice to avoid a second
concurrent edit to `llm/mod.rs` while M11's streaming work was landing in it at the same time —
but it's a real fidelity gap against the original design intent, not a stylistic equivalent.
Upgrade path if this matters later: give `run_turn` (or a sibling method) a real
`history: &[Message]` parameter and have `AnswerAll` build actual alternating user/assistant
turns instead of one growing text blob.

**Post-M8 fix (live testing, real Forms tab)**: `find_active_forms_url` (used by both
`forms_answer_page` and `forms_answer_all`) failed outright even with a real Forms tab open and
focused — root cause was `Browser::pages()` racing chromiumoxide's target-discovery handshake
right after the fresh `Browser::connect()` this function makes on every hotkey press. Fixed with
a short bounded retry. While fixing it, also closed the "first Forms-looking tab wins" gap this
function's own comment had flagged as unverified: it now requires the *focused* tab
(`document.hasFocus()`) to be the Forms tab, failing clearly instead of silently answering a
stale/background one if multiple tabs are open. See design doc §7.3 for the full writeup.
Verified: `cargo check`/`clippy --all-targets -- -D warnings`/`test` clean, 54/54.

**Second post-M8 fix, same testing pass — three iterations, real upstream bug found**:
`open_forms_tab` opened a brand new top-level browser *window* instead of a new tab in the debug
window. A dedicated dev-only test hotkey (`debug_open_tab`, Ctrl+Shift+Alt+Z) was added
specifically to iterate on this without a full Forms/LLM run each time:
1. Bare URL (`new_window` unset) — opens a new window every time.
2. Explicit `new_window(false)` — CDP error -32000 "Failed to open new tab - no browser is
   open" outright.
3. `for_tab(true)` — user-confirmed working (real tab, existing window), but then chromiumoxide
   0.9.1's own handler panics (`Browser::new_page` requires the new target already be in its
   internal map when the `createTarget` response arrives; hard-`panic!`s instead of retrying if
   the separate `Target.targetCreated` event hasn't landed yet — confirmed as a genuine crate bug
   by reading its source, which has its own `// TODO can this even happen?` next to the panic).

**Fix at the time**: keep `for_tab(true)`, stop calling `Browser::new_page` (the panicking path).
Issue the raw `Target.createTarget` via `Browser::execute` (plain passthrough, no special
post-processing), then poll `Browser::get_page` with a short bounded retry for the same
"event hasn't landed yet" race, tolerated instead of asserted impossible. Also (from the first
attempt, kept): `SetFormIndicator`/`FormIndicatorState` (top-right pixel, stacked under the main
indicator) were fully wired in the UI layer since scaffolding but never actually sent —
`forms_run.rs`'s own header comment said so. Wired now: `Running` before `execute_form_flow`,
`Hidden` right after, regardless of outcome. Verified: `cargo check`/`clippy --all-targets -- -D
warnings`/`test` clean, 54/54.

**Correction — this was not actually the final fix.** Live testing hit a new, different error on
the very next real run: `new Forms tab was created but never became attachable: Requested value
not found.` — the retry loop above exhausted all 10 attempts and always would have, no matter how
long it retried. Root cause, found by reading `chromiumoxide::handler::target::Target::poll`
(0.9.1): it unconditionally returns early (`if !self.is_page() { return None; }`) *before* ever
issuing the `Target.attachToTarget` command that sets `session_id` — and `session_id` being `Some`
is the only thing `get_or_create_page` actually waits on. `for_tab(true)` creates a target whose
CDP `type` field is the literal string `"tab"` (Chrome's newer tab-strip-era target kind).
`TargetType::new` only recognizes `"page"/"background_page"/"service_worker"/"shared_worker"/
"other"/"browser"/"webview"` — `"tab"` falls through to `Unknown`, so `is_page()` is false forever
for this target and chromiumoxide structurally never attaches to it. This is a second, separate
upstream gap from the earlier panic, not a variant of the same race — no retry count fixes it.

**Actual final fix**: stop using CDP `Target.createTarget` for this entirely. `open_forms_tab` now
takes the already-attached source page (the one `find_active_forms_url` found) and runs
`window.open(url, '_blank')` as JS inside it — a page-initiated `window.open` creates an ordinary
`"page"`-type target, fully compatible with chromiumoxide's normal attach/poll path, while still
landing as a real tab in the same window (exactly what `window.open` does in a real browser). The
new tab has no CDP target id obtainable from JS, so it's found by diffing `Browser::pages()`
before/after `window.open` for a target id that wasn't there before, with the same short bounded
retry used elsewhere in this file for the `Target.targetCreated` race. `find_active_forms_url` was
refactored to return the source `Page` alongside its URL (was previously discarded after reading
`document.hasFocus()`), and its focused-tab-finding logic was pulled out into a shared
`find_focused_page` helper so `debug_open_tab` (which has no Forms tab to speak of) can find a
source page the same way instead of hard-coding `"about:blank"` with no source page at all — a gap
the previous version of this fix would have hit immediately had it been tested standalone.
Verified: `cargo check`/`clippy --all-targets -- -D warnings`/`test`, both default and `--features
debug`, all clean, 54/54.

**Immediate follow-up bug, same live-testing pass**: the very next run hit `opening new Forms tab
via window.open: Error -32000: Object reference chain is too long`. Root cause: `window.open(...)`
itself evaluates to a `Window` object reference — deeply self-referential (`window.window`,
`window.self`, `window.top` all point back into the same cyclic structure) — and CDP's
`Runtime.evaluate` tries to build a serializable preview of the expression's completion value
before handing the result back, which fails walking a `Window` object's reference graph. Fixed by
appending `; void 0` to the evaluated JS so the expression's completion value is plain `undefined`
instead of the `Window` reference — nothing left to try serializing. Verified: `cargo
check`/`clippy --all-targets -- -D warnings`/`test`, both default and `--features debug`, all
clean, 54/54.

**Third follow-up bug, same live-testing pass — page read before it finished loading**: next run
logged `forms: page 1 — 0 field(s)`, then `page 1 already fully answered, skipping ahead`
(`all_fields_answered` is vacuously true on an empty list), then `advance_to_next_page: found
neither a Next nor a Submit control on this page`. Root cause: `open_forms_tab` hands back the new
`Page` the moment its CDP target is discoverable via `Browser::pages()`, which can be well before
Chrome has actually finished navigating `window.open`'s target to `form_url` — `read_page` was
running against a still-loading/blank document. Fixed with `wait_for_document_ready` in
`tab_lifecycle.rs`: polls `document.readyState === 'complete'` with a short bounded retry (30 ×
200ms) right after the new tab is found, before handing it back to any caller. Best-effort — if it
never reports ready, callers proceed anyway rather than hanging, and any real failure surfaces
downstream same as before.

**Fourth follow-up bug, same live-testing pass — real run, once the tab actually had content**:
with the page-ready fix in place, the model got into a genuine infinite loop calling
`fill_page({"answers":{}})` every round, forever (confirmed live past round 25 with no change and
no cap — `MAX_LOOP_ROUNDS` was removed entirely per earlier explicit request, so nothing was going
to stop this). Root cause: `fill_page`'s tool schema had no `minProperties` on `answers`, so an
empty object is schema-valid, and `fill_page()`'s own loop (`for (field_id, raw_answer) in
answers`) is a no-op on an empty map — it silently returned `"[]"`, which reads to the model as a
harmless success rather than a rejected call, giving it no signal to ever try something different.
Fixed at both layers: `minProperties: 1` added to the tool's JSON schema (rejects it up front for
any model that actually validates tool-call arguments against the schema), and `FillPageTool::
execute` now explicitly errors on an empty `answers` map (`"no answers provided — include at least
one question id..."`) as a backstop for models that don't validate, so the model gets a real
corrective error back instead of a fake-success empty array. This doesn't guarantee a given model
recovers from a genuine capability gap, but it stops a no-op from ever looking like success — the
actual bug, independent of which model is misbehaving.

Verified (all four fixes together): `cargo check`/`clippy --all-targets -- -D warnings`/`test`,
both default and `--features debug`, all clean, 54/54.

---

## M9 — Stealth: capture exclusion + unbranded identity — DONE

**Goal**: design doc §6's two concrete stealth items, both currently explicit no-ops (§16).

**Status**: implemented directly (not via subagent — see build log note below), verified via
`cargo check`/`clippy --all-targets`/`test`, all clean, 7/7 tests passing.
- `apply_capture_exclusion` now real: `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` on all
  four stealth windows, gated on `RtlGetVersion` (not `GetVersionExW`, which lies about the OS
  version above 8.1 without an app manifest) reporting build >= 19041.
- One new Cargo feature needed and added: `Wdk_System_SystemServices` (for `RtlGetVersion`
  specifically — `SetWindowDisplayAffinity` itself needed no new feature, confirmed during
  scaffolding).
- Window class/title renamed from the placeholder `SPv3Wnd`/`SPv3` to `SysIndicatorHost`/`System
  Notification`.
- Not yet done: the actual visual verification (screen-recording tool, confirm invisible) — still
  a manual step for the user, as this milestone always specified.

**Post-M9 fix (live testing)**: exactly that unverified step came back as a real user report —
the answer overlay showed up in a screenshot. Two independent responses, since the root cause
(OS silently not applying the flag, vs. the specific capture tool not honoring it, vs. something
else) isn't yet isolated: `apply_capture_exclusion` now reads back `GetWindowDisplayAffinity`
right after setting it and logs loudly on a mismatch, so the next debug-build run's log says
definitively whether Windows actually recorded the flag. Separately, Screenshot Query's own
region capture now explicitly hides the overlay/help windows immediately before reading the
screen and restores them right after — regardless of the OS-level flag's real status, this
app's own feature can no longer bleed a leftover answer into a brand new capture. Verified:
`cargo check`/`clippy --all-targets -- -D warnings`/`test` clean, 54/54. Still open: which exact
capture method surfaced the leak — needed to close this out for real. See design doc §6.

**Answer, and third post-M9 fix, same testing pass**: Windows Snipping Tool, specifically while
the gradient-crawl effect was animating (not while the overlay was static). `SetWindowDisplayAffinity`
was only ever set once, at window creation — `draw_and_present` now re-asserts it after every
single `UpdateLayeredWindow` call, removing any possibility that crawl mode's repeated
resize/reposition of the layered surface silently drops the exclusion on whatever new backing
surface each frame allocates. Verified: `cargo check`/`clippy --all-targets -- -D warnings`/
`test` clean, 54/54. Still genuinely unverified without a live capture: whether this specific
fix is what closes the gap, since the readback diagnostic from the previous fix hadn't been run
yet when this was made.

- Implement `apply_capture_exclusion` for real: `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`
  on the indicator/overlay/help windows, gated on `RtlGetVersion` reporting build ≥ 19041 —
  confirm the gate itself works (test on whatever Windows build is actually available; if it's
  ≥19041, confirm the flag applies without error; the older-build code path can only be verified
  by inspection unless an older test machine is available).
  - **Already confirmed, no Cargo change needed**: `SetWindowDisplayAffinity`/
    `WDA_EXCLUDEFROMCAPTURE` were tested directly against the real scaffold (a temporary probe
    call, compiled then removed) and compile cleanly under the `Win32_UI_WindowsAndMessaging`
    feature already enabled in `Cargo.toml` — no new feature flag needed. (An earlier draft of
    this plan guessed a new feature would be required; verified and corrected rather than left
    as a guess.)
- Pick and apply the real replacement for the placeholder `SPv3Wnd`/`SPv3` window class/title
  (§16) — anything generic/unbranded works; the specific string is a free choice, not a design
  constraint.
- Manually verify capture exclusion actually works: start a screen-recording tool (or Windows'
  own Snipping Tool / Xbox Game Bar capture) while the daemon is running, confirm the indicator/
  overlay/help windows don't appear in the recording while still being visible normally on the
  actual screen.

**Exit criteria**: the stealth windows are confirmed invisible to at least one real capture tool
on a modern Windows build, and nothing in the process/window list says "ShadowPrompt" anywhere.

**Depends on**: M0 (no functional dependency on the LLM/browser work — this could technically run
in parallel with M1–M8, but is sequenced here so it doesn't compete for attention with the
higher-risk core-loop and browser milestones first).

---

## M10 — Gradient-crawl overlay — DONE (also closes M3's switch_model half)

**Goal**: design doc §6's overlay redesign — currently still v2's plain static paint (§16).

- Switch the overlay window from `LWA_COLORKEY` to `UpdateLayeredWindow` with a real per-pixel-
  alpha ARGB bitmap rendered offscreen.
- Implement word-wrap against the overlay's configured width, and the moving ~1.5-line window
  (line 1 opaque, line 2 fading, line 3+ invisible) driven by a repaint timer, looping back to the
  top at the end — using `crawl_trigger_lines`/`crawl_ms_per_line` from config (already in the
  schema, unused until now).
- Confirm a short answer (under `crawl_trigger_lines`) still displays as a static single paint,
  no crawl — this was an explicit design requirement (design doc §6), not just "always crawl."
  Confirm a long answer actually crawls and loops correctly.

**Exit criteria**: a deliberately long test answer visibly crawls with the fading effect; a short
one doesn't.

**Depends on**: M1 (needs real answers to actually test against, not placeholder text).

**Status**: verified (cargo check/clippy/test all clean, 24/24). Real `UpdateLayeredWindow` +
32bpp ARGB DIB rendering, replacing `LWA_COLORKEY`. Static paint below `crawl_trigger_lines`,
`WM_TIMER`-driven ~1.5-line crawl above it, correct premultiplied-alpha technique for the fade
(verified independently, not just taken on faith). `switch_model`'s `FlashNotice` now genuinely
transient (3s auto-clear, reverts to whatever was showing before) instead of sticking
permanently — closes the second half of M3. Not verified, and cannot be without a live Windows
GUI session: actual visual appearance (fade look, timing feel, on-screen positioning) — still a
manual step for the user.

**Post-M10 fix (found in live testing)**: the M10 agent's own report flagged that it only touched
`hwnd_overlay` and left `hwnd_help` (the cheat sheet) on the old static `WM_PAINT` +
`LWA_COLORKEY` path with a separate `CreateFontW` call. User caught exactly this in live use —
"it's like they were made by different agents" — because the help panel and the answer overlay
really did look and behave differently. Fixed: `hwnd_help` now renders through the identical
`render_help_panel` → `draw_and_present` pipeline as the overlay (same font, same per-pixel-alpha
technique, same corner-anchor helper), `paint_static_panel` deleted as fully dead code, `WM_PAINT`
now a no-op ack for both windows, and `ToggleHide` re-renders the help panel too (it previously
never touched `hwnd_help` at all, so hiding "everything" left an open cheat sheet visible).
Verified: `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` all clean, 54/54
tests passing, zero warnings.

---

## M11 — Real SSE streaming — DONE

**Goal**: design doc §2's streaming commitment, currently non-streaming (`stream: false`, §16).

- Implement real SSE frame parsing against OpenRouter's chat/completions streaming format
  (`docs/OPENROUTER.md` §5's documented shape — keep-alive comment lines, the terminal `[DONE]`,
  the extra usage-carrying chunk before it).
- Accumulate tool-call argument deltas across chunks correctly (a tool call can arrive as
  `tool-input-start`/`-delta`/`-end`-shaped pieces, not as one atomic block) before treating a
  tool call as complete and ready to execute.
- Feed partial text into the overlay as it arrives, flip `stream: true` in the request builder.
- Confirm this doesn't break the tool-calling loop (`run_turn` needs to keep working correctly
  when the assistant's turn is entirely tool-calls with no visible text, or a mix of both).

**Exit criteria**: `clipboard_query` visibly streams text into the overlay token-by-token; the
tool-calling loop still works correctly end to end with streaming on.

**Depends on**: M4/M5 (real tool-calling with actual tools attached — memory and search — needs
to already be proven working non-streaming before adding streaming on top; the Forms-specific
work in M6–M8 isn't actually exercised by this milestone's own exit criteria, so it's not a real
dependency, just conservative over-sequencing in the original draft, corrected after an
independent check).

**Status**: verified (cargo check/clippy/test all clean, 38/38). Real SSE frame parsing (comment
lines discarded, `[DONE]` terminator, the documented double-`finish_reason` quirk handled
correctly), real tool-call argument accumulation across fragmented chunks keyed by `index` with
correct gap-handling, mid-stream `finish_reason: "error"` surfaced as a real fatal error rather
than folded into partial output. `run_turn` gained a trailing `on_delta` callback parameter
(rippled into 3 call sites outside this agent's owned files — `clipboard_query.rs`,
`screenshot_query.rs`, `browser/forms/mod.rs` — each a one-line no-op fix, flagged honestly and
confirmed by the orchestrator to still compile cleanly crate-wide, including alongside M8's
concurrent edits to the same `browser/forms/mod.rs`). Callback isn't wired to the UI overlay yet
— that's a real remaining step for whenever the overlay-streaming UX itself gets built.

---

## M12 — Release polish — PARTIALLY DONE (everything doable without a live key/machine is done)

**Goal**: everything needed to actually ship a v3 release, matching design doc §12.

- Confirm the size-optimized release profile (`opt-level = "z"`, `lto = true`, `strip = true`,
  already in `Cargo.toml`) actually produces a meaningfully smaller binary than a default release
  build — measure both, note the real numbers (don't assert a size claim without measuring it).
  **Measured for real**: release build (with the size-optimized profile) is **5.0 MB**; a dev
  build (unoptimized, with debug symbols) is 25 MB for comparison. Genuinely small, single-EXE,
  USB-portable.
- Implement `--probe` for real (currently a stub, §16) — a lighter, non-daemon version of M1/M2's
  manual testing, automated: round-trip text/vision checks against the configured model chain,
  print pass/fail, matching v2's existing `--probe` shape but updated for the new 5-model chain
  and tool-calling. **Done**: new `src/probe.rs`, uses the same `simple_call`/`run_turn` real
  code paths every other action uses (not a separate parallel implementation) — text round-trip,
  vision round-trip (same red-square-image technique v2 used), and a tools-attached round-trip
  confirming the request shape with the full tool schema doesn't error. Deliberately does **not**
  try to force a search call — v3 has no `:online`-suffix-style static flag to check anymore
  (search is always available as a tool, whether the model uses it on a given turn is its own
  judgment call, not something a fixed probe question can deterministically trigger). Compiles
  and passes review; cannot be run against the live API in this environment (no real key) — that
  remains yours to try once you have one in place.
- Run `install.ps1`/`uninstall.ps1` end to end against a real fresh install (a clean user profile
  or VM, if available) — confirm the one-liner actually works, confirm uninstall actually removes
  everything it claims to.
- Confirm CI (`check_v3.yml`) is green on a real push.
- Do the manual pre-release checklist pass design doc §13 committed to instead of automated Forms
  testing — every hotkey, every config option, on a fresh install.

**Exit criteria**: a tagged release, built by CI, installable via the one-liner on a machine that's
never seen this project before, passes a full manual hotkey-by-hotkey check.

**Depends on**: all of the above.

---

**Live-testing fix — Screenshot Query grabbed the wrong region on a scaled display**: reported
while testing on a second machine at non-100% Windows display scaling — the captured image never
matched the dragged rectangle. Root cause: the process never declared any DPI awareness (no
manifest, no `SetProcessDpiAwarenessContext` call anywhere, despite the `Win32_UI_HiDpi` Cargo
feature having been enabled and unused since it was added). For a DPI-unaware process, Windows
transparently virtualizes both `GetSystemMetrics(SM_CXSCREEN/CYSCREEN)` and GDI screen capture
(`GetDC`/`BitBlt` from the screen DC) down to the scaled/logical resolution — but `rdev`'s
low-level mouse hook (`WH_MOUSE_LL`) always reports real physical pixel coordinates, unaffected by
the hooking process's own DPI awareness. At 100% scaling the two coordinate spaces are identical
by coincidence, which is why this never surfaced on the original dev machine; at any other scale
factor the drag rectangle (physical pixels) and the capture (virtualized pixels) disagree, and
`capture_region` grabs the wrong area. **Fix**: `SetProcessDpiAwarenessContext
(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)` called once, first thing in `lib::run()`, before any
window or screen-metrics call — makes every Win32 coordinate in the process (mouse hook, window
placement, `GetSystemMetrics`, GDI capture) consistently physical, on any monitor/scale
combination. No new dependency; `Win32_UI_HiDpi` was already in `Cargo.toml`.

Fixing this also surfaced a separate, unrelated problem: the newly-installed Rust toolchain
(1.98.1, needed because this dev machine had no Rust/MSVC build tools at all before this session)
ships a clippy lint (`chunks_exact_to_as_chunks`) that didn't exist in whatever toolchain last
built this repo clean. It failed `cargo clippy --all-targets -- -D warnings` on three pre-existing,
unrelated call sites (`capture/screen.rs`, `lifecycle/path_cleanup.rs`, `ui/manager.rs`) —
confirmed pre-existing by reproducing the same failures on a clean `git stash`. Fixed as three
mechanical one-line rewrites to `slice::as_chunks`/`as_chunks_mut` (clippy's own suggested form,
stable in this toolchain), no behavior change. Verified: `cargo check`/`clippy --all-targets -- -D
warnings`/`test`, both default and `--features debug`, all clean, 54/54.

---

## M13 — v3 legacy Forms: reading + already-answered enforcement — DONE, 2026-09-20

**Goal**: after extended live testing left v3 new (§7.3) unable to reliably type or answer
questions at all, build v3 legacy (design doc §7.4) as the new default Forms engine. This
milestone covers the *reading* half only — extraction and the already-answered write-lock, no
answering/filling yet (M14).

- Port v2's `EXTRACTOR_JS` (`shadow_prompt_v2/src/browser/forms/extractor.rs`) into
  `shadow_prompt_v3`, run via `chromiumoxide`'s `Page::evaluate()` instead of `headless_chrome`'s
  sync `tab.evaluate()`. Same `Question`/`QuestionKind` shape as v2, ported as-is where it isn't
  one of the four patches below.
- Patch the extractor for design doc §7.4's four fixes:
  1. Remove `if (!text) return;` — heading-less/image-only questions still produce a `Question`.
  2. Remove the `data:`-prefix filter in the image-collection line — inline images survive.
  3. Add page-level heading/paragraph collection (outside any `[role="listitem"]`) into a new
     `page_context: String` field the extractor returns alongside the question array.
  4. Grid questions report each row's filled/blank state individually (not one collective
     `current_value` JSON blob) — add a per-row blank/filled shape to `Question` for `Grid`/
     `CheckboxGrid` kinds specifically.
- Unit-test the patch against hand-built HTML/DOM fixtures (or the same kind of hand-built
  fixture-JSON approach `read.rs`'s tests already used for v3 new) covering: a heading-less image
  question, an inline `data:` image, page-level context text, and a partially-filled grid.
- Already-answered write-lock: compute, from the extractor's own output, which fields/rows are
  blank vs. filled — this list is what M14's fill step is allowed to touch, nothing else. Checkbox
  questions stay whole-question skip (any existing pick locks the whole question), matching §7.3's
  reasoning.

**Exit criteria**: extraction runs against a real Google Form via `chromiumoxide` and correctly
reports all four fixes by inspection (page context populated, heading-less image question present,
inline image URL present, partially-filled grid reports per-row state) — verified live, not just
against unit fixtures.

**Depends on**: nothing new — reuses v3's existing `chromiumoxide`/debugger connection from §7.3.

---

**Status**: verified (`cargo check`/`clippy --all-targets -- -D warnings`/`test`, default and
`--features debug`, all clean, 69/69 — up from 54/54 pre-M13). Ported `EXTRACTOR_JS` into
`browser/forms/legacy/extractor.rs`, run via `Page::evaluate()` (chromiumoxide's async
equivalent of v2's sync `tab.evaluate()`) instead of a second, separate browser-automation crate —
no `headless_chrome` dependency added, keeping one Chrome-automation stack in the binary. All four
design-doc §7.4 patches implemented directly in the JS (no `if (!text) return`, no `data:` filter,
page-level heading/description capture, per-row `blank_rows` on Grid/CheckboxGrid) plus a new
`QuestionKind::Unknown` variant for a field with no recognized input control at all, so a
heading-less, control-less field still surfaces as *something* rather than vanishing. Tested via
hand-built JSON fixtures matching the extractor's own `JSON.stringify` output shape (same
constraint M6/M7/read.rs already documented: the JS itself can't run outside a real Chrome in this
environment, only the Rust-side parsing/`needs_attention` logic is unit-testable here — consistent
with design doc §13's "test the real thing" posture, not a gap specific to this milestone).
`Question::needs_attention()` is the already-answered gate: whole-field for every kind except
Grid/CheckboxGrid, which are judged by `blank_rows` — directly fixes the v2 defect found while
designing this (a partially-filled grid's collective `current_value` reads as "answered" the
instant any one row has a pick).

## M14 — v3 legacy Forms: agentic answer (no fill tool) + parse/inject — DONE, 2026-09-20

**Goal**: the answering and filling half. Model keeps real reasoning/search/docs capability via
`run_turn`, but filling is code-driven from one final structured-text answer, never a tool call —
the specific design choice that removes v3 new's fill-loop hang/infinite-loop failure class
(design doc §7.4).

- Add `delivery_forms_legacy.txt` (design doc §9): instructs the model to answer with one JSON
  object mapping question id → answer, restates that already-answered content must be skipped and
  that there is no submit capability and no way to invoke one.
- Call `run_turn` with `list_docs`/`read_doc`/`web_search` attached as tools — **no `fill_page`
  tool, no browser-interaction tool of any kind**. Build the initial user content from M13's
  extracted unanswered fields/rows (page context + questions + images as content parts, same
  shape as v2's `build_user_message`).
- Parse the model's final text with v2's existing balanced-brace `parse_answers` (port as-is from
  `shadow_prompt_v2/src/browser/forms/mod.rs` — already handles a clean JSON reply and a
  prose-wrapped one).
- Port v2's `injector.rs` (fuzzy label matching, not exact-string) into `shadow_prompt_v3`, run via
  `Page::evaluate()`, patched to only ever target the blank fields/rows M13 identified — this is
  the hard enforcement layer, independent of whatever the parsed answer map happens to contain.
- Test the never-overwrite guarantee directly: manually pre-fill some fields (including a
  partially-filled grid) on a real test form, run the flow, confirm only the blank fields/rows
  changed and the pre-filled ones are byte-identical afterward.

**Exit criteria**: a real Forms page with a mix of question types (including at least one image
question and one partially-filled grid) gets correctly answered and filled end to end, using real
search/`list_docs` calls where the test question calls for them, with zero tool-loop involved in
the fill step.

**Depends on**: M13.

---

**Status**: verified. `delivery_forms_legacy.txt` added (`forms_legacy_prompt()` in
`system_prompts.rs`) — restates the already-answered-skip and no-submit rules directly, on top of
`base.txt`'s general posture, matching design doc §9's addendum convention. `execute_legacy_form_flow`
calls `run_turn` with only `list_docs`/`read_doc`/`web_search` attached (`web_search` is
auto-added inside `run_turn` itself, same as every other entry point) — **no `fill_page` tool, no
browser-interaction tool of any kind**. The model's final turn is parsed by `parse_answers`, ported
verbatim from `shadow_prompt_v2/src/browser/forms/mod.rs` (clean-JSON path plus the
balanced-brace/prose-wrapped fallback), then injected via `browser/forms/legacy/injector.rs`
(v2's `injector.rs` ported, patched with a live-DOM re-check in every branch — `if (dateInput.value)
continue`, `groupAlreadyChecked(g)`, etc. — so the write-lock is enforced at the moment of acting,
regardless of what the parsed answer map contains, not just relying on the model having been asked
nicely). This is the direct fix for the failure class that made v3 new unusable: no tool call
drives filling, so there is no tool schema for a model to call with empty/malformed arguments and
no tool-loop for it to get stuck retrying.

## M15 — v3 legacy Forms: current-page multi-step flow + hotkey split — DONE, 2026-09-20

**Goal**: wire M13/M14 into the two Forms hotkeys, operating directly on the already-open tab (no
new background tab, no tab-closing logic, no page-count cap — all explicitly dropped per design
doc §7.4), and split the hotkey surface so v3 legacy takes the current default binds while v3 new
moves to new, secondary ones.

- `forms_answer_page`: find the currently-focused Forms tab (reuse §7.3's
  `document.hasFocus()`-based lookup — that part is generic tab-detection, not new-tab-opening, so
  it's shared rather than reimplemented), run M13 read → M14 answer/fill on the current page, stop.
- `forms_answer_all`: same, then find a Next-labeled control and click it, repeat from the current
  page onward until only a Submit control remains (never clicked — same hard non-negotiable as
  §7.3, and there is still no submit tool anywhere in the codebase). Cross-page memory via the same
  flattened-text-prepend approach §7.3/M8 already used and flagged (`run_turn` still has no
  multi-turn history parameter — inherited gap, not a new one).
- No page-count cap, no tab-open/tab-close code at all for this engine — confirm by reading the
  diff that none of §7.3's `tab_lifecycle.rs` open/close logic is called from this path.
- Config/hotkey split: `forms_answer_page`/`forms_answer_all` (`ctrl+shift+alt+g`/`f`, unchanged
  binds) route to v3 legacy. Add `forms_answer_page_axtree`/`forms_answer_all_axtree`
  (`ctrl+shift+alt+j`/`u`, new) routing to v3 new's existing, untouched `execute_form_flow`. Both
  pairs active at all times — no config flag gating which engine runs; update
  `config/schema.rs`/`config.toml`/`config.example.toml`/the help-overlay cheat sheet (13 → 15
  rows) accordingly.

**Exit criteria**: `forms_answer_page`/`forms_answer_all` solve a real multi-page form end to end
via v3 legacy on the default binds; `forms_answer_page_axtree`/`forms_answer_all_axtree` still
invoke v3 new, unchanged, on the new binds — both reachable in the same build.

**Depends on**: M14.

**Status**: verified (`cargo check`/`clippy --all-targets -- -D warnings`/`test`, default and
`--features debug`, all clean, 69/69). `browser/forms/legacy/mod.rs`'s `advance_to_next_page`
ported v2's `submit_guard.rs` selectors almost verbatim (`div[role="button"][aria-label*="Next"/
"Submit"]`), using chromiumoxide's own `Page::find_element`/`Element::click` — the same
CSS-selector-based public API v3 new's own module doc (`fill.rs`) already confirmed is the only
public way to get an `Element` from this crate, just used directly here instead of needing the
AX-tree/`backend_dom_node_id` bridge `fill.rs` had to build for its AX-driven lookups. `forms_run.rs`
(v3 new's action handler) is untouched; a new `forms_run_legacy.rs` mirrors its `FormIndicator`
wiring for the legacy engine. `HotkeysConfig` gained `forms_answer_page_axtree`/
`forms_answer_all_axtree` with `#[serde(default)]` (`ctrl+shift+alt+j`/`u`) so an existing
`config.toml` written before this pair existed still parses without a forced `--init` rerun.
`InputEvent`/`bindings.rs`/`actions/mod.rs::dispatch` all extended with the two new variants;
`help_toggle.rs`'s cheat sheet grew from 13 to 15 rows. Confirmed by reading the diff (not just
asserted): no call anywhere in `browser/forms/legacy/` touches `tab_lifecycle::open_forms_tab`/
`safe_to_close`, and there is no page-count loop bound at all in `execute_legacy_form_flow` — both
match design doc §7.4's explicit drops. Not verified, and cannot be without a live Google Form (no
browser available in this environment, the same honest constraint every prior Forms milestone in
this plan has carried): whether the ported extractor/injector selectors and the best-effort
page-description heuristic actually match Google's current live markup. That is the actual reason
v3 legacy exists — v2's underlying selector approach was the proven-reliable half of this rebuild,
carried forward rather than re-invented — but "v2's approach worked before" is not the same claim
as "confirmed working today," and this should be the first thing tried against a real form.
