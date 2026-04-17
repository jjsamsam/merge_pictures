param(
  [switch]$InstallMissing
)

$ErrorActionPreference = "Stop"

function Write-Step($message) {
  Write-Host ""
  Write-Host "==> $message" -ForegroundColor Cyan
}

function Test-Command($name) {
  $null -ne (Get-Command $name -ErrorAction SilentlyContinue)
}

function Ensure-Command($commandName, $displayName, $wingetId) {
  if (Test-Command $commandName) {
    Write-Host "$displayName is already installed." -ForegroundColor Green
    return $true
  }

  Write-Host "$displayName is not installed." -ForegroundColor Yellow

  if (-not $InstallMissing) {
    Write-Host "Run this script with -InstallMissing to install missing dependencies automatically." -ForegroundColor Yellow
    return $false
  }

  if (-not (Test-Command "winget")) {
    throw "winget is not available. Please install $displayName manually."
  }

  Write-Host "Installing $displayName with winget..." -ForegroundColor Cyan
  winget install --id $wingetId --exact --accept-source-agreements --accept-package-agreements
  return (Test-Command $commandName)
}

Write-Step "Checking Windows build prerequisites"

$nodeOk = Ensure-Command -commandName "node" -displayName "Node.js" -wingetId "OpenJS.NodeJS.LTS"
$cargoOk = Ensure-Command -commandName "cargo" -displayName "Rust" -wingetId "Rustlang.Rustup"

if (-not $nodeOk -or -not $cargoOk) {
  Write-Host ""
  Write-Host "Some required tools are still missing." -ForegroundColor Red
  Write-Host "After installing them, close and reopen PowerShell, then rerun this script." -ForegroundColor Yellow
  exit 1
}

Write-Step "Version check"
node -v
npm -v
cargo --version

Write-Step "Checking recommended native build tools"
if (Test-Command "cl") {
  Write-Host "MSVC build tools detected." -ForegroundColor Green
} else {
  Write-Host "MSVC build tools were not detected in this shell." -ForegroundColor Yellow
  Write-Host "If Tauri build fails later, install Visual Studio Build Tools with Desktop development for C++." -ForegroundColor Yellow
}

Write-Step "Installing project dependencies"
npm install

Write-Step "Done"
Write-Host "You can now run:" -ForegroundColor Green
Write-Host "  npm run tauri:build -- --config src-tauri/tauri.windows.conf.json" -ForegroundColor White
