# Helper

A small native **Rust** desktop app for a personal workspace. One window, four tabs:

| Tab | What it is |
| --- | --- |
| **Files** | Pinned files and folders, with configurable **Open with** (VS Code, Grok, Finder, …) |
| **Notes** | A local collection of notes |
| **Snippets** | Copy-ready text (clipboard) |
| **Settings** | Launchers and data location |

This is a development-first slice. More tabs and features can land later without changing the layout.

## Run

```bash
cargo run
```

Requires a working Rust toolchain. On macOS, `code` / `cursor` / `grok` must be on `PATH` for those launchers to work. You can add or change launchers in **Settings**.

## Data

User data is **not** in this repo. It lives in the OS app-data directory:

- macOS: `~/Library/Application Support/helper-app/`
  - `config.toml` — launchers
  - `store.json` — pins, notes, snippets

See `config.example.toml` for the launcher format. Placeholders:

- `{path}` — the selected file or folder
- `{dir}` — the folder itself, or the parent of a file
- `{name}` — the file or folder name

## Grok

This repo is set up for Grok Build: `AGENTS.md`, `.grok/lsp.json` (rust-analyzer), and a git history. There is no `grok init` CLI command; those files are the project init.
