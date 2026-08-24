#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${1:-5500}"

if [[ ! -f "$ROOT/dist/index.html" ]]; then
  echo "Missing dist/index.html. Run ./scripts/build-all.sh first." >&2
  exit 1
fi

cd "$ROOT/dist"
python3 -m http.server "$PORT"
