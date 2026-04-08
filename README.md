# EDIE Runner

Endless runner starring **EDIE**, the ADrive robot. Runs in Chrome via WebAssembly.

## Phase 1 (Greybox)

Mechanically complete game using colored rectangles for art. Phase 2 adds bespoke pixel art; Phase 3 adds juice and audio.

## Required tooling

- Rust 1.84 (auto-installed via `rust-toolchain.toml`)
- `rustup target add wasm32-unknown-unknown`
- Python 3 (for local serving) or any other static file server

## Build & run

Linux / macOS / Git Bash:

```bash
./scripts/build.sh
cd web && python -m http.server 8080
```

Windows PowerShell:

```powershell
./scripts/build.ps1
cd web; python -m http.server 8080
```

Then open <http://localhost:8080> in Chrome.

## Controls

- **Space / ↑** — jump (hold for higher jump)
- **↓** — duck
- **Shift** — Aurora Dash (costs 1 aurora stone)
- **P** — pause
- **Space** — confirm on Title / Game Over

## Run unit tests

```bash
cargo test --lib
```

All game logic is host-testable through trait seams in `src/platform/`.
