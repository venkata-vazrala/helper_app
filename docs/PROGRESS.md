# Progress log

## Already shipped (before this slice)

- README, git, Grok files, Cargo crate
- Config (TOML launchers), JSON store, `{path}` / `{dir}` / `{name}` open-with
- Files / Notes / Snippets / Settings tabs
- Cards, chrome, accent, app icon
- `scripts/package.sh` → `dist/Helper.app`

## This slice (2026-09-10)

| Item | Status |
| --- | --- |
| Local PLAN / CHECKLIST / PROGRESS | done |
| Library crate + baseline tests | done |
| Reorder (`move_up` / `move_down`, `⌥↑`/`⌥↓`) | done |
| Recents (unpin → recents, pin again, cap 50) | done |
| Settings Data (reload / import / export, parse-before-write) | done |
| Tab chips (name + superscript shortcut) | done |
| Appearance (dark/light/system, accent, density, scale) | done |
| Debounced save (400ms, flush on tab/close) | done |
| Verify (`cargo test` 24 passed, clippy `-D warnings`, fmt) | done |
| Snippets editor: always-on scrollbars + list preview | done |

## How to check it

```bash
cargo test
cargo run --release
```

Settings → Appearance and Data. Files → Unpin, then Recents → Pin again. Tab labels show `⌘1` in the top-right of each chip.
