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

Write-Host "[3/4] Copying wasm to web\..."
New-Item -ItemType Directory -Force -Path web | Out-Null
Copy-Item target\wasm32-unknown-unknown\release\edie_runner.wasm web\edie_runner.wasm -Force

Write-Host "[4/4] Optimizing + stripping wasm..."
if (Get-Command wasm-opt -ErrorAction SilentlyContinue) {
    wasm-opt -Oz --strip-debug --strip-producers -o web\edie_runner.wasm web\edie_runner.wasm
} else {
    Write-Host "  SKIP: wasm-opt not installed"
}

# Remove any leftover PNG / WAV files from web/
Get-ChildItem -Path web -File -Include *.png, *.wav -ErrorAction SilentlyContinue | Remove-Item -Force

Write-Host ""
Write-Host "Build complete. Serve the game with:"
Write-Host "  cd web; python -m http.server 8080"
Write-Host "Then open http://localhost:8080 in Chrome."
