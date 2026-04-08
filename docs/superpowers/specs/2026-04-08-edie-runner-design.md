# EDIE Runner — Design Spec

**Date:** 2026-04-08
**Status:** Draft v2 (post code-reviewer feedback)
**Owner:** helle

## 1. Overview

A browser-playable endless runner written in Rust, compiled to WebAssembly, themed around **EDIE**, the AeiROBOT robot. It is *inspired by* the Chrome offline T-Rex game but is **not a reskin**: it introduces a core resource-spending mechanic (Aurora Dash, §3.5), an AeiROBOT lab world, and bespoke art produced specifically for the game.

**Success criteria**

1. Loads in Chrome from a static `web/` folder in under 2 s on a typical laptop.
2. 60 FPS on integrated GPU at 1280×400 logical resolution.
3. Single keyboard control (Space/↑ jump, ↓ duck, **Shift dash**) plus touch (tap = jump, swipe-down = duck, on-screen Dash button bottom-right = dash). No multi-finger gestures (avoids pinch-zoom conflict).
4. Feels alive: parallax background, particle dust, screen shake on hit, score popups, dash trail.
5. Persistent high score across reloads.
6. Game pauses cleanly on tab blur, resumes on focus.

## 2. Tech stack

| Layer | Choice | Why |
|---|---|---|
| Language | Rust stable, pinned in `rust-toolchain.toml` (1.84) | Reproducible builds |
| Engine | **macroquad 0.4** | Tiny, 2D-first, single-binary WASM |
| Target | `wasm32-unknown-unknown` | Static files in Chrome |
| Persistence | `quad-storage` behind a `Storage` trait (§5) | Testable |
| Audio | macroquad audio (`.ogg`), gated on first user gesture | Chrome autoplay policy |
| Build | `cargo build --release` + `wasm-opt -Oz` | Bundle size |

## 3. Game design

### 3.1 Core loop

EDIE runs left→right at increasing speed across an AeiROBOT lab floor. The player jumps over or ducks under obstacles and collects **Aurora Stones**. Holding cells lets the player **Dash** — a short invulnerable burst that destroys obstacles in its path. Hitting an obstacle without dashing ends the run.

### 3.2 World theme & art direction

Background is the inside of an AeiROBOT dev lab — server racks (far layer), workbenches with oscilloscopes (mid), smooth floor (foreground). Distinct from "desert with cacti".

**Art direction: pixel art**, dictated by the EDIE reference art (`assets/source/edie_ref_*.png`):

- Visible pixel grid; nearest-neighbor scaling (`FilterMode::Nearest`)
- ~1 px black outline on the EDIE silhouette and matching obstacles
- White/off-white body with subtle 1-shade grey dithering for form
- Warm orange accent for the eye and select highlights
- Cute round chibi proportions, not utilitarian sci-fi
- Source frames authored at low resolution (EDIE ~64–96 px tall) and rendered with integer scale at runtime

The whole game (EDIE, obstacles, background, UI, pickups) is authored in this single pixel-art style so EDIE never looks like a guest from another game.

### 3.3 Player — EDIE

States: `Running`, `Jumping`, `Falling`, `Ducking`, `Dashing`, `Hit`. Transitions are a flat table in `player.rs`.

**Physics constants** (authoritative — all tests reference these):

| Constant | Value | Notes |
|---|---|---|
| Logical resolution | 1280 × 400 | All entity coords in this space |
| Ground Y | 320 | Top of floor in logical px |
| Gravity | 2400 px/s² | |
| Jump initial velocity | -780 px/s | |
| Jump hold extra velocity | -260 px/s, applied while held, max 120 ms | Variable jump height |
| Coyote time | 80 ms | After leaving ground |
| Duck hitbox shrink | 45% vertical | |
| Hitbox inset | 4 px each side | Forgiving |
| Dash duration | 280 ms | Invulnerable, +60% horizontal |
| Dash cost | 1 aurora | |
| Dash cooldown | 400 ms after end | |

Jump arc target: at base speed (320 px/s) the player clears a 96 px tall obstacle and lands in time to clear another 48 px obstacle 192 px later. Verified by unit test.

### 3.4 Obstacles (AeiROBOT-themed)

| Name | Layer | Height | Counter | Destroyable by Dash |
|---|---|---|---|---|
| Coiled Cable | ground low | 32 px | Jump | yes |
| Charging Dock | ground tall | 96 px | Jump | no (heavy) |
| Tool Cart | ground wide | 48×128 | Jump (long) | yes |
| Sensor Cone Field | ground group | 32 px ×3 | Jump | yes |
| Quadcopter Drone | air mid | hover at duck height | Duck | yes |
| Spark Burst | air high | drops on period | run-through | n/a (passes) |

Spawning is a **seeded** weighted random (§4.3). Minimum spacing between obstacles is computed at spawn time from current speed so a fair reaction window always exists.

### 3.5 Aurora Dash (the "not a reskin" mechanic)

- **Aurora Stones** are pickups: small glowing orbs (보라색/초록색) that pulse and emit a soft particle halo. Spawn ~1 every 8–12 s, sometimes mid-air to reward jumps. Two color variants alternate purely cosmetically (purple `#9D6BFF` and green `#5BE3A8`); both fill the same energy meter.
- HUD shows current charge count (0–3, capped).
- Pressing **Shift** spends 1 aurora and triggers Dash:
  - 280 ms invulnerability
  - +60% horizontal speed during dash
  - Destroyable obstacles in the path are smashed (score popup `+25` each)
  - Non-destroyable (Charging Dock) just lets you pass through harmlessly *if* you started the dash before contact, otherwise you die normally
- **Strategic depth**: do you spend a charge to power through a tight cluster, or save it for a higher-tier section? Dash on a Drone gives a brief slow-mo (200 ms at 0.6× time scale) for style. Slow-mo scales the `dt` passed into `world.update`, not the fixed-step accumulator: the loop still runs at real 1/120 s, but each step advances simulated time by `DT * 0.6`. Determinism preserved.
- Dash cannot be triggered with 0 batteries; UI plays a soft "denied" sound.

This is the design pivot that makes EDIE Runner not a dino reskin: there is now a **resource you collect, manage, and spend** that meaningfully changes moment-to-moment decisions.

### 3.6 Difficulty progression

- Base scroll: 320 px/s. +20 px/s per 500 score, capped at 720 px/s.
- Spawn weight table is keyed on `difficulty_tier = floor(score / 500).min(8)`. Tiers stored as a const `[[u8; N_OBSTACLES]; 9]` table in `difficulty.rs`.
- Spark Burst unlocks at tier 3.

### 3.7 Feedback / juice

- Dust particles on landing.
- 80 ms screen shake + red flash on collision (red flash adjustable to colorblind-safe yellow via a simple `--cb` URL query flag — cheap accessibility win).
- `+50` / `+25` score popups.
- Dash leaves a 6-frame motion trail.
- Speed-tier vignette darkens slightly per tier.

### 3.8 UI / states

- **Title**: "EDIE RUNNER", high score, "Press SPACE to start". First key press also unlocks audio (Chrome autoplay).
- **Playing**: score top-right, aurora icons top-left, speed-tier indicator.
- **Paused** (tab blur, or `P`): dim overlay, no simulation.
- **Game Over**: score, high score, "Press SPACE to retry". New high score highlighted.

## 4. Architecture

Single binary, modular by file. Target ~150–300 lines per file.

```
src/
├── main.rs              // macroquad entry, window cfg, state machine, fixed-step loop
├── time.rs              // Fixed timestep accumulator (§4.2)
├── game/
│   ├── mod.rs
│   ├── state.rs         // GameState: Title | Playing | Paused | GameOver
│   ├── world.rs         // World: player, obstacles, pickups, bg, particles, rng
│   ├── player.rs        // EDIE physics + state machine
│   ├── obstacles.rs     // Spawn table, update, collision shapes
│   ├── pickups.rs       // Aurora cells
│   ├── dash.rs          // Dash state, invulnerability, slow-mo
│   ├── background.rs    // Parallax layers
│   ├── particles.rs     // Dust, hit flash, dash trail, debris
│   ├── difficulty.rs    // Tier curve, spawn weights, const tables
│   └── score.rs         // Score, high-score persistence (uses Storage trait)
├── render/
│   ├── mod.rs
│   ├── camera.rs        // Logical→screen, screen shake, DPR
│   ├── sprites.rs       // Atlas + frame lookup, animation clock
│   └── ui.rs            // Title, HUD, pause, game over
├── platform/
│   ├── mod.rs
│   ├── storage.rs       // trait Storage { get/set } + QuadStorage impl + InMemoryStorage for tests
│   ├── input.rs         // trait InputSource + MacroquadInput + ScriptedInput for tests
│   └── visibility.rs    // tab blur/focus handling
├── assets.rs            // AssetHandles, async load_all() with progress
└── lib.rs               // Re-exports for unit tests
```

### 4.1 Boundaries

- `world` owns mutable game state; `render` only reads.
- `platform::*` are the **only** modules that touch macroquad globals; everything game-logic uses traits, so unit tests inject mocks.
- `assets` is the only module that knows file paths.

### 4.2 Update model — fixed timestep

The game uses a **fixed timestep** of 1/120 s with an accumulator pattern:

```
acc += clamp(get_frame_time(), 0.0, 0.1)  // clamp = no death-spiral on tab return
while acc >= DT:
    world.update(DT)
    acc -= DT
render(world)
// Note: render interpolation alpha is intentionally NOT threaded through in v1.
// We re-evaluate after Phase 1 if motion looks judder-y at 60 Hz display.
```

DT is a `const f32 = 1.0 / 120.0`. All physics constants in §3.3 are expressed in continuous units (px/s, px/s²) and consumed by `world.update(dt)`. This makes the simulation **deterministic for a given input + seed sequence** — required by §8 tests.

### 4.3 Randomness

`world` owns a `SmallRng` seeded from either the current time (production) or a fixed seed (tests). All random draws — spawn table, pickup placement, particle jitter — go through this single RNG. No `rand::thread_rng()` anywhere.

### 4.4 Async asset loading

macroquad's `load_texture` is `async`. Load order:

1. `main` enters an `async` `Loading` state on first frame.
2. `assets::load_all()` awaits every texture and audio file in parallel via `futures::join!`.
3. While loading, a **count-based** progress indicator is rendered (`loaded / total`). macroquad does not expose per-future progress hooks, so we resolve futures one at a time and increment a counter — slightly slower than full parallel `join!` but gives real progress. Total assets is small (~20), so the cost is negligible.
4. **Missing asset = hard fail with on-screen error message**, not panic. The game shows "Asset failed to load: <name>" and stops. This is debuggable in the browser.
5. Only after `load_all` completes does the state transition to `Title`.

### 4.5 Data flow per frame

```
InputSource.poll() -> Vec<Action>
  -> state.handle(actions, world)
  -> (fixed-step loop) world.update(DT)
  -> render.draw(world)
```

`Action` enum: `Jump`, `JumpRelease`, `Duck`, `DuckRelease`, `Dash`, `Confirm`, `Pause`.

## 5. Persistence

- Behind `trait Storage { fn get(&self, key:&str)->Option<String>; fn set(&mut self, key:&str, val:&str); }`.
- Production impl wraps `quad-storage`. Test impl is a `HashMap`.
- Stored value is JSON: `{"version":1,"high_score":1234}`. Unknown version → treat as 0, overwrite on next save. Future fields (best distance, settings) extend this object.
- Defensive parse: malformed value → 0, never panic.

## 6. Phased delivery (3 implementation plans)

The reviewer flagged that engine + full art set + juice is too much for one plan. We split:

### Phase 1 — Greybox (own implementation plan)

Goal: a **playable** game with placeholder art (colored rectangles) that proves all mechanics and architecture.

Includes:
- All of §4 (architecture, fixed timestep, traits, async loader)
- All physics constants and player states from §3.3
- All obstacle types as colored rectangles with correct hitboxes
- Aurora pickups as purple/green pulsing squares
- **Aurora Dash mechanic fully working**, including slow-mo and obstacle smashing
- Difficulty curve and seeded RNG
- Score, high score, persistence
- Pause on tab blur
- Title / Playing / Paused / GameOver states
- All unit tests in §8
- Build script and `web/index.html` shell that runs in Chrome

Excludes: real art, audio, particle polish, screen shake, dash trail, vignette.

**Done = the user can play it in Chrome and the game is mechanically complete and tuned.**

### Phase 2 — Art pass (own implementation plan)

Goal: replace every placeholder rectangle with bespoke art in a coherent style.

- Lock palette (§6.2 below)
- Author EDIE sprite sheets (run, jump, duck, dash, hit) using user-provided references as style anchor
- Author all obstacle sprites
- Author parallax background tiles
- Author aurora pickup
- Author UI font choice
- Wire each sheet into `assets.rs` and `sprites.rs`

Done = the game is visually finished. No new mechanics.

### Phase 3 — Juice & polish (own implementation plan)

Goal: make it feel great.

- Particle systems (dust, hit flash, dash trail, debris on smash)
- Screen shake
- Score popups
- Speed-tier vignette
- Slow-mo on dash-through-drone
- All SFX (jump, hit, pickup, dash, smash, deny)
- Audio gated on first user gesture
- Colorblind flag
- DPR / high-DPI scaling: canvas backing store sized at `min(devicePixelRatio, 2) * cssSize`; logical 1280×400 unchanged; camera scales output. Cap at 2× to bound bundle and fill rate on 4K screens.
- `wasm-opt -Oz` and final bundle-size pass

Done = ship-ready.

## 6.2 Asset plan (governs Phase 2)

### Palette (locked, sampled from EDIE references)

```
edie-outline      #1A1A1A   (1 px hard outline on EDIE and matched obstacles)
edie-white        #FFFFFF   (main body)
edie-shade        #D8D8D8   (1-step grey body shading)
edie-eye-dark     #1A1A1A   (eye outline)
edie-orange       #E8923C   (warm orange — eye fill, accents)
edie-orange-deep  #B86A1F   (eye shadow)

bg-sky            #F5EFE4   (warm cream — sky behind racks)
bg-far            #C9C2B2   (taupe silhouettes — far servers)
bg-mid            #8E8676   (warm grey — mid workbenches)
floor             #4A4438   (dark warm brown — lab floor)
floor-line        #2E2A22   (floor accent lines)

aurora-purple     #9D6BFF
aurora-purple-hi  #D3B8FF
aurora-green      #5BE3A8
aurora-green-hi   #B8F5DD
aurora-glow       #FFFFFF   (additive halo, low alpha)

hazard            #E63946   (collision flash, danger UI only)
ok                #2EC4B6   (high-score highlight, dash-ready glow)
```

The palette is warm and friendly, matching the EDIE cute pixel tone — not the cool blue sci-fi I had originally specced. The lab is rendered as a *cozy* workshop, not a sterile clean room.

### Conventions

- **Pixel grid.** Source art is authored at low resolution and rendered with `FilterMode::Nearest` at integer scale to keep pixels crisp. No bilinear filtering.
- **1 px black outline** on the EDIE silhouette and on obstacles that share the EDIE style; background and UI elements may omit outlines for visual hierarchy.
- **Asset sizes are author-tuned, not enforced.** I (the assistant) will pick frame sizes that look right and re-tune at the vertical-slice gate. Rough starting targets:
  - EDIE frames: ~80×80 source, rendered at ~3× = 240 px tall in-world
  - Small obstacles: ~32×32 source, rendered at 3×
  - Tool cart: ~64×32 source, rendered at 3×
  - Background tiles: 256 px wide source, tiled horizontally
- **Pivot**: bottom-center for ground entities, center for air entities.
- **Frame timing**: 12 fps for run, 8 fps for idle bobs (can be tuned).
- **Format**: PNG with straight alpha. **1 px transparent padding + 1 px edge extrude between frames** to prevent bleeding even with nearest-neighbor at non-integer camera offsets.
- **Sheet layout**: one sheet per entity, frames laid out horizontally.

### Authoring order (vertical slice first)

1. EDIE run cycle (6 frames). **Stop, drop into greybox build, evaluate.**
2. One ground obstacle (Coiled Cable).
3. One background floor tile.
4. *Visual gate*: does this triple look like one game? If no, redo before continuing.
5. EDIE jump (3 frames) → duck (2) → dash (3) → hit (1).
6. Remaining obstacles in order: dock, cart, cone, drone, spark.
7. Background mid + far layers.
8. Aurora Stone pickup — 6-frame pulse + soft halo, two color variants (purple, green).
9. UI font selection (free-licensed pixel font matching the EDIE pixel-art tone; license file committed alongside).

### Tooling

- Pixel editor of choice (Aseprite, Piskel, Krita with pixel brush, etc.); choice is per-asset, not enforced.
- If any frame is AI-generated, it must be hand-cleaned to match palette and pivot conventions before commit. Frames that don't match are redone.

### Provided references

User-supplied images saved to `assets/source/edie_ref_*.png` and **never shipped**, only used as style reference during authoring.

### Design coherence policy

Resolved on 2026-04-08 after inspecting the provided refs: the entire game is authored in the EDIE flat-vector cartoon style (§3.2, §6.2 palette and conventions). EDIE is the visual anchor; everything else conforms to it.

- EDIE run/jump/duck/dash/hit frames are derived from `assets/source/edie_ref_*.png`, keeping the silhouette, proportions, palette, and expression. Additional poses/frames are authored in the same style.
- Obstacles, background, pickups, UI all use the locked palette and the same "smooth shape, no outline, soft contrast" treatment.
- Asset sizing is the assistant's responsibility — I pick frame dimensions that look right in-game and re-tune at the vertical-slice gate. The user is not asked to specify pixel sizes.
- Vertical-slice gate (after EDIE run + first obstacle + first floor tile) is the explicit checkpoint where the user approves the look before the rest of the assets are produced.

The user has explicitly endorsed this approach: I (the assistant) author the actual game art; the user gives direction and approves at the gate.

## 7. Build & deploy

- `rust-toolchain.toml` pins Rust 1.84.
- `cargo build --release --target wasm32-unknown-unknown`
- `wasm-opt -Oz` post-process step (Phase 3).
- `web/index.html` vendors macroquad's JS shim (committed, not fetched at runtime).
- `scripts/build.ps1` and `scripts/build.sh` do build → copy `.wasm` → print `python -m http.server` instructions.
- No CI in v1; document local commands in README.

**Required local tooling** (README must list these):
- Rust 1.84 (auto-installed via `rust-toolchain.toml`)
- `rustup target add wasm32-unknown-unknown`
- `binaryen` (for `wasm-opt`, Phase 3 only) — install via `winget install WebAssembly.Binaryen` on Windows or `brew install binaryen` on macOS
- Python 3 (for `python -m http.server` local serving) **or** any other static file server

## 8. Testing

The trait seams in §4.1 make these executable, not aspirational.

**Unit tests** (`#[cfg(test)]`):

- `player::tests`
  - jump arc apex matches expected (within 1 px) given §3.3 constants
  - variable jump height: hold for 120 ms vs tap → measurable apex difference
  - duck shrinks hitbox by 45%
  - coyote time: jump 79 ms after leaving ground succeeds, 81 ms fails
- `obstacles::tests`
  - spawn table respects minimum spacing at every speed tier (use `ScriptedInput` + fixed seed, simulate 60 s, assert no overlaps)
  - destroyable flag matches §3.4 table
- `dash::tests`
  - costs exactly 1 aurora, fails at 0
  - invulnerable for exactly 280 ms simulated time
  - smashes destroyable obstacles, dies on Charging Dock contact mid-run-up
- `difficulty::tests`
  - speed curve monotonic and capped at 720
  - tier table indexable for score 0..10000 without panic
- `score::tests`
  - high-score round-trip via `InMemoryStorage`
  - malformed JSON → returns 0, no panic
  - schema version mismatch → returns 0
- `input::tests`
  - `ScriptedInput` events map to expected `Action`s
- `time::tests`
  - accumulator clamps frame time, no death-spiral
  - large frame time (tab return) drops frames cleanly

**Manual smoke (per phase)**:

- Phase 1: Title→Play→Die→Retry loop, dash works, high score persists, pause on tab blur.
- Phase 2: visual diff against the palette and pivot conventions; no placeholder rectangles remain.
- Phase 3: 60 FPS in Chrome on integrated GPU; all SFX play after first input; bundle ≤ 3 MB.

Headless rendering tests are out of scope.

## 9. Out of scope (v1)

Multiplayer, leaderboards, mobile-optimized layout beyond touch fallback, localization, achievements, music, in-game settings menu, level editor.

## 10. Risks

| Risk | Mitigation |
|---|---|
| Authoring full sprite set is the largest single piece of work | Phase 2 is its own plan; vertical-slice gate after first three assets |
| WASM bundle past ~3 MB | `wasm-opt -Oz` in Phase 3, audio kept short |
| `quad-storage` quirks across browsers | Storage trait + best-effort, never blocks gameplay |
| Difficulty tuning is subjective | Constants centralized in `difficulty.rs` |
| Chrome autoplay blocks audio | Audio init gated on first user gesture |
| Tab blur kills physics on return | Fixed-step accumulator with frame-time clamp + Pause state on `visibilitychange` |
| AI-generated frames break visual coherence | Mandatory hand-clean + palette gate before commit |
| Font licensing | Use a permissively-licensed pixel font, commit `LICENSE` next to it |
| Determinism flakes in obstacle tests | Single seeded RNG in `world`; no thread_rng anywhere |

## 11. Open questions

(none — all prior open questions resolved by user decisions on 2026-04-08:
A=aurora-dash mechanic chosen, scope split into 3 phases approved.)
