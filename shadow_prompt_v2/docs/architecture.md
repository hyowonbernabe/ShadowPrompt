# ShadowPrompt v2 — Architecture

> Source of truth for the runtime architecture, module boundaries, dependency choices, and concurrency model.
> Companion to `features.md` (what we build) and `agents.md` (how we talk to the model).
> Last updated: 2026-05-17

---

## Goals

The architecture must serve five non-negotiable properties from `features.md`:

1. **Single executable, lightweight.** Small binary, minimal cold start.
2. **Stealth.** No taskbar, no console in release, no visible window.
3. **Stateless by default.** No persistent conversation, no on-disk session state.
4. **One-command install.** No GUI setup, no runtime dependencies.
5. **Provider-transport, model-agnostic.** OpenRouter as transport; Claude as default; any multimodal model as drop-in.

Everything below serves those.

---

## High-Level Runtime Topology

```
                ┌──────────────────────────────────────────────────┐
                │              shadowprompt.exe                    │
                │                                                  │
   OS Thread A  │   ┌──────────────────────────────────────────┐   │
   (rdev hook)  │   │ Input listener                            │   │
                │   │  • Global keyboard hook                   │   │
                │   │  • Mouse hook (only during OCR mode)      │   │
                │   │  • Hotkey state machine (double-tap arm)  │   │
                │   └────────────────┬─────────────────────────┘   │
                │                    │ InputEvent (mpsc)            │
                │                    ▼                              │
   Main thread  │   ┌──────────────────────────────────────────┐   │
   (tokio)      │   │ Orchestrator (main event loop)            │   │
                │   │  • Receives InputEvent                    │   │
                │   │  • Dispatches to action handlers          │   │
                │   │  • Manages in-flight task handle          │   │
                │   └─┬──────────────────────────────────┬─────┘   │
                │     │ spawn task                       │ UICommand│
                │     ▼                                  │ (mpsc)   │
                │   ┌──────────────────────────────┐    ▼          │
                │   │ Action handlers (free async   │  ┌─────────┐  │
                │   │  fns):                        │  │ UI loop │  │
                │   │  - clipboard_query            │  │ (Win32  │  │
                │   │  - ocr_query                  │  │  GetMsg │  │
                │   │  - forms_run                  │  │  pump)  │  │
                │   │  - launch_debugger            │  │         │  │
                │   │  - hide_toggle                │  │ Owns    │  │
                │   │  - abort_active               │  │ pixel + │  │
                │   │  - panic_kill                 │  │ overlay │  │
                │   │  - restart_daemon             │  │ windows │  │
                │   │  - self_delete                │  └─────────┘  │
                │   └──────────────────────────────┘    ▲          │
                │     │                                 │          │
                │     │ uses                            │          │
                │     ▼                                 │          │
                │   ┌──────────────────────────────────────────┐   │
                │   │ Service layer                             │   │
                │   │  • LlmClient (OpenRouter)                 │   │
                │   │  • Clipboard, OCR, screen capture         │   │
                │   │  • Chrome session (headless_chrome)       │   │
                │   │  • Forms extractor / injector             │   │
                │   │  • Image resize / encode                  │   │
                │   └──────────────────────────────────────────┘   │
   OS Thread B  │                                                  │
   (Win32 pump) │                                                  │
                └──────────────────────────────────────────────────┘
```

Three execution contexts:

1. **Input thread (OS thread, sync)** — `rdev` listener. Translates raw key/mouse events into `InputEvent` values. Owns the hotkey state machine (double-tap arming).
2. **UI thread (OS thread, sync)** — Win32 message loop (`GetMessage` / `DispatchMessage`). Owns all visible windows. Consumes `UICommand` via a non-blocking channel poll inside the pump.
3. **Tokio runtime (main thread + worker pool)** — orchestrator + async action handlers + network IO + browser IO. Multi-thread flavor.

Channels:

- `mpsc<InputEvent>` — input thread → orchestrator.
- `mpsc<UICommand>` — orchestrator (and action tasks) → UI thread.

This is the same channel-per-concern shape as v1, intentionally. It works.

---

## Layered Module Boundaries

```
src/
├── main.rs                    -- thin entry: CLI parse, runtime init, wiring
├── lib.rs                     -- public surface for integration tests
│
├── config/                    -- TOML schema, defaults, path resolution
│   ├── mod.rs
│   ├── paths.rs               -- exe-relative path helpers
│   ├── schema.rs              -- Config struct + serde derives
│   └── load.rs                -- load-or-init, validation
│
├── lifecycle/                 -- process lifecycle / nuke helpers
│   ├── mod.rs
│   ├── panic.rs               -- clipboard wipe + immediate exit
│   ├── self_restart.rs        -- detached cmd.exe relaunch
│   ├── self_delete.rs         -- nuke folder + PATH + Chrome procs
│   └── chrome_cleanup.rs      -- kill spawned debug Chrome processes
│
├── input/                     -- raw keyboard/mouse → InputEvent
│   ├── mod.rs                 -- InputManager, rdev thread
│   ├── events.rs              -- InputEvent enum
│   ├── parser.rs              -- "ctrl+shift+v" → key combo
│   └── state_machine.rs       -- double-tap arming for insta-delete
│
├── ui/                        -- Win32 visible windows
│   ├── mod.rs                 -- UIManager, message loop thread
│   ├── commands.rs            -- UICommand enum
│   ├── state.rs               -- centralized UiState struct
│   ├── indicator.rs           -- ready / processing pixel
│   ├── form_indicator.rs      -- Forms status pixel
│   ├── overlay.rs             -- text overlay window
│   └── debug_rect.rs          -- OCR region preview rectangle
│
├── actions/                   -- one async fn per hotkey-driven action
│   ├── mod.rs                 -- ActionContext, dispatcher, task handle slot
│   ├── clipboard_query.rs
│   ├── ocr_query.rs
│   ├── forms_run.rs
│   ├── launch_debugger.rs
│   ├── hide_toggle.rs
│   └── abort_active.rs
│
├── llm/                       -- OpenRouter client + request shaping
│   ├── mod.rs                 -- LlmClient
│   ├── request.rs             -- build_request(model, msgs, caps)
│   ├── response.rs            -- parse OpenAI-compat response
│   ├── messages.rs            -- Message, ContentPart (text/image)
│   ├── capabilities.rs        -- static table: model_id → caps
│   ├── system_prompts.rs      -- embedded system prompts
│   └── retry.rs               -- 429/5xx backoff
│
├── capture/                   -- screen + clipboard + image pipeline
│   ├── mod.rs
│   ├── clipboard.rs           -- arboard wrapper
│   ├── ocr.rs                 -- Windows.Media.Ocr (WinRT)
│   ├── screen.rs              -- GDI region capture → PNG bytes
│   └── image.rs               -- resize to 1568px + base64 encode
│
├── browser/                   -- Chrome control + Google Forms flow
│   ├── mod.rs                 -- ChromeSession (attach or launch)
│   ├── debugger.rs            -- spawn incognito with --remote-debugging-port
│   ├── cookies.rs             -- rookie cookie extraction
│   └── forms/
│       ├── mod.rs             -- execute_form_flow orchestrator
│       ├── extractor.rs       -- EXTRACTOR_JS + parse to typed Question structs
│       ├── injector.rs        -- inject model answers into DOM
│       ├── answered.rs        -- detect user-filled questions, skip them
│       └── submit_guard.rs    -- Submit button detection + refusal
│
└── logger.rs                  -- simplelog init
```

Dependency direction is strictly downward — `actions` may depend on `llm`, `capture`, `browser`; none of those may depend on `actions` or `ui`. `ui` and `input` are sinks/sources at the edge.

---

## Concurrency Model

### Single in-flight non-Forms query

Only one clipboard or OCR query runs at a time. Firing a new query while one is still in flight cancels the previous one.

Implementation: `ActionContext` holds `Arc<Mutex<Option<JoinHandle<()>>>>`. Each action that wants exclusivity:

1. Lock the mutex.
2. If a handle is present, call `handle.abort()` and drop it.
3. Spawn the new task, store its handle.
4. Release the lock.

This is the same pattern v1 uses for Forms abort, generalized to all queries.

### Forms runs are exclusive

A Forms run takes the same handle slot. While a Forms run is active, clipboard and OCR queries are dropped (with an error indicator flash) rather than queued. The abort hotkey is the only supported way to cancel mid-Forms.

Rationale: Forms holds the Chrome session and rapid-fires LLM calls; interleaving stateless queries would either fight for the Chrome session or pollute UI state. Cleaner to refuse.

### Tokio runtime flavor

Multi-thread runtime, default worker count (number of CPU cores). Forms + image resize + HTTP all benefit from parallelism. Memory cost (a few hundred KB extra) is acceptable.

### UI thread is sync, never blocks the orchestrator

The Win32 message pump on the UI thread polls the `UICommand` channel via `PeekMessage` between dispatch calls. Painting and child-window updates happen in response to received commands. The orchestrator never waits on the UI thread.

### Action cancellation safety

Every action handler is written so that being cancelled mid-await leaves no global state in a corrupt form. Concretely:

- Clipboard/OCR queries: stateless; nothing to clean up.
- Forms runs: ephemeral Forms conversation is task-local on the stack, dropped automatically. Chrome session is held by the action; on cancellation, the Chrome session drops, the browser keeps the form in whatever state it was last in. No rollback.
- All other actions: short-lived, no cancellation concerns.

---

## State Management

There is no persistent application state on disk beyond `config.toml`. In-memory state is partitioned:

| State | Owner | Lifetime |
|---|---|---|
| `Config` | `Arc<Config>` shared via `ActionContext` | Process lifetime, reloaded only on restart |
| `LlmClient` | `Arc<LlmClient>` (wraps `reqwest::Client`) | Process lifetime |
| Active task handle | `Arc<Mutex<Option<JoinHandle>>>` in `ActionContext` | Per query / Forms run |
| Forms ephemeral conversation | `Vec<Message>` local to `forms_run::execute` | Single form flow |
| UI state | `UiState` struct guarded by `Mutex`, owned by UI thread | Process lifetime |
| Hotkey arm state | `HotkeyStateMachine` owned by input thread | Per double-tap window |
| Chrome session | `Browser` handle, owned by Forms task | Single form flow |

Notable absences: no conversation history across queries, no token counter, no cache state, no recent-Q ring buffer. By design.

---

## Lifecycle

### Cold start

```
main()
  ├─ Parse CLI args (--version, --reset-config, default = start daemon)
  ├─ Initialize logger (simplelog to data/logs/app.log)
  ├─ Load config (or fail with helpful message if missing/invalid)
  ├─ Validate model id supports vision (via capabilities table)
  ├─ Construct ActionContext (config, llm_client, channels, task slot)
  ├─ Spawn input thread (rdev listener)
  ├─ Spawn UI thread (Win32 message loop)
  ├─ Enter tokio runtime
  └─ Run orchestrator event loop
```

### Hot operation

Orchestrator loop:

```
loop {
  let evt = input_rx.recv().await;
  match evt {
    InputEvent::Clipboard      => spawn(clipboard_query(ctx.clone())),
    InputEvent::Ocr            => spawn(ocr_query(ctx.clone())),
    InputEvent::FormsAuto      => spawn(forms_run(ctx.clone(), Auto)),
    InputEvent::FormsSingle    => spawn(forms_run(ctx.clone(), Single)),
    InputEvent::Abort          => abort_active(&ctx),
    InputEvent::LaunchDebugger => spawn(launch_debugger(ctx.clone())),
    InputEvent::Hide           => ctx.ui_tx.send(UICommand::ToggleHide),
    InputEvent::Restart        => self_restart::execute(&ctx).await,
    InputEvent::InstaDelete    => self_delete::execute(&ctx).await,
    InputEvent::Panic          => panic::execute(&ctx),
  }
}
```

### Shutdown

- **Normal exit:** all channels drop, threads observe close, tokio runtime joins workers, process exits.
- **Panic key:** clipboard cleared, immediate `std::process::exit(0)`, no graceful shutdown.
- **Restart key:** spawn detached `cmd.exe` to relaunch; exit cleanly.
- **Insta-delete:** double-tap arm flow; on confirm, spawn detached cleanup `cmd.exe` (PATH strip, folder deletion); exit cleanly.

---

## Error Handling

`anyhow::Result<T>` is used throughout the application. No `thiserror`, no custom error enums. Errors are built up with `.context("doing X")` at boundaries so the resulting chain reads top-down.

Error reporting:

- **At an action boundary**, the action sends `UICommand::ShowError(short_message)` and logs the full chain at `error` level.
- **At the orchestrator**, panics in spawned tasks are caught (`JoinHandle::join_error`) and logged, but do not kill the process. The user sees a red indicator and can retry.

The only path that intentionally bypasses graceful error handling is the panic key, which exits without flushing logs.

---

## Capability Detection

The daemon must adapt request shape to the configured model. Capabilities are detected from the model id prefix:

```rust
// llm/capabilities.rs
pub struct Capabilities {
    pub vision: bool,
    pub reasoning: bool,
    pub prompt_caching: bool,
    pub context_window: usize,
}

pub fn for_model(id: &str) -> Capabilities {
    match id {
        // Claude family
        s if s.starts_with("anthropic/claude-sonnet-4")  // 4.6, 4.7, ...
          || s.starts_with("anthropic/claude-opus-4")
            => Capabilities { vision: true, reasoning: true, prompt_caching: true, context_window: 1_000_000 },

        // Gemini family
        s if s.starts_with("google/gemini-2.5-pro")
          || s.starts_with("google/gemini-3-pro")
            => Capabilities { vision: true, reasoning: true, prompt_caching: false, context_window: 1_000_000 },

        // OpenAI family
        s if s.starts_with("openai/gpt-5") || s.starts_with("openai/gpt-4o")
            => Capabilities { vision: true, reasoning: true, prompt_caching: false, context_window: 200_000 },

        // Unknown — conservative defaults; startup validation will reject if vision missing
        _ => Capabilities { vision: false, reasoning: false, prompt_caching: false, context_window: 128_000 },
    }
}
```

Static, compiled into the binary, rebuilt when new model families land. Rejected alternative: fetch from OpenRouter's `/models` endpoint at startup — adds network dependency on cold start, slows launch, adds a failure mode for offline-then-online users.

---

## Request Building

`llm/request.rs::build_request(model_id, messages, caps)` constructs the OpenRouter request body:

```
1. Base request: { model, messages, max_tokens, stream: false }
2. If caps.reasoning:  add { reasoning: { effort: "high", exclude: true } }
3. If caps.prompt_caching:  add cache_control: ephemeral 1h on system message
4. If Claude model + caps.context_window == 1_000_000:
     add extra header anthropic-beta: context-1m-2025-08-07
5. Return Request struct ready for reqwest::Client::send()
```

This is the only place provider-specific logic exists. Action handlers stay provider-agnostic.

---

## Image Pipeline

Every image flowing into a request passes through one function:

```
capture/image.rs::prepare(png_bytes) -> ContentPart
  1. Decode PNG → DynamicImage
  2. If max(width, height) > 1568: resize keeping aspect ratio
  3. Re-encode as PNG
  4. base64 encode
  5. Wrap as ContentPart::Image { data_url, media_type: "image/png" }
```

Forms image policy (max 20 images per request, <18MB total) is enforced one level up in `forms/mod.rs` before calling `prepare()`.

---

## Hotkey State Machine

Most hotkeys translate directly: keypress → `InputEvent`. The exception is `Ctrl+Shift+Del` (insta-delete) which requires a double-tap within 2 seconds.

```
state: Idle | Armed { at: Instant }

on hotkey_press(InstaDelete):
  match state:
    Idle:
      state = Armed { at: now() }
      emit InputEvent::InstaDeleteArmed   -- UI flashes red
    Armed { at } if now() - at <= 2.seconds:
      state = Idle
      emit InputEvent::InstaDeleteConfirmed
    Armed { _ }:  -- expired, treat as fresh first press
      state = Armed { at: now() }
      emit InputEvent::InstaDeleteArmed

on tick (every 200ms):
  if let Armed { at } = state if now() - at > 2.seconds:
    state = Idle
    emit InputEvent::InstaDeleteDisarmed   -- UI clears red flash
```

Lives in `input/state_machine.rs`. Owned by the input thread.

---

## Self-Delete and Self-Restart

Running EXEs cannot delete themselves on Windows. Both restart and insta-delete use the same pattern:

```
1. Daemon writes a small helper batch script to %TEMP% (or generates an inline cmd line).
2. Daemon spawns cmd.exe with CREATE_NO_WINDOW | DETACHED_PROCESS.
3. The script:
   - Waits 1 second (gives daemon time to exit).
   - Performs its job (delete folder + PATH cleanup, OR relaunch exe).
   - Deletes itself.
4. Daemon exits.
```

For insta-delete the script also:

- Removes install path from user `PATH` (via PowerShell `[Environment]::SetEnvironmentVariable('Path', ...)`).
- Kills any leftover `chrome.exe` / `msedge.exe` processes spawned by this daemon (matched by command-line containing the daemon's temp profile path).

For restart the script just `start "" "shadowprompt.exe"`.

---

## Build Profile

`Cargo.toml` release profile (binary size optimization):

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

Trade-offs:

- `opt-level = "z"`: smallest code, slightly slower than `"s"` or `"3"`. Acceptable — we are IO-bound (network, screen capture, browser) not CPU-bound.
- `lto = true`: full link-time optimization. Slow compile (~2× longer). Worth it for releases.
- `codegen-units = 1`: better optimization, slow compile. Same trade.
- `panic = "abort"`: no unwinding tables. Saves 10–15% size. Panics still produce logs; they just abort instead of unwinding.
- `strip = true`: strips debug symbols. Massive size win on Windows MSVC builds.

Combined: roughly 40–60% smaller release binary vs the default profile.

Debug profile stays default (fast incremental builds).

---

## Testing Strategy

- **Unit tests** colocated with code in each module. Pure-logic functions (hotkey parser, capability table, image resize math, message builder) get unit coverage.
- **Integration tests** in `tests/` against the `lib.rs` public surface. Stub the LLM transport (mock `reqwest::Client`) and exercise action handlers end-to-end without hitting the network.
- **Manual Forms testing** required before release — there is no good way to mock real Google Forms behavior; a sample test form must be exercised manually.

---

## Locked Open Questions

For reference, the architecture questions resolved during planning:

1. **Single in-flight query model** — yes.
2. **`lib.rs` + `main.rs` split** — yes (enables integration tests).
3. **Capability table** — static (compiled in).
4. **Error handling** — `anyhow::Result` everywhere; no `thiserror`.
5. **Action shape** — free async functions; no Action trait.
6. **Tokio runtime flavor** — multi-thread, default worker count.
7. **Single crate vs workspace** — single crate.

---

## Locked Dependency Stack

| Concern | Crate | Notes |
|---|---|---|
| Async runtime | `tokio` | Multi-thread, full features |
| HTTP / SSE | `reqwest` (slim) | `default-features = false`, features = `["json", "rustls-tls"]` |
| Global hotkeys + mouse | `rdev` | v1 proven; supports F12 via low-level hook |
| Clipboard | `arboard` | 89% docs |
| Win32 + WinRT | `windows` (latest) | 100% Microsoft docs |
| Browser | `headless_chrome` | v1 proven |
| Cookies | `rookie` | v1 proven |
| Image | `image` | resize + encode |
| Config | `serde` + `toml` | standard |
| CLI args | `clap` | lightweight feature set |
| Logging | `simplelog` + `log` | v1 proven |
| Build resources | `embed-resource` | icon embedding |

Rejected during research:

- `win-hotkeys` (17% doc coverage — too sparse)
- `chromiumoxide` (key patterns we depend on are undocumented)
- `tracing` (overkill for v2.0; revisit in v2.1 if structured logs become valuable)
- `windows-capture` (marginal benefit, extra dep)
- `ureq` / `attohttpc` (sync; bad fit with tokio)

---

## What Survives From v1 vs What Changes

| Aspect | v1 | v2 |
|---|---|---|
| Two-phase startup | setup wizard → daemon | single startup, no wizard |
| Top-level modules | input/ui/llm/knowledge/browser/setup/... | input/ui/llm/capture/browser/lifecycle/actions/config |
| `main.rs` | large, handlers inline | thin, handlers in `actions/` |
| LLM module | 3-provider cascade | single OpenRouter client + cap table |
| UI state | scattered statics | centralized `UiState` |
| Conversation | none | stateless + ephemeral Forms |
| Hotkey handling | inline | dedicated state machine |
| Concurrency | parallel queries | single in-flight, new cancels previous |
| Image policy | configurable | locked: 1568px, 20 per request |
| Self-delete | none | new |
| Self-restart | none | new |
| `lib.rs` | no | yes (integration tests) |
| Release profile | default | aggressive size optimization |

The shape of the runtime (channels, two OS threads + tokio) is preserved. The structure of the code (layered modules, free-fn actions, centralized UI state) is rebuilt.

---

## Future Considerations (v2.1+)

- **Tracing-based structured logging** if debugging needs grow.
- **`chromiumoxide` migration** when their connect-to-existing-port pattern and image-bytes fetch are documented.
- **OpenRouter `/models` startup probe** as a fallback if static capability table grows stale faster than release cadence.
- **Plugin trait for additional automations** (Canvas, Moodle, …) if demand justifies it.
- **Optional streaming overlay** if user feedback requests it.

Out of scope for v2.0.
