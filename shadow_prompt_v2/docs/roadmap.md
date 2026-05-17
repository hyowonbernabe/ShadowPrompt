# ShadowPrompt v2 — Roadmap

Feature-by-feature build plan. One milestone at a time. Each milestone is independently shippable to the `v2` branch and verifiable end-to-end before the next begins.

Order is chosen so each milestone unlocks the next: foundation → input → output → intelligence → capture → automation → polish → release.

---

## M0 — Foundation (config, logging, lifecycle, CLI)

Goal: daemon starts, reads config, logs, accepts CLI flags, exits cleanly. No hotkeys yet.

- `src/config/schema.rs` — finalize `Config` struct (already stubbed)
- `src/config/loader.rs` — resolve exe-relative `config/config.toml`, create from template on first run
- `src/config/paths.rs` — `get_exe_dir()`, data/log dirs
- `src/logger.rs` — `simplelog` → `data/logs/shadowprompt.log`, rotates on size
- `src/lifecycle/startup.rs` — single-instance lock (named mutex), panic hook → log
- `src/lifecycle/shutdown.rs` — graceful drop of tokio runtime, UI thread join
- `src/cli.rs` — clap: `--version`, `--config <path>`, `--debug`, `--uninstall`
- `src/main.rs` — wire CLI → config load → logger init → tokio runtime → idle event loop

Exit criteria: `shadowprompt.exe --debug` runs, logs startup, sits idle, Ctrl+C exits clean.

---

## M1 — Input layer (hotkeys + double-tap state machine)

Goal: every hotkey from the spec fires a typed event into the main loop. No actions yet — just log "would do X".

- `src/input/manager.rs` — `rdev::listen` thread, emits `InputEvent` over mpsc
- `src/input/parser.rs` — parse hotkey strings from config (`"Ctrl+Shift+V"`)
- `src/input/state_machine.rs` — finalize double-tap arm window (already stubbed)
- `src/input/events.rs` — `InputEvent` enum: `Query`, `OcrRegion`, `OcrFullscreen`, `FormsRun`, `InstaDelete`, `Hide`, `Cancel`
- Wire into `main.rs` event loop with `tokio::select!`

Exit criteria: every configured hotkey logs its intent. Double-tap insta-delete fires only on 2nd press within window.

---

## M2 — UI layer (indicator + text overlay)

Goal: visual feedback works. Daemon shows state via colored pixel + can display text on screen.

- `src/ui/manager.rs` — Win32 message loop thread, `UICommand` mpsc consumer
- `src/ui/indicator.rs` — `ShadowPromptIndicator` window, color per state
- `src/ui/overlay.rs` — `ShadowPromptTextOverlay` window with `LWA_COLORKEY` transparency
- `src/ui/colors.rs` — state → color mapping from config
- `src/ui/commands.rs` — `UICommand` enum: `SetState`, `ShowText`, `Hide`, `Clear`
- Wire main loop to publish state transitions (Ready → Working → Ready/Error)

Exit criteria: indicator visible at corner, color changes on hotkey, text overlay renders an arbitrary string from a test command.

---

## M3 — Clipboard query (end-to-end LLM path)

Goal: `Ctrl+Shift+V` → reads clipboard → OpenRouter → writes answer to clipboard + overlay. The vertical slice that proves the whole stack.

- `src/clipboard.rs` — `arboard` wrapper, 3-retry read/write
- `src/llm/capabilities.rs` — finalize capability table (already stubbed)
- `src/llm/request.rs` — finalize request builder (already stubbed)
- `src/llm/client.rs` — `reqwest` POST to OpenRouter, retry with backoff
- `src/llm/response.rs` — parse OpenRouter response, extract content
- `src/llm/system_prompts/` — finalize embedded prompts (already stubbed)
- `src/actions/query.rs` — handler: read clipboard → call LLM → write clipboard → overlay
- `src/actions/cancel.rs` — single in-flight slot; new query aborts previous `JoinHandle`

Exit criteria: copy a question, hit hotkey, answer lands in clipboard + appears on screen. Pressing hotkey again mid-query cancels and restarts.

---

## M4 — OCR capture (region + fullscreen)

Goal: hotkey captures screen pixels, OCRs to text, runs through M3 query path.

- `src/capture/screen.rs` — GDI bitmap capture (full screen + rect)
- `src/capture/region_select.rs` — drag-select rect via `ShadowPromptDebug` window
- `src/capture/ocr.rs` — `Windows.Media.Ocr` WinRT call → text
- `src/capture/image.rs` — finalize resize/encode pipeline (already stubbed)
- `src/actions/ocr_query.rs` — region/fullscreen → OCR → LLM (text-only, no image attach)
- Vision-fallback path: if OCR confidence low, send image bytes to vision-capable model

Exit criteria: region-select hotkey crops, OCRs, answers. Fullscreen hotkey same. Low-OCR-quality image falls back to vision.

---

## M5 — Google Forms automation

Goal: hotkey opens active Forms tab, reads questions (incl. images), fills only unanswered, never submits.

- `src/browser/launch.rs` — connect to existing Chrome via CDP port 9222, or launch detached
- `src/browser/cookies.rs` — `rookie` cookie extraction → inject into headless tab
- `src/browser/forms/extractor.rs` — finalize `EXTRACTOR_JS` port from v1 (already stubbed)
- `src/browser/forms/answered.rs` — finalize unanswered filter (already stubbed)
- `src/browser/forms/images.rs` — fetch image bytes per question, run through `capture/image.rs`
- `src/browser/forms/llm_call.rs` — ephemeral conversation: system prompt + question batch + images
- `src/browser/forms/injector.rs` — finalize answer injection (already stubbed)
- `src/browser/forms/submit_guard.rs` — finalize next-vs-submit detection (already stubbed)
- `src/browser/forms/pagination.rs` — click "Next" up to N pages, stop at "Submit"
- `src/actions/forms_run.rs` — orchestrator

Exit criteria: real Google Form opens, daemon fills unanswered radio/checkbox/dropdown/text/date/grid, stops at Submit. Image questions handled. Already-answered untouched.

---

## M6 — Lifecycle hotkeys (insta-delete + restart + hide)

Goal: panic key works. Daemon can wipe itself or restart on demand.

- `src/lifecycle/self_delete.rs` — detached `cmd.exe` spawn pattern, removes EXE + config dir
- `src/lifecycle/path_cleanup.rs` — remove install dir from user PATH
- `src/lifecycle/chrome_kill.rs` — terminate Chrome instances launched by us
- `src/lifecycle/restart.rs` — re-exec self via detached spawn
- `src/actions/insta_delete.rs` — chrome kill → path cleanup → self-delete
- `src/actions/hide.rs` — toggle indicator visibility

Exit criteria: double-tap insta-delete leaves no traces (EXE gone, PATH clean, no Chrome). Restart hotkey relaunches daemon clean.

---

## M7 — Install pipeline

Goal: `irm <url> | iex` from a fresh Windows box drops a working binary in PATH with API key set.

- `install.ps1` — finalize: fetch latest release zip, extract to `%LOCALAPPDATA%\ShadowPrompt`, prompt for API key, write `config.toml`, add to PATH
- `uninstall.ps1` — mirror of install (also invoked by `--uninstall` flag)
- `.github/workflows/release.yml` — on tag push: `cargo build --release`, zip with `config.example.toml`, attach to GitHub Release
- Release artifact naming: `shadowprompt-v2.0.0-windows-x64.zip`

Exit criteria: clean Windows VM → run one-liner → working install → first hotkey query succeeds.

---

## M8 — Polish, hardening, release

Goal: ship 2.0.0 stable.

- CI: `.github/workflows/check.yml` — clippy + `cargo test` on `windows-latest`
- Integration tests: mock OpenRouter, snapshot request bodies per capability tier
- Manual QA pass: every hotkey, every config option, install + uninstall on fresh VM
- README at repo root — update for v2 install instructions
- `docs/troubleshooting.md` — common issues (Chrome not found, key invalid, OCR empty)
- Tag `v2.0.0`, cut release, merge `v2` → `main`

Exit criteria: green CI, clean Windows VM install works, release published, branch merged.

---

## Working rules

- One milestone at a time. Don't start M3 work while M2 is half-done.
- Each milestone ends with a passing `cargo check` + manual verification of exit criteria.
- Commit per logical sub-step. Push `v2` branch after each milestone.
- TODO comments in stubs reference `docs/architecture.md` sections — read those before implementing.
- If a milestone's scope grows, split it. Don't expand the current one.
