# Contributing

## Prerequisites

- A recent **stable** Rust toolchain (`rust-toolchain.toml` pins `stable` plus `rustfmt` and `clippy`)
- On Linux, GTK 3 headers (`libgtk-3-dev` and related xcb packages) to build `rfd` / `eframe`

## Checks (run before a PR)

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --locked
```

CI runs the same on macOS, Ubuntu, and Windows.

For GitHub, set `user.email` to your GitHub noreply address so commits do not publish a personal inbox.

## Scope

- Keep the app native Rust (no webview / Electron).
- User data stays outside git (`helper.json` in the OS app-data directory).
- Do not commit personal names, emails, machine paths, or local `helper.json`.
- Default launchers are per-OS; do not hardcode one person's editor as the only option.

## Packaging

macOS app bundle: `./scripts/package.sh` (Darwin only).
