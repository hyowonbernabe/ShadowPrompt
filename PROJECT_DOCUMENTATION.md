# ShadowPrompt: Portable Stealth AI Architecture

**Version:** 1.0.0
**Target OS:** Windows 10/11
**Language:** Rust

---

## 1. Project Overview
ShadowPrompt is a **Portable** AI assistant designed to run entirely from a USB drive. It prioritizes stealth, portability, and speed. The application runs as a background daemon with no visible window or taskbar icon. It interacts with the user solely through global hotkeys and clipboard manipulation.

### Core Philosophy
* **Contained:** All logic, dependencies (DLLs), authentication, and vector indices reside on the USB. No files are written to the host machine.
  > **Note:** "Local Model" support via Ollama requires an external installation and is technically **NOT** contained. It is provided as a developer/debug option or for users with pre-existing setups (e.g., USB-portable Ollama).
* **Invisible:** The UI is non-intrusive. Interactions occur via invisible overlays and clipboard injections.
* **Agnostic:** Capable of reading screen context (OCR) or clipboard content to query generic LLMs or project-specific RAG data.

---

## 2. Technical Stack
We utilize **Rust** for its memory safety, zero-dependency compilation (static binary), and direct access to Windows APIs.

| Component | Choice | Justification |
| :--- | :--- | :--- |
| **Language** | **Rust** | Compiles to a single `.exe`. Extremely performant and lightweight. |
| **OCR Engine** | **Windows.Media.Ocr** | Native Windows 10/11 API. Adds **0MB** to binary size. Privacy-friendly (local). |
| **Windowing/GUI** | **Windows API (Win32)** | Required for creating the "invisible" overlay and handling global keyboard hooks. |
| **Setup Framework** | **eframe (egui)** | Lightweight GUI used exclusively for the First-Run Setup Wizard. |
| **Vector DB** | **Lightweight (JSON/Memory)** | Simplified RAG using in-memory vectors and JSON storage for maximum portability. |
| **Embeddings** | **FastEmbed-rs** | Local ONNX-based embedding generation (BGE-Small-EN-v1.5). No API keys required. |
| **LLM Provider** | **Groq (Primary) / OpenRouter (Secondary) / Ollama** | **Groq**: Ultra-fast, Free (`llama-3.1-8b-instant`). <br> **OpenRouter**: General purpose. <br> **Ollama**: Local dev/debug. |

---

## 3. Workflow & UX Strategy

### A. First-Run GUI Setup
On initial launch (or when using the `--setup` flag), ShadowPrompt launches a visible configuration wizard.
1. **Welcome**: Warns that setup runs only once.
2. **API Config**: Configures the primary LLM provider and API keys.
3. **Hotkeys**: Records user-preferred keybinds.
4. **Resources**: Downloads essential `fastembed` models to `data/models`.
5. **Re-exec Logic**: Spawns the stealth background process and exits the wizard.

### B. The "Stealth" Loop (Startup)
1. **User Action:** Plug in USB, run `shadow_prompt.exe`.
2. **State:** Background process starts.
3. **Visual Feedback:** A single **Green Pixel** appears to indicate "Loaded/Ready".
4. **Process Persistence:** If USB is removed, the process dies immediately/fails gracefully.

### C. Scenario 1: Clipboard Trigger (The Main Way)
1. **Action:** User selects text and copies (`Ctrl + C`).
2. **Trigger:** User presses `Model Key` (e.g., `Ctrl + Shift + V`).
3. **Visual:** **Red Pixel** (Processing).
4. **Processing:** App reads clipboard, sends to LLM (Groq/OpenRouter/Ollama).
5. **Output:** App overwrites system clipboard with the Answer.
6. **Visual:** Red Pixel disappears.
7. **User Action:** Paste (`Ctrl + V`) to see the answer.

### D. Scenario 2: OCR Trigger (Fallback)
1. **Trigger:** User presses `OCR Key` (e.g., `Ctrl + Shift + Space`).
2. **Visual:** **Red Pixel** (Waiting for Selection).
3. **Action:** User clicks and drags region. **Invisible Selection** (no border).
4. **Processing:** `Windows.Media.Ocr` extracts text -> System Clipboard.
5. **Follow-up:** User proceeds to **Scenario 1** (Trigger Model).

### E. Scenario 3: Multiple Choice Questions (MCQ)
ShadowPrompt uses **context-aware MCQ detection** to identify answers (e.g., "A", "B", "C", "D").
1. **Trigger:** Standard Model Query.
2. **Visual Feedback:** **Secondary Pixel** appears:
    * **Cyan:** A / 1
    * **Magenta:** B / 2
    * **Yellow:** C / 3
    * **Black:** D / 4
    * **White:** No MCQ detected
3. **Clipboard:** Full answer text is still copied.

---

## 4. Directory Structure & Portability
The application uses exe-relative paths via `get_exe_dir()`. The USB drive letter can change without breaking functionality.

```text
/ShadowPrompt                     # Release folder
├── shadow_prompt.exe             # Main executable
├── onnxruntime.dll               # Required for ONNX embeddings
├── config/
│   ├── config.toml               # User settings
│   ├── .setup_complete           # Marker file
│   └── system_prompt.txt         # Custom system prompt
├── data/
│   ├── models/                   # FastEmbed models
│   └── rag_index/                # RAG Index
└── knowledge/                    # Local RAG documents
    └── (your files)
```

### Best Practices for USB Usage
1. **Drive Speed**: Use a USB 3.0 or faster drive. Loading embedding models from a slow USB 2.0 stick may add startup latency.
2. **Drive Letters**: ShadowPrompt uses relative paths, so it does not matter if your USB mounts as `E:` on one computer and `F:` on another.
3. **Ejection**: Always close ShadowPrompt (Panic Key or Task Manager) before ejecting the drive to prevent config corruption.

---

## 5. Security & Safety Mechanisms

### A. The Panic Button
* **Keybind:** `Ctrl + Shift + F12`
* **Behavior:** Terminates process immediately and clears system clipboard.

### B. High-Risk Scenarios & Mitigations
| Risk | Impact | Strategy |
| :--- | :--- | :--- |
| **Visual Feedback** | Unsure of state | **Green Pixel** (Ready), **Red Pixel** (Busy). |
| **Blocked USBs** | App wont run | System-level restriction. Cannot bypass if executables are blocked. |
| **Network Filtering** | API blocked | Use offline models (Ollama) if possible, though this affects portability. |
| **Clipboard Collisions** | Data loss | We overwrite. User accepts risk for stealth. |

---

## 6. Configuration Specification (`config.toml`)

```toml
[general]
mode = "default"
wake_key = "Alt + Space"     # Enter OCR Selection Mode
model_key = "Ctrl + Space"   # Send Clipboard to Model
panic_key = "F8"             # Force Exit

[rag]
enabled = true
knowledge_path = "knowledge"    # Relative path to scan
index_path = "data/rag_index"   # Internal DB path
max_results = 3
min_score = 0.7

[visuals]
indicator_color = "#FF0000" # Processing Color
ready_color = "#00FF00"
position = "top-right"      # top-right, top-left, bottom-right, bottom-left
size = 5
offset = 0

# MCQ Colors
color_mcq_a = "#00FFFF" # Cyan
color_mcq_b = "#FF00FF" # Magenta
color_mcq_c = "#FFFF00" # Yellow
color_mcq_d = "#000000" # Black

[models]
provider = "groq" # Options: groq, openrouter, ollama

[models.groq]
api_key = "gsk_..."
model_id = "llama-3.1-8b-instant"

[models.openrouter]
api_key = "sk-or-..."
model_id = "google/gemma-3-27b-it:free"
```

## 7. Build & Deployment
1. **Build**: Use `Launcher.bat` or `build_release.bat`.
2. **Run**: Double-click `Launcher.bat`.
3. **Knowledge**: Add .md/.txt files to `knowledge/`. Restart to index.
