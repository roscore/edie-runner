$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
cargo build --release --target wasm32-unknown-unknown --bin edie_runner
Copy-Item target\wasm32-unknown-unknown\release\edie_runner.wasm web\edie_runner.wasm -Force
Write-Host ""
Write-Host "Build complete. Serve the game with:"
Write-Host "  cd web; python -m http.server 8080"
Write-Host "Then open http://localhost:8080 in Chrome."
