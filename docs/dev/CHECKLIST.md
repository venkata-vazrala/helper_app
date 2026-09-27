# Implementation checklist

Tick a box when that step is in the tree and covered by tests (where the plan requires them).

## 1. Library crate and baseline tests

- [x] `src/lib.rs` exports app modules; `main.rs` is a thin window entry
- [x] `tempfile` dev-dependency
- [x] Existing store/config/launch tests still pass as lib tests

## 2. Reorder

- [x] `move_up` / `move_down` helpers with bound tests
- [x] Pins: up / down on the selected row
- [x] Notes: up / down
- [x] Snippets: up / down
- [x] Launchers: up / down
- [x] `⌥↑` / `⌥↓` when a list row is selected

## 3. Recents

- [x] `RecentItem` on `Store` (`#[serde(default)]`)
- [x] Unpin moves to recents; cap 50; dedupe by path
- [x] Restore re-pins and drops the recent row
- [x] Files tab Recents section: Pin again, Clear
- [x] Tests: unpin, restore, duplicate path, cap eviction

## 4. Settings — Data

- [x] Reload from disk
- [x] Export workspace JSON
- [x] Export settings TOML
- [x] Import workspace (confirm replace; parse-before-write)
- [x] Import settings (confirm replace)
- [x] Invalid import does not clobber existing files (test)

## 5. Tabs

- [x] Chip: name at body size
- [x] Shortcut as small superscript in the top-right corner
- [x] Selected state uses accent bar, not concatenated `Files  ⌘1` text

## 6. Appearance

- [x] Dark / Light / System
- [x] Accent presets: teal, blue, amber, rose
- [x] Density: comfortable / compact
- [x] Font scale 0.9–1.2, clamped
- [x] Persist `[appearance]` in `config.toml`
- [x] Apply style only when appearance (or system theme) changes
- [x] Palette contrast tests for dark and light

## 7. Persistence performance

- [x] Debounce note/snippet saves (400ms)
- [x] Flush on tab switch and close-requested

## 8. Verify

- [x] `cargo test` (24 lib tests)
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo fmt`
