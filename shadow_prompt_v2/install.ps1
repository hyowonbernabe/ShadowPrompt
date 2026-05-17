# ShadowPrompt v2 — installer
# Usage:
#   irm https://raw.githubusercontent.com/<owner>/ShadowPrompt/main/shadow_prompt_v2/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Owner   = "hyowonbernabe"
$Repo    = "ShadowPrompt"

$InstallDir = Join-Path $env:LOCALAPPDATA "ShadowPrompt"
$ConfigDir  = Join-Path $InstallDir "config"
$ConfigPath = Join-Path $ConfigDir "config.toml"

Write-Host ""
Write-Host "ShadowPrompt v2 installer" -ForegroundColor Cyan
Write-Host "-------------------------"

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $ConfigDir  | Out-Null

# Fetch latest release zip (contains exe + config example + knowledge/ tree).
$ReleaseApi = "https://api.github.com/repos/$Owner/$Repo/releases/latest"
$Release = Invoke-RestMethod -Uri $ReleaseApi -Headers @{ "User-Agent" = "shadowprompt-installer" }
$Zip = $Release.assets | Where-Object { $_.name -like "*windows-x64.zip" } | Select-Object -First 1
if (-not $Zip) { throw "No windows-x64.zip asset on latest release." }

$TempZip = Join-Path $env:TEMP "shadowprompt-install.zip"
Write-Host "Downloading $($Zip.name) ..."
Invoke-WebRequest -Uri $Zip.browser_download_url -OutFile $TempZip -UseBasicParsing

Write-Host "Extracting to $InstallDir ..."
Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
Remove-Item $TempZip -Force

# Move example config to config.toml on first install only.
$ExamplePath = Join-Path $ConfigDir "config.example.toml"
if ((Test-Path $ExamplePath) -and (-not (Test-Path $ConfigPath))) {
    Copy-Item $ExamplePath $ConfigPath
    Write-Host "Wrote default config -> $ConfigPath"
}

# Add install dir to user PATH if missing.
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    Write-Host "Added $InstallDir to user PATH (restart terminal to take effect)."
}

# API key prompt.
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
Write-Host "Knowledge folder: $InstallDir\knowledge\" -ForegroundColor Cyan
Write-Host "Add your own .md notes under knowledge\<subject>\ then activate via config.toml." -ForegroundColor Cyan
Write-Host ""
Write-Host "Done. Run:  shadowprompt" -ForegroundColor Green
Write-Host ""
