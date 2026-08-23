#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
APP="$ROOT/src-tauri/target/release/bundle/macos/Desktop Migam Mac.app"
DMG="$ROOT/src-tauri/target/release/bundle/dmg/Desktop Migam Mac_0.1.0_aarch64.dmg"
REQUIREMENT='=designated => identifier "com.migam.desktop.mac"'

if [ ! -d "$APP" ]; then
  echo "macOS app bundle not found: $APP" >&2
  exit 1
fi

codesign --force --deep --sign - \
  --identifier com.migam.desktop.mac \
  --requirements "$REQUIREMENT" \
  "$APP"
codesign --verify --deep --strict --verbose=2 "$APP"

STAGING=$(mktemp -d "${TMPDIR:-/tmp}/desktop-migam-dmg.XXXXXX")
trap 'rm -rf "$STAGING"' EXIT
ditto "$APP" "$STAGING/Desktop Migam Mac.app"
ln -s /Applications "$STAGING/Applications"
rm -f "$DMG"
hdiutil create -quiet -volname "Desktop Migam Mac" -srcfolder "$STAGING" -ov -format UDZO "$DMG"

echo "Signed app and rebuilt DMG with stable local designated requirement."
