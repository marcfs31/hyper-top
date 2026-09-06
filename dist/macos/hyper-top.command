#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$ROOT_DIR"

if command -v hyper-top >/dev/null 2>&1; then
  exec hyper-top "$@"
else
  exec cargo run -- "$@"
fi
