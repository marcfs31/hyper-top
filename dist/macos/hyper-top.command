#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ -x "$SCRIPT_DIR/hyper-top" ]]; then
  exec "$SCRIPT_DIR/hyper-top" "$@"
fi

if command -v hyper-top >/dev/null 2>&1; then
  exec hyper-top "$@"
fi

echo "hyper-top binary not found next to this launcher or on PATH." >&2
echo "Install it with: cargo install --path . --locked" >&2
exit 1
