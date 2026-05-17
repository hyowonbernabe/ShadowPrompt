# ShadowPrompt v2 — uninstaller
# Removes install dir, PATH entry, and any debug profile temp dir.

$ErrorActionPreference = "SilentlyContinue"

$InstallDir = Join-Path $env:LOCALAPPDATA "ShadowPrompt"
$TempProfile = Join-Path $env:TEMP "shadowprompt-debug-profile"

Write-Host "Removing $InstallDir ..."
Remove-Item -Recurse -Force $InstallDir

Write-Host "Removing $TempProfile ..."
Remove-Item -Recurse -Force $TempProfile

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath) {
    $Cleaned = ($UserPath -split ';' | Where-Object { $_ -and ($_ -ne $InstallDir) }) -join ';'
    [Environment]::SetEnvironmentVariable("Path", $Cleaned, "User")
    Write-Host "Removed install dir from user PATH."
}

Write-Host "Done." -ForegroundColor Green
