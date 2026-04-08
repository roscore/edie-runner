#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# 1. Regenerate art assets (requires python + Pillow + numpy)
if command -v python >/dev/null 2>&1; then
    echo "[1/4] Regenerating art assets..."
    python tools/generate_art.py
else
    echo "[1/4] SKIP: python not found; using existing assets/gen/"
fi

# 2. Build wasm
echo "[2/4] Building wasm release..."
cargo build --release --target wasm32-unknown-unknown --bin edie_runner

# 3. Copy assets into web/
echo "[3/4] Copying assets to web/..."
mkdir -p web
cp target/wasm32-unknown-unknown/release/edie_runner.wasm web/edie_runner.wasm
cp assets/gen/*.png web/
cp assets/gen/*.wav web/ 2>/dev/null || true

# 4. Optional wasm-opt
if command -v wasm-opt >/dev/null 2>&1; then
    echo "[4/4] Optimizing wasm with wasm-opt..."
    wasm-opt -Oz -o web/edie_runner.wasm web/edie_runner.wasm
else
    echo "[4/4] SKIP: wasm-opt not installed"
fi

echo
echo "Build complete. Serve the game with:"
echo "  cd web && python -m http.server 8080"
echo "Then open http://localhost:8080 in Chrome."
