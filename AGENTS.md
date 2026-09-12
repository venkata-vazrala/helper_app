# Helper App

Native Rust desktop workspace for pinned files/folders, notes, and copy-ready snippets.

## Stack

- Rust (edition 2024), binary crate `helper`
- GUI: `eframe` + `egui` (no web/JS frontend)
- Persistence: one `helper.json` under the OS app-data directory
- Open-with targets are user-configurable, per-OS defaults; VS Code is the default editor when present

## Commands

- Run: `cargo run --release`
- Test: `cargo test`
- Format: `cargo fmt`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Package macOS app: `./scripts/package.sh` (or `make app`)
- Optional install: `./scripts/install.sh` (or `make install`)

## Conventions

- Keep new features in Rust. Do not add Electron, Tauri webviews, or a separate frontend.
- Launchers use `{path}`, `{dir}`, and `{name}` placeholders.
- User data (pins, notes, snippets, launchers) lives outside the git tree.
- Prefer small modules: `config`, `store`, `launch`, then one UI module per tab.
- Track this slice in `docs/PLAN.md`, `docs/CHECKLIST.md`, and `docs/PROGRESS.md`.
- `cargo fmt` before finishing a change.
