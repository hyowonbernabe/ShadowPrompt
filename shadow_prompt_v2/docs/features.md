# ShadowPrompt v2 — Features

> Living document. Updated as decisions are made.
> Last updated: 2026-05-17

---

## Mission

A lighter, simpler, single-executable revamp of ShadowPrompt v1. Stealth AI assistant for Windows, deployable to any computer with a single terminal command. No bloat, no half-built features, no third-party fallbacks. One provider, swappable model, one purpose: fast, private, exam-grade assistance.

---

## Core Principles

1. **Lightweight first.** Every dependency must justify its size cost. If a feature pulls in 50MB of ONNX runtime for marginal benefit, it does not ship.
2. **Single executable.** One `shadowprompt.exe`. No installer wizard, no companion binaries, no runtime dependencies beyond what Windows ships with.
3. **Windows-only.** No cross-platform abstraction overhead. Native Win32 + WinRT where it makes sense.
4. **One-command install.** Install on any new computer from terminal in under one minute. No GUI, no manual file copying.
5. **GitHub-only hosting.** No custom domain, no backend services, no database. Repository releases serve binaries; raw URLs or GitHub Pages serve the install script.
6. **No backend, no accounts.** Users own their API key end-to-end. Nothing leaves their machine except calls to OpenRouter.
7. **Stealth by default.** No taskbar entry, no window, no console in release builds. Visual feedback exists but is unobtrusive enough to be invisible to anyone who does not know where to look.
8. **Stateless where possible.** Every query is independent. State only exists where it provably improves results (Forms ephemeral context).
9. **Provider transport, model agnostic.** OpenRouter is the only transport. The actual model is configurable — Claude family is the default and primary target, but the system must work with any OpenRouter-hosted multimodal model (Gemini, GPT-4 vision, etc.).
10. **Start from scratch.** v1 codebase is reference material only — v2 begins fresh to avoid inheriting architectural debt.

---

## Surviving Features from v1

These features proved their value and carry over to v2, often simplified.

### Hotkey daemon

Global keyboard hook running in a background OS thread. Listens for configured key combinations and dispatches commands to the main event loop. Survives in essentially the same form as v1.

### Clipboard query

User copies a question to the clipboard, presses the hotkey, the daemon reads the clipboard, sends the contents to the LLM as a single stateless request, and writes the response back to the clipboard. Answer also appears briefly on the text overlay.

### OCR query

User presses the OCR hotkey, selects a screen region, the daemon captures the region as an image, sends it to a multimodal model on OpenRouter as a single stateless request, and returns the answer the same way clipboard query does. Vision is the primary path; raw OCR text extraction may serve as a fallback or auxiliary signal.

### Google Forms automation

The marquee feature. User presses a hotkey, the daemon connects to a running Chrome instance via the DevTools Protocol (or launches one), extracts the current form's DOM as structured JSON, sends questions and any embedded images to the LLM, and injects answers back into the form. Supports both auto-pagination (advances through every page automatically) and single-page mode.

Critical Forms behaviors:

- **Image-bearing questions** are sent to the multimodal model alongside the question text. Many quizzes embed diagrams or photos as primary context — this is mandatory.
- **Skip already-answered questions.** Before injecting answers on a page, the daemon detects which questions the user has already filled in manually and excludes them from the model's task. The model is told only about the unanswered questions and never overwrites the user's manual answers. This protects the user's intentional choices.
- **Never submit.** The daemon will never press the "Submit" button. It advances through pages via the "Next" button (auto-paginate mode) and stops at the final page. The user is always the one who submits. The submit button is treated as untouchable.

### Visual feedback

A tiny colored pixel sits in a configurable screen corner. Color indicates state: ready, processing, error, answer-delivered. A second pixel handles Forms-specific status (running, succeeded, failed, aborted). A text overlay displays the model's response in transparent-background text. All UI windows are Win32, marked as tool windows so they do not appear in the taskbar or Alt+Tab list. Visibility is toggleable with a single hotkey.

### Panic key

Single hotkey kills the process and wipes the clipboard immediately. No prompts, no save state.

### Hide / show UI

Toggle all visible windows (indicator, overlay, form indicator) on or off.

### Single executable, USB-portable layout

Config and any per-user data live next to the executable (or in a known per-user location), keeping the binary portable.

---

## New Features for v2

### Stateless query model

Clipboard queries and OCR queries are completely stateless. Every request is independent — system prompt plus this question, nothing else. There is no persistent conversation history across the daemon's lifetime.

Rationale: per-question cost drops roughly 5x compared to a growing-history approach. Latency drops because payloads stay small. Code stays simple — no token counters, no auto-wipe logic, no edge cases. The 5% of questions that genuinely need cross-question context can be handled by the user copying more text into their selection.

### Ephemeral Forms run context

A single Google Forms run (start of flow → final page reached or user aborts) carries a short-lived conversation. The model sees its own earlier-page answers when generating answers for later pages, ensuring consistency across multi-page forms.

This context exists only for the duration of one form execution. It is built at flow start and discarded the moment the flow ends (success, failure, abort, or page cap reached). It is never shared with clipboard or OCR queries.

### Adaptive thinking (model-side reasoning)

When the configured model supports extended thinking (Claude 4.x family, Gemini 2.5/3 Pro thinking, GPT-5 reasoning, etc.), the daemon enables it on every request. The model itself decides how much to think based on the difficulty of the question — modern reasoning models self-modulate, thinking briefly on trivial questions and longer on hard ones.

The daemon does not classify question difficulty itself. It simply requests reasoning with a high upper bound and trusts the model's internal adaptivity. No configuration knob is exposed; the behavior is fully dynamic and provider-driven.

For models that do not support extended thinking, the request proceeds normally without it.

### Insta-delete hotkey

A nuclear hotkey that wipes every trace of ShadowPrompt from the machine.

Safety: requires a **double-tap within 2 seconds**. A single press shows a brief warning indicator (e.g., the indicator pixel flashes red) and arms the second press. If the second press does not arrive within 2 seconds, the arm is canceled. This prevents accidental nukes from a stray key combination.

When fired (second press confirmed), the daemon:

1. Wipes the clipboard.
2. Aborts any in-flight Forms or query task.
3. Kills any Chrome / Edge debug processes the daemon spawned (those launched with `--remote-debugging-port=9222` and the temporary profile path).
4. Spawns a detached helper (`cmd.exe`) that:
   - Waits ~1 second for the daemon to exit.
   - Removes the install path from the user's `PATH` environment variable.
   - Recursively deletes the install folder, including the executable.
   - Deletes itself.
5. Exits the daemon process.

After execution, no trace remains on disk and no Chrome debug processes leak.

### Restart hotkey

A hotkey that fully restarts the daemon process — same self-spawning technique as insta-delete, but the spawned helper relaunches the executable instead of deleting it. Use cases: recover from a crash without going to File Explorer, force a config reload, clear any wedged state. Full process restart is preferred over in-process config reload because it is simpler and safer.

### Answer-mode tone (model behavior)

The model treats every incoming message as an exam or quiz question, not a chat turn. Responses contain only the answer — no greetings, no acknowledgements, no meta-commentary. Formatting (MCQ letter, true/false, short answer, paragraph) is decided dynamically by the model based on the question, guided by the system prompt. See `agents.md` for the full system prompt rules.

### Model agnosticism

ShadowPrompt v2 talks to OpenRouter, which itself fronts dozens of model families. The daemon must work with any multimodal model OpenRouter hosts. Claude (Sonnet/Opus) is the default and the one we tune for, but switching to Gemini 2.5 Pro, GPT-4o, or any other multimodal model is a single config change.

Hard requirement: the chosen model must support image inputs natively. Text-only models are not supported because OCR and Forms image questions are core features.

Provider-specific features (extended thinking, prompt caching, etc.) are enabled when available and silently skipped when not. The daemon never assumes a capability is present.

### One-command install with key prompt

The install flow is a single PowerShell one-liner that pulls the latest release from GitHub, drops the executable into `%LOCALAPPDATA%\ShadowPrompt`, adds the install path to the user's `PATH`, and prompts for an OpenRouter API key. The user pastes their key and presses Enter, or presses Enter immediately to skip. Skipping leaves the key field empty in the config file, which the user can edit manually later.

Install URL is hosted on GitHub — either via raw repository URL or via GitHub Pages for a slightly cleaner path. No custom domain, no third-party hosting.

---

## Dropped Features (with reasoning)

| Feature | Reason for removal |
|---|---|
| RAG / local knowledge base | FastEmbed + BGE model = ~100MB+ on disk and significant cold-start cost. Conflicts with "lightweight" principle. |
| LLM provider cascade (Groq + OpenRouter + Ollama) | Multiple providers multiply config complexity, retry logic, and failure surface. OpenRouter alone reaches every major model family. |
| Web search (Serper / DuckDuckGo) | Third-party search introduces latency, key management, scraping fragility, and rate limits. Models with native web search (Claude with web tools, Gemini with grounding) handle this internally with better quality. |
| Model cycling hotkeys | Picking one strong model consistently produces better results than swapping among ten mediocre ones. Removes three hotkeys and a chunk of config. |
| Setup wizard (egui / eframe) | A multi-page GUI wizard contradicts the one-command install model. eframe and egui together add roughly 5–10MB to the binary for a single-use flow that runs once per machine. |
| MCQ / true-false parsing code | The model handles formatting dynamically based on the system prompt. No parser, no color-coding state machine. |
| Encrypted API key storage (DPAPI) | Marginal security benefit for a single-user local config. Adds debugging surface. Users can secure their machine through other means. |
| Persistent global conversation | Per-question cost grows linearly with conversation length, and the cache-invalidation problem makes context management complex. Stateless wins on cost, latency, and simplicity. |
| Manual context wipe hotkey | No state to wipe in the new stateless model. |
| Token cap and auto-wipe logic | Not needed when state is stateless or short-lived. |
| Approaching-limit warnings / overlays | Not needed. |
| Streaming overlay (character-by-character output) | Deferred. Marginal UX win for added complexity. Final-answer-only is sufficient for exam use. |
| GitHub Copilot provider stub | Never finished, no current plan to ship. |
| OAuth flow (Google account linking) | Not needed for the BYO-key model. |
| Telemetry / crash reporting / auto-update | Out of scope for v2.0. |
| Prompt profiles, CLI ask subcommand, Canvas / Moodle support | Deferred. Possibly v2.1 if demand justifies. |

---

## Hotkeys (Preliminary)

Final assignments to be locked in the hotkey-finalization discussion. Listed here in feature order.

All hotkeys are reconfigurable via `config.toml`. Defaults:

| Action | Default |
|---|---|
| Clipboard query | `Ctrl+Shift+V` |
| OCR query | `Ctrl+Shift+Space` |
| Google Forms — auto-paginate | `Ctrl+Shift+9` |
| Google Forms — single page | `Ctrl+Shift+7` |
| Abort active Forms task | `Ctrl+Shift+0` |
| Launch incognito Chrome debugger | `Ctrl+Shift+I` |
| Hide / show UI | `Ctrl+Shift+H` |
| Restart daemon | `Ctrl+Shift+R` |
| Insta-delete (double-tap within 2s) | `Ctrl+Shift+Del` |
| Panic kill | `Ctrl+Shift+F12` |

---

## Install and Bootstrap Flow

```powershell
# Fresh computer, one line:
irm https://raw.githubusercontent.com/<owner>/ShadowPrompt/main/install.ps1 | iex
```

The script:
1. Downloads the latest `shadowprompt.exe` from GitHub Releases.
2. Creates `%LOCALAPPDATA%\ShadowPrompt\` and places the executable inside.
3. Adds the install path to the user's `PATH` (user scope, no admin required).
4. Writes a default `config.toml` next to the executable.
5. Prompts: `Paste your OpenRouter API key (or press Enter to skip):`
6. Writes the key into `config.toml` (or leaves it blank if skipped).
7. Prints next-step instructions: `Run: shadowprompt`.

If the user skipped the key, running `shadowprompt` for the first time exits with a clear message pointing to the config file path.

---

## Tech Stack

| Concern | Choice |
|---|---|
| Language | Rust 2021 edition |
| Async runtime | Tokio (multi-thread) |
| LLM transport | OpenRouter (single provider) |
| Default model family | Claude (Anthropic) via OpenRouter |
| Supported model families | Any multimodal OpenRouter model (Claude, Gemini, GPT-4o, etc.) |
| Global hotkeys | `rdev` |
| Clipboard | `arboard` |
| OCR / screen capture | `windows` crate (WinRT `Windows.Media.Ocr` + GDI) |
| HTTP / SSE | `reqwest` (async, JSON, stream support) |
| Browser control | `headless_chrome` |
| Cookie extraction | `rookie` |
| Config | `toml` + `serde` |
| Logging | `simplelog` + `log` |
| CLI argument parsing | `clap` (lightweight feature set) |
| Build resources | `embed-resource` (icon embedding) |

Removed compared to v1: `fastembed`, `text-splitter`, `eframe`, `egui`, `oauth2`, `open`, `url`, and the bundled `protoc` toolchain.

---

## Forms Image Handling Policy

Locked, model-agnostic to support Claude, Gemini, and GPT-4o equally.

- **Resize before send:** every image is resized in-process so its longest edge is at most **1568 px**, preserving aspect ratio. This is Claude's recommended size for fastest time-to-first-token and is well within Gemini's and GPT-4o's optimal ranges.
- **Format:** PNG.
- **Per-request image cap:** **20 images** per request. Above this, Claude forces a 2000×2000 downscale; we stay under the threshold instead.
- **Total request size cap:** target **under 18 MB** (Gemini's 20 MB limit is the smallest among supported providers; 2 MB margin for the rest of the payload).
- **Pages with more than 20 images:** split into multiple requests within the same ephemeral Forms run, preserving conversation order. Rare in practice.
- **Image transport:** all images of a page sent in parallel inside a single user message's content array.

---

## Open Items (Pending Decision)

All v2.0 design decisions are now locked. Future iterations (v2.1+) may revisit:

1. Optional streaming overlay if user demand justifies the complexity.
2. Adaptive image resolution per provider (e.g., higher resolution on Gemini 3 via `media_resolution`).
3. Tool use / function calling for richer Forms automation.
