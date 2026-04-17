# Merge Picture

Desktop app skeleton for merging image files and PDFs into a single PDF document.

## Stack

- Tauri
- React
- TypeScript
- Rust
- Vitest

## Current status

This repository currently contains:
- a modern semi-dark React desktop UI
- native file picker and desktop drag-and-drop
- image thumbnail and PDF queue previews
- a Rust merge engine for mixed image and PDF inputs
- image page layout options:
  - page size
  - margins
  - fit mode
- frontend unit tests
- backend unit and integration tests
- packaging metadata and app icon assets for Tauri bundling

## Project structure

- `src/`: React UI
- `src/lib/`: frontend helper logic
- `src-tauri/src/commands/`: Tauri command layer
- `src-tauri/src/services/`: backend service layer
- `src-tauri/src/domain/`: shared domain types

## Install

Run the following once dependencies are available:

```bash
npm install
```

## Development

Frontend only:

```bash
npm run dev
```

Tauri desktop app:

```bash
npm run tauri:dev
```

## Tests

Frontend:

```bash
npm run test
```

Backend:

```bash
cd src-tauri
cargo test
```

## Package build

Desktop bundle build:

```bash
npm run tauri:build
```

Build outputs on macOS:

- App bundle:
  - `src-tauri/target/release/bundle/macos/Merge Picture.app`
- DMG:
  - `src-tauri/target/release/bundle/dmg/Merge Picture_0.1.0_aarch64.dmg`

Windows bundle build on a Windows machine:

```bash
npm run tauri:build -- --config src-tauri/tauri.windows.conf.json
```

Expected build outputs on Windows:

- NSIS installer:
  - `src-tauri/target/release/bundle/nsis/`
- MSI installer:
  - `src-tauri/target/release/bundle/msi/`

## Build guide

### Build on macOS

```bash
cd /Users/jonghochoi/workspace/merge_picture
npm install
npm run tauri:build
```

### Build on Windows

```bash
cd <project-folder>
npm install
npm run tauri:build -- --config src-tauri/tauri.windows.conf.json
```

Notes:
- `src-tauri/icons/` contains the generated app icon assets used for bundling.
- `src-tauri/icons/generated/app.icns` and `app.ico` are wired into the Tauri bundle config.
- `src-tauri/tauri.windows.conf.json` contains Windows-specific NSIS/MSI settings, including WebView2 bootstrapper mode and a fixed WiX upgrade code.

## Packaging status

- macOS `.app` and `.dmg` were built successfully on this machine.
- Windows packaging is prepared in config, but not executed here because this environment is macOS.
- Expected Windows targets:
  - NSIS `.exe` installer
  - MSI installer

## Windows release checklist

1. Run the Windows build command on a Windows machine.
2. Verify install, launch, merge flow, and uninstall behavior.
3. Confirm Start Menu shortcut placement and app icon.
4. Verify WebView2 bootstrapper behavior on a clean machine.
5. Add code signing certificate settings before public release.

## Next implementation steps

1. Verify signed macOS and Windows installer outputs.
2. Add richer PDF preview rendering for first-page thumbnails.
3. Expand merge options with page background and orientation controls.
4. Add release automation for tagged builds.
