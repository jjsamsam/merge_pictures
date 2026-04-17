# PDF Merge App Development Plan

## 1. Goal

Build a desktop application that merges input images and PDFs into a single PDF document.

Target platforms:
- macOS
- Windows

Core expectations:
- Modern semi-dark GUI
- Easy installation and distribution
- Optional portable build if practical
- Reliable uninstall support for installer-based distributions
- Unit tests and integration tests written and passing

## 2. Recommended Stack

### Primary choice
- App shell: Tauri
- Frontend: React + TypeScript + Vite
- Backend processing: Rust
- Styling: Tailwind CSS or CSS Modules with design tokens
- Testing:
  - Frontend unit tests: Vitest + Testing Library
  - Backend unit tests: Rust `cargo test`
  - Integration tests: Rust integration tests and end-to-end app workflow tests

### Why this stack
- Tauri packages well for macOS and Windows
- Installers are straightforward to produce
- Portable-style distribution is easier than with heavier runtimes
- Rust is strong for file I/O, PDF processing, and cross-platform reliability
- React makes it easy to build a polished modern GUI

## 3. Product Scope

### In scope for v1
- Add multiple files
- Support image inputs:
  - JPG/JPEG
  - PNG
  - WEBP
- Support PDF inputs
- Merge mixed inputs into one PDF
- Reorder files before merge
- Remove selected files
- Show thumbnail or file preview when possible
- Save output PDF to user-selected location
- Basic progress and error feedback

### Nice to have after v1
- Rotate images before merge
- Page size options
- Margin options
- Fit mode options:
  - Contain
  - Cover
  - Original size within page bounds
- Drag-and-drop folder import
- Recent output location memory
- Open merged PDF after export

### Out of scope for initial release
- PDF editing beyond merge
- OCR
- Cloud sync
- Multi-user collaboration
- Mobile support

## 4. Functional Requirements

### File input
- Users can add files by:
  - Clicking an "Add Files" button
  - Drag and drop
- The app validates supported file types before processing
- Unsupported files are rejected with a clear message

### File list management
- Show selected files in a reorderable list
- Each list item shows:
  - File name
  - File type
  - Page count for PDFs if available
  - Thumbnail or placeholder preview
- Users can:
  - Reorder
  - Remove one item
  - Clear all items

### Merge behavior
- If input is an image:
  - Convert each image into one PDF page
- If input is a PDF:
  - Preserve all existing pages
- Merge all pages in the list order into one PDF

### Output
- Users choose output path and file name
- Default output name example:
  - `merged-YYYYMMDD-HHMMSS.pdf`
- On success:
  - Show success message
  - Optionally open file or reveal in folder

### Error handling
- Graceful handling for:
  - Corrupted files
  - Password-protected PDFs
  - Read permission failures
  - Write permission failures
  - Invalid image data
  - Empty merge request

## 5. Non-Functional Requirements

### Performance
- UI should remain responsive during merge
- Large files should be processed off the UI thread
- Memory usage should be bounded where possible

### Reliability
- Temporary files must be cleaned up
- Partial failure should not leave broken output without explanation
- File paths with spaces, Unicode, and long names must work

### Cross-platform
- Same core behavior on macOS and Windows
- Platform-specific packaging supported

### UX
- Semi-dark visual design
- Clear primary action
- Low-friction file import
- Obvious progress state and error feedback

## 6. Technical Architecture

### Frontend responsibilities
- Window layout and interactions
- Drag-and-drop handling
- Reorderable file list
- Preview rendering
- Output settings form
- Progress, success, and error display
- Invoking Tauri commands

### Backend responsibilities
- File validation and metadata extraction
- Image decoding and PDF page generation
- PDF loading and page copying
- Merge orchestration
- Temporary file management
- Final PDF writing
- Structured error reporting

### Proposed module layout

#### Frontend
- `src/app/`
- `src/components/`
- `src/features/file-list/`
- `src/features/merge/`
- `src/features/settings/`
- `src/lib/`
- `src/styles/`

#### Backend
- `src-tauri/src/commands/`
- `src-tauri/src/services/`
- `src-tauri/src/domain/`
- `src-tauri/src/errors/`
- `src-tauri/src/tests/`

## 7. UI / UX Plan

### Visual direction
- Semi-dark background
- High-contrast primary action color
- Subtle elevation and borders
- Modern desktop layout, not mobile-first imitation

### Suggested layout
- Header:
  - App title
  - Add files button
  - Clear all button
- Main area:
  - Left or top: drag-and-drop import panel
  - Center: reorderable file queue
  - Right or bottom: output settings and merge action
- Footer/status area:
  - Validation messages
  - Progress
  - Success/failure notices

### Key states to design
- Empty state
- Files loaded state
- Processing state
- Success state
- Recoverable error state

## 8. PDF Processing Strategy

### Candidate libraries
- Rust PDF library for merging existing PDFs
- Rust image library for decoding image files
- PDF generation library for creating pages from images

### Processing pipeline
1. Accept selected files in UI order
2. Inspect each file type
3. For image files:
   - Decode image
   - Convert to PDF page
4. For PDF files:
   - Read page structure
   - Append pages to output document
5. Write final PDF
6. Return success or structured error

### Important implementation concerns
- Image orientation metadata
- DPI and page scaling
- PDF compatibility level
- Encrypted PDFs
- File handles released promptly on Windows

## 9. Testing Strategy

### Unit tests

#### Frontend
- File list reducer or state logic
- Validation helpers
- Output filename generation
- Settings form behavior
- Progress state transitions

#### Backend
- File type detection
- Image-to-page conversion helpers
- PDF append logic
- Error mapping
- Temp file cleanup helpers

### Integration tests
- Merge multiple images into one PDF
- Merge multiple PDFs into one PDF
- Merge mixed images and PDFs into one PDF
- Reject unsupported file types
- Handle empty input correctly
- Handle write failure correctly
- Handle corrupted PDF correctly

### Test fixtures
- Small sample JPG
- Small sample PNG
- Multi-page sample PDF
- Corrupted PDF sample
- Unsupported sample file

### CI test gates
- Frontend unit tests must pass
- Rust unit tests must pass
- Integration tests must pass
- Lint and type checks should pass

## 10. Packaging and Distribution Plan

### macOS
- Primary:
  - `.app`
  - `.dmg`
- Optional:
  - zipped app bundle for simpler distribution

### Windows
- Primary:
  - installer package such as MSI or NSIS
- Optional:
  - portable executable/bundle if supported by packaging setup

### Uninstall expectations
- Installer-based build must:
  - register correctly in system uninstall list
  - remove app files cleanly
  - preserve or remove user data based on chosen policy

### Signing and trust
- Consider code signing for release builds
- Especially important for macOS Gatekeeper and Windows SmartScreen

## 11. Delivery Milestones

### Milestone 1: Project foundation
- Initialize Tauri + React + TypeScript project
- Set up styling system
- Set up linting, formatting, and test runners
- Define domain models and command interfaces

### Milestone 2: Core merge flow
- Add file import
- Show file list
- Reorder and remove items
- Implement backend merge pipeline
- Save merged PDF

### Milestone 3: UI polish and validation
- Semi-dark design refinement
- Progress and error UX
- Better previews and metadata display
- Edge case handling

### Milestone 4: Testing hardening
- Complete unit tests
- Add integration fixtures and scenarios
- Ensure all tests pass locally and in CI

### Milestone 5: Packaging and release
- macOS build artifacts
- Windows build artifacts
- Installer verification
- Uninstall verification
- Release notes and usage guide

## 12. Risks and Mitigations

### Risk: PDF library limitations
- Mitigation:
  - validate library choices with a prototype early
  - test mixed PDF/image merge before UI polish work

### Risk: Cross-platform file path issues
- Mitigation:
  - add tests using spaces and Unicode file names

### Risk: Windows file locking
- Mitigation:
  - keep backend file scope tight
  - explicitly close handles
  - test overwrite/retry scenarios

### Risk: Large file memory pressure
- Mitigation:
  - stream or chunk where library permits
  - start with practical limits and document them

## 13. Suggested First Implementation Order

1. Scaffold the Tauri app
2. Implement backend proof of concept for image/PDF merge
3. Add drag-and-drop and reorderable list UI
4. Connect frontend to backend commands
5. Add save dialog and merge execution flow
6. Add unit tests
7. Add integration tests with fixtures
8. Package for macOS and Windows

## 14. Definition of Done

The project is ready for initial release when:
- The app runs on macOS and Windows
- Users can merge images and PDFs into one PDF
- The GUI is polished and semi-dark
- Installer or portable distribution works
- Uninstall works correctly for installer builds
- Unit tests and integration tests are present and passing
- Basic user documentation exists

## 15. Recommended Next Step

Create the project skeleton with:
- Tauri
- React
- TypeScript
- Test setup
- Initial semi-dark shell UI

Then immediately validate the most important technical risk:
- Can the selected Rust PDF/image libraries reliably merge mixed image and PDF inputs into one output PDF?
