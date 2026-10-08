#!/bin/sh
# Downloads the local AI runtime (Ollama) that Errandly bundles, so users don't
# need Homebrew. Pinned to an exact release and verified by SHA-256.
# Output: src-tauri/runtime/ (git-ignored; bundled as an app resource). Apple Silicon only.
set -eu

VERSION="v0.40.1"
SHA256="66e1587711f3a06315b23782ba74897001da6c8b8edf6c0371f7533015a076dd"
URL="https://github.com/ollama/ollama/releases/download/${VERSION}/ollama-darwin.tgz"

cd "$(dirname "$0")/../src-tauri"
if [ -x runtime/ollama ] && [ "$(cat runtime/VERSION 2>/dev/null)" = "$VERSION" ]; then
  echo "runtime $VERSION already present"
  exit 0
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
echo "downloading Ollama $VERSION…"
# Resume interrupted downloads instead of starting over.
for attempt in 1 2 3 4 5 6; do
  if curl -fsSL --retry 3 --retry-all-errors -C - -o "$tmp/ollama.tgz" "$URL"; then break; fi
  [ "$attempt" = 6 ] && { echo "download failed" >&2; exit 1; }
  echo "connection dropped; resuming (attempt $((attempt + 1)))…"
  sleep 3
done
echo "$SHA256  $tmp/ollama.tgz" | shasum -a 256 -c -

mkdir -p "$tmp/full"
tar -xzf "$tmp/ollama.tgz" -C "$tmp/full"

# Keep only what Apple Silicon needs (41 MB instead of 520 MB): the arm64 half
# of the server and its llama.cpp runner (Metal is embedded), plus licences.
# The other libraries are Intel-only, and the MLX engine isn't used for the
# GGUF models Errandly runs.
rm -rf runtime
mkdir -p runtime
lipo "$tmp/full/ollama" -thin arm64 -output runtime/ollama
lipo "$tmp/full/llama-server" -thin arm64 -output runtime/llama-server
cp "$tmp/full/"*LICENSE* runtime/ 2>/dev/null || true
echo "$VERSION" > runtime/VERSION
echo "runtime ready in src-tauri/runtime"
