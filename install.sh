#!/bin/bash
# Bocah Timer — free, no-Gatekeeper-warning installer.
#
# Downloading via the terminal (not a browser) means the app is never
# quarantined, so macOS does NOT show the "cannot verify developer / malware"
# prompt. We also strip any quarantine flag just in case.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/pradityaaldi/bocah-timer/main/install.sh | bash

set -euo pipefail

REPO="pradityaaldi/bocah-timer"
ASSET="Bocah.Timer_universal.app.tar.gz"
URL="https://github.com/$REPO/releases/latest/download/$ASSET"
DEST="/Applications"

echo "==> Downloading Bocah Timer…"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
curl -fsSL "$URL" -o "$TMP/app.tar.gz"

echo "==> Extracting…"
tar -xzf "$TMP/app.tar.gz" -C "$TMP"

APP_PATH="$(/usr/bin/find "$TMP" -maxdepth 1 -name '*.app' -print -quit)"
if [ -z "$APP_PATH" ]; then
  echo "Error: no .app found in the release archive." >&2
  exit 1
fi
APP_NAME="$(basename "$APP_PATH")"

echo "==> Installing to $DEST/$APP_NAME…"
rm -rf "${DEST:?}/$APP_NAME"
mv "$APP_PATH" "$DEST/"

# strip quarantine so Gatekeeper stays quiet even if it was set
xattr -dr com.apple.quarantine "$DEST/$APP_NAME" 2>/dev/null || true

echo "==> Done. Launching Bocah Timer 🎉"
open "$DEST/$APP_NAME"
