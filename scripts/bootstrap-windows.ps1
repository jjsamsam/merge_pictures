param(
  [switch]$InstallMissing
)

$ErrorActionPreference = "Stop"
$VsBuildToolsPackageId = "Microsoft.VisualStudio.2022.BuildTools"
$VsCppWorkloadId = "Microsoft.VisualStudio.Workload.VCTools"
$VsCppToolsComponentId = "Microsoft.VisualStudio.Component.VC.Tools.x86.x64"

function Write-Step($message) {
  Write-Host ""
  Write-Host "==> $message" -ForegroundColor Cyan
}

function Test-Command($name) {
  $null -ne (Get-Command $name -ErrorAction SilentlyContinue)
}

function Get-VsWherePath() {
  $programFilesX86 = ${env:ProgramFiles(x86)}

  if (-not $programFilesX86) {
    return $null
  }

  $candidate = Join-Path $programFilesX86 "Microsoft Visual Studio\Installer\vswhere.exe"

  if (Test-Path $candidate) {
    return $candidate
  }

  return $null
}

function Get-VsInstallationPath() {
  $vsWhere = Get-VsWherePath

  if (-not $vsWhere) {
    return $null
  }

  $installationPath = & $vsWhere -latest -products * -requires $VsCppToolsComponentId -property installationPath 2>$null

  if ([string]::IsNullOrWhiteSpace($installationPath)) {
    return $null
  }

  return $installationPath.Trim()
}

function Get-VsDevCmdPath() {
  $installationPath = Get-VsInstallationPath

  if (-not $installationPath) {
    return $null
  }

  $candidate = Join-Path $installationPath "Common7\Tools\VsDevCmd.bat"

  if (Test-Path $candidate) {
    return $candidate
  }

  return $null
}

function Import-VsBuildEnvironment() {
  $vsDevCmd = Get-VsDevCmdPath

  if (-not $vsDevCmd) {
    return $false
  }

  Write-Host "Loading Visual Studio build environment into the current PowerShell session..." -ForegroundColor Cyan
  $envDump = & cmd.exe /s /c "`"$vsDevCmd`" -arch=amd64 -host_arch=amd64 >nul && set"

  foreach ($line in $envDump) {
    if ($line -match "^(.*?)=(.*)$") {
      [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
    }
  }

  return (Test-Command "link")
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

function Ensure-VsBuildTools() {
  if (Get-VsInstallationPath) {
    Write-Host "Visual Studio Build Tools with C++ tools are already installed." -ForegroundColor Green
    return $true
  }

  Write-Host "Visual Studio Build Tools with Desktop development with C++ are not installed." -ForegroundColor Yellow

  if (-not $InstallMissing) {
    Write-Host "Run this script with -InstallMissing to install Visual Studio Build Tools automatically." -ForegroundColor Yellow
    return $false
  }

  if (-not (Test-Command "winget")) {
    throw "winget is not available. Please install Visual Studio Build Tools manually."
  }

  Write-Host "Installing Visual Studio Build Tools with Desktop development with C++..." -ForegroundColor Cyan
  winget install --id $VsBuildToolsPackageId --exact --accept-source-agreements --accept-package-agreements --override "--wait --passive --add $VsCppWorkloadId --add $VsCppToolsComponentId --includeRecommended"

  return [bool](Get-VsInstallationPath)
}

Write-Step "Checking Windows build prerequisites"

$nodeOk = Ensure-Command -commandName "node" -displayName "Node.js" -wingetId "OpenJS.NodeJS.LTS"
$cargoOk = Ensure-Command -commandName "cargo" -displayName "Rust" -wingetId "Rustlang.Rustup"
$vsOk = Ensure-VsBuildTools

if (-not $nodeOk -or -not $cargoOk -or -not $vsOk) {
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
if (Test-Command "link") {
  Write-Host "MSVC linker detected in the current shell." -ForegroundColor Green
} elseif (Import-VsBuildEnvironment) {
  Write-Host "MSVC linker loaded into the current shell from the Visual Studio build environment." -ForegroundColor Green
} else {
  Write-Host "Visual Studio Build Tools are installed, but link.exe is not available in this shell." -ForegroundColor Yellow
  Write-Host "Open Developer PowerShell for Visual Studio, or close and reopen PowerShell after installation." -ForegroundColor Yellow
}

Write-Step "Installing project dependencies"
npm install

Write-Step "Done"
Write-Host "You can now run:" -ForegroundColor Green
Write-Host "  npm run tauri:build -- --config src-tauri/tauri.windows.conf.json" -ForegroundColor White
