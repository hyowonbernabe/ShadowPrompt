<p align="center">
  <img src="shadow_prompt_v2/assets/logo_512.png" width="140" alt="ShadowPrompt">
</p>

<h1 align="center">ShadowPrompt</h1>

<p align="center">
  <em>Lightweight, hotkey-driven AI assistant for Windows. Single executable. No taskbar entry. Built for exam workflows.</em>
</p>

<p align="center">
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-blue.svg"></a>
  <a href="https://github.com/hyowonbernabe/ShadowPrompt/releases/latest"><img alt="Release" src="https://img.shields.io/github/v/release/hyowonbernabe/ShadowPrompt?include_prereleases&color=success"></a>
  <a href="https://github.com/hyowonbernabe/ShadowPrompt/actions/workflows/check_v2.yml"><img alt="CI" src="https://github.com/hyowonbernabe/ShadowPrompt/actions/workflows/check_v2.yml/badge.svg?branch=v2"></a>
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%2010%20%2F%2011-0078D6?logo=windows&logoColor=white">
  <img alt="Rust" src="https://img.shields.io/badge/built%20with-Rust-orange?logo=rust&logoColor=white">
</p>

---

## Install in one line

Open **PowerShell** (regular user, no admin needed) and run:

```powershell
irm https://raw.githubusercontent.com/hyowonbernabe/ShadowPrompt/main/shadow_prompt_v2/install.ps1 | iex
```

This will:

1. Download the latest release zip
2. Extract to `%LOCALAPPDATA%\ShadowPrompt`
3. Bundle a 7-day OpenRouter trial key so you can run immediately
4. Add the install directory to your user PATH

Then launch from any terminal:

```powershell
shadowprompt
```

A tiny green pixel appears at the top-right corner of your screen. ShadowPrompt is now listening for hotkeys.

> **Already had a previous trial?** Replace the bundled key with your own from <https://openrouter.ai/keys> by editing `%LOCALAPPDATA%\ShadowPrompt\config\config.toml`.

---

## What it does

| Hotkey | What it does |
|---|---|
| `Ctrl+Shift+V` | Answer the question on your clipboard. Result lands back on the clipboard + on-screen overlay. |
| `Ctrl+Shift+Alt+V` | Same as above, but with live web search enabled. |
| `Ctrl+Shift+Space` | Draw a rectangle on screen with two clicks. Sent to a vision model. |
| `Ctrl+Shift+Alt+Space` | Same as above, with live web search. |
| `Ctrl+Shift+9` | **Google Forms auto-paginate.** Fills every unanswered question across all pages. Never clicks Submit. |
| `Ctrl+Shift+7` | Google Forms single page. Fills only the current page. |
| `Ctrl+Shift+I` | Open a debug Chrome window so the Forms hotkeys can attach. |
| `Ctrl+Shift+0` | Abort the current task. |
| `Ctrl+Shift+H` | Hide / show all UI. |
| `Ctrl+Shift+?` | Toggle on-screen cheat sheet of every hotkey. |
| `Ctrl+Shift+R` | Restart the daemon. |
| `Ctrl+Shift+Delete` | Insta-delete. Tap twice within 2 sec to wipe install dir + PATH entry. |
| `Ctrl+Shift+F12` | Panic. Wipe clipboard, exit immediately. |

All hotkeys are configurable in `config.toml`.

---

## Why it exists

Standard AI tools demand visible windows, browser tabs, and constant focus switching. ShadowPrompt was built for moments where that's not an option:

- **One process, no window** — runs invisibly. Only a single-pixel state indicator on screen.
- **Clipboard in, clipboard out** — copy a question, hit a hotkey, paste the answer.
- **Vision when text isn't enough** — pick a region of your screen, send to a multimodal model.
- **Forms automation** — fill an entire multi-page Google Form (radio, checkbox, dropdown, grid, text, date, time, scale, with image questions) while you watch. Never clicks Submit, never touches questions you've already answered.
- **Subject knowledge** — drop markdown notes into a folder, daemon caches them, every query gets that context for the cost of one cached lookup.
- **Panic button** — two-tap hotkey deletes the install dir, strips PATH, kills the process. No trace.

---

## Architecture at a glance

```
hotkey  ───►  input thread (rdev)
                    │
                    ▼
              tokio runtime
              ├─► clipboard / screen capture
              ├─► OpenRouter (Anthropic-pinned)
              │     └─► prompt cache (5min ephemeral)
              ├─► headless_chrome (Forms automation)
              └─► UI thread (Win32 layered windows)
                    └─► pixel indicator + overlay + help + form indicator
```

- **Language**: Rust 2021, single executable
- **HTTP**: reqwest + rustls (no native-tls)
- **LLM transport**: OpenRouter, pinned to Anthropic provider for Claude models
- **Default model**: `anthropic/claude-sonnet-4.6`
- **Caching**: Anthropic prompt caching, 5-minute ephemeral, refreshed on each hit
- **Browser automation**: headless_chrome attached to a user-launched Chrome on `:9222`
- **Stealth**: `windows_subsystem = "windows"` (no console), layered + topmost windows, chroma-keyed transparent overlay

Full details: [`shadow_prompt_v2/docs/architecture.md`](shadow_prompt_v2/docs/architecture.md).

---

## Subject knowledge (the killer feature for exams)

ShadowPrompt loads markdown files at startup and injects them as a cached system prompt block on every query. The first call pays a small write fee; every subsequent call within 5 minutes pays one-tenth of the input cost. Two reviewers are bundled out of the box.

Drop your own notes into:

```
%LOCALAPPDATA%\ShadowPrompt\knowledge\<subject>\*.md
```

Then enable in `config.toml`:

```toml
[knowledge]
enabled = true
active_subjects = ["methods_of_research", "structure_of_programming_language"]
cache_ttl = "5m"
max_chars = 800000
```

Restart the daemon. Every query now gets your full subject reference for ~$0.03 instead of ~$0.30.

See [`shadow_prompt_v2/docs/plans/2026-05-17-subject-context-injection.md`](shadow_prompt_v2/docs/plans/2026-05-17-subject-context-injection.md) for the full design.

---

## Configuration

Config lives at `%LOCALAPPDATA%\ShadowPrompt\config\config.toml`. Generated automatically on first run.

```toml
[openrouter]
api_key = "sk-or-v1-..."
model_id = "anthropic/claude-sonnet-4.6"

[hotkeys]
clipboard_query        = "ctrl+shift+v"
clipboard_query_search = "ctrl+shift+alt+v"
ocr_query              = "ctrl+shift+space"
ocr_query_search       = "ctrl+shift+alt+space"
forms_auto             = "ctrl+shift+9"
forms_single           = "ctrl+shift+7"
abort                  = "ctrl+shift+0"
launch_debugger        = "ctrl+shift+i"
hide_toggle            = "ctrl+shift+h"
help_toggle            = "ctrl+shift+slash"
restart_daemon         = "ctrl+shift+r"
insta_delete           = "ctrl+shift+delete"
panic_kill             = "ctrl+shift+f12"

[visuals]
indicator_corner = "top_right"
indicator_size   = 4
indicator_offset = [0, 0]
overlay_corner   = "top_left"
overlay_offset   = [0, 0]
overlay_font_size = 11

[knowledge]
enabled = true
active_subjects = ["methods_of_research", "structure_of_programming_language"]
cache_ttl = "5m"
max_chars = 800000
```

Full schema in [`shadow_prompt_v2/config/config.example.toml`](shadow_prompt_v2/config/config.example.toml).

---

## Verify everything is working

The daemon ships with a self-check probe:

```powershell
shadowprompt --probe
```

Runs five round-trips against the configured model and prints pass/fail for: text answering, vision, knowledge recency, multi-step reasoning, and whether web search is enabled.

For Google Forms specifically, run the 15-scenario verification plan documented in [`shadow_prompt_v2/docs/forms-testing.md`](shadow_prompt_v2/docs/forms-testing.md).

---

## Build from source

Requires Rust 2021 (stable) and Windows 10/11.

```powershell
git clone https://github.com/hyowonbernabe/ShadowPrompt.git
cd ShadowPrompt\shadow_prompt_v2
cargo build --release
```

Binary at `target\release\shadowprompt.exe`. ~5.5 MB stripped.

For development builds with verbose logging:

```powershell
cargo build --features debug
.\target\debug\shadowprompt.exe --debug
```

---

## CLI flags

| Flag | What it does |
|---|---|
| `--debug` | Run with stdout logging + console window |
| `--init` | Write a fresh `config.toml` from the embedded template, then exit |
| `--probe` | Run the capability self-check, then exit |
| `--uninstall` | Spawn detached cleanup script, remove install dir, exit |
| `--config <path>` | Override the config file path |
| `--version` | Print version |

---

## Uninstall

```powershell
irm https://raw.githubusercontent.com/hyowonbernabe/ShadowPrompt/main/shadow_prompt_v2/uninstall.ps1 | iex
```

Or from inside the daemon: tap `Ctrl+Shift+Delete` twice within 2 seconds.

Both methods:
- Remove `%LOCALAPPDATA%\ShadowPrompt`
- Remove the install dir from your user PATH
- Wipe the temp Chrome debug profile

---

## Project layout

```
ShadowPrompt/
├── README.md                          ← you are here
├── LICENSE                            ← Apache 2.0
├── shadow_prompt_v2/                  ← active codebase
│   ├── Cargo.toml
│   ├── src/
│   ├── config/config.example.toml
│   ├── knowledge/                     ← bundled reviewer markdown
│   ├── install.ps1
│   ├── uninstall.ps1
│   └── docs/                          ← architecture, agents, roadmap, testing
└── shadow_prompt/                     ← legacy v1 (archived, do not modify)
```

---

## Documentation

- [Roadmap](shadow_prompt_v2/docs/roadmap.md) — feature-by-feature build plan, M0–M8
- [Architecture](shadow_prompt_v2/docs/architecture.md) — runtime topology, module boundaries, concurrency
- [Agents](shadow_prompt_v2/docs/agents.md) — LLM transport, system prompts, conversation model
- [Features](shadow_prompt_v2/docs/features.md) — full feature spec
- [Forms testing plan](shadow_prompt_v2/docs/forms-testing.md) — 15 verification scenarios
- [Context injection design](shadow_prompt_v2/docs/plans/2026-05-17-subject-context-injection.md) — knowledge caching plan

---

## Known limitations

- Windows only. Linux/macOS are out of scope; the Win32 layered-window pixel indicator is core to the discretion story.
- Forms automation needs a Chrome window launched via `Ctrl+Shift+I`. Cannot attach to your everyday Chrome session.
- Custom JS date pickers in Google Forms are not yet supported (standard `<input type="date">` works).
- File-upload questions in Google Forms are skipped.
- The daemon never clicks Submit. You must click manually on the final Forms page.
- Anti-cheat / DRM-protected screens may capture as black via GDI.

---

## License

[Apache 2.0](LICENSE). Use at your own risk; this is research / personal-productivity tooling, not a sanctioned tool for any institution's evaluation.
