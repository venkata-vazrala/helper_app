# Helper

A native **Rust** desktop workspace: pinned files and folders, notes, copy-ready snippets, and configurable **Open with** apps. Data stays on your machine.

| Tab | What it is |
| --- | --- |
| **Files** | Pin files and folders; open them in VS Code, Finder, Antigravity IDE, or any launcher you add |
| **Notes** | Local notes |
| **Snippets** | Copy-ready text |
| **Settings** | Launchers, appearance, import/export |

Unpinning a path moves it to **Recents**. Reorder with the chevron buttons or `Alt+↑` / `Alt+↓` (`Option` on macOS).

## Supported devices

| Platform | Status | How to run |
| --- | --- | --- |
| **macOS** 11+ (Intel and Apple Silicon) | Primary | `cargo run --release` or `./scripts/package.sh` → `dist/Helper.app` |
| **Linux** (X11 / Wayland) | Builds and runs from source | `cargo run --release` (needs GTK 3 headers to compile) |
| **Windows** 10+ | Builds and runs from source | `cargo run --release` |

Default Open-with entries are **per OS** (Finder/`open` on macOS, `xdg-open` on Linux, Explorer on Windows). VS Code is the default editor on all three if `code` is on `PATH`. You can add or change launchers in Settings.

There is no iOS, Android, or web build.

## Requirements

- [Rust](https://rustup.rs/) **stable** (see `rust-toolchain.toml`)
- Linux packages for a source build: `libgtk-3-dev` plus the usual xcb / xkb libraries

## Run from source

```bash
cargo test --locked
cargo run --release
```

## macOS app bundle (no Cargo needed to run)

```bash
./scripts/package.sh
open dist/Helper.app
```

Optional copy into `/Applications`:

```bash
./scripts/install.sh
```

`make app` and `make install` wrap the same scripts. Packaging is Darwin-only.

## Data

Helper does **not** store workspace data in this git repo. One JSON file holds pins, notes, snippets, recents, launchers, and appearance:

| OS | Path |
| --- | --- |
| macOS | `~/Library/Application Support/helper-app/helper.json` |
| Linux | `~/.local/share/helper-app/helper.json` |
| Windows | `%APPDATA%\helper-app\helper.json` |

Older `store.json` / `config.toml` files in that folder are migrated on first launch.

Launcher placeholders: `{path}`, `{dir}`, `{name}`. See `config.example.toml`.

## Security

Helper is local-only. **Open with** runs programs you configure (or import). Do not import a `helper.json` from an untrusted source. See [SECURITY.md](SECURITY.md).

## License

MIT. See [LICENSE](LICENSE). Copyright is held by Helper contributors — this repository contains no personal contact details.
