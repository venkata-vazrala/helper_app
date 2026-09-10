# Helper App

Native Rust desktop workspace for pinned files/folders, notes, and copy-ready snippets.

## Stack

- Rust (edition 2024), binary crate `helper`
- GUI: `eframe` + `egui` (no web/JS frontend)
- Persistence: TOML config + JSON store under the OS app-data directory
- Open-with targets are user-configurable launchers, never a hardcoded editor

## Commands

- Run: `cargo run`
- Test: `cargo test`
- Format: `cargo fmt`
- Lint: `cargo clippy --all-targets -- -D warnings`

## Conventions

- Keep new features in Rust. Do not add Electron, Tauri webviews, or a separate frontend.
- Launchers use `{path}`, `{dir}`, and `{name}` placeholders.
- User data (pins, notes, snippets, launchers) lives outside the git tree.
- Prefer small modules: `config`, `store`, `launch`, then one UI module per tab.
- `cargo fmt` before finishing a change.
