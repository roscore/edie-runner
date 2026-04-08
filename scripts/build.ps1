$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

Write-Host "[1/4] Regenerating art assets..."
if (Get-Command python -ErrorAction SilentlyContinue) {
    python tools\generate_art.py
} else {
    Write-Host "  SKIP: python not found"
}

Write-Host "[2/4] Building wasm release..."
cargo build --release --target wasm32-unknown-unknown --bin edie_runner

Write-Host "[3/4] Copying assets to web\..."
New-Item -ItemType Directory -Force -Path web | Out-Null
Copy-Item target\wasm32-unknown-unknown\release\edie_runner.wasm web\edie_runner.wasm -Force
Copy-Item assets\gen\*.png web\ -Force
Copy-Item assets\gen\*.wav web\ -Force -ErrorAction SilentlyContinue

Write-Host "[4/4] Optimizing wasm..."
if (Get-Command wasm-opt -ErrorAction SilentlyContinue) {
    wasm-opt -Oz -o web\edie_runner.wasm web\edie_runner.wasm
} else {
    Write-Host "  SKIP: wasm-opt not installed"
}

Write-Host ""
Write-Host "Build complete. Serve the game with:"
Write-Host "  cd web; python -m http.server 8080"
Write-Host "Then open http://localhost:8080 in Chrome."
