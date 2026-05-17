# ShadowPrompt v2 — silent installer
# Usage:
#   irm https://raw.githubusercontent.com/hyowonbernabe/ShadowPrompt/main/shadow_prompt_v2/install.ps1 | iex
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

# Seed config.toml from the bundled example on first install. If it
# already exists but has an empty api_key, lift the key from the new
# example so users get the bundled trial key after re-installing.
$ExamplePath = Join-Path $ConfigDir "config.example.toml"
if (Test-Path $ExamplePath) {
    if (-not (Test-Path $ConfigPath)) {
        Copy-Item $ExamplePath $ConfigPath
    } else {
        $existing = Get-Content $ConfigPath
        $hasEmptyKey = ($existing | Select-String '^api_key\s*=\s*""\s*$').Count -gt 0
        if ($hasEmptyKey) {
            $newKey = (Get-Content $ExamplePath | Select-String '^api_key\s*=').ToString()
            if ($newKey -and $newKey -notmatch 'api_key\s*=\s*""') {
                $patched = $existing -replace '^api_key\s*=\s*""\s*$', $newKey
                Set-Content -Path $ConfigPath -Value $patched -Encoding UTF8
            }
        }
    }
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
    Log "ShadowPrompt is running. Indicator pixel at top-right corner."
} else {
    Write-Host "ERROR: shadowprompt.exe not found at $ExePath" -ForegroundColor Red
    exit 1
}
