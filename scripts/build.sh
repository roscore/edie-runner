#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

## 1. Regenerate art + SFX (required for the embedded bundle)
if command -v python >/dev/null 2>&1; then
    echo "[1/4] Regenerating art assets..."
    python tools/generate_art.py
    echo "[1/4] Regenerating extras (store shops, ERICA gate, BGM)..."
    python tools/generate_extras.py
else
    echo "[1/4] SKIP: python not found; using existing assets/gen/"
fi

# 2. Build wasm (build.rs embeds every file from assets/gen/ into the binary)
echo "[2/4] Building wasm release..."
cargo build --release --target wasm32-unknown-unknown --bin edie_runner

# 3. Copy wasm into web/. No PNG/WAV files are shipped — they're all inside
#    the wasm binary in XOR-scrambled form.
echo "[3/4] Copying wasm to web/..."
mkdir -p web
cp target/wasm32-unknown-unknown/release/edie_runner.wasm web/edie_runner.wasm

# 4. Optional wasm-opt -Oz --strip-debug
if command -v wasm-opt >/dev/null 2>&1; then
    echo "[4/4] Optimizing + stripping wasm..."
    wasm-opt -Oz --strip-debug --strip-producers -o web/edie_runner.wasm web/edie_runner.wasm
else
    echo "[4/4] SKIP: wasm-opt not installed"
fi

# Sanity check: the web/ folder should NOT contain PNG/WAV
find web -maxdepth 1 \( -name '*.png' -o -name '*.wav' \) -delete 2>/dev/null || true

echo
echo "Build complete. Serve the game with:"
echo "  cd web && python -m http.server 8080"
echo "Then open http://localhost:8080 in Chrome."
