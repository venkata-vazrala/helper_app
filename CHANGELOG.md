# Changelog

## Unreleased

- H-plus brand mark: Dock / window icon (`assets/icon.png`), README logo (`assets/logo.png`)
- MSRV recorded as 1.95 (eframe). Lockfile refreshed. CI audits crates with RustSec and reads the repo only.

## 0.2.0

- App catalog in Settings (Add / Add anyway) plus Add custom
- Product zip `dist/Helper.zip` (app only); unsigned sign stub
- Import confirm lists launcher commands
- Large folders capped at 400 items; clearer open-with errors
- No silent rewrite of `helper.json` on every launch
- About card with version and data path

## 0.1.0

- Files, notes, snippets, and settings in one native window
- Configurable Open-with launchers (VS Code default)
- Recents for unpinned paths, reorder, light/dark/system appearance
- Single `helper.json` workspace + settings file
- macOS `.app` packaging scripts
