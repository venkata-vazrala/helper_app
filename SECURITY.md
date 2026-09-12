# Security

Helper is a **local** desktop app. It does not phone home, and it does not open network sockets of its own.

## Trust model

- **Workspace data** (`helper.json`) is stored only in the OS application-data directory.
- **Import** parses JSON in memory first. Invalid files never overwrite the live store. Imports larger than 8 MiB are rejected.
- **Open with** runs the command you configured, with `{path}` / `{dir}` / `{name}` substituted. Commands are spawned as argv (not a shell). A launcher is therefore equivalent to running that program yourself. Do not import a `helper.json` from someone you do not trust: it can point launchers at arbitrary executables.

## Reporting

Please open a GitHub issue for vulnerabilities in this repository. Do not include secrets or personal workspace dumps.
