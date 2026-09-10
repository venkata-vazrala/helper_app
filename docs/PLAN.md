# Helper — next slice

Approved plan. Implementation is tracked in [CHECKLIST.md](CHECKLIST.md) and [PROGRESS.md](PROGRESS.md).

## Goals

1. Reorder pins, notes, snippets, and launchers (move up / down; `⌥↑` / `⌥↓`).
2. Settings: load / reload / import / export workspace data.
3. Tabs: name is the label; shortcut is a small superscript in the top-right of the chip.
4. Recents: unpin must not lose the item; re-pin from Recents (cap 50).
5. Appearance: light / dark / system, accent presets, density, font scale.
6. Tests for domain logic, without weakening atomic saves or UI-frame cost.

## Defaults

- No drag-and-drop this slice.
- Import **replaces** after confirm (parse first, then atomic write).
- Recents cap **50**, newest first.
- Note/snippet save debounce **400ms**, flush on tab switch and window close.
- Library crate so tests do not open a window.

## Out of scope

- Drag-and-drop reorder, cloud sync, arbitrary hex color picker, changing the app-data root.
