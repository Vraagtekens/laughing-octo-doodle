#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SKETCH="${1:-}"

if [[ -z "$SKETCH" ]]; then
  echo "Usage: scripts/build-sketch.sh <sketch-name>" >&2
  exit 1
fi

if ! command -v trunk >/dev/null 2>&1; then
  echo "Trunk is not installed. Install it with: cargo install trunk --version 0.20.3 --locked" >&2
  exit 1
fi

"$ROOT/scripts/write-sketch-page.sh" "$SKETCH"

ID="$(jq -r --arg id "$SKETCH" '.[] | select(.id == $id or .crate == $id) | .id' "$ROOT/public/manifest.json")"
ENTRY="$ROOT/.trunk/$ID/index.html"
DIST="$ROOT/dist/$ID"
RUSTFLAGS_VALUE="${RUSTFLAGS-}"
RUSTFLAGS_VALUE="${RUSTFLAGS_VALUE:+$RUSTFLAGS_VALUE }--cfg getrandom_backend=\"wasm_js\""

rm -rf "$DIST"
mkdir -p "$(dirname "$DIST")"

cd "$ROOT"
env -u NO_COLOR RUSTFLAGS="$RUSTFLAGS_VALUE" trunk build "$ENTRY" --release --dist "$DIST" --public-url ./

perl -0pi -e "s/await init\((['\"][^'\"]+_bg\.wasm['\"])\)/await init({ module_or_path: \$1 })/g" "$DIST/index.html"
