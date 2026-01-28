# Project Documentation: ShadowPrompt

## 1. Technical Overview

ShadowPrompt is a **portable** academic interface built in Rust. It is engineered to operate without installation, making it ideal for use from removable media (USB drives).

### Architecture
- **Language**: Rust (for memory safety, speed, and zero runtime dependencies).
- **GUI Framework**: `egui` (used only for the initial setup wizard; the main runtime is headless).
- **OCR Engine**: Windows.Media.Ocr (Native Windows 10/11 API) - ensures high accuracy with 0MB additional weight.
- **LLM Client**: `reqwest` based client supporting OpenAI-compatible endpoints (Groq, OpenRouter) and Ollama.
- **Data Persistence**: All configuration and data are stored in relative paths within the `shadow_prompt` directory.

---

## 2. Portability Guide

ShadowPrompt is "Portable-First". This means it assumes no guarantees about the host filesystem other than the directory it resides in.

### Folder Structure
When placed on a USB drive (e.g., `D:\ShadowPrompt\`), the structure looks like this:

```
D:\ShadowPrompt\
├── shadow_prompt.exe          # Main executable
├── config\
│   └── config.toml           # User settings (keys, providers)
├── data\
│   └── models\               # Local embedding models
├── knowledge\                # User provided RAG documents (.md, .txt)
└── logs\                     # (Optional) Troubleshooting logs
```

### Best Practices for USB Usage
1. **Drive Speed**: Use a USB 3.0 or faster drive. Loading embedding models from a slow USB 2.0 stick may add startup latency.
2. **Drive Letters**: ShadowPrompt uses relative paths, so it does not matter if your USB mounts as `E:` on one computer and `F:` on another.
3. **Ejection**: Always close ShadowPrompt (Panic Key or Task Manager) before ejecting the drive to prevent config corruption.

---

## 3. Stealth Features

ShadowPrompt is designed to be discrete.

### Visual Footprint
- **No Taskbar Icon**: The application does not register a window in the taskbar once the setup wizard is closed.
- **Pixel Indicators**: A 1x1 or 2x2 pixel overlay (configurable) provides status feedback.
    - **Position**: Top-Right corner by default.
    - **Opacity**: High alpha blend to appear as a "stuck pixel" or system glitch rather than a UI element.

### Panic Mode (`Ctrl+Shift+F12`)
- **Action**: Immediately terminates the `shadow_prompt.exe` process.
- **Cleanup**: Clears the system clipboard to remove any evidence of AI-generated answers.

---

## 4. Configuration Reference

The `config.toml` file is the brain of the application.

```toml
[general]
wake_key = "Ctrl+Shift+Space"   # trigger OCR capture
model_key = "Ctrl+Shift+V"      # trigger AI generation from clipboard
panic_key = "Ctrl+Shift+F12"    # kill switch
use_rag = true                  # enable/disable local knowledge base

[ui]
pixel_size = 3                  # Size of the indicator pixel
pixel_position = "TopRight"     # TopRight, TopLeft, BottomRight, BottomLeft

[models]
provider = "groq"               # active provider (groq, openrouter, ollama)

[models.groq]
api_key = "gsk_..."
model_id = "llama-3.1-8b-instant"

[models.openrouter]
api_key = "sk-or-..."
model_id = "openai/gpt-4o-mini"

[rag]
enabled = true
# Relative path to knowledge folder
knowledge_path = "knowledge"
# Max results to inject into prompt
top_k = 3
```

---

## 5. Troubleshooting High-Risk Scenarios

- **Blocked USBs**: If an institution blocks USB executables, ShadowPrompt will not run. This is a system-level restriction.
- **Network Filtering**: If Groq/OpenRouter is blocked on the network, consider using the "Ollama" provider, though this breaks the "Portable" aspect as Ollama usually requires a host installation (unless a portable version of Ollama is used).
