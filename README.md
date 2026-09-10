# Helper

A small native **Rust** desktop app for a personal workspace. One window, four tabs:

| Tab | What it is |
| --- | --- |
| **Files** | Pinned files and folders, with configurable **Open with** (VS Code, Grok, Finder, …) |
| **Notes** | A local collection of notes |
| **Snippets** | Copy-ready text (clipboard) |
| **Settings** | Launchers and data location |

This is a development-first slice. More tabs and features can land later without changing the layout.

Unpinning a file or folder moves it to **Recents** (not gone). Reorder lists with ▴ ▾ or `⌥↑` / `⌥↓`. Settings covers appearance (light / dark / system) and import/export of workspace data. Plan and task tracking live in `docs/`.

## Run (from source)

```bash
cargo run --release
```

On macOS, `code` / `cursor` / `grok` must be on `PATH` for those launchers to work. You can add or change launchers in **Settings**.

## Ready-to-run app (no install)

Rust is only needed to **build**. After that you can run Helper without Cargo:

```bash
./scripts/package.sh
open dist/Helper.app
```

That writes:

- `dist/Helper.app` — double-click in Finder
- `dist/helper` — the same binary, for the terminal
- `dist/Open Helper.command` — double-click helper next to the app

Optional install into `/Applications`:

```bash
./scripts/install.sh
open -a Helper
```

`make app` and `make install` do the same.

## Data

User data is **not** in this repo. It lives in the OS app-data directory:

- macOS: `~/Library/Application Support/helper-app/helper.json`
  - one JSON file for workspace (pins, notes, snippets, recents) **and** settings (launchers, appearance)

Older `store.json` / `config.toml` files are migrated into `helper.json` on first launch.

See `config.example.toml` for the launcher format. Placeholders:

- `{path}` — the selected file or folder
- `{dir}` — the folder itself, or the parent of a file
- `{name}` — the file or folder name

## Grok

This repo is set up for Grok Build: `AGENTS.md`, `.grok/lsp.json` (rust-analyzer), and a git history. There is no `grok init` CLI command; those files are the project init.
