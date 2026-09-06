#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

cargo test --locked
cargo build --release

if command -v docker >/dev/null 2>&1; then
  docker build -f docker/Dockerfile.verify -t hyper-top-verify .
  docker run --rm hyper-top-verify
else
  ./target/release/hyper-top --version
fi
