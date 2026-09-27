#!/usr/bin/env bash
# Codesign Helper.app when APPLE_IDENTITY is set. No-op otherwise.
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "sign.sh only runs on macOS." >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/dist/Helper.app"

if [[ ! -d "$APP" ]]; then
  echo "Build first: ./scripts/package.sh" >&2
  exit 1
fi

if [[ -z "${APPLE_IDENTITY:-}" ]]; then
  echo "APPLE_IDENTITY is not set. Skipping codesign."
  echo "Example: export APPLE_IDENTITY='Developer ID Application: Example Inc (TEAMID)'"
  exit 0
fi

codesign --force --deep --options runtime --sign "$APPLE_IDENTITY" "$APP"
echo "Signed $APP"
echo "Notarize next with notarytool (not automated here)."
