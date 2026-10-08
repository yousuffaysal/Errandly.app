#!/bin/sh
# Signs the bundled AI runtime with your Developer ID and the hardened runtime,
# so Apple's notarization accepts the app. Does nothing without an identity
# (local development and unsigned builds keep working).
#   APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
set -eu
cd "$(dirname "$0")/../src-tauri/runtime"
if [ -z "${APPLE_SIGNING_IDENTITY:-}" ]; then
  echo "APPLE_SIGNING_IDENTITY not set; leaving the runtime unsigned (fine for development)"
  exit 0
fi
for bin in ollama llama-server; do
  codesign --force --options runtime --timestamp --sign "$APPLE_SIGNING_IDENTITY" "$bin"
  codesign --verify --strict "$bin"
done
echo "runtime signed"
