param(
  [switch]$InstallMissing,
  [switch]$SkipBootstrap
)

$ErrorActionPreference = "Stop"

function Write-Step($message) {
  Write-Host ""
  Write-Host "==> $message" -ForegroundColor Cyan
}

function Assert-Path($path, $label) {
  if (Test-Path $path) {
    Write-Host "$label found: $path" -ForegroundColor Green
  } else {
    Write-Host "$label not found: $path" -ForegroundColor Yellow
  }
}

$projectRoot = Split-Path -Parent $PSScriptRoot
$bootstrapScript = Join-Path $PSScriptRoot "bootstrap-windows.ps1"

Set-Location $projectRoot

if (-not $SkipBootstrap) {
  Write-Step "Running Windows bootstrap"

  if ($InstallMissing) {
    & $bootstrapScript -InstallMissing
  } else {
    & $bootstrapScript
  }
}

Write-Step "Building Windows installers"
npm run tauri:build -- --config src-tauri/tauri.windows.conf.json

$nsisDir = Join-Path $projectRoot "src-tauri\target\release\bundle\nsis"
$msiDir = Join-Path $projectRoot "src-tauri\target\release\bundle\msi"

Write-Step "Build output summary"
Assert-Path $nsisDir "NSIS bundle folder"
Assert-Path $msiDir "MSI bundle folder"

Write-Host ""
Write-Host "Done. Check these folders for Windows installers:" -ForegroundColor Green
Write-Host "  $nsisDir" -ForegroundColor White
Write-Host "  $msiDir" -ForegroundColor White
