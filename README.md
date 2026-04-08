# EDIE Runner

Endless runner starring **EDIE**, the AeiROBOT mascot. Runs in Chrome via WebAssembly.

**Play in your browser:** <https://roscore.github.io/edie-runner/>

EDIE has been left behind in a Pangyo pop-up store and must find the long way back to AeiROBOT HQ, dashing through coffee cups, traffic, balloon drones, and AeiROBOT bots along the way. Survive all the way to 50000 points and a very unwelcome guest shows up for the final boss fight.

## Story

1. **Pangyo Department Store** — luxury pop-up floor, marble, gold watches
2. **Pangyo Street** — sidewalks, tech offices, early bots
3. **Highway to Ansan** — cars charging, deer leaping, balloons drifting
4. **Hanyang ERICA** — campus stretch, AeiROBOT bots begin
5. **AeiROBOT Office** — home is in sight, but virus warnings flash everywhere
6. **AeiROBOT CEO Room** — dense robot swarm, extreme difficulty
7. **Corona Boss Fight** (50000+) — survive 60s of falling virus rain + laser attacks

## Controls

| Key | Action |
|---|---|
| **Space / ↑** | Jump (hold for higher arc) |
| **↓** | Duck under drones |
| **Shift** | Aurora Dash (costs 1 aurora, smashes everything) |
| **Left / Right** or **A / D** | Dodge (boss fight only) |
| **P** | Pause |
| **H** | Help screen |
| **T** | Story intro (Star Wars style) |
| **Esc** | Back |

## Mechanics

- Collect **Aurora Stones** (purple/green orbs) to fuel the dash
- Collect **Hearts** for extra lives (max 3)
- Dash grants 400 ms of invulnerability and smashes any obstacle
- Cross tier thresholds to unlock new obstacle pools and zones
- Scroll speed climbs smoothly from 280 to 640+ px/s

## Required tooling

- Rust 1.84 (auto-installed via `rust-toolchain.toml`)
- `rustup target add wasm32-unknown-unknown`
- Python 3 + Pillow + numpy (for art/audio regeneration)
- Optional: `wasm-opt` (Binaryen) for `-Oz` optimization

## Build locally

```bash
# Linux / macOS / Git Bash
./scripts/build.sh
cd web && python -m http.server 8080
```

```powershell
# Windows PowerShell
./scripts/build.ps1
cd web; python -m http.server 8080
```

Then open <http://localhost:8080> in Chrome.

`scripts/build.sh` runs the Python art generator, builds the wasm release, copies artifacts into `web/`, and (if available) runs `wasm-opt -Oz`.

## Deployment (GitHub Pages)

The game auto-deploys to GitHub Pages on every push to `main` via
`.github/workflows/deploy.yml`. The workflow:

1. Installs Python + Rust + wasm32 target
2. Runs `python tools/generate_art.py`
3. Builds the wasm release
4. Copies assets into `web/`
5. Optimizes with `wasm-opt -Oz`
6. Publishes `web/` to GitHub Pages

To enable Pages for a fresh fork:

1. Settings → Pages → Source: "GitHub Actions"
2. Push to `main`
3. Wait for the workflow to complete
4. Open `https://<user>.github.io/<repo>/`

## Unit tests

```bash
cargo test --lib
```

60+ host-side tests cover physics, obstacles, dash, score, difficulty, camera,
day/night cycle, and boss mode. All game logic is host-testable through the
trait seams in `src/platform/`.

## Project structure

```
edie-runner/
|- src/
|  |- main.rs            # macroquad entry, main loop
|  |- lib.rs             # re-exports for tests
|  |- assets.rs          # async texture/sound loader
|  |- time.rs            # fixed-timestep accumulator
|  |- platform/          # Storage/Input/Visibility trait seams
|  |- game/              # pure game logic (no macroquad deps)
|  |  |- player.rs, obstacles.rs, pickups.rs
|  |  |- dash.rs, effects.rs, boss.rs
|  |  |- world.rs, state.rs, difficulty.rs
|  `- render/            # camera, sprites, UI, day/night
|- tools/generate_art.py # procedural art + SFX generator
|- assets/
|  |- source/            # user-provided EDIE reference art (GIFs)
|  `- gen/               # generator output (PNGs + WAVs)
|- web/                  # WASM host + static files served by GitHub Pages
|- docs/superpowers/     # design spec + implementation plans
`- .github/workflows/    # CI deploy
```
