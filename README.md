<p align="center">
  <img src="shadow_prompt_v3/assets/logo_512.png" width="120" alt="ShadowPrompt">
</p>

<h1 align="center">ShadowPrompt</h1>

<p align="center">
  <em>Hotkey-driven AI answers for Windows. One executable, no window, no taskbar entry.</em>
</p>

<p align="center">
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-blue.svg"></a>
  <a href="https://github.com/hyowonbernabe/ShadowPrompt/releases/latest"><img alt="Release" src="https://img.shields.io/github/v/release/hyowonbernabe/ShadowPrompt?color=success"></a>
  <a href="https://github.com/hyowonbernabe/ShadowPrompt/actions/workflows/check_v3.yml"><img alt="CI" src="https://github.com/hyowonbernabe/ShadowPrompt/actions/workflows/check_v3.yml/badge.svg"></a>
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%2010%20%2F%2011-0078D6?logo=windows&logoColor=white">
</p>

---

## Install

Paste one line. It downloads the latest release, installs it and starts it. No admin needed.

**PowerShell**

```powershell
irm https://raw.githubusercontent.com/hyowonbernabe/ShadowPrompt/main/shadow_prompt_v3/install.ps1 | iex
```

**Command Prompt**

```cmd
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/hyowonbernabe/ShadowPrompt/main/shadow_prompt_v3/install.ps1 | iex"
```

Installs to `%LOCALAPPDATA%\AppSupport` with the study docs, adds it to your PATH and starts it. Run `shadowprompt` to start it again later. Running the line again updates to the newest release and keeps your `config.toml`.

**Portable:** download the zip from the [latest release](https://github.com/hyowonbernabe/ShadowPrompt/releases/latest), extract it anywhere (a USB stick works), run `shadowprompt.exe`. Everything stays in that folder.

## First launch

No API key ships with the app. Every time you start it, a small dialog asks how to start:

- **Log in with a code**: loads the OpenRouter key saved under your code.
- **Sign up**: pick a code and save your OpenRouter key under it.
- **Continue**: use the login or key you already have (works offline).

The login is saved in `config\config.toml`. Restarts the app does itself (restart hotkey, crash recovery) skip the dialog. Get a key at <https://openrouter.ai/keys>.

## Hotkeys

| Hotkey | Action |
|---|---|
| `Ctrl+Shift+Alt+V` | Answer the question on the clipboard |
| `Ctrl+Shift+Alt+S` | Answer a screen region (click two corners) |
| `Ctrl+Shift+Alt+W` | Answer everything on all monitors and put a dot on each answer |
| `Ctrl+Shift+Alt+D` | Clear the dots |
| `Ctrl+Shift+Alt+A` | Save a screen region as context (no answer) |
| `Ctrl+Shift+Alt+N` | New conversation (forget context) |
| `Ctrl+Shift+Alt+G` | Google Forms: fill this page |
| `Ctrl+Shift+Alt+F` | Google Forms: fill every page |
| `Ctrl+Shift+Alt+B` | Open the Chrome window the Forms hotkeys attach to |
| `Ctrl+Shift+Alt+M` | Switch between Claude and Gemini |
| `Ctrl+Shift+Alt+T` | Show which model is answering |
| `Ctrl+Shift+Alt+K` | Account: change key or code, log out, delete |
| `Ctrl+Shift+Alt+C` | Abort |
| `Ctrl+Shift+Alt+H` | Hide / show everything |
| `Ctrl+Shift+Alt+/` | Hotkey cheat sheet |
| `Ctrl+Shift+Alt+R` | Restart |
| `Ctrl+Shift+Alt+X` | Self-delete (press twice) |
| `Ctrl+Alt+End` | Panic: wipe the clipboard and exit |

**Backup keys**, for when Ctrl/Shift/Alt are blocked: hold **Caps Lock**, tap **Tab**, press the hotkey's letter. Caps Lock state is left as it was.

All hotkeys can be changed in `config.toml`.

## How it works

- **Conversation memory.** Every question, screenshot and form page is sent with the conversation so far, so context from earlier in a test carries over. Save a reading passage or the top of a long question with `A` before asking. Up to 20 screenshots are kept.
- **Screen answer.** `W` reads every monitor, answers, and marks where to click or type. Dots follow the page as you scroll, for 10 minutes.
- **Study docs.** Markdown in the `knowledge\` folder, plus the shared online docs, searchable by the model. A `CLAUDE.md` in that folder is always read. The release ships with the four midterm reviewers.
- **Models.** Claude Sonnet 5.5 first, Gemini 3.8 Flash as backup, then free models if both are down. Web search when needed.
- **Hidden.** No taskbar entry; the answer text, dots and dialogs don't show up in screenshots or screen sharing.
- **Bad internet.** Small screenshots, automatic retries, and frozen connections are dropped and retried after 25 s instead of hanging.
- **Never submits.** Forms are filled, never submitted, and answers you already gave are left alone.

## Command line

| Flag | Action |
|---|---|
| `--debug` | Write a log to `data\logs\` |
| `--init` | Write a fresh `config.toml`, then exit |
| `--probe` | Check the model chain, then exit |
| `--uninstall` | Remove the install, then exit |

## Uninstall

`Ctrl+Shift+Alt+X` twice, or:

```powershell
irm https://raw.githubusercontent.com/hyowonbernabe/ShadowPrompt/main/shadow_prompt_v3/uninstall.ps1 | iex
```

## Build from source

Rust stable, Windows 10/11:

```powershell
git clone https://github.com/hyowonbernabe/ShadowPrompt.git
cd ShadowPrompt\shadow_prompt_v3
cargo build --release
```

`--features admin` builds the elevated variant.

## Limitations

- Windows only.
- Forms need the Chrome window from `Ctrl+Shift+Alt+B`; it can't attach to your everyday Chrome.
- Hotkeys don't reach windows running as administrator unless you use the admin build.
- The model only knows what it can see. Save cut-off context with `A`.

## License

[Apache 2.0](LICENSE). Use at your own risk.
