# ShadowPrompt v2 — installer
# Usage:
#   irm https://raw.githubusercontent.com/<owner>/ShadowPrompt/main/shadow_prompt_v2/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Owner   = "hyowonbernabe"
$Repo    = "ShadowPrompt"
$ExeName = "shadowprompt.exe"

$InstallDir = Join-Path $env:LOCALAPPDATA "ShadowPrompt"
$ExePath    = Join-Path $InstallDir $ExeName
$ConfigDir  = Join-Path $InstallDir "config"
$ConfigPath = Join-Path $ConfigDir "config.toml"

Write-Host ""
Write-Host "ShadowPrompt v2 installer" -ForegroundColor Cyan
Write-Host "-------------------------"

# Create install dir
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $ConfigDir  | Out-Null

# Fetch latest release asset URL for the EXE
$ReleaseApi = "https://api.github.com/repos/$Owner/$Repo/releases/latest"
$Release = Invoke-RestMethod -Uri $ReleaseApi -Headers @{ "User-Agent" = "shadowprompt-installer" }
$Asset = $Release.assets | Where-Object { $_.name -eq $ExeName } | Select-Object -First 1
if (-not $Asset) { throw "No $ExeName asset on latest release." }

Write-Host "Downloading $ExeName ..."
Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $ExePath -UseBasicParsing
Write-Host "  -> $ExePath"

# Fetch example config if not present
if (-not (Test-Path $ConfigPath)) {
    $ExampleUrl = "https://raw.githubusercontent.com/$Owner/$Repo/main/shadow_prompt_v2/config/config.example.toml"
    Invoke-WebRequest -Uri $ExampleUrl -OutFile $ConfigPath -UseBasicParsing
    Write-Host "Wrote default config -> $ConfigPath"
}

# Add to user PATH if missing
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    Write-Host "Added $InstallDir to user PATH (restart terminal to take effect)."
}

# Prompt for OpenRouter API key (trial key is bundled by default)
Write-Host ""
Write-Host "A 7-day trial OpenRouter key ships with this build." -ForegroundColor Cyan
Write-Host "Press Enter to use it, or paste your own key to override." -ForegroundColor Cyan
$Key = Read-Host "OpenRouter API key"
if ($Key) {
    (Get-Content $ConfigPath) `
        -replace '^api_key\s*=\s*".*"$', "api_key = `"$Key`"" `
        | Set-Content $ConfigPath
    Write-Host "Saved your key to config." -ForegroundColor Green
} else {
    Write-Host "Using bundled trial key. Replace it at $ConfigPath after expiry." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Done. Run:  shadowprompt" -ForegroundColor Cyan
Write-Host ""
