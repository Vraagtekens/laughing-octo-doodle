#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="$ROOT/dist"
MANIFEST="$ROOT/public/manifest.json"

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to read $MANIFEST" >&2
  exit 1
fi

rm -rf "$DIST"
mkdir -p "$DIST"
cp "$ROOT/web/index.html" "$DIST/index.html"
cp "$MANIFEST" "$DIST/manifest.json"

jq -r '.[].id' "$MANIFEST" | while IFS= read -r sketch; do
  "$ROOT/scripts/build-sketch.sh" "$sketch"
done
