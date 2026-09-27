#!/usr/bin/env bash
# Build a double-clickable macOS app. Rust is needed to *build*, not to *run*.
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "package.sh builds a macOS .app and only runs on Darwin." >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> cargo build --release"
cargo build --release

DIST="$ROOT/dist"
APP="$DIST/Helper.app"
MACOS="$APP/Contents/MacOS"
RES="$APP/Contents/Resources"

rm -rf "$APP"
mkdir -p "$MACOS" "$RES"

BIN="$ROOT/target/release/helper"
if [[ ! -x "$BIN" ]]; then
  echo "missing release binary: $BIN" >&2
  exit 1
fi

cp "$BIN" "$MACOS/Helper"
chmod +x "$MACOS/Helper"
cp "$ROOT/packaging/macos/Info.plist" "$APP/Contents/Info.plist"

ICON_SRC="$ROOT/assets/icon.png"
if [[ -f "$ICON_SRC" ]] && command -v sips >/dev/null && command -v iconutil >/dev/null; then
  ICONSET="$DIST/Helper.iconset"
  rm -rf "$ICONSET"
  mkdir -p "$ICONSET"
  sips -z 16 16     "$ICON_SRC" --out "$ICONSET/icon_16x16.png" >/dev/null
  sips -z 32 32     "$ICON_SRC" --out "$ICONSET/icon_16x16@2x.png" >/dev/null
  sips -z 32 32     "$ICON_SRC" --out "$ICONSET/icon_32x32.png" >/dev/null
  sips -z 64 64     "$ICON_SRC" --out "$ICONSET/icon_32x32@2x.png" >/dev/null
  sips -z 128 128   "$ICON_SRC" --out "$ICONSET/icon_128x128.png" >/dev/null
  sips -z 256 256   "$ICON_SRC" --out "$ICONSET/icon_128x128@2x.png" >/dev/null
  sips -z 256 256   "$ICON_SRC" --out "$ICONSET/icon_256x256.png" >/dev/null
  sips -z 512 512   "$ICON_SRC" --out "$ICONSET/icon_256x256@2x.png" >/dev/null
  sips -z 512 512   "$ICON_SRC" --out "$ICONSET/icon_512x512.png" >/dev/null
  sips -z 1024 1024 "$ICON_SRC" --out "$ICONSET/icon_512x512@2x.png" >/dev/null
  iconutil -c icns "$ICONSET" -o "$RES/AppIcon.icns"
  rm -rf "$ICONSET"
fi

ZIP="$DIST/Helper.zip"
rm -f "$ZIP"
ditto -c -k --keepParent "$APP" "$ZIP"

echo
echo "Ready to run (unsigned):"
echo "  open \"$APP\""
echo "Product zip:"
echo "  \"$ZIP\""
ls -lh "$APP" "$ZIP" "$MACOS/Helper"
echo "Optional install into /Applications:"
echo "  ./scripts/install.sh"
echo "Optional signing (needs APPLE_IDENTITY):"
echo "  ./scripts/sign.sh"
