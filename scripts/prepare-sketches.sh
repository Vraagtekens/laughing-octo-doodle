#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/public/manifest.json"

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to read $MANIFEST" >&2
  exit 1
fi

jq -r '.[].id' "$MANIFEST" | while IFS= read -r sketch; do
  "$ROOT/scripts/write-sketch-page.sh" "$sketch"
done
