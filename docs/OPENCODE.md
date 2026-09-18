# OpenCode Architecture Reference (for ShadowPrompt v3)

> Research document. Purpose: extract OpenCode's agentic-loop / tool-calling / permission /
> provider-abstraction design so ShadowPrompt v3 (a Rust, single-provider, ~6-8-tool, stealth
> exam-assistant daemon) can reimplement the *ideas* from scratch, without depending on or
> forking OpenCode. Every claim below is backed by a `file:line` citation into a local clone of
> the OpenCode monorepo. Paths are relative to the OpenCode repo root
> (`packages/opencode/...`, `packages/llm/...`, `specs/...`).
>
> **Scope note.** OpenCode is mid-migration from a legacy ("V1") runtime — Hono routes, Zod
> schemas, the Vercel `ai` SDK doing the actual model calls — to a from-scratch Effect-native
> "V2" stack (`specs/v2/*.md`, `packages/core`, the new `packages/llm` wire layer, an
> Effect HttpApi server). **The V1 runtime is what actually ships and runs today**; V2 is a
> parallel, partially-wired rewrite of the same ideas with a cleaner design, described in
> `specs/v2/*.md` and `specs/storage/*.md` but not yet the production request path for a normal
> chat turn. This document treats V1 as "how it actually behaves" and V2 as "the idealized
> version of the same architecture, useful as a design reference." Out of scope per the
> assignment: TUI rendering (`packages/tui`), the desktop/Tauri app (`packages/desktop`), the
> marketing site (`packages/web`), Slack (`packages/slack`), `packages/enterprise`, and CI/build
> tooling — all exist, all irrelevant to a single-purpose Rust daemon.

---

## 1. What OpenCode is, and the one idea worth stealing

OpenCode is a general-purpose, multi-provider, multi-client coding agent: a headless Node/Bun
server (`packages/opencode`) that runs an agentic tool-calling loop against ~75+ LLM providers,
persists every step of the conversation to SQLite, and is driven by a TUI, a desktop app, a web
UI, or raw HTTP — all talking to the same server process over HTTP + Server-Sent Events.

**The one idea worth stealing:** the entire system is built around a strict separation of three
concerns that never leak into each other:

1. **A dumb, replayable event stream from the model** (`LLMEvent`: `text-delta`, `tool-call`,
   `tool-result`, `step-finish`, `finish`, ...) that is *identical* regardless of which of the
   dozen wire protocols (OpenAI chat, OpenAI responses, Anthropic messages, Bedrock, Gemini,
   OpenRouter, ...) actually produced it.
2. **A single, boring `while(true)` loop** (`SessionPrompt.runLoop`,
   `packages/opencode/src/session/prompt.ts:1081-1341`) that pulls one user turn's worth of
   conversation history off durable storage, hands it to the model, drains that `LLMEvent`
   stream into durably-persisted message parts, and decides — from *only* the persisted state,
   never from in-memory turn state — whether to call the model again, run a subagent, compact
   history, or stop.
3. **Tools that are plain data + a function**: a tool is a JSON-schema description plus an
   `execute(input, ctx) -> Result` function. It never talks to the model, never decides whether
   it's allowed to run (that's `ctx.ask(...)`, a separate permission service), and never decides
   how its own output gets trimmed (that's a separate truncation service). Tools are inert;
   the loop and its surrounding services provide all policy.

Everything else in this document — persistence, compaction, permission-gating, provider
abstraction, subagents — is a consequence of taking that separation seriously. That's the part
worth copying into a from-scratch Rust daemon; the 75-provider catalog, the multi-client HTTP/SSE
protocol, and the Effect-TS plumbing are overhead specific to OpenCode's product shape (see
§7).

---

## 2. The agentic loop: full request lifecycle

### 2.0 Control-flow diagram

```
 HTTP POST /session/:id/message                      (server/routes/.../handlers/session.ts:295-316)
        │
        ▼
 SessionPrompt.prompt(input)                          (session/prompt.ts:1052-1071)
   - creates + persists the user Message + Parts       (createUserMessage, :635-1050)
   - calls loop({ sessionID })                         (:1070)
        │
        ▼
 SessionPrompt.loop(input)                             (:1343-1347)
   - SessionRunState.ensureRunning(sessionID, ...)      (session/run-state.ts)
     one Runner-fiber per sessionID; a second prompt
     for the SAME session joins the existing fiber
     instead of starting a race
        │
        ▼
 SessionPrompt.runLoop(sessionID)   <──────────────────────────────────┐  (:1081-1341)
   ┌─────────────────────────────────────────────────────────────┐    │
   │ 1. reload + filter history from SQLite                      │    │  loop again
   │    MessageV2.filterCompactedEffect(sessionID)  (:1092)       │    │  ("continue")
   │ 2. latest(msgs) -> {lastUser, lastAssistant, finished,tasks} │    │
   │ 3. exit if last assistant turn is "finished" and has         │    │
   │    no pending tool calls                        (:1111-1130)│    │
   │ 4. pop a queued subtask/compaction "task" if present         │    │
   │    (subagent -> handleSubtask, compaction -> compaction.process)
   │ 5. check context-overflow -> enqueue compaction (:1161-1168) │    │
   │ 6. resolve agent, model, reminders, system prompt            │    │
   │ 7. SessionTools.resolve(...) -> AI-SDK-shaped tool map        │    │
   │ 8. processor.process({ system, messages, tools, model })  ───┼────┘
   │    == ONE PROVIDER TURN (may itself contain N tool calls)    │
   │ 9. inspect result: "stop" | "compact" | "continue"           │
   └─────────────────────────────────────────────────────────────┘
        │ "stop"                                   
        ▼
 return last assistant message+parts to caller (also pushed live via SSE)
```

One `processor.process(...)` call (§2.3) is "one provider turn": one HTTP/streaming call to the
model that may itself execute an arbitrary number of tool calls *inside* the AI-SDK's own
tool-loop before it emits `step-finish`/`finish`. The **outer** `runLoop` `while(true)` is what
repeats provider turns — it re-enters because the assistant's `finish` reason was `tool-calls`
(there's more to do), because compaction was requested, or because a subagent/task needs to run
before the next provider turn. Termination is not "no more tool calls the model wants to make"
tracked in memory — it is recomputed **every iteration by re-reading persisted state**
(`lastAssistant.finish && !hasToolCalls`, `prompt.ts:1111-1130`). This is the single most
important structural property for a Rust port: **the loop's exit condition is a pure function of
durable history, not of stack-local state**, so a crash mid-turn is recoverable by just re-running
the loop against what's on disk.

### 2.1 HTTP entry point

- `POST /session/:sessionID/message` is declared in
  `packages/opencode/src/server/routes/instance/httpapi/groups/session.ts:95,316-328` (path
  constant `prompt`, endpoint id `session.prompt`).
- The handler lives in
  `packages/opencode/src/server/routes/instance/httpapi/handlers/session.ts:295-316`, and does
  nothing but forward `ctx.payload` + `ctx.params.sessionID` to
  `SessionPrompt.Service.prompt(...)` (`:301`, `:316`).
- There is also `POST /session/:sessionID/prompt_async`
  (`groups/session.ts:96,329-337`) which accepts the same payload but returns immediately
  (`HttpApiSchema.NoContent`) — the loop still runs, but the HTTP caller doesn't block on it;
  progress is observed exclusively over the SSE event stream (§6).

### 2.2 `SessionPrompt.prompt` → `SessionPrompt.loop`

`packages/opencode/src/session/prompt.ts:1052-1071`:

1. Loads the `Session.Info` row, cleans up any pending "revert" state.
2. `createUserMessage(input)` (`:635-1050`) — this is the single choke point where all"user
   input" shapes get normalized into `Part`s before anything is model-visible:
   - Plain text and file/image attachments pass through largely as-is (with image
     normalization/resizing, `:1011-1020`).
   - `type: "agent"` parts (an explicit `@subagent` mention) get rewritten into a synthetic
     text part instructing the model to call the `task` tool (`:974-990`).
   - `type: "file"` parts whose `url` is `file:` get **eagerly read now**, using the same `read`
     tool the model would call later, and the file content is spliced into the message as a
     synthetic "Called the Read tool with the following input: ..." pair of text parts
     (`:808-970`) — so the model sees tool-call-shaped context even though no real tool call
     happened.
   - MCP `resource://` sources are read via `mcp.readResource` and inlined similarly
     (`:702-784`), with byte-size and MIME allowlist checks (`:65-72`, `:734-753`).
   - Every user message and part is schema-validated (`decodeMessageInfo`/`decodeMessagePart`,
     `:1022-1044`) — validation failures are logged, not silently swallowed, but writes still
     proceed (defense-in-depth logging, not a hard gate).
3. Persists the message + parts (`sessions.updateMessage`/`updatePart`, `:1046-1047`) — this
   happens **before** any model call. Nothing is "in-flight-only"; the user's turn exists
   durably the instant it's accepted.
4. If `noReply` isn't set, calls `loop({ sessionID })` (`:1070`), whose only job is to hand off to
   `SessionRunState.ensureRunning` (`:1343-1347`), which either joins an already-running fiber for
   this session or starts `runLoop(sessionID)` fresh (`session/run-state.ts`). This is what makes
   "prompt while already busy" safe: a second `prompt()` call for the same session doesn't start
   a second competing loop, it attaches to the one that's already running and gets the same
   final result when it settles.

### 2.3 One provider turn: `runLoop` body + `SessionProcessor`

Inside `runLoop` (`prompt.ts:1081-1341`), each iteration:

- Re-reads history: `MessageV2.filterCompactedEffect(sessionID)` (`:1092`) — this both loads
  messages from SQLite (`message-v2.ts:97-124` hydrate helpers) **and** re-splices any completed
  compaction into `[compaction-user, summary-assistant, ...retained tail..., continue-user]`
  order (`message-v2.ts:498-572`, `filterCompacted`). Position in this array is *not*
  chronological once compaction has happened — see §4.
- `latest(msgs)` (`message-v2.ts:582-598`) walks that array once and extracts: the most recent
  user message, the most recent assistant message, the most recent *finished* assistant message,
  and any pending `subtask`/`compaction` "tasks" attached to messages after the last finished
  turn. This is the entire piece of "what should happen next" state — it is recomputed from
  scratch every loop iteration, never carried in a local variable across iterations.
- Exit check (`:1111-1130`): if the last assistant message has a terminal `finish` reason (not
  `"tool-calls"` or `"unknown"`) and none of its tool parts are still pending, the loop breaks.
  There's a specific carve-out for providers that report `finish: "stop"` even though the
  message still contains real tool calls (`hasToolCalls`, `:1106-1109`) — the loop keeps running
  in that case so those tool results still get sent back to the model.
- If a queued `subtask` exists, `handleSubtask(...)` runs it (§5) and `continue`s the loop
  without calling the model this iteration (`:1144-1147`).
- If a queued `compaction` task exists, `compaction.process(...)` runs it (§4) and `continue`s
  (`:1149-1159`).
- If the last finished turn's token usage crosses the model's context budget,
  `compaction.create(...)` enqueues a compaction task and `continue`s *without* calling the model
  (`:1161-1168`) — compaction always happens as its own loop iteration, never inline with a
  normal provider turn.
- Otherwise: resolve the `Agent.Info`, apply reminders (`SessionReminders.apply`, `:1180-1184`),
  build a new assistant `Message` row (`:1186-1201`), and call
  `SessionProcessor.Service.create({ assistantMessage, sessionID, model })` (`:1213-1219`) to get
  a `Handle` for this one provider turn.
- `SessionTools.resolve(...)` (`session/tools.ts:41-134`, called from `prompt.ts:1226-1241`)
  builds the actual AI-SDK `Record<string, Tool>` map for this turn: it asks
  `ToolRegistry.tools(...)` for the currently-visible `Tool.Def[]` (permission-filtered, model
  gated — e.g. `apply_patch` vs `edit`/`write` depending on whether the model is GPT-family,
  `registry.ts:296-302`), and wraps each one's `execute` in a closure that also fires
  `plugin.trigger("tool.execute.before"/"tool.execute.after", ...)` around the call
  (`tools.ts:99-134`).
- `handle.process({ system, messages, tools, model, ... })` (`prompt.ts:1272-1286`) is the actual
  model call — see next.
- The `Result` it returns (`"stop" | "compact" | "continue"`, `processor.ts:30`) plus a check for
  structured-output completion (`prompt.ts:1288-1293`) and content-filter/refusal detection
  (`:1296-1317`) together decide the outer loop's next step (`:1319-1329`).

**`SessionProcessor.create(...).process(streamInput)`** (`packages/opencode/src/session/processor.ts:98-707`)
is where the wire event stream becomes durable state:

- `llm.stream(streamInput)` (`:654`) returns an Effect `Stream<LLMEvent>` (§5 for what actually
  produces this).
- `Stream.tap(handleEvent)` (`:656-660`) processes each event as it arrives:
  - `reasoning-start/-delta/-end` → create/append/finalize a `reasoning` Part (`:280-313`).
  - `text-start/-delta/-end` → create/append/finalize a `text` Part, streamed to any live SSE
    listener via `session.updatePartDelta` on every delta (`:513-524`) so a UI can render
    token-by-token without waiting for the part to finish.
  - `tool-input-start/-delta/-end` → `ensureToolCall` creates a `tool` Part in `"pending"` state
    keyed by the provider's `toolCallID`, before arguments have even finished streaming
    (`:216-253`, `:315-329`). This is what lets a UI show "calling `bash`..." before the full
    command string has arrived.
  - `tool-call` → the Part flips to `"running"` with the fully-parsed input (`:331-351`), **plus**
    a "doom loop" guard: if the same tool name+input has now repeated 3 times in a row for this
    assistant message (`DOOM_LOOP_THRESHOLD = 3`, `:29`, `:356-369`), the processor calls
    `permission.ask({ permission: "doom_loop", ... })` (`:372-379`) — i.e. it forces a
    human-approval gate specifically to break a model stuck in a tool-call loop. **The actual
    tool `execute()` call itself does not happen here** — it happens inside the AI SDK's own
    tool-invocation machinery, which calls the `execute` closure that `SessionTools.resolve`
    attached to each AI-SDK `Tool` (`tools.ts:99-134`); `SessionProcessor` only ever observes the
    `tool-call`/`tool-result`/`tool-error` events the SDK's internal loop emits.
  - `tool-result`/`tool-error` → mark the Part `"completed"`/`"error"`, normalizing image
    attachments (`:390-413`).
  - `step-finish` → compute usage/cost, persist a `step-finish` Part with token/cost totals
    (`:435-469`), snapshot the working tree for undo/diff purposes (`:436`, `:472-484`), and flag
    `ctx.needsCompaction` if usage now overflows the model's context window (`:491-496`).
  - `finish` → no-op; the terminal state was already captured by the last `step-finish`.
- `Stream.takeUntil(() => ctx.needsCompaction)` (`:658`) — the stream is force-stopped as soon as
  a `step-finish` handler flags overflow, even mid-turn, rather than letting the provider keep
  streaming into a request that's already too big.
- `Effect.retry(SessionRetry.policy(...))` (`:674-688`) wraps the whole stream-drain in a
  provider-aware retry policy (rate limits, transient 5xx, etc. — see §5's error taxonomy).
- `Effect.ensuring(cleanup())` (`:690`) always runs on the way out: it finalizes any open
  text/reasoning parts, waits up to 250ms for in-flight tool calls to settle, and marks anything
  still `"pending"`/`"running"` as `error: "Tool execution aborted", metadata.interrupted: true`
  (`:553-611`) — this is the durable marker that `isOrphanedInterruptedTool` (`prompt.ts:96-100`)
  later uses to distinguish "this turn really finished, some tool call just got orphaned by a
  crash" from "this turn is still waiting on a tool call."
- `process()` returns `"compact"` if `ctx.needsCompaction`, `"stop"` if permission was denied
  (`ctx.blocked`) or the message errored, else `"continue"` (`:693-695`).

### 2.4 Concurrency model

One `Runner` fiber per `sessionID`, tracked in an in-memory map
(`session/run-state.ts:33-45`; the fiber-per-session abstraction itself is `effect/runner.ts`,
not read in depth here since it's Effect-specific plumbing). A second `prompt()`/`loop()` call for
a session that's already running **joins** the existing fiber rather than racing it; a
`shell()` call additionally supports a `Latch` so the caller can wait until the shell command has
at least been durably recorded before returning (`run-state.ts:19-24`, used from
`prompt.ts:1349-1354`). Cancellation (`cancel(sessionID)`) interrupts that one fiber; interrupting
never deletes already-durable rows, it only stops future work and runs the `Effect.onInterrupt`
finalizers described above. Different sessions run fully concurrently with no shared lock.

---

## 3. Tool system

### 3.1 The interface a tool implements

Two coexisting tool type shapes appear in the codebase — they matter for a port because they
represent two different points on the "how much can change without recompiling" spectrum:

**V1 (production), `packages/opencode/src/tool/tool.ts:55-181`:**

```ts
interface Def<Parameters, M> {
  id: string
  description: string
  parameters: Parameters                 // an Effect Schema decoder
  jsonSchema?: JSONSchema7               // optional pre-rendered schema for the wire
  execute(args: Schema.Type<Parameters>, ctx: Context): Effect.Effect<ExecuteResult<M>>
  formatValidationError?(error: unknown): string
}

interface Context<M> {
  sessionID: SessionID
  messageID: MessageID
  agent: string
  abort: AbortSignal
  callID?: string
  extra?: Record<string, unknown>          // ad-hoc typed escape hatch, e.g. { model, bypassAgentCheck }
  messages: SessionV1.WithParts[]          // full session history, read-only, for tools that need context
  metadata(input: { title?: string; metadata?: M }): Effect.Effect<void>   // stream progress mid-execution
  ask(input: Omit<PermissionV1.Request, "id"|"sessionID"|"tool">): Effect.Effect<void>
}

interface ExecuteResult<M> {
  title: string
  metadata: M
  output: string
  attachments?: Omit<FilePart, "id"|"sessionID"|"messageID">[]
}
```

`Tool.define(id, init)` (`tool.ts:151-169`) wraps a raw definition with a `wrap(...)` closure
(`:99-149`) that is applied **once at tool-registration time**, not per-call, and does three
things uniformly for every tool without each tool author having to remember them:

1. Decodes/validates the model's raw JSON args against `parameters` before `execute` ever runs;
   a decode failure becomes an `InvalidArgumentsError` (`:24-34`) whose `.message` getter is
   literally the text that gets fed back to the model as the tool result — "rewrite your
   arguments" is a first-class, typed failure mode, not a thrown exception the model never sees.
2. Runs `execute`, then applies the shared output-truncation policy (§3.3) unless the tool
   already set `metadata.truncated` itself (`:130-144`).
3. Wraps everything in `Effect.withSpan("Tool.execute", { "tool.name", "session.id",
   "message.id", "tool.call_id" })` (`:145`) for tracing.

**V2 (spec, `specs/v2/tools.md`, aspirational — see §0):** a slightly stricter, opaque
`Tool.Definition<Input, Output>` built by `Tool.make({ description, input, output, execute,
toModelOutput? })`, with an explicit `Tool.Context { sessionID, agent, assistantMessageID,
toolCallID }` and a formal set of invariants ("Laws" in the spec, `tools.md` bottom): single
executor, codec boundary between decoded-input/encoded-output, captured-execution (a tool
mid-flight can't be affected by later registration changes), stale-call rejection. This is a
cleaner restatement of the same contract V1 already has in practice — worth reading for the
invariants, not for new mechanism.

### 3.2 Concrete tool examples

**`read` — filesystem read, `tool/read.ts:64-386`.** Demonstrates: path resolution relative to a
project root with symlink/escape checks delegated to a separate `assertExternalDirectoryEffect`
gate (`:250-253`); a single `ctx.ask({ permission: "read", patterns: [relativePath], always:
["*"] })` call before any bytes are touched (`:255-260`); binary/image/PDF sniffing from a small
sample read before deciding how to handle the rest of the file (`:301-329`); line-based paging
with an explicit byte cap (`MAX_BYTES = 50KB`, `:16`) *independent* of the line-count cap
(`DEFAULT_READ_LIMIT = 2000` lines, `:13`), because a file can blow the byte budget in far fewer
than 2000 lines; and it returns machine-actionable continuation hints in its own output text
(`"Use offset=<next> to continue"`, `:345-350`) rather than silently truncating.

**`task` — subagent spawn, `tool/task.ts:81-372`.** See §5 in full; the short version is it
creates a *new, independent* `Session` row with `parentID` set to the current session, derives a
restricted permission ruleset for it, and recursively calls the exact same
`SessionPrompt.prompt`/`loop` machinery (via a `TaskPromptOps` capability object,
`:18-22`, threaded through `ctx.extra.promptOps`) rather than having any separate "run a subagent"
code path.

**`bash`/shell — `tool/shell.ts`.** Demonstrates permission scoped by *argument content*, not
just tool name: it parses the shell command with `web-tree-sitter` to classify which arguments
name files/paths (`CWD`/`FILES`/`CMD_FILES` keyword sets, `:26-59`) so the permission `patterns`
passed to `ctx.ask` can be e.g. `rm -rf ./foo` → deny-pattern-tested against `./foo` specifically,
not just against the literal tool name `bash`. (Full grammar in `permission/arity.ts`, not
exhaustively read — the pattern to copy is "generate permission patterns from the *parsed*
argument, not from the tool name alone.")

**`webfetch` — network fetch, `tool/webfetch.ts:23-40+`.** Demonstrates a hard producer-side cap
independent of the generic truncation service: `MAX_RESPONSE_SIZE = 5 * 1024 * 1024` bytes and a
30s/120s default/max timeout (`:9-11`) enforced while *reading* the HTTP response, before the
truncation-service ever sees the text — because truncation happens on a `string` you already
have in memory, and a producer that can emit gigabytes must be capped before that string exists
(this exact distinction is called out explicitly in `specs/v2/tools.md` under "Output Bounding":
*"Model-output bounding is not producer memory management... A producer cannot claim a complete
retained output after it has already discarded bytes."*).

### 3.3 Permission gating

Mechanism (`packages/opencode/src/permission/index.ts:12-176`):

- A **ruleset** is an ordered list of `{ permission, pattern, action: "allow"|"deny"|"ask" }`.
  `evaluate(permission, pattern, ...rulesets)` (`:28-38`) flattens every ruleset and does
  `findLast` over wildcard matches — **last match wins**, so more specific/later rules override
  earlier/broader ones, and the default when nothing matches is `"ask"`.
- `Permission.ask(input)` (`:67-107`) evaluates every one of the request's `patterns` against
  `ruleset` (the agent's static config) plus `approved` (this-process-lifetime "always allow"
  grants). If any pattern resolves to `"deny"`, it fails immediately with `DeniedError` — no
  prompt, no wait. If every pattern resolves to `"allow"`, it returns immediately — again no
  prompt. **Only** if at least one pattern needs asking does it create a `Deferred<void,
  RejectedError>`, register it in a `pending` map, and publish a `permission.asked` event over
  the SSE bridge (`:100`), then `yield* Deferred.await(deferred)` (`:102`) — this is the pause:
  **the executing fiber literally blocks on an unresolved `Deferred`, inside the same Effect
  fiber that's running `Tool.execute`**, with no separate "suspend and serialize state" step
  needed, because Effect's runtime can park a fiber indefinitely without holding a thread.
- `Permission.reply(input)` (`:109-167`), called from a separate HTTP request
  (`POST /session/:id/permissions/:requestID`, `groups/session.ts:395-408`), resolves that
  `Deferred`: `"reject"` fails it (optionally with user feedback text as a `CorrectedError` the
  tool can surface to the model as "try again, differently"); `"once"` succeeds it without
  changing the ruleset; `"always"` succeeds it **and** appends new `{ action: "allow" }` rules to
  the process-lifetime `approved` list for every pattern in `request.always`, then immediately
  re-evaluates every other still-pending request for the same session against the updated
  `approved` list and auto-resolves any that now pass (`:145-166`) — so approving one wildcard
  can unblock several already-queued asks at once.
- On session/process teardown, every still-pending `Deferred` is force-failed with
  `RejectedError` (`:54-61`) — nothing is left silently hanging.
- What's persisted: **only the fact that a request is pending**, in an in-memory `Map` scoped to
  the process (`InstanceState`, not SQLite). A crash loses pending asks; the *tool call itself*
  is still durable as a `"pending"`/`"running"` Part and gets marked
  `error: "Tool execution aborted"` on restart (§2.3's cleanup path) rather than silently
  resuming — OpenCode does not currently persist permission requests across a process restart.
- `PermissionV1.Ruleset` composition for a call site is always `Permission.merge(agent.permission,
  session.permission ?? [])` (e.g. `prompt.ts:87`, `tools.ts:87`) — static per-agent config plus
  session-level overrides (e.g. a subagent's derived restrictions, §5) are just concatenated in
  order, relying on the same "last match wins" evaluator.

### 3.4 Output-size handling

Two independent layers, deliberately not merged (`tool/truncate.ts:1-157`):

1. **Producer-side caps**, per tool, enforced *while bytes are being read/generated* — `read`'s
   `MAX_BYTES` line-streaming cutoff (`read.ts:16,148-179`), `webfetch`'s `MAX_RESPONSE_SIZE`
   (`webfetch.ts:9`). These exist because you cannot "truncate" memory you've already
   over-allocated; the cap has to apply during production.
2. **Generic model-output truncation**, `Truncate.output(text, options, agent)`
   (`truncate.ts:85-141`), applied uniformly to *every* tool's already-produced output string by
   the `Tool.define` wrapper (§3.1) unless the tool opted out by setting its own
   `metadata.truncated`. Defaults: 2000 lines / 50KB (`MAX_LINES`, `MAX_BYTES`, `:14-15`),
   overridable per-deployment via `tool_output.max_lines`/`max_bytes` config (`:80-82`). When the
   content fits, it passes through unchanged (`:93-95`). When it doesn't: the **full** text is
   still written to a scratch file under a truncation directory with a 7-day retention sweep
   (`RETENTION = 7 days`, `:12`, `:53-66`, swept hourly `:143-148`); the tool's *model-visible*
   output becomes a head-or-tail preview plus an explicit continuation hint
   (`"...N lines truncated...\nUse Grep to search the full content or Read with offset/limit"`,
   `:129-137`) — and if the current agent actually has the `task` tool available, the hint
   instead says *"delegate to the explore agent to process this file... do NOT read the full
   file yourself, to save context"* (`:129-131`), i.e. the truncation hint is itself
   agent-capability-aware.

The V2 spec (`specs/v2/tools.md`, "Output Bounding") restates this as a hard rule worth keeping
verbatim for a Rust port: *tools return complete, untruncated domain output; exactly one
settlement-boundary step measures and bounds what actually reaches the provider; if full
retention of the overflow fails, that's an operational failure, not a silently-lossy success.*

---

## 4. Session / message / part model, and compaction

### 4.1 What's persisted, and why

Storage is SQLite via Drizzle (`packages/opencode/src/storage/db.ts`, schema tables referenced
from `message-v2.ts:21` as `MessageTable`/`PartTable`/`SessionTable`). Two levels:

- **`Message`** (`Info` in `message-v2.ts`) — one row per user turn or per assistant turn.
  Carries role, parent linkage (`parentID`, used both for "which user message does this
  assistant reply to" and for subagent session parenting, §5), the resolved `{providerID,
  modelID,variant}` for that turn, cumulative `tokens`/`cost`, a `finish` reason once settled, an
  optional `error` (a small **typed taxonomy**: `AbortedError`, `AuthError`, `APIError`,
  `ContextOverflowError`, `OutputLengthError`, `ContentFilterError`, `StructuredOutputError`,
  `NamedError.Unknown` — `message-v2.ts:606-734`), and a `summary: true` flag marking a
  compaction-generated assistant message.
- **`Part`** — one row per *piece* of a message's content, each independently updatable/streamed:
  `text`, `reasoning`, `tool`, `file`, `step-start`, `step-finish`, `patch` (working-tree diff
  snapshot), `compaction` (a marker part on a *user* message meaning "this turn is a compaction
  request"), `subtask` (a marker meaning "this turn should spawn a subagent"), `agent` (an
  explicit `@mention`). Splitting message from part is what lets a UI (or, for us, an in-memory
  renderer) subscribe to a single growing text part's deltas without re-fetching the whole
  message, and what lets a `tool` part transition `pending → running → completed/error`
  independently of everything else in the same assistant turn.
- Every row is keyed with a **lexicographically sortable, monotonically increasing ID**
  (`MessageID.ascending()`, `PartID.ascending()`, ULID-style — seen throughout, e.g.
  `prompt.ts:657,696`), so `ORDER BY id` reconstructs chronological order without needing a
  separate sequence column, and IDs generated in different processes still sort correctly enough
  to break ties (`message-v2.ts:600-604`, `isAfter`).
- Everything is written **synchronously into the same durable store the loop reads from** —
  there is no separate "conversation cache" that could drift from disk. `runLoop` re-reads from
  SQLite every iteration (§2.3); nothing about "what happens next" lives only in a Rust-equivalent
  local variable.

### 4.2 Compaction

Two independent mechanisms, both driven from `runLoop`, never from inside a normal provider turn:

**Full compaction** (`session/compaction.ts:319-557`, `processCompaction`), triggered either when
usage crosses budget after a `step-finish` (`prompt.ts:1161-1168`) or when the provider itself
rejects the request as too large (`processor.ts:621-631`, a `ContextOverflowError`, handled as an
"overflow-triggered" compaction with a stricter one-retry policy per `specs/v2/session.md`'s
"Automatic Compaction" section). Sequence:

1. A synthetic **user** message is inserted with a `compaction` Part (`create`, `:559-582`) — this
   is how "please compact" becomes a queued task the next `runLoop` iteration picks up
   (`latest()`'s `tasks` field, `message-v2.ts:592-597`).
2. `select(...)` (`:223-269`) decides what to keep verbatim: it walks conversation "turns"
   backward (a turn = one user message through the next user message, `turns()`, `:122-138`),
   accumulating a token budget (`preserveRecentBudget`, `:115-120`, default 25% of the model's
   usable context, clamped to 2K–15K tokens) until the budget is exhausted, optionally
   **splitting** a single oversized turn to keep only its tail (`splitTurn`, `:140-163`) rather
   than dropping it whole.
2b. Everything *before* that kept tail gets `serialize()`d into a flat text transcript
   (`:54-85`) — text/reasoning/tool-calls-with-truncated-outputs (`TOOL_OUTPUT_MAX_CHARS =
   2000`, `:30,51-52`) joined with `[User]:`/`[Assistant]:`/`[Assistant tool call]:` prefixes —
   and handed to the model as a **summarization prompt** built by `buildPrompt(...)`
   (imported from `@opencode-ai/core/session/compaction`, not read in depth; role is "ask the
   model to write a rolling summary given the previous summary + this new transcript chunk").
3. That summarization runs through the **exact same** `SessionProcessor.create(...).process(...)`
   machinery as a normal turn (`:420-448`) — compaction is not a special code path for calling
   the model, it's a normal provider turn whose output happens to be marked `summary: true`
   (`:401`) and whose only "tool" is none (`tools: {}`, `:429`).
4. On success, `filterCompacted` (`message-v2.ts:498-572`, called every time history is loaded,
   `:574-576`) rewrites the *view* of history to `[compaction-user, summary-assistant, ...kept
   tail (chronological)..., anything after]` — **the original messages are never deleted**, only
   reordered/hidden for model-consumption purposes. Repeated compactions fold forward: a new
   compaction summarizes "previous summary + newly-accumulated turns" (`previousSummary`,
   `:366,381-391`), so the summary chain never needs to re-read everything from the beginning.
5. If compaction itself fails to fit ("Session too large to compact... even after stripping
   media"), the turn is marked as a hard error, not retried in a loop (`:450-459`).
6. Optionally auto-continues afterward: replays the original over-budget user turn stripped of
   media (`:468-495`) or injects a synthetic "Continue if you have next steps..." user message
   (`:519-548`) so the session doesn't just go idle after compacting.

**Pruning** (`compaction.ts:273-317`, `prune`) is a much cheaper, non-model second mechanism: walk
backward through *already-completed* tool-call Parts, and once more than `PRUNE_PROTECT` (40K)
tokens' worth of old tool output has accumulated beyond the most recent 2 turns, mark the
**oldest** excess ones (past `PRUNE_MINIMUM` = 20K tokens' worth) with `state.time.compacted =
now` (`:308-316`) — `serialize()` then renders those as `"[Old tool result content cleared]"`
(`:76-78`) instead of their real (possibly huge) output, without touching the surrounding
text/reasoning or requiring a model call at all. Runs opportunistically after the loop exits
(`prompt.ts:1338`, fire-and-forget).

---

## 5. Provider / model abstraction

### 5.1 Two parallel engines, one canonical event stream

`session/llm.ts` is the seam: it holds a per-model choice between `LLMAISDK` (wraps the Vercel
`ai` package's `streamText`, the current production path for most providers) and
`LLMNativeRuntime` (the new from-scratch `packages/llm` engine). **Both converge on the same
`LLMEvent` stream type** (`@opencode-ai/llm`'s `schema/events.ts`) that `SessionProcessor`
consumes (§2.3) — this is the actual seam to copy: *pick one canonical, provider-agnostic event
enum, and make every transport implementation, however different its wire format, emit only
that enum.* `LLMEvent`'s sixteen tags (`packages/llm/src/schema/events.ts:79-201`, unioned via
`Schema.toTaggedUnion("type")`, `:209-226`): `step-start`, `text-start/-delta/-end`,
`reasoning-start/-delta/-end`, `tool-input-start/-delta/-end`, `tool-call`, `tool-result`,
`tool-error`, `step-finish`, `finish`, `provider-error`.

### 5.2 `packages/llm`'s Protocol/Route/Endpoint/Auth/Framing split

This is the part explicitly worth re-deriving for a Rust OpenRouter-only client, even though we
only need one branch of it. The doc-comment at `packages/llm/src/route/protocol.ts:4-35` states
the design intent directly:

> A `Protocol` owns the parts of a route that are intrinsic to "what does this API look like":
> how a common `LLMRequest` becomes a provider-native body, what schema that body must satisfy...
> and how the streaming response decodes back into common `LLMEvent`s. A `Protocol` is **not** a
> deployment. It does not know which URL, which headers, or which auth scheme to use... This
> separation is what lets DeepSeek, TogetherAI, Cerebras, etc. all reuse `OpenAIChat.protocol`
> without forking 300 lines per provider.

Concretely, four orthogonal pieces (`route/client.ts:36-53`, doc-comment at `:306-320`):

- **`Protocol<Body, Frame, Event, State>`** (`route/protocol.ts:36-63`) — pure, transport-free:
  `body.from(LLMRequest) -> Body` (build the provider-native JSON), `body.schema` (validate it
  before sending), `stream.initial/step/onHalt` — a fold (`Stream.mapAccumEffect`,
  `route/client.ts:288-292`) that turns a sequence of provider-native decoded `Event`s into
  `LLMEvent`s plus updated `State`, i.e. **the streaming parser is just `reduce`, not a stateful
  class**.
- **`Endpoint`** — base URL + path.
- **`Auth`** — how the request gets signed/bearer-authed (`route/auth.ts`, `route/auth-options.ts`
  — for OpenRouter this is `AuthOptions.bearer(input, "OPENROUTER_API_KEY")`,
  `providers/openrouter.ts:84`).
- **`Framing`** — how the raw HTTP byte stream is cut into protocol frames before JSON-decoding
  each one; SSE (`Framing.sse`) for chat/completions-style APIs, a binary AWS event-stream framer
  for Bedrock (`protocols/bedrock-event-stream.ts`, not read in depth — irrelevant to us).

`Route.make({ protocol, endpoint, auth, framing })` (`route/client.ts:303-339`) composes these
into a callable `Route` whose `streamPrepared(...)` does: HTTP-transport frames → decode each
frame against `protocol.stream.event` schema (`:283-284`) → fold through `protocol.stream.step`
into `LLMEvent`s (`:288-291`) → wrap stream-level failures into a typed `LLMError`
(`:293`, `streamError`, `:220-224`).

`compile(request)` (`route/client.ts:344-359`) is the top of this pipeline for a single call:
merge route/model/request-level generation & provider options (`resolveRequestOptions`,
`:167-180`), apply cache-hint policy (`applyCachePolicy`, not read in depth), build+validate the
provider body, then `route.prepareTransport(body, resolved)` to get an HTTP-ready `Prepared`
value. `LLMClient.stream`/`generate` (`:396-408`) are the two public entry points — `generate`
just folds the whole `stream` into one `LLMResponse` (`:382-391`) via `Stream.runFold`, useful
for non-interactive calls like title-generation (`prompt.ts:225-242`, uses `llm.stream(...)`
directly and reduces via `Stream.mkString`) or `generateObject` (see below).

### 5.3 OpenRouter specifically

`packages/llm/src/providers/openrouter.ts` — the entire OpenRouter adapter is ~98 lines because
it reuses the OpenAI-chat protocol wholesale:

- Base URL `https://openrouter.ai/api/v1` (`providers/openai-compatible-profile.ts:12`, one entry
  in a shared table alongside `groq`, `deepseek`, `cerebras`, `fireworks`, `togetherai`, `xai`,
  `deepinfra`, `baseten` — **OpenRouter is treated as just another OpenAI-compatible chat
  endpoint**, not a bespoke integration).
- `protocol = Protocol.make({ id: "openrouter-chat", body: { schema: OpenRouterBody, from: ... },
  stream: OpenAIChat.protocol.stream })` (`openrouter.ts:38-54`) — the *request* body schema is
  `OpenAIChat`'s body **plus an open extra-fields record** (`Schema.StructWithRest(...,
  [Schema.Record(String, Any)])`, `:33-36`) so OpenRouter-specific fields (`usage.include`,
  `reasoning`, `prompt_cache_key`) can ride alongside standard OpenAI-chat fields
  (`bodyOptions`, `:56-67`); the *streaming* parser is reused **unchanged** from `OpenAIChat`
  (`:53`) because OpenRouter's SSE stream is wire-compatible with OpenAI chat/completions.
  Practical takeaway for a Rust port: implementing "OpenAI chat/completions SSE" gets you
  OpenRouter (and Groq et al.) for free; OpenRouter needs zero protocol-level special-casing
  beyond a different base URL, bearer token env var, and a small bag of optional extra body
  fields.
- Route: `POST {baseURL}/chat/completions`, SSE framing, bearer auth from `OPENROUTER_API_KEY`
  (`:69-86`).
- `configure(input)` / `provider.model(id)` (`:88-99`) is the generic per-provider factory
  pattern every provider file in `packages/llm/src/providers/` implements, taking an optional
  override `baseURL`/`apiKey`/headers so self-hosted or proxy deployments still work.

### 5.4 Model-capability detection

Two layers, both in the **V1** production provider system (`packages/opencode/src/provider/`),
not `packages/llm`:

- **Catalog data** comes from `models.dev` (`ModelsDev` service, referenced `provider.ts:12`),
  a community-maintained JSON metadata source giving, per model: `temperature`, `reasoning`,
  `attachment`, `tool_call` support, input/output `modalities` (`text`/`audio`/`image`/`video`/
  `pdf`), context/output token limits, and pricing tiers.
- That's normalized into a `ProviderCapabilities` shape at `provider.ts:1035-1036` and populated
  per-model at `provider.ts:1520-1546`: `{ temperature, reasoning, attachment, toolcall,
  input: {text,audio,image,video,pdf}, output: {...}, interleaved (reasoning-field variant, e.g.
  "reasoning_content" vs "reasoning_text") }`.
- These flags gate request-shaping in `ProviderTransform` (`provider/transform.ts`, 1909 lines,
  not read in depth) — e.g. whether to send `reasoning_effort`, whether to strip images from a
  tool result for a model that can't accept media there and instead synthesize a follow-up user
  message carrying the image (`message-v2.ts:141-150`'s `supportsMediaInToolResult` allowlist by
  provider SDK package name).
- The **V2 spec's** `ModelV2.Info.capabilities` (`specs/v2/provider-model.md`) simplifies this to
  just `{ tools: boolean, input: string[], output: string[] }` plus a separate `status`
  (`alpha|beta|deprecated|active`) and `enabled` flag, with provider-level `enabled` gating
  computed from *how* the provider became available (`env`/`account`/`custom`) — a smaller,
  cleaner shape than V1's, worth copying over V1's for a from-scratch port since we don't need
  the historical cruft.
- Structured/JSON output is deliberately **not** implemented via provider-native JSON modes at
  all (`packages/llm/src/llm.ts:146-186`, `generateObject`): it forces a synthetic tool call
  (`GENERATE_OBJECT_TOOL_NAME = "generate_object"`, `ToolChoice.named(...)`) and decodes the
  tool's input against the target schema — uniform behavior across every provider, including
  ones with no native JSON mode at all. The V1 runtime does the same thing for its own
  `StructuredOutput` tool (`session/prompt.ts:1565-1591`, `createStructuredOutputTool`).

### 5.5 Error taxonomy

`packages/llm/src/route/executor.ts` is the layer worth copying almost verbatim into a Rust
OpenRouter client: it classifies every non-2xx response into a typed reason
(`AuthenticationReason` 401/403, `RateLimitReason` 429 with parsed `Retry-After`/
`X-RateLimit-*`/Anthropic-style rate headers, `QuotaExceededReason` (429 + "insufficient quota"
body pattern), `InvalidRequestReason` (400/404/409/413/422, with a `classification:
"context-overflow"` sub-case detected from the response body, `:253-265`), `ContentPolicyReason`
(body matches `/content[-_\s]?policy|content_filter|safety/i`, `:233-235`),
`ProviderInternalReason` (5xx / 429/503/504/529), `TransportReason` (network/timeout failures
before any HTTP response), `UnknownProviderReason` (fallback) — `executor.ts:225-275`. It also:
redacts every plausible secret (header values, query params, JSON body fields matching a
sensitive-name regex, and a second literal pass that replaces any actual outbound secret value
that got echoed back) before logging/erroring (`:41-201`) — worth copying wholesale for any
OpenRouter client that logs request/response bodies for debugging; and retries only requests
whose reason is explicitly `retryable`, honoring the provider's `Retry-After` when present,
otherwise capped exponential backoff with jitter (`retryDelay`, `:345-364`).

---

## 6. Subagent / Task mechanism

`tool/task.ts:81-372` (§3.2 named this as a concrete tool example; here's the full mechanism).

- **A subagent is not a different runtime** — it's a brand-new `Session` row
  (`sessions.create({ parentID: ctx.sessionID, agent: next.name, permission: [...] })`,
  `:156-172`) that gets run through the exact same `SessionPrompt.prompt`/`loop` the parent uses,
  via a narrow capability object `TaskPromptOps { cancel, resolvePromptParts, prompt }`
  (`:18-22`) threaded in through `ctx.extra.promptOps` (bound at the top level in
  `prompt.ts:144-150`, `ops()`). This is why `runLoop` treats a queued `subtask` Part specially
  (`handleSubtask`, `prompt.ts:255-449`) *and* why the `task` tool's own `execute` can also
  directly call `ops.prompt(...)` (`task.ts:200-225`) — there are two call sites for "run a
  subagent" (one from a `/command`-style pre-declared subtask, one from the model calling `task`
  mid-conversation) but they converge on the same recursive prompt call.
- **What's shared vs isolated:**
  - *Isolated*: the child gets its **own** message/part history (a fresh session — the parent's
    conversation is not copied in; only the `prompt` text the model/user supplied is passed,
    `:201-212`), its own model selection (defaults to the parent turn's model unless the agent
    config pins one, `:181-184`), and its own permission ruleset computed by
    `deriveSubagentSessionPermission` (`agent/subagent-permissions.ts:14-27`): **only the
    parent's `deny` rules and `external_directory` rules carry over**; everything the subagent
    is otherwise allowed to do comes from its *own* agent config, not inherited allow-rules from
    the parent. Plus two hardcoded denies unless the subagent's own config explicitly grants them:
    it cannot call `todowrite` or spawn its own `task` (nested subagents) by default.
  - *Shared*: nothing else — no shared mutable state, no shared tool-call cache. The **link**
    back to the parent is purely `Session.parentID` plus the `task_id` (== child session ID)
    the model can pass back in to *resume* the same child session on a follow-up call
    (`Parameters.task_id`, `:47-50`, `:136-138`) instead of creating a new one each time.
  - A hard **depth limit** (`cfg.subagent_depth ?? 1`, default 1) is enforced by walking
    `parentID` chains before allowing another nested spawn (`:104-117`) — subagents cannot spawn
    subagents by default, and even when allowed, depth is bounded, not just permission-gated.
- **Foreground mode** (default): the tool call blocks (`Effect.raceFirst` between the child
  session finishing and a "promoted to background" signal, `:328-358`) — if the *parent's* tool
  call gets aborted, it propagates cancellation into the *child* session too (`ops.cancel`,
  `:322-326`, `onAbort`), rather than leaving an orphaned child loop running.
- **Background mode** (`background: true`, gated behind an experimental flag, `:96-102`):
  `BackgroundJob.Service.start(...)` runs the child session's prompt loop on a forked fiber and
  returns an immediate "task started, you'll be notified" tool result
  (`BACKGROUND_STARTED` text, `:31-35`, `renderOutput`, `:299-319`); when it later finishes, the
  result is **injected back into the parent session as a new synthetic user-role prompt**
  (`inject(...)`, `:227-254`, calling `ops.prompt(...)` again on the *parent* session) — i.e. a
  background subagent's completion re-enters the parent's normal loop as if the user had sent a
  new message, rather than needing any special "resume with subagent result" code path.
- The `task` tool's own description is dynamically extended per-call with the live list of
  available (non-`primary`-mode, permission-visible) agent names and descriptions
  (`ToolRegistry.describeTask`, `registry.ts:265-278`) — the model always sees an accurate,
  live-filtered menu of what it's allowed to delegate to, computed fresh per turn rather than
  baked into a static system prompt.

---

## 7. Client/server split, permission round-trip, and extensibility (brief — out of primary scope)

- **Process boundary**: `packages/opencode` is a headless server exposing an Effect `HttpApi`
  (`server/routes/instance/httpapi/`, one `HttpApiGroup` per resource — `session`, `permission`,
  `provider`, `mcp`, `file`, `pty`, ... — assembled in `api.ts`). The TUI/desktop/web clients are
  pure HTTP+SSE consumers of this server; none of them touch SQLite or the model directly. This
  is the client/server boundary — everything in §2-§6 lives entirely server-side.
- **Live updates** are one global `GET /event` SSE stream per client connection
  (`handlers/event.ts:24-50`): every domain event (message updated, part delta, permission asked,
  session status changed, ...) is published to an in-process bus and re-emitted as an
  `{id, type, properties}` SSE frame, filtered to the connecting client's
  directory/workspace (`:37-40`). There is no per-resource polling; a client opens one stream and
  gets everything relevant to its scope.
- **Permission round-trip over HTTP**: `permission.asked` arrives over that same SSE stream; the
  client's answer is a normal `POST /session/:id/permissions/:requestID` REST call
  (`groups/session.ts:395-408`) that resolves the server-side `Deferred` described in §3.3. The
  loop-pausing mechanism (§3.3) and the client transport are fully decoupled — a CLI, a TUI, and
  a web dashboard all resolve the exact same in-process `Deferred` through the exact same REST
  call.
- **Extensibility**: two mechanisms, both explicitly out of deep scope here but worth naming.
  (1) **Plugins** (`packages/opencode/src/plugin/`) are npm packages (or local
  `.opencode/plugins/*.ts` files) loaded at startup (`plugin/loader.ts`) that register hook
  callbacks (`tool.execute.before/after`, `chat.message`, `provider.update`,
  `experimental.session.compacting`, `experimental.chat.messages.transform`, `tool.definition`,
  ...) — every `plugin.trigger(name, input, output)` call seen throughout §2-§6 is one of these
  hook points, and plugins can also directly contribute new tools (`ToolRegistry`'s
  `fromPlugin(...)`, `registry.ts:125-181`) or providers. (2) **MCP servers** are configured
  remote/local tool providers; their tool defs get converted through `McpCatalog.convertTool` and
  merged into the same per-turn tool map as built-ins (`session/tools.ts:390-489`), and their
  *resources* (not just tools) are separately exposed via three built-in
  `list_mcp_resources`/`list_mcp_resource_templates`/`read_mcp_resource` tools
  (`session/tools.ts:27-31, 136-386`) rather than being silently inlined.

---

## 8. What ShadowPrompt v3 should steal vs skip

ShadowPrompt v3's shape is fundamentally simpler than OpenCode's on every axis that drives
OpenCode's complexity: **one** LLM provider (OpenRouter) instead of 75+, **one** client (the
stealth daemon itself, no separate TUI/desktop/web/HTTP-API surface to serve), **6-8 fixed,
narrow tools** (no filesystem/shell access, no plugin-contributed tools, no MCP) instead of an
open-ended registry, and (presumably) **no durable multi-session history to browse/resume** in
the way a coding agent's session list is a first-class product feature. That changes the
cost/benefit of almost everything in this document.

### Steal

- **The event-stream/loop/tool separation itself (§1, §2).** This is the actual architectural
  win and it costs nothing extra to do right from the start: define one canonical `LlmEvent` enum
  (text-delta, tool-call-start/args/end, tool-result, finish, error) up front, have your
  OpenRouter SSE parser emit *only* that enum, and write the turn-loop against the enum, never
  against OpenRouter's wire shapes directly. Even with one provider, this keeps your streaming
  parser and your "what do I do with a tool call" logic from tangling together, and it's exactly
  what would let you swap OpenRouter for a direct provider later without touching the loop.
- **Recompute loop state from durable history every iteration, don't carry it in a local
  variable across turns (§2.0, §2.3).** ShadowPrompt already writes to a RAG index and (per
  `PROJECT_DOCUMENTATION.md`'s data flow) reads clipboard → calls LLM → writes clipboard as one
  shot per hotkey, so there's currently no multi-turn "session" to speak of. If v3 adds any
  multi-step tool-calling (e.g. "search web, then answer"), copy this rule anyway: whatever
  decides "call the model again vs. stop" should be a pure function of what's actually been
  persisted/observed so far, not of an in-memory counter that a panic/crash could desync from
  reality.
- **The typed error taxonomy + redaction + retry policy from §5.5.** This is cheap, provider-count
  independent, and directly protects ShadowPrompt's "auto-cascade Groq → OpenRouter → Ollama"
  retry logic (`CLAUDE.md`'s `llm.rs`) from silently retrying non-retryable errors (bad API key,
  content policy) or leaking the API key into `data/logs/error.log`. Concretely: classify
  OpenRouter HTTP failures into `Auth | RateLimit(retry_after) | QuotaExceeded | InvalidRequest |
  ContentPolicy | ServerError(retryable) | Transport`, and scrub `Authorization`/`api_key`-shaped
  fields before anything gets logged.
- **Output-size handling as two separate layers (§3.4).** ShadowPrompt's tools are narrow (OCR
  text, web search results, RAG hits — not arbitrary file reads), but a runaway OCR capture or a
  bloated search-result page is exactly the same shape of problem: cap bytes *while producing*
  the string (before it ever reaches the LLM-context builder), and separately cap what actually
  gets sent to the model, with a short deterministic "...N chars omitted..." marker rather than a
  silent cut. Given ShadowPrompt is stealth/latency-sensitive with no user-facing "read the full
  file" follow-up UI, skip OpenCode's "write full output to a scratch file for later" half of
  this — just cap-and-mark; there's no interactive follow-up loop to make the saved file useful.
- **The permission-ask-as-a-blocked-fiber pattern, drastically simplified (§3.3).** ShadowPrompt
  is a single-user stealth tool with no team-shared config and (today) no "ask before running a
  tool" UX at all — but *if* v3 adds any tool with real-world side effects (e.g. actually
  submitting a form via the browser-automation module, not just reading one), copy the shape:
  synchronous allow/deny/ask evaluation against a small static ruleset, and only pay for an
  actual pause (a channel/oneshot the tool-executing task awaits on) in the "ask" case. Skip
  OpenCode's wildcard-pattern-matching ruleset language and "always allow this pattern for the
  rest of the session" bookkeeping — with 6-8 fixed tools and one user, a flat per-tool
  allow/deny/ask enum in `config.toml` covers it.
- **Provider abstraction's four-axis split (§5.2), collapsed to the one axis you need.** You
  don't need `Protocol`/`Endpoint`/`Auth`/`Framing` as four separate swappable pieces when you
  have exactly one endpoint (OpenRouter chat/completions) and it's already OpenAI-chat-shaped —
  but keep the *reason* for the split: put "how do I turn my internal request struct into
  OpenRouter's JSON body" and "how do I fold an SSE frame into my `LlmEvent` enum" behind their
  own small functions/modules, not inlined into the HTTP call site, so that adding a second
  provider later (or OpenRouter's alternate response shapes, e.g. non-streaming `generate`) is a
  new small module, not a rewrite.

### Skip

- **Multi-provider catalog, models.dev integration, per-model capability negotiation (§5.4).**
  ShadowPrompt already hardcodes its provider cascade and model choices in `config.toml`
  (`[models.groq]`, `[models.openrouter]`, `[models.ollama]` per `CLAUDE.md`); there is no product
  need to dynamically discover "does this model support vision/tools/reasoning" from a live
  catalog. Keep `ModelCapabilities` (`capabilities.rs`) as a small hand-maintained struct per
  configured model, not a registry.
- **The whole SQLite message/part persistence model + compaction (§4).** This exists because
  OpenCode sessions are long-lived, resumable, and browsable as a product feature (you can close
  the TUI and reopen a session from a week ago). ShadowPrompt's actual use case per
  `PROJECT_DOCUMENTATION.md` is single-shot Q&A per hotkey press with a short-lived RAG-augmented
  prompt — there is no multi-hour conversation to compact. If v3 grows a genuine multi-turn
  "chat mode" that can run long enough to hit a context window, a much smaller version of §4.2's
  pruning idea (drop old tool-output text once a token budget is exceeded, keep recent turns
  verbatim) is enough; full rolling-summary compaction via an extra LLM call is not worth the
  complexity or the extra OpenRouter spend for what's meant to be a fast, invisible exam aid.
- **Subagent/Task mechanism (§6) as a general recursive-session system.** Spinning up an
  isolated child "session" with its own permission derivation is solving a problem ShadowPrompt
  doesn't have (no team of specialized coding subagents). If v3 wants something conceptually
  similar — e.g. a background web-search call that runs concurrently with OCR and reports back —
  a single `tokio::spawn`ed task with a oneshot/mpsc result channel gets the same effect (§6's
  "background mode re-injects as a new prompt" idea, simplified to "background task, channel,
  and the next hotkey-triggered turn checks the channel") without needing session objects,
  parent/child linkage, or a permission-inheritance algorithm.
- **Client/server HTTP+SSE split and the multi-client protocol (§7).** ShadowPrompt is
  deliberately a single, no-window, no-server process — there is no second client to serve, so
  there is no reason to introduce a process boundary, an HTTP API, or an SSE event bus. Keep the
  existing in-process `mpsc` channel design (`InputEvent`/`UICommand`) exactly as documented in
  `CLAUDE.md`; it already achieves, in-process, what OpenCode needs a server for.
  Correspondingly, skip plugins and MCP entirely (§7) — a stealth tool with a small fixed toolset
  built for one purpose has no third-party-extensibility use case, and adding one would only
  increase the attack/detection surface a stealth tool is specifically trying to minimize.
  Similarly skip OpenCode's Effect-TS-specific plumbing (`LayerNode`, `Context.Service`,
  `InstanceState`) — it exists to manage per-workspace service lifecycles across many concurrent
  client sessions, which has no equivalent in a single-instance Rust daemon; plain structs, `Arc`,
  and Tokio channels/tasks are the right-sized replacement.

---

## Appendix: file index for this document

| Area | Key files (all under the OpenCode clone root) |
|---|---|
| Specs (design intent) | `specs/v2/{session,tools,provider-model,provider-policy,config,instructions,todo}.md`, `specs/storage/*.md` |
| Main loop | `packages/opencode/src/session/prompt.ts`, `session/processor.ts`, `session/run-state.ts` |
| Message/part persistence + compaction | `session/message-v2.ts`, `session/compaction.ts`, `session/overflow.ts` |
| Tool interface + registry | `tool/tool.ts`, `tool/registry.ts`, `session/tools.ts`, `tool/truncate.ts` |
| Concrete tools read | `tool/read.ts`, `tool/task.ts`, `tool/shell.ts`, `tool/webfetch.ts` |
| Permission | `permission/index.ts`, `permission/arity.ts`, `agent/subagent-permissions.ts` |
| Provider/model (V1, production) | `provider/provider.ts`, `provider/transform.ts` |
| LLM wire layer (V2, Effect-native) | `packages/llm/src/route/{client,protocol,executor}.ts`, `packages/llm/src/providers/openrouter.ts`, `packages/llm/src/providers/openai-compatible-profile.ts`, `packages/llm/src/schema/{events,messages}.ts`, `packages/llm/src/llm.ts` |
| HTTP/SSE server boundary | `server/routes/instance/httpapi/groups/session.ts`, `.../handlers/{session,event}.ts` |
| Extensibility | `plugin/loader.ts`, `mcp/catalog.ts`, `skill/discovery.ts` |
