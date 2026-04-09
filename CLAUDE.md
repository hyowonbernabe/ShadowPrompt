# ShadowPrompt — CLAUDE.md

> Portable stealth AI assistant (Rust, Windows 10/11, USB-deployable). No taskbar, no window — hotkeys + clipboard only. v1.5.0

---

## Tech Stack

| Concern | Crate / Technology |
|---|---|
| Language | Rust 2021, single EXE |
| Async | Tokio (multi-thread, full) |
| GUI (setup only) | eframe 0.29 / egui 0.29 |
| Win32 / WinRT | `windows` 0.58 (GDI, OCR, keyboard, console) |
| Global hooks | `rdev` 0.5 |
| Clipboard | `arboard` 3.4 |
| OCR | `Windows.Media.Ocr` (native WinRT) |
| HTTP / LLM | `reqwest` 0.12 (async, JSON) |
| LLM providers | Groq → OpenRouter → Ollama (auto-cascade) |
| Embeddings / RAG | `fastembed` 4, BGE-Small-EN-v1.5 (ONNX local) |
| Browser automation | `headless_chrome` 1.0 + `rookie` 0.1 |
| Config format | TOML (`toml` 0.8) + `serde` |
| Serialization | `serde_json` |
| Logging | `simplelog` 0.12 + `log` 0.4 |
| Build | `embed-resource` 2 (icon embedding) |

---

## Repository Tree

```
ShadowPrompt/
├── CLAUDE.md                         ← this file
├── README.md                         ← user-facing docs, feature matrix
├── PROJECT_DOCUMENTATION.md          ← technical spec (v1.5.0)
├── LICENSE                           Apache 2.0
├── build_release.bat                 build + ZIP packaging script
│
├── .github/
│   └── workflows/check.yml           CI: clippy + cargo test (windows-latest)
│
└── shadow_prompt/                    ← Rust crate root
    ├── Cargo.toml                    package manifest (v1.5.0), all deps
    ├── Cargo.lock
    ├── build.rs                      embeds resources.rc (EXE icon)
    ├── resources.rc                  Windows resource file (icon)
    ├── Launcher.bat                  dev launcher (sets PROTOC, runs cargo run)
    │
    ├── .cargo/
    │   └── config.toml               sets PROTOC env var (bundled protoc)
    │
    ├── assets/
    │   ├── logo_512.ico              EXE icon
    │   └── logo_512.png              logo for egui setup wizard
    │
    ├── config/
    │   ├── config.example.toml       committed template (no secrets)
    │   └── system_prompt.txt         LLM system prompt (stealth, MCQ rules)
    │   # config.toml is gitignored (user-generated)
    │   # .setup_complete is gitignored (presence = setup done)
    │
    ├── data/
    │   └── rag_index/index.json      persisted RAG vector index (JSON)
    │   # data/models/ = fastembed download target (gitignored)
    │   # data/logs/   = runtime log output (gitignored)
    │
    ├── knowledge/
    │   ├── password.md               example knowledge doc (RAG source)
    │   └── sample.md                 example knowledge doc (RAG source)
    │
    ├── docs/
    │   ├── DEVELOPMENT.md            dev setup, Google OAuth guide
    │   └── plans/
    │       └── 2026-02-19-multi-format-question-support.md  historical plan
    │
    ├── tools/
    │   └── protoc/                   bundled protobuf compiler
    │
    ├── test_overlay.rs               standalone test binary (overlay window)
    ├── test_ws.rs                    standalone test binary (WebSocket)
    ├── test_tabs.rs                  standalone test binary (browser tabs)
    ├── test_hc.rs                    standalone test binary (headless Chrome)
    │
    └── src/
        ├── main.rs                   entry point; setup gate, tokio runtime, event loop
        ├── config.rs                 Config structs, TOML de/serialization, path resolution
        ├── input.rs                  InputManager: rdev global hook thread, InputEvent enum
        ├── ui.rs                     UIManager: Win32 GDI windows, UICommand enum
        ├── clipboard.rs              arboard wrapper (3-retry read/write)
        ├── ocr.rs                    Windows.Media.Ocr capture + base64 PNG encode
        ├── llm.rs                    LlmClient: Groq/OpenRouter/Ollama, retry, vision
        ├── capabilities.rs           ModelCapabilities: supports_search, supports_vision
        ├── utils.rs                  parse_keys(), parse_hex_color(), unit tests
        ├── logger.rs                 simplelog init → data/logs/error.log
        ├── setup.rs                  8-page egui Setup Wizard
        ├── hotkey_recorder.rs        egui widget: records hotkey combos
        ├── color_picker.rs           egui widget: color picker with presets
        ├── tos_text.rs               TOS text + version constant
        │
        ├── knowledge/
        │   ├── mod.rs                KnowledgeProvider: gather_context() (search + RAG)
        │   ├── rag.rs                RagSystem: FastEmbed ingest/query, cosine sim, 4 tests
        │   └── search.rs             web search: Serper.dev → DuckDuckGo fallback
        │
        └── browser/
            ├── mod.rs                execute_form_flow(), launch_incognito_debugger()
            ├── injector.rs           EXTRACTOR_JS (reads Google Form DOM), build_injector_call()
            └── cookies.rs            rookie cookie extraction + headless_chrome tab injection
```

---

## Architecture

### Two-phase startup

```
main()
  └─ config/.setup_complete exists?
       No  → SetupWizard::show()  (egui, writes config.toml + .setup_complete)
       Yes → run_app()            (stealth daemon, no visible window)
```

### Stealth daemon channels

```
InputManager (OS thread)          UIManager (OS thread)
  rdev::listen global hooks         Win32 message loop
       │ InputEvent (mpsc)               ▲ UICommand (mpsc)
       ▼                                 │
  main event loop  ──── tokio::spawn tasks ────►  LlmClient
  (tokio block_on)                                 KnowledgeProvider
                                                   ClipboardManager
                                                   OcrCapture
                                                   BrowserFlow
```

### Primary data flow (clipboard query)

1. User copies text → presses `Ctrl+Shift+V`
2. `InputManager` → `InputEvent::Model`
3. Main loop spawns tokio task:
   - `ClipboardManager::read()` → question text
   - `KnowledgeProvider::gather_context()` → web search + RAG results
   - Prepend as `"Context:\n...\nQuestion:\n..."`
   - `LlmClient::query()` → Groq / OpenRouter / Ollama
   - `ClipboardManager::write(answer)`
   - `UICommand::SetOverlayText(answer)` → text overlay on screen

---

## Key Entry Points

| Symbol | File:Line | Role |
|---|---|---|
| `fn main()` | `src/main.rs:32` | setup gate → daemon |
| `async fn run_app()` | `src/main.rs:77` | stealth runtime (event loop) |
| `SetupWizard::show()` | `src/setup.rs` | launches 8-page egui wizard |
| `InputManager::start()` | `src/input.rs:29` | spawns rdev global hook thread |
| `UIManager::start()` | `src/ui.rs:43` | spawns Win32 message loop thread |
| `LlmClient::query()` | `src/llm.rs:11` | dispatches to provider with retry |
| `KnowledgeProvider::gather_context()` | `src/knowledge/mod.rs:38` | search + RAG orchestration |
| `execute_form_flow()` | `src/browser/mod.rs:49` | Google Forms automation entry |

---

## Configuration System

- **Path resolution** (`config.rs::get_config_path`): checks exe-relative `config/config.toml` first, then CWD — this is the USB portability mechanism.
- **`get_exe_dir()`**: used everywhere; all data dirs are exe-relative (never hardcoded absolute).
- **Setup marker**: `config/.setup_complete` — file existence check only.
- **Config sections**: `[general]`, `[visuals]`, `[models]` (with `[models.groq]`, `[models.openrouter]`, `[models.ollama]`), `[search]`, `[rag]`, `[safety]`, `[http]`
- `config/config.example.toml` = committed template (no keys). `config/config.toml` = gitignored.

---

## Visual System (Win32, not egui)

Three layered windows created by `ui.rs`, all topmost, no taskbar entry:

| Window class | Purpose |
|---|---|
| `ShadowPromptIndicator` | Tiny colored pixel at screen corner; color = state |
| `ShadowPromptDebug` | Semi-transparent rect shown during OCR region selection |
| `ShadowPromptTextOverlay` | Transparent-background text; uses `LWA_COLORKEY` |

Indicator colors: green=ready, red=processing, cyan/magenta/yellow/black=MCQ A/B/C/D.

Static mutable globals in `ui.rs` (`CURRENT_COLOR`, `OVERLAY_TEXT`, `IS_HIDDEN`, etc.) are safe because only the UI thread reads/writes them in the message loop.

---

## RAG System

File: `src/knowledge/rag.rs`

- Model: BGE-Small-EN-v1.5 via `fastembed`, downloaded to `data/models/` on first run
- Storage: `data/rag_index/index.json` (all docs + embeddings in one JSON file)
- Indexing: incremental by file mtime; only re-embeds changed files
- Search: in-memory cosine similarity, filtered by `min_score`, capped at `max_results`
- Sources: `knowledge/*.md`, `knowledge/*.txt` (and subdirectories)
- Has 4 unit tests: cache logic, incremental re-index, operational flag, corrupt index recovery

---

## Browser Automation

File: `src/browser/mod.rs`

Two modes:
- **`launch_incognito_debugger()`**: spawns Chrome/Edge with `--remote-debugging-port=9222` as detached process
- **`execute_form_flow()`**: connects to debug session on port 9222 (preferred) or launches headless Chrome with cookies from `rookie`; injects `EXTRACTOR_JS` to read Google Form DOM as JSON; sends to LLM; injects answer actions back; supports auto-pagination (≤10 pages) or single-page mode

Cookie strategy: `rookie` reads Google cookies from host Chrome/Edge/Firefox/Brave profiles and injects them into the headless tab.

---

## Build & Run

```bash
# Dev run (from shadow_prompt/)
Launcher.bat                       # sets PROTOC, runs cargo run

# Release build + ZIP
build_release.bat                  # outputs to release/

# Feature flags
cargo build --features debug       # enables console window (AllocConsole)
cargo build                        # release: no console (windows_subsystem = "windows")

# CI
.github/workflows/check.yml        # clippy + cargo test on windows-latest
```

---

## Patterns & Conventions

| Pattern | Where | Detail |
|---|---|---|
| Exe-relative paths | everywhere | `get_exe_dir().join(...)` — never hardcoded absolute paths |
| Retry with backoff | `llm.rs` | 3 attempts, 1s base; Groq → OpenRouter → Ollama cascade in "auto" mode |
| Channel-per-concern | `main.rs` | `InputEvent` (input→main) and `UICommand` (main→UI) are separate mpsc channels |
| Spawn-per-event | `main.rs` | each hotkey fires a fresh `tokio::spawn`; only browser task handle stored for abort |
| `unsafe` confined | `ui.rs`, `ocr.rs` | Win32 API calls and raw GDI bitmap ops only |
| No console in release | `main.rs:1` | `#![cfg_attr(not(feature="debug"), windows_subsystem="windows")]` |

---

## Known Gaps / In-Progress

- **MCQ pixel detection**: Color constants and config fields exist in `ui.rs` and `config.rs`, but the `InputEvent::Model` handler in `main.rs` does **not** yet perform MCQ response parsing or set secondary indicator colors — this was planned (see `docs/plans/2026-02-19-multi-format-question-support.md`).
- **`text-splitter`**: Imported in `rag.rs` but chunking is not yet applied — documents are stored as whole units.
- **`oauth2` + `open` + `url`**: Present in `Cargo.toml` and referenced in `DEVELOPMENT.md` for Google OAuth, but the OAuth flow is not wired into the current runtime.
- **Developer scratch files**: `shadow_prompt/compile_errors2.txt` and `test_err.txt` are tracked in git accidentally.
