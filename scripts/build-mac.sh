#!/usr/bin/env bash
#
# Build a local, signed macro_paste.app for development/testing on macOS.
#
# Signs with the self-signed "macro_paste-dev" identity so the Accessibility
# (TCC) permission survives rebuilds — macOS keys the permission on the
# signature's Designated Requirement (stable cert + bundle id), not on the
# binary hash. Grant Accessibility once; subsequent rebuilds keep working.
#
# Create the identity once (see README / one-time setup) if it is missing.

set -euo pipefail

cd "$(dirname "$0")/.."

IDENTITY="macro_paste-dev"
APP="target/release/macro_paste.app"

if ! security find-identity -p codesigning | grep -q "$IDENTITY"; then
    echo "error: code-signing identity '$IDENTITY' not found in keychain." >&2
    echo "       Run the one-time identity setup first (see scripts/setup-signing.sh)." >&2
    exit 1
fi

echo "==> Building release binary"
cargo build --release

echo "==> Assembling app bundle"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/macro_paste "$APP/Contents/MacOS/macro_paste"
cp assets/Info.plist "$APP/Contents/Info.plist"
cp assets/macro_paste.icns "$APP/Contents/Resources/macro_paste.icns"

echo "==> Signing with '$IDENTITY'"
codesign --force --deep --sign "$IDENTITY" "$APP"
codesign --verify --verbose=2 "$APP"

echo "==> Done: $APP"
echo "    Launch with: open $APP"
