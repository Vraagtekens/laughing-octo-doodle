#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/public/manifest.json"
TEMPLATE="$ROOT/web/sketch.html"
SKETCH_ID="${1:-}"

if [[ -z "$SKETCH_ID" ]]; then
  echo "Usage: scripts/write-sketch-page.sh <sketch-name>" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to read $MANIFEST" >&2
  exit 1
fi

SKETCH="$(jq -c --arg id "$SKETCH_ID" '.[] | select(.id == $id or .crate == $id)' "$MANIFEST")"
if [[ -z "$SKETCH" ]]; then
  echo "Unknown sketch in public/manifest.json: $SKETCH_ID" >&2
  exit 1
fi

ID="$(jq -r '.id' <<<"$SKETCH")"
NAME="$(jq -r '.name | @html' <<<"$SKETCH")"
SUMMARY="$(jq -r '.summary | @html' <<<"$SKETCH")"
ASSET_DIR="$(jq -r '.assetDir // empty' <<<"$SKETCH")"
ENTRY="$ROOT/.trunk/$ID/index.html"
CARGO_MANIFEST="../../sketches/$ID/Cargo.toml"
ASSET_LINKS=""

if [[ ! -d "$ROOT/sketches/$ID" ]]; then
  echo "Missing sketch directory: sketches/$ID" >&2
  exit 1
fi

if [[ ! -f "$TEMPLATE" ]]; then
  echo "Missing sketch HTML template: web/sketch.html" >&2
  exit 1
fi

mkdir -p "$(dirname "$ENTRY")"

if [[ -n "$ASSET_DIR" ]]; then
  if [[ "$ASSET_DIR" == "assets" ]]; then
    ASSET_LINKS="$(printf '    <link data-trunk rel="copy-dir" href="../../sketches/%s/assets" />' "$ID")"
  else
    ASSET_LINKS="$(printf '    <link data-trunk rel="copy-dir" href="%s" />' "$ASSET_DIR")"
  fi
fi

awk \
  -v title="$NAME" \
  -v summary="$SUMMARY" \
  -v asset_links="$ASSET_LINKS" \
  -v cargo_manifest="$CARGO_MANIFEST" \
  '{
    gsub(/__TITLE__/, title);
    gsub(/__SUMMARY__/, summary);
    gsub(/__ASSET_LINKS__/, asset_links);
    gsub(/__CARGO_MANIFEST__/, cargo_manifest);
    print;
  }' "$TEMPLATE" >"$ENTRY"
