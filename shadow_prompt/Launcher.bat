@echo off
set "SCRIPT_DIR=%~dp0"
set "PROTOC=%SCRIPT_DIR%tools\protoc\bin\protoc.exe"

echo [*] Setting PROTOC Path: %PROTOC%
echo [*] Starting ShadowPrompt...

cargo run
pause
