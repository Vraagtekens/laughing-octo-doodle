#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SKETCH="${1:-}"

if [[ -z "$SKETCH" ]]; then
  echo "Usage: scripts/serve-sketch.sh <sketch-name> [trunk args...]" >&2
  exit 1
fi

shift || true
"$ROOT/scripts/write-sketch-page.sh" "$SKETCH"

ID="$(jq -r --arg id "$SKETCH" '.[] | select(.id == $id or .crate == $id) | .id' "$ROOT/public/manifest.json")"
cd "$ROOT"
env -u NO_COLOR trunk serve "$ROOT/.trunk/$ID/index.html" --open "$@"
