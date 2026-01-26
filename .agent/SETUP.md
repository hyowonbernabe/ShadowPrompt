# Rust Development Environment Setup Guide

This guide will help you set up the complete Rust environment required to build **ShadowPrompt**.

## 1. Install Rust (Rustup)
The recommended way to install Rust is via `rustup`.

### Option A: Automatic Download (PowerShell)
Run this command in your terminal to download and run the installer:
```powershell
winget install Rustlang.Rustup
```
*   **Restart your terminal** after installation.

### Option B: Manual Download
1.  Go to [rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).
2.  Download `RUSTUP-INIT.EXE` (64-bit).
3.  Run the executable.
4.  Press `1` to proceed with default installation.

## 2. Verify Installation
Open a **new** terminal (PowerShell or Command Prompt) and run:
```bash
rustc --version
cargo --version
```
You should see output similar to `rustc 1.83.0 ...`.

## 3. Install Build Tools (Visual Studio C++)
Rust on Windows requires the MSVC Linker.
1.  Download **Visual Studio Build Tools** from [visualstudio.microsoft.com/downloads](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022).
2.  Run the installer.
3.  Select **"Desktop development with C++"**.
4.  Ensure the **Windows 10/11 SDK** is checked on the right side.
5.  Click **Install**.

## 4. Install Project Dependencies
Once Rust is working, our project will require the following:

```bash
# We will run this inside the project folder later
cargo build
```

The project will automatically download crates (`windows`, `rdev`, `arboard`) when we first build.

## 5. (Optional) Install LLM Tools
If you want to test models locally (though we will use APIs for the final app):
*   **Ollama:** [ollama.com](https://ollama.com/download) (Good for testing prompts).

---
**You are now ready to build ShadowPrompt.**
