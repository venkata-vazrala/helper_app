#!/usr/bin/env bash
# Copy the packaged app into /Applications. Run scripts/package.sh first.
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "install.sh copies Helper.app into /Applications and only runs on macOS." >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/dist/Helper.app"

if [[ ! -d "$APP" ]]; then
  echo "No app bundle at $APP — run ./scripts/package.sh first." >&2
  exit 1
fi

DEST="/Applications/Helper.app"
rm -rf "$DEST"
cp -R "$APP" "$DEST"
echo "Installed $DEST"
echo "Open with: open -a Helper"
