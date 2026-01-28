@echo off
setlocal enabledelayedexpansion

echo ============================================
echo       ShadowPrompt Release Builder
echo ============================================
echo.

cd /d "%~dp0shadow_prompt"

:: Extract version from Cargo.toml
for /f "tokens=3 delims= " %%v in ('findstr /r "^version" Cargo.toml') do (
    set "RAW_VERSION=%%v"
)
:: Remove quotes from version string
set "VERSION=%RAW_VERSION:"=%"
echo [*] Building version: v%VERSION%
echo.

:: Set PROTOC path for build
set "PROTOC=%CD%\tools\protoc\bin\protoc.exe"
echo [*] PROTOC Path: %PROTOC%

:: Clean previous build artifacts
echo [*] Cleaning previous build...
cargo clean --release 2>nul

:: Build release binary
echo [*] Building release binary...
cargo build --release
if errorlevel 1 (
    echo [!] Build failed. See errors above.
    pause
    exit /b 1
)

:: Create release directory
set "RELEASE_DIR=%~dp0release\ShadowPrompt"
echo [*] Preparing release directory: %RELEASE_DIR%

if exist "%RELEASE_DIR%" rmdir /s /q "%RELEASE_DIR%"
mkdir "%RELEASE_DIR%"

:: Copy executable
echo [*] Copying executable...
copy "target\release\Shadow_Prompt.exe" "%RELEASE_DIR%\" >nul

:: Copy required DLLs
echo [*] Copying DLLs...
for %%f in (target\release\*.dll) do (
    echo     - %%~nxf
    copy "%%f" "%RELEASE_DIR%\" >nul
)

:: Create placeholder directories
echo [*] Creating directories...
mkdir "%RELEASE_DIR%\config" 2>nul
mkdir "%RELEASE_DIR%\knowledge" 2>nul
mkdir "%RELEASE_DIR%\data" 2>nul

:: Copy system prompt if exists
if exist "config\system_prompt.txt" (
    echo [*] Copying system_prompt.txt...
    copy "config\system_prompt.txt" "%RELEASE_DIR%\config\" >nul
)

:: Create README for users
echo [*] Creating README...
(
echo ShadowPrompt v%VERSION% - Portable AI Assistant
echo =====================================
echo.
echo QUICK START:
echo   1. Run shadow_prompt.exe
echo   2. Complete the Setup Wizard ^(configure API keys and hotkeys^)
echo   3. Place your documents in the 'knowledge' folder for RAG
echo   4. Use your configured hotkeys to interact
echo.
echo HOTKEYS ^(Default^):
echo   - Ctrl+Shift+Space: OCR Mode ^(screen capture^)
echo   - Ctrl+Shift+V: Query AI with clipboard content
echo   - Ctrl+Shift+F12: Panic ^(exit immediately^)
echo.
echo For more info: https://github.com/hyowonbernabe/ShadowPrompt
) > "%RELEASE_DIR%\README.txt"

:: Calculate size
echo.
echo ============================================
echo              BUILD COMPLETE!
echo ============================================
echo.
echo Release location: %RELEASE_DIR%
echo.
dir "%RELEASE_DIR%" /s /b
echo.

:: Create versioned zip (requires PowerShell)
set "ZIP_FILE=%~dp0release\ShadowPrompt-v%VERSION%-windows-x64.zip"
if exist "%ZIP_FILE%" del "%ZIP_FILE%"
echo [*] Creating ZIP archive: ShadowPrompt-v%VERSION%-windows-x64.zip
powershell -Command "Compress-Archive -Path '%RELEASE_DIR%' -DestinationPath '%ZIP_FILE%' -Force"

if exist "%ZIP_FILE%" (
    echo [+] ZIP created: %ZIP_FILE%
    echo.
    echo Ready for release!
) else (
    echo [!] Failed to create ZIP
)

echo.
pause
