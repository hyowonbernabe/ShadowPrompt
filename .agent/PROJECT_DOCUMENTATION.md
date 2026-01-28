# ShadowPrompt: Portable Stealth AI Architecture

**Version:** 1.0.0
**Target OS:** Windows 10/11
**Language:** Rust

---

## 1. Project Overview
ShadowPrompt is a "Zero-Install" AI assistant designed to run entirely from a USB drive. It prioritizes stealth, portability, and speed. The application runs as a background daemon with no visible window or taskbar icon. It interacts with the user solely through global hotkeys and clipboard manipulation.

### Core Philosophy
* **Contained:** All logic, dependencies (DLLs), authentication, and vector indices reside on the USB. No files are written to the host machine.
  > **Note:** "Local Model" support via Ollama requires an external installation and is technically **NOT** contained. It is provided as a developer/debug option or for users with pre-existing setups (e.g. USB-portable Ollama).
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
| **LLM Provider** | **Groq (Primary) / OpenRouter (Secondary) / Ollama (Dev)** | **Justification:** Configurable provider selection. <br> 1. **Groq:** `llama-3.1-8b-instant` (Ultra-fast, Free). <br> 2. **OpenRouter:** `google/gemma-3-27b-it:free` (High-quality). <br> 3. **Ollama:** `llama3` (Local Dev/Debug). |

---

## 3. Workflow & UX Strategy

### A. First-Run GUI Setup
On initial launch (or when using the `--setup` flag), ShadowPrompt launches a visible configuration wizard.

1.  **Welcome**: Warns that setup runs only once and settings must be edited manually in `config.toml` afterwards.
2.  **API Config**: Configures the primary LLM provider and API keys.
3.  **Hotkeys**: Records user-preferred keybinds for Wake, Model, and Panic operations.
4.  **Resources**: downloads essential `fastembed` models to `data/models` and verifies `onnxruntime.dll`.
5.  **Re-exec Logic**: To ensure a clean transition, the Wizard spawns a new stealth instance of the app and exits itself immediately. This prevents "Not Responding" hangs on Windows.

### B. The "Stealth" Loop (Startup)
1.  **User Action:** Plug in USB, run `shadow_prompt.exe`.
2.  **State:** Background process starts.
3.  **Visual Feedback:** A single **Green Pixel** appears in the top-right (or very right) of the screen to indicate "Loaded/Ready".
4.  **Process Persistence:** If USB is removed, the process dies immediately.

### B. Scenario 1: Clipboard Trigger (The Main Way)
This is the primary method. The user supplies the text via standard system copy.

1.  **Action:** User selects text (e.g., quiz question) and presses `Ctrl + C`.
2.  **Trigger:** User presses `Model Key` (e.g., `Ctrl + Shift + V`).
3.  **Visual:** **Red Pixel** appears (Top Right) to indicate "Thinking/Processing".
4.  **Processing:**
    *   App reads current clipboard content.
    *   **Logic:** Uses the provider specified in `config.toml` (Strict Mode).
        *   **Groq:** Fast inference using `llama-3.1-8b-instant`.
        *   **OpenRouter:** General purpose API.
        *   **Ollama:** Local development/debug mode (Requires running instance).
    *   *Note: If "Web Search" is strictly needed (live data), we will need to add a specialized search tool (e.g., DuckDuckGo API), but for now, we rely on the generic intelligence of the LLM.*
5.  **Output:** App overwrites system clipboard with the Answer.
6.  **Visual:** Red Pixel disappears.
7.  **User Action:** User checks Clipboard History (`Win + V`) or pastes (`Ctrl + V`) to see the answer.

### C. Scenario 2: OCR Trigger (Fallback)
Used when copy is disabled or text is an image.

1.  **Trigger:** User presses `OCR Key` (e.g., `Ctrl + Shift + Space`).
2.  **Visual:** **Red Pixel** appears (Top Right) to indicate "Waiting for Selection".
3.  **Action:** User clicks and drags to define a region.
    *   *Visuals:* **Completely Invisible Selection.** No border, no shading.
    *   *Logic:* Captures from the screen/monitor where the cursor is focused.
4.  **Processing:**
    *   Use `Windows.Media.Ocr` to extract text.
    *   **Result:** Extracted text is placed into System Clipboard.
5.  **Visual:** Red Pixel disappears (indicating text is ready).
6.  **Follow-up:** User proceeds to **Scenario 1** (Presses `Model Key` to process the now-copied text).

### D. Scenario 3: Multiple Choice Questions (MCQ)
ShadowPrompt uses **context-aware MCQ detection** to identify answers even when the model outputs only the value (e.g., "4" instead of "A) 4").

1.  **Trigger:** Standard Model Query (Scenario 1).
2.  **Detection:** The system prompt enforces MCQ format. Additionally, the parser cross-references answer values with choices from the original question.
3.  **Visual Feedback:** A **Secondary Pixel** appears just below the main indicator:
    *   **Cyan:** Answer is **A** or **1**
    *   **Magenta:** Answer is **B** or **2**
    *   **Yellow:** Answer is **C** or **3**
    *   **Black:** Answer is **D** or **4**
    *   **White:** No MCQ detected (default state)
4.  **Clipboard:** The full answer text is still copied to the clipboard.
5.  **Reset:** The secondary indicator resets to white when a new query is started.

---

## 4. Directory Structure (USB Layout)
The application uses exe-relative paths via `get_exe_dir()`. The USB drive letter can change without breaking functionality.

```text
/ShadowPrompt                     # Release folder (from ZIP)
├── shadow_prompt.exe             # Main executable (run this!)
├── onnxruntime.dll               # Required for ONNX embeddings
├── config/
│   ├── config.toml               # Created by Setup Wizard (API keys, hotkeys)
│   ├── .setup_complete           # Marker file indicating setup is done
│   └── system_prompt.txt         # Optional: Custom system prompt
├── data/
│   ├── models/                   # FastEmbed models (downloaded on first run)
│   └── rag_index/                # JSON index for RAG
└── knowledge/                    # Drop .md/.txt files here for local RAG
    └── (your files)
```

### Distribution
- **GitHub Releases**: Pre-built `ShadowPrompt-windows-x64.zip` is created by GitHub Actions on version tags
- **Local Build**: Use `build_release.bat` at repo root to create release locally


---

## 5. Security & Safety Mechanisms

### A. The Panic Button
* **Keybind:** `Ctrl + Shift + F12`
* **Behavior:**
    1. Terminate `shadow_prompt.exe` immediately.
    2. Clear system clipboard (Safety wipe).

### B. Cost & Limits
* **Daily Limit:** Track usage in `data/usage.json`.
* **RAG Timeout:** If RAG is too slow, skip it and use raw LLM knowledge/Search.

---

## 6. Risks & Mitigations

| Risk | Impact | Mitigation Strategy |
| :--- | :--- | :--- |
| **Missing Visual Feedback** | User unsure of state | **Green Pixel** (Loaded), **Red Pixel** (Active/Waiting). |
| **Missing DLLs** | App fails on guest PC | **Side-load DLLs:** Ship `onnxruntime.dll` in `/bin`. |
| **Firewall Blocks** | Network requests fail | **User Assumption:** User handles network/AV (mentioned in scenario). |
| **Clipboard Collisions** | User loses data | **Simplicity:** We overwrite. User accepts risk. |
| **USB Removal** | Process hangs | **Heartbeat:** Check drive existence; die if gone. |

---

## 7. Configuration Specification (`config.toml`)

```toml
[general]
mode = "default"
wake_key = "Alt + Space"     # Enter OCR Selection Mode
model_key = "Ctrl + Space"   # Send Clipboard to Model
panic_key = "F8"             # Force Exit

# Keybind Configuration:
# - Supported Modifiers: Ctrl, Shift, Alt, Win/Meta
# - Supported Keys: A-Z, 0-9, F1-F12, Space, Enter, Esc, Tab, Backspace, CapsLock
# - Format: "Modifier + Key" or "Modifier + Modifier + Key"
# - Note: Overlapping keybinds will trigger a warning on startup.

# - Note: Overlapping keybinds will trigger a warning on startup.
 
[rag]
enabled = true
knowledge_path = "knowledge"    # Relative path to scan
index_path = "data/rag_index"   # Internal DB path
max_results = 3
min_score = 0.7

[visuals]
indicator_color = "#FF0000" # Processing Color
ready_color = "#00FF00"
cursor_change = false

# New Configurable Options
position = "top-right" # Options: top-right, top-left, bottom-right, bottom-left
size = 5               # Pixel size (default 5)
offset = 0             # Distance from corner (e.g. 10 to move in)
x_axis = 0            # Add to move Right (Desktop coords)
y_axis = 0            # Subtract to move Down (User preference logic: -Y = Down)

# MCQ Colors
color_mcq_a = "#00FFFF" # Cyan
color_mcq_b = "#FF00FF" # Magenta
color_mcq_c = "#FFFF00" # Yellow
color_mcq_d = "#000000" # Black
color_mcq_none = "#FFFFFF" # White (default when no MCQ detected)
color_processing = "#FF0000" # Red

[models]
# Options: "openrouter", "github_copilot", "ollama", "groq"
provider = "groq"

[models.groq]
api_key = "gsk_..."
model_id = "llama-3.1-8b-instant"

[models.openrouter]
api_key = "sk-or-..."
model_id = "google/gemma-3-27b-it:free"

[models.ollama]
base_url = "http://localhost:11434"
model_id = "llama3"

[models.github_copilot]
token_path = "config/copilot_token.json"
```

## 8. Build & Deployment
To ensure portability across Windows machines:
1.  **Build**: Use the provided `Launcher.bat` or manually set `PROTOC` env var to `tools/protoc/bin/protoc.exe`.
2.  **Run**: Double-click `Launcher.bat`.
3.  **Knowledge**: Add files to `knowledge/`. Restart app to re-index.
```
