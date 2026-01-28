<p align="center">
  <img src="shadow_prompt/assets/logo_512.png" width="150" alt="ShadowPrompt Logo">
</p>

# ShadowPrompt: Zero-Install Discrete Academic Interface

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/hyowonbernabe/ShadowPrompt?include_prereleases)](https://github.com/hyowonbernabe/ShadowPrompt/releases)
[![Automated Checks](https://github.com/hyowonbernabe/ShadowPrompt/actions/workflows/check.yml/badge.svg)](https://github.com/hyowonbernabe/ShadowPrompt/actions/workflows/check.yml)
[![Windows](https://img.shields.io/badge/platform-Windows%2010%2F11-0078D6?logo=windows)](https://github.com/hyowonbernabe/ShadowPrompt/releases)

ShadowPrompt is a lightweight, stealthy AI assistant designed for high-stakes environments where focus and discretion are paramount. Built with Rust and designed to run entirely from a **USB Drive**, it provides real-time knowledge synthesis without leaving a footprint on the host machine.

---

## 💡 The "Why" (Motivation)

In modern academic and professional settings, the gap between "having data" and "understanding context" can be overwhelming. Standard AI tools are often intrusive, demanding browser tabs, installation permissions, and constant context switching.

ShadowPrompt was born from the need for a **Discrete Academic Interface**—a tool that lives in the shadows (literally) to support students and professionals during intensive sessions.

### The Shadow Advantage
| Feature | Benefit |
|---------|---------|
| **Zero-Install Portability** | Run directly from USB. No registry, no temp files, no trace. |
| **Cognitive Flow Preservation** | Global hotkeys—no window switching required. |
| **Stealth by Design** | No taskbar icon, no windows. Just a single pixel indicator. |
| **Panic Button** | Instantly kill process and wipe clipboard with one hotkey. |

---

## ✨ Key Features

- **🔴 Stealth Pixel Indicators**: Discrete visual cues (Green/Red/Cyan/etc.) indicate status and answers
- **📋 Clipboard Injection**: Copy question → trigger → answer appears in clipboard
- **👁️ OCR Region Capture**: Extract text from images or locked PDFs with invisible selection
- **📚 Local RAG**: Index your `.md`/`.txt` notes for project-specific AI context
- **🔄 Multi-Provider**: Switch between **Groq**, **OpenRouter**, or **Ollama** (local)
- **🎯 MCQ Detection**: Automatic color-coded pixel for multiple choice answers

---

## 🛠️ Tech Stack

| Component | Technology |
|:----------|:-----------|
| **Engine** | Rust (Memory-safe, single binary) |
| **LLM Providers** | Groq (Llama 3.1), OpenRouter, Ollama |
| **Embeddings** | FastEmbed-rs (BGE-Small-EN-v1.5, local) |
| **OCR** | Windows.Media.Ocr (Native API, 0MB overhead) |
| **Hooks** | Win32 API / Rdev (Global Keyboard) |
| **GUI** | egui (Setup Wizard only) |

---

## 📋 Prerequisites

- **OS**: Windows 10 or 11 (64-bit)
- **Hardware**: Any USB 2.0/3.0 drive with ~100MB free space
- **Internet**: Required for API-based models (Groq/OpenRouter)
- **API Key**: Free account from [Groq](https://console.groq.com/) or [OpenRouter](https://openrouter.ai/)

---

## 🚀 Installation & Setup

### Download (Recommended)

1. **Download**: Get `ShadowPrompt-windows-x64.zip` from [Releases](https://github.com/hyowonbernabe/ShadowPrompt/releases/latest)
2. **Extract**: Unzip to your USB drive
3. **Run**: Double-click `shadow_prompt.exe` → Setup Wizard opens
4. **Configure**: Enter API key, set hotkeys, wait for model download
5. **Ready**: Look for the **Green Pixel** in the top-right corner

### Get Your API Key

<details>
<summary><b>Groq (Recommended - Free & Fast)</b></summary>

1. Go to [console.groq.com](https://console.groq.com/)
2. Sign up or log in
3. Navigate to **API Keys** → **Create API Key**
4. Copy the key (starts with `gsk_...`)
</details>

<details>
<summary><b>OpenRouter (More Models)</b></summary>

1. Go to [openrouter.ai](https://openrouter.ai/)
2. Sign up or log in
3. Navigate to **Keys** → **Create Key**
4. Copy the key (starts with `sk-or-...`)
</details>

### Build from Source (Development)

<details>
<summary>Click to expand build instructions</summary>

**Prerequisites:**
- Rust toolchain (stable)
- Visual Studio Build Tools with C++ workload
- Protobuf compiler

**Steps:**
```bash
git clone https://github.com/hyowonbernabe/ShadowPrompt.git
cd ShadowPrompt
.\build_release.bat
```

The release will be created in the `release/` folder.
</details>

---

## 🎮 Usage Guide

### Quick Start
```
1. Copy text (Ctrl+C) or OCR capture (Ctrl+Shift+Space)
2. Query AI (Ctrl+Shift+V)
3. Wait for Green pixel
4. Paste answer (Ctrl+V)
```

### Default Hotkeys

| Hotkey | Action |
|--------|--------|
| `Ctrl+Shift+Space` | Enter OCR capture mode |
| `Ctrl+Shift+V` | Send clipboard to AI |
| `Ctrl+Shift+F12` | **PANIC** - Kill process & wipe clipboard |

> **Tip**: Hotkeys are fully configurable during setup or in `config/config.toml`

### Visual Indicators

| Pixel Color | Meaning |
|-------------|---------|
| 🟢 **Green** | Ready / Answer available |
| 🔴 **Red** | Processing / Waiting |
| 🟦 **Cyan** | MCQ Answer: **A** |
| 🟪 **Magenta** | MCQ Answer: **B** |
| 🟨 **Yellow** | MCQ Answer: **C** |
| ⬛ **Black** | MCQ Answer: **D** |

---

## ⚙️ Configuration

After setup, you can edit `config/config.toml` directly:

```toml
[general]
wake_key = "Ctrl+Shift+Space"    # OCR mode
model_key = "Ctrl+Shift+V"       # Query AI
panic_key = "Ctrl+Shift+F12"     # Emergency exit
use_rag = true                   # Enable local knowledge

[models]
provider = "groq"                # Options: groq, openrouter, ollama

[models.groq]
api_key = "gsk_your_key_here"
model_id = "llama-3.1-8b-instant"

[rag]
enabled = true
knowledge_path = "knowledge"     # Drop .md/.txt files here
```

To re-run the Setup Wizard: `shadow_prompt.exe --setup`

---

## 🔒 Security & Privacy

- **Local Processing**: OCR and embeddings run 100% locally (no data sent)
- **No Telemetry**: ShadowPrompt does not collect any usage data
- **Portable**: All data stays on your USB drive
- **Panic Wipe**: Clipboard is cleared on panic to prevent data leakage

> **Note**: Queries sent to Groq/OpenRouter are subject to their privacy policies.

---

## ❓ Troubleshooting

<details>
<summary><b>Setup Wizard doesn't appear</b></summary>

Delete `config/.setup_complete` and run the exe again.
</details>

<details>
<summary><b>"Failed to initialize FastEmbed"</b></summary>

Ensure you have internet connection for first-run model download. Check that `data/models/` is writable.
</details>

<details>
<summary><b>Hotkeys don't work</b></summary>

- Check for conflicts with other applications
- Try running as Administrator
- Verify hotkeys in `config/config.toml`
</details>

<details>
<summary><b>OCR returns empty text</b></summary>

Ensure you're on Windows 10/11 with English language pack installed. OCR uses the system's Windows.Media.Ocr API.
</details>

<details>
<summary><b>API errors / "Failed to get response"</b></summary>

- Verify your API key is correct
- Check your internet connection
- Ensure you haven't exceeded rate limits
</details>

---

## 🗺️ Roadmap

- [ ] **Web Search Integration**: Perplexity or Brave Search APIs for live data
- [ ] **More Providers**: Anthropic, OpenAI, DeepSeek support
- [ ] **Enhanced RAG**: Chunking strategies and hybrid search
- [ ] **Linux/macOS**: Cross-platform stealth daemons
- [ ] **32-bit Support**: Windows x86 builds

---

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📄 License

Distributed under the **Apache License 2.0**. See [`LICENSE`](LICENSE) for details.

---

<p align="center">
  <i>Disclaimer: This tool is intended for personal productivity and research.<br>
  Users are responsible for adhering to their institutional policies.</i>
</p>
