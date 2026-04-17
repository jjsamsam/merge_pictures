# Windows Build Guide

## Recommended quick start

Open PowerShell in the project folder and run:

```powershell
.\scripts\bootstrap-windows.ps1 -InstallMissing
```

If you see a message like `running scripts is disabled on this system`, allow scripts for the current PowerShell window only, then run the script again:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
.\scripts\bootstrap-windows.ps1 -InstallMissing
```

This is the recommended option because it only affects the current PowerShell session.

What this script does:
- checks whether `node`, `npm`, and `cargo` are available
- checks whether `Visual Studio Build Tools 2022` with `Desktop development with C++` is available
- installs missing `Node.js LTS` and `Rust` with `winget` if needed
- installs missing `Visual Studio Build Tools 2022` with the required C++ workload when `-InstallMissing` is used
- tries to load the Visual Studio build environment into the current PowerShell session
- runs `npm install`
- prints the build command for Windows

If you prefer not to auto-install anything, run:

```powershell
.\scripts\bootstrap-windows.ps1
```

If you want a longer-lasting setting for your own account, you can use:

```powershell
Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned
```

## One-step release build

If you want a single command that:
- checks required tools
- installs missing dependencies when allowed
- runs the Windows Tauri build
- prints the output folders

use:

```powershell
.\scripts\release-windows.ps1 -InstallMissing
```

## Build command

After the environment is ready:

```powershell
npm run tauri:build -- --config src-tauri/tauri.windows.conf.json
```

## Expected output folders

- NSIS installer:
  - `src-tauri\target\release\bundle\nsis\`
- MSI installer:
  - `src-tauri\target\release\bundle\msi\`

## If `npm` is not recognized

That usually means Node.js is not installed, or PowerShell has not been reopened after installation.

Fix:
1. Install Node.js LTS
2. Close PowerShell
3. Open PowerShell again
4. Run:

```powershell
node -v
npm -v
```

## If Tauri build fails on Windows

Most common missing dependency:
- Visual Studio Build Tools 2022 with Desktop development with C++

Recommended workload:
- Desktop development with C++

You may also need:
- WebView2 runtime support on the target machine

## If you see `link.exe not found`

That means the Microsoft C++ linker is not available in the current shell.

Recommended fix:
1. Run the bootstrap script with installation enabled:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
.\scripts\bootstrap-windows.ps1 -InstallMissing
```

2. If it still fails, open one of these shells and run the build there:
- `Developer PowerShell for Visual Studio`
- `x64 Native Tools Command Prompt for VS`

3. Confirm the linker is visible:

```powershell
where.exe link
```
