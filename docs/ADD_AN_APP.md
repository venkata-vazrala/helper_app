# Add an app to Helper

## As a user (Settings)

1. Open **Settings → Launchers**.
2. Under **Add from catalog**, click **Add** (or **Add anyway** if the app is not detected).
3. Or click **Add custom**: set name, command, and args.
4. Placeholders: `{path}` selected file/folder, `{dir}` folder (or parent of a file), `{name}` file name.
5. Mark **Default** for the accent Open-with button.
6. **Export JSON** backs up launchers with the rest of the workspace.

CLI apps that accept a path should put `--` before `{path}` so names like `-foo` are not treated as flags:

```
command = code
args    = -- {path}
```

macOS GUI apps:

```
command = open
args    = -a "App Name" {path}
```

## As a maintainer (catalog)

1. Append an `[[apps]]` table in `src/catalog.toml`.
2. Set `id`, `name`, `platforms` (`macos` / `linux` / `windows`), `command`, `args`, and `detect` (PATH names or `/Applications/...`).
3. Do not add a new `fn *_launcher()` in `config.rs`.
4. Run `cargo test` (catalog ids must stay unique).
5. Click **Add** in Settings on a machine that has the app.

Relevant apps to consider next: Zed, iTerm, Ghostty, Warp, Xcode, Sublime, GitHub Desktop, Fork, JetBrains IDEs (`idea`, `webstorm`, `pycharm`), Neovim in a terminal.
