<p align="center">
  <img src="shadow_prompt/src/assets/logo_512.png" width="150" alt="ShadowPrompt Logo">
</p>

# ShadowPrompt: Zero-Install Discrete Academic Interface

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-1.0.0-green.svg)](#)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](#)

ShadowPrompt is a lightweight, stealthy AI assistant designed for high-stakes environments where focus and discretion are paramount. Built with Rust and designed to run entirely from a **USB Drive**, it provides real-time knowledge synthesis without leaving a footprint on the host machine.

---

## 💡 The "Why" (Motivation)
In modern academic and professional settings, the gap between "having data" and "understanding context" can be overwhelming. Standard AI tools are often intrusive, demanding browser tabs, installation permissions, and constant context switching.

ShadowPrompt was born from the need for a **Discrete Academic Interface**—a tool that lives in the shadows (literally) to support students and professionals during intensive sessions (quizzes, exams, or complex research).

### The Shadow Advantage:
- **Zero-Install Portability**: Run directly from a USB stick. No registry changes, no temp files, and no trace left behind.
- **Cognitive Flow Preservation**: No switching windows. Use global hotkeys to query models and receive answers directly in your workflow.
- **Stealth by Design**: No taskbar icon, no windows. Visual feedback is limited to single, configurable pixels on the corner of your screen.
- **Safety First**: A dedicated "Panic Button" kills the process and wipes the clipboard instantly.

---

## ✨ Key Features
- **Stealth Pixel Indicators**: Discrete visual cues (Green/Red/Cyan/etc.) indicate readiness, processing, and multi-choice answers.
- **Clipboard Injection**: Select text, trigger the model, and have the answer automatically placed back in your clipboard.
- **OCR Region Capture**: Capture text from restricted environments (images, PDFs with disabled copy) using invisible region selection.
- **Local RAG (Retrieval-Augmented Generation)**: Index your own `.md` or `.txt` notes locally for project-specific intelligence.
- **Multi-Provider Support**: Seamlessly switch between **Groq** (Primary), **OpenRouter** (Secondary), and **Ollama** (Local Dev).
- **Automated MCQ Detection**: Intelligent parsing of multiple-choice questions with color-coded pixel shortcuts.

---

## 🛠️ Tech Stack
| Component | Technology |
| :--- | :--- |
| **Engine** | **Rust** (Memory-safe, Static Binaries) |
| **Intelligence** | Groq (Llama 3.1), OpenRouter, Ollama |
| **Embeddings** | **FastEmbed-rs** (Local BGE-Small-EN-v1.5) |
| **OCR** | Windows.Media.Ocr (Native Windows 10/11 API) |
| **Hooks** | Win32 API / Rdev (Global Keyboard Hooks) |
| **Interface** | egui (Setup Wizard only) |

---

## 📋 Prerequisites
- **OS**: Windows 10 or 11.
- **Hardware**: Any standard USB 2.0/3.0 drive.
- **Internet**: Required for API-based models (Groq/OpenRouter).

---

## 🚀 Installation & Setup (The "Happy Path")
1. **Clone the Repository**:
   ```bash
   git clone https://github.com/yourusername/ShadowPrompt.git
   ```
2. **Transfer to USB**:
   Copy the `ShadowPrompt` folder to your USB drive.
3. **Initialize**:
   Run `Launcher.bat`. On the first launch, the **Stealth Setup Wizard** will appear.
4. **Configure**:
   - Enter your **Groq** or **OpenRouter** API key.
   - Set your preferred hotkeys.
   - Wait for the embedding models to download to `data/models`.
5. **Finalize**:
   The Wizard will close and spawn the background daemon. Look for the **Green Pixel** in the top-right corner.

---

## 🎮 Usage Guide

### Standard Workflow
1. **Prepare Context**: Copy text (`Ctrl + C`) or use **OCR Mode** (`Ctrl + Shift + Space`) to grab text from the screen.
2. **Query Model**: Press `Ctrl + Shift + V` (default).
3. **Watch the Indicator**:
   - **🔴 Red Pixel**: Thinking...
   - **🟢 Green Pixel**: Ready/Answer available in clipboard.
4. **Retrieve**: Paste (`Ctrl + V`) to see the answer.

### MCQ Shortcuts (Visual Only)
ShadowPrompt uses the **CMYB** (Cyan, Magenta, Yellow, Black) color pattern for Multiple Choice Questions. This mnemonic is designed for rapid recall and covers four options where standard RGB falls short:
- **Cyan**: Answer is **A / 1**
- **Magenta**: Answer is **B / 2**
- **Yellow**: Answer is **C / 3**
- **Black**: Answer is **D / 4**

### The Panic Button
Press `Ctrl + Shift + F12` to instantly terminate ShadowPrompt and wipe your clipboard history.

---

## 🗺️ Roadmap
- [ ] **High-Fidelity Search**: Integration with Perplexity or Brave Search APIs for live data synthesis.
- [ ] **Expanded Providers**: Support for Anthropic, OpenAI, and DeepSeek.
- [ ] **Optimized RAG**: Precision improvements for large local knowledge bases.
- [ ] **Cross-Platform Stealth**: Experimental support for Linux/macOS background daemons.

---

## 🤝 Contributing
Contributions are welcome! Whether it's fixing bugs or suggesting new stealth features, please follow these steps:
1. Fork the Project.
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`).
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`).
4. Push to the Branch (`git push origin feature/AmazingFeature`).
5. Open a Pull Request.

---

## 📄 License
ShadowPrompt is distributed under the **Apache License 2.0**. See `LICENSE` for more information. Attribution is required for all derivatives.

*Disclaimer: This tool is intended for personal productivity and research. Users are responsible for adhering to their institutional policies.*
