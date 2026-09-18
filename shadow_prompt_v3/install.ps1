# ShadowPrompt v3 — silent installer
# Usage:
#   irm https://raw.githubusercontent.com/hyowonbernabe/ShadowPrompt/main/shadow_prompt_v3/install.ps1 | iex
#
# For verbose output: pipe with -Verbose flag
#   & ([scriptblock]::Create((irm https://.../install.ps1))) -Verbose

param([switch]$Verbose)

$ErrorActionPreference = "Stop"
$ProgressPreference = if ($Verbose) { "Continue" } else { "SilentlyContinue" }

$Owner = "hyowonbernabe"
$Repo  = "ShadowPrompt"

$InstallDir = Join-Path $env:LOCALAPPDATA "ShadowPrompt"
$ConfigDir  = Join-Path $InstallDir "config"
$ConfigPath = Join-Path $ConfigDir "config.toml"
$ExePath    = Join-Path $InstallDir "shadowprompt.exe"

function Log($msg) { if ($Verbose) { Write-Host $msg -ForegroundColor Cyan } }

# Stop any running instance so we can overwrite the exe.
Get-Process shadowprompt -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300

Log "Creating $InstallDir ..."
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $ConfigDir  | Out-Null

Log "Fetching latest release metadata ..."
$ReleaseApi = "https://api.github.com/repos/$Owner/$Repo/releases/latest"
$Release = Invoke-RestMethod -Uri $ReleaseApi -Headers @{ "User-Agent" = "shadowprompt-installer" }
$Zip = $Release.assets | Where-Object { $_.name -like "*windows-x64.zip" } | Select-Object -First 1
if (-not $Zip) { throw "No windows-x64.zip asset on latest release." }

$TempZip = Join-Path $env:TEMP "shadowprompt-install.zip"
Log "Downloading $($Zip.name) ..."
Invoke-WebRequest -Uri $Zip.browser_download_url -OutFile $TempZip -UseBasicParsing

Log "Extracting ..."
Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
Remove-Item $TempZip -Force -ErrorAction SilentlyContinue

# Design doc §12: API key is hardcoded for now — the released config.example.toml already has
# a real key patched in at release-build time (a CI step, same pattern v2 used). This just seeds
# config.toml from that example on first install; it never overwrites a config.toml that's
# already there, so re-installing never clobbers a key you've since swapped in yourself.
$ExamplePath = Join-Path $ConfigDir "config.example.toml"
if ((Test-Path $ExamplePath) -and (-not (Test-Path $ConfigPath))) {
    Copy-Item $ExamplePath $ConfigPath
}

# Add install dir to user PATH if missing.
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
}

# Launch the daemon immediately, detached, no window. The exe is built with
# windows_subsystem="windows" so it has no console; Start-Process returns
# instantly and the daemon keeps running after this script exits.
if (Test-Path $ExePath) {
    Start-Process -FilePath $ExePath -WindowStyle Hidden -ErrorAction SilentlyContinue
    Log "ShadowPrompt is running."
} else {
    Write-Host "ERROR: shadowprompt.exe not found at $ExePath" -ForegroundColor Red
    exit 1
}
