#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown --bin edie_runner
cp target/wasm32-unknown-unknown/release/edie_runner.wasm web/edie_runner.wasm
echo
echo "Build complete. Serve the game with:"
echo "  cd web && python -m http.server 8080"
echo "Then open http://localhost:8080 in Chrome."
