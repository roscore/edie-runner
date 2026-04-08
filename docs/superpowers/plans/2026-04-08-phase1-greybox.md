# EDIE Runner — Phase 1 (Greybox) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A mechanically-complete, playable EDIE Runner in Chrome using colored rectangles for all art. Every Phase 1 item from `docs/superpowers/specs/2026-04-08-edie-runner-design.md` §6 must be present and tested. Phase 2 (art) and Phase 3 (juice) are out of scope.

**Architecture:** Rust → `wasm32-unknown-unknown` using macroquad 0.4. Single binary, modules under `src/` per spec §4. Trait seams (`Storage`, `InputSource`) in `src/platform/` enable unit testing of all game logic without macroquad globals. Fixed timestep accumulator at 1/120 s drives a deterministic update loop seeded by a single `SmallRng`.

**Tech Stack:** Rust 1.84 (pinned), macroquad 0.4, quad-storage, rand 0.8 (`SmallRng`), serde + serde_json (persistence), futures (asset loader), Chrome as the only target browser.

**Spec reference:** All section numbers (§N.M) refer to `docs/superpowers/specs/2026-04-08-edie-runner-design.md`.

**Testing convention:** Pure logic modules use `#[cfg(test)] mod tests` and run via `cargo test` on the host (NOT wasm). macroquad-touching modules are *only* `src/main.rs`, `src/platform/visibility.rs`, the macroquad impls in `src/platform/storage.rs` and `src/platform/input.rs`, and `src/render/*`. Everything else is host-testable.

**Working directory:** All paths in this plan are relative to `C:\Users\helle\jun_ws\edie-runner\`.

---

## Task 0: Project scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `.gitignore`
- Create: `src/main.rs` (placeholder)
- Create: `src/lib.rs` (placeholder)

- [ ] **Step 0.1: Initialize git repo**

```bash
cd /c/Users/helle/jun_ws/edie-runner
git init
git config core.autocrlf false
```

- [ ] **Step 0.2: Create rust-toolchain.toml**

Create `rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.84.0"
targets = ["wasm32-unknown-unknown"]
profile = "minimal"
components = ["rustfmt", "clippy"]
```

- [ ] **Step 0.3: Create Cargo.toml**

Create `Cargo.toml`:

```toml
[package]
name = "edie_runner"
version = "0.1.0"
edition = "2021"
authors = ["helle"]
description = "Endless runner starring EDIE the ADrive robot"

[lib]
name = "edie_runner"
path = "src/lib.rs"

[[bin]]
name = "edie_runner"
path = "src/main.rs"

[dependencies]
macroquad = "0.4"
quad-storage = "0.1"
rand = { version = "0.8", default-features = false, features = ["small_rng"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
# Host-side test deps (none extra needed yet)

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

- [ ] **Step 0.4: Create .gitignore**

Create `.gitignore`:

```
/target
**/*.rs.bk
Cargo.lock.bak
.DS_Store
Thumbs.db
web/edie_runner.wasm
```

(Note: we ignore the built `.wasm` in `web/` because it's a build artifact. Source `web/index.html` IS committed.)

- [ ] **Step 0.5: Create placeholder src/lib.rs and src/main.rs**

Create `src/lib.rs`:

```rust
//! EDIE Runner library entry point (re-exports for unit tests).
pub mod game;
pub mod platform;
pub mod time;
```

Create `src/main.rs`:

```rust
//! EDIE Runner binary entry point.
//! Phase 1 placeholder — replaced in Task 21.

fn main() {
    println!("EDIE Runner — placeholder");
}
```

(`game`, `platform`, `time` modules will be created in subsequent tasks. Compilation will fail until Task 1 — that's expected.)

- [ ] **Step 0.6: Commit scaffold**

```bash
git add Cargo.toml rust-toolchain.toml .gitignore src/lib.rs src/main.rs
git commit -m "chore: project scaffold (Cargo, toolchain, placeholders)"
```

---

## Task 1: time::FixedStep accumulator

**Files:**
- Create: `src/time.rs`

Implements spec §4.2: fixed timestep at 1/120 s, frame-time clamped at 0.1 s to prevent death-spiral on tab return.

- [ ] **Step 1.1: Write the failing test**

Create `src/time.rs`:

```rust
//! Fixed-timestep accumulator. See spec §4.2.

pub const DT: f32 = 1.0 / 120.0;
const MAX_FRAME: f32 = 0.1;

#[derive(Debug, Default)]
pub struct FixedStep {
    accumulator: f32,
}

impl FixedStep {
    pub fn new() -> Self {
        Self { accumulator: 0.0 }
    }

    /// Feed a real-time delta and return how many fixed steps to run this frame.
    pub fn advance(&mut self, frame_time: f32) -> u32 {
        let clamped = frame_time.clamp(0.0, MAX_FRAME);
        self.accumulator += clamped;
        let mut steps = 0;
        while self.accumulator >= DT {
            self.accumulator -= DT;
            steps += 1;
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_frame_time_yields_zero_steps() {
        let mut s = FixedStep::new();
        assert_eq!(s.advance(0.0), 0);
    }

    #[test]
    fn one_dt_yields_one_step() {
        let mut s = FixedStep::new();
        assert_eq!(s.advance(DT), 1);
    }

    #[test]
    fn small_frames_accumulate_then_fire() {
        let mut s = FixedStep::new();
        // half a DT — no step yet
        assert_eq!(s.advance(DT / 2.0), 0);
        // another half — should fire one step
        assert_eq!(s.advance(DT / 2.0), 1);
    }

    #[test]
    fn large_frame_clamped_no_death_spiral() {
        let mut s = FixedStep::new();
        // 5-second frame would naively be 600 steps; clamp caps it
        let steps = s.advance(5.0);
        assert!(steps <= (MAX_FRAME / DT).ceil() as u32 + 1);
    }
}
```

- [ ] **Step 1.2: Run the tests, verify pass**

```bash
cargo test --lib time::tests
```

Expected: 4 passed.

- [ ] **Step 1.3: Commit**

```bash
git add src/time.rs
git commit -m "feat(time): fixed-timestep accumulator with frame-time clamp"
```

---

## Task 2: platform::storage trait + InMemoryStorage

**Files:**
- Create: `src/platform/mod.rs`
- Create: `src/platform/storage.rs`

Spec §5: storage behind a trait, JSON `{"version":1,"high_score":N}`, malformed → 0.

- [ ] **Step 2.1: Create platform module**

Create `src/platform/mod.rs`:

```rust
//! Platform abstractions (storage, input, visibility) behind traits
//! so game logic is unit-testable. See spec §4.1.
pub mod storage;
```

- [ ] **Step 2.2: Write the failing test + trait + InMemoryStorage**

Create `src/platform/storage.rs`:

```rust
//! Persistent key-value storage behind a trait. See spec §5.

use std::collections::HashMap;

pub trait Storage {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str);
}

#[derive(Default)]
pub struct InMemoryStorage {
    map: HashMap<String, String>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for InMemoryStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.map.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: &str) {
        self.map.insert(key.to_string(), value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut s = InMemoryStorage::new();
        assert_eq!(s.get("k"), None);
        s.set("k", "v");
        assert_eq!(s.get("k"), Some("v".to_string()));
    }

    #[test]
    fn overwrite() {
        let mut s = InMemoryStorage::new();
        s.set("k", "a");
        s.set("k", "b");
        assert_eq!(s.get("k"), Some("b".to_string()));
    }
}
```

- [ ] **Step 2.3: Wire module into lib.rs**

`src/lib.rs` already references `pub mod platform;` from Task 0. No edit needed.

- [ ] **Step 2.4: Run tests**

```bash
cargo test --lib platform::storage::tests
```

Expected: 2 passed.

- [ ] **Step 2.5: Commit**

```bash
git add src/platform/mod.rs src/platform/storage.rs
git commit -m "feat(platform): Storage trait + InMemoryStorage for tests"
```

---

## Task 3: platform::input::Action + InputSource trait + ScriptedInput

**Files:**
- Create: `src/platform/input.rs`
- Modify: `src/platform/mod.rs`

Spec §4.5: `Action` enum, trait + scripted impl for tests. Macroquad impl is added in Task 21 to keep this module host-testable.

- [ ] **Step 3.1: Add module declaration**

Edit `src/platform/mod.rs` to add:

```rust
//! Platform abstractions (storage, input, visibility) behind traits
//! so game logic is unit-testable. See spec §4.1.
pub mod storage;
pub mod input;
```

- [ ] **Step 3.2: Write the failing test + types**

Create `src/platform/input.rs`:

```rust
//! Input abstraction. See spec §4.5.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Jump,
    JumpRelease,
    Duck,
    DuckRelease,
    Dash,
    Confirm,
    Pause,
}

pub trait InputSource {
    /// Drain pending actions for this frame.
    fn poll(&mut self) -> Vec<Action>;
}

/// Test-only input source: feed a list of (frame_index, Action) pairs and the
/// scripted input returns the actions whose frame_index matches the current
/// internal frame counter, then advances the counter.
#[derive(Default)]
pub struct ScriptedInput {
    script: Vec<(u32, Action)>,
    frame: u32,
}

impl ScriptedInput {
    pub fn new(script: Vec<(u32, Action)>) -> Self {
        Self { script, frame: 0 }
    }
}

impl InputSource for ScriptedInput {
    fn poll(&mut self) -> Vec<Action> {
        let now = self.frame;
        self.frame += 1;
        self.script
            .iter()
            .filter(|(f, _)| *f == now)
            .map(|(_, a)| *a)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_script_yields_nothing() {
        let mut s = ScriptedInput::new(vec![]);
        assert!(s.poll().is_empty());
        assert!(s.poll().is_empty());
    }

    #[test]
    fn fires_at_correct_frame() {
        let mut s = ScriptedInput::new(vec![(0, Action::Jump), (2, Action::Dash)]);
        assert_eq!(s.poll(), vec![Action::Jump]);
        assert!(s.poll().is_empty());
        assert_eq!(s.poll(), vec![Action::Dash]);
    }

    #[test]
    fn multiple_actions_same_frame() {
        let mut s = ScriptedInput::new(vec![(0, Action::Jump), (0, Action::Dash)]);
        let actions = s.poll();
        assert!(actions.contains(&Action::Jump));
        assert!(actions.contains(&Action::Dash));
    }
}
```

- [ ] **Step 3.3: Run tests**

```bash
cargo test --lib platform::input::tests
```

Expected: 3 passed.

- [ ] **Step 3.4: Commit**

```bash
git add src/platform/mod.rs src/platform/input.rs
git commit -m "feat(platform): Action enum + InputSource trait + ScriptedInput"
```

---

## Task 4: game::difficulty (tier curve + spawn weights)

**Files:**
- Create: `src/game/mod.rs`
- Create: `src/game/difficulty.rs`

Spec §3.6: base scroll 320 px/s, +20 per 500 score, capped 720. Tier table.

- [ ] **Step 4.1: Create game module**

Create `src/game/mod.rs`:

```rust
//! Pure game logic. No macroquad dependencies in this module tree
//! except behind trait seams.
pub mod difficulty;
```

- [ ] **Step 4.2: Write the failing test + implementation**

Create `src/game/difficulty.rs`:

```rust
//! Difficulty tier curve. See spec §3.6.

pub const BASE_SPEED: f32 = 320.0;
pub const SPEED_STEP: f32 = 20.0;
pub const SPEED_CAP: f32 = 720.0;
pub const SCORE_PER_TIER: u32 = 500;
pub const MAX_TIER: u32 = 8;
pub const SPARK_BURST_UNLOCK_TIER: u32 = 3;

pub fn tier_for_score(score: u32) -> u32 {
    (score / SCORE_PER_TIER).min(MAX_TIER)
}

pub fn speed_for_score(score: u32) -> f32 {
    let tier = tier_for_score(score);
    let raw = BASE_SPEED + SPEED_STEP * tier as f32;
    raw.min(SPEED_CAP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_speed_at_zero() {
        assert_eq!(speed_for_score(0), BASE_SPEED);
    }

    #[test]
    fn monotonic() {
        let mut prev = speed_for_score(0);
        for s in (0..10000).step_by(100) {
            let cur = speed_for_score(s);
            assert!(cur >= prev, "speed went down at {s}: {prev} -> {cur}");
            prev = cur;
        }
    }

    #[test]
    fn capped() {
        for s in (0..50000).step_by(500) {
            assert!(speed_for_score(s) <= SPEED_CAP);
        }
    }

    #[test]
    fn tier_indexable_no_panic() {
        for s in (0..50000).step_by(123) {
            let _ = tier_for_score(s);
        }
    }

    #[test]
    fn tier_caps_at_max() {
        assert_eq!(tier_for_score(999999), MAX_TIER);
    }
}
```

- [ ] **Step 4.3: Run tests**

```bash
cargo test --lib game::difficulty::tests
```

Expected: 5 passed.

- [ ] **Step 4.4: Commit**

```bash
git add src/game/mod.rs src/game/difficulty.rs
git commit -m "feat(difficulty): tier curve, speed cap, monotonic test"
```

---

## Task 5: game::score (score + persistent high score)

**Files:**
- Create: `src/game/score.rs`
- Modify: `src/game/mod.rs`

Spec §5: JSON `{"version":1,"high_score":N}`, malformed → 0, version mismatch → 0.

- [ ] **Step 5.1: Add module declaration**

Edit `src/game/mod.rs`:

```rust
//! Pure game logic. No macroquad dependencies in this module tree
//! except behind trait seams.
pub mod difficulty;
pub mod score;
```

- [ ] **Step 5.2: Write the failing test + implementation**

Create `src/game/score.rs`:

```rust
//! Score and persistent high score. See spec §5.

use crate::platform::storage::Storage;
use serde::{Deserialize, Serialize};

pub const STORAGE_KEY: &str = "edie_runner.high_score";
const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct StoredScore {
    version: u32,
    high_score: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Score {
    pub current: u32,
    pub high: u32,
}

impl Score {
    pub fn load<S: Storage>(storage: &S) -> Self {
        let high = storage
            .get(STORAGE_KEY)
            .and_then(|s| serde_json::from_str::<StoredScore>(&s).ok())
            .filter(|s| s.version == SCHEMA_VERSION)
            .map(|s| s.high_score)
            .unwrap_or(0);
        Self { current: 0, high }
    }

    pub fn save_if_new_high<S: Storage>(&self, storage: &mut S) -> bool {
        if self.current > self.high {
            let stored = StoredScore {
                version: SCHEMA_VERSION,
                high_score: self.current,
            };
            let json = serde_json::to_string(&stored).expect("serializable");
            storage.set(STORAGE_KEY, &json);
            true
        } else {
            false
        }
    }

    pub fn add(&mut self, points: u32) {
        self.current = self.current.saturating_add(points);
        if self.current > self.high {
            self.high = self.current;
        }
    }

    pub fn reset(&mut self) {
        self.current = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::storage::InMemoryStorage;

    #[test]
    fn fresh_load_is_zero() {
        let s = InMemoryStorage::new();
        let score = Score::load(&s);
        assert_eq!(score.high, 0);
        assert_eq!(score.current, 0);
    }

    #[test]
    fn round_trip_high_score() {
        let mut storage = InMemoryStorage::new();
        let mut score = Score::load(&storage);
        score.add(1234);
        assert!(score.save_if_new_high(&mut storage));

        let reloaded = Score::load(&storage);
        assert_eq!(reloaded.high, 1234);
    }

    #[test]
    fn save_only_if_new_high() {
        let mut storage = InMemoryStorage::new();
        let mut score = Score::load(&storage);
        score.add(100);
        assert!(score.save_if_new_high(&mut storage));

        let mut later = Score::load(&storage);
        later.add(50);
        assert!(!later.save_if_new_high(&mut storage));
        assert_eq!(Score::load(&storage).high, 100);
    }

    #[test]
    fn malformed_json_returns_zero() {
        let mut storage = InMemoryStorage::new();
        storage.set(STORAGE_KEY, "this is not json");
        assert_eq!(Score::load(&storage).high, 0);
    }

    #[test]
    fn wrong_version_returns_zero() {
        let mut storage = InMemoryStorage::new();
        storage.set(STORAGE_KEY, r#"{"version":99,"high_score":500}"#);
        assert_eq!(Score::load(&storage).high, 0);
    }

    #[test]
    fn add_updates_high_in_memory() {
        let s = InMemoryStorage::new();
        let mut score = Score::load(&s);
        score.add(50);
        assert_eq!(score.high, 50);
        score.add(20);
        assert_eq!(score.high, 70);
        score.reset();
        assert_eq!(score.current, 0);
        assert_eq!(score.high, 70);
    }
}
```

- [ ] **Step 5.3: Run tests**

```bash
cargo test --lib game::score::tests
```

Expected: 6 passed.

- [ ] **Step 5.4: Commit**

```bash
git add src/game/score.rs src/game/mod.rs
git commit -m "feat(score): score + versioned persistent high score"
```

---

## Task 6: game::player physics + state machine

**Files:**
- Create: `src/game/player.rs`
- Modify: `src/game/mod.rs`

Spec §3.3 physics constants table is the source of truth. Tests verify jump apex within 1 px and variable jump height.

- [ ] **Step 6.1: Add module declaration**

Edit `src/game/mod.rs`:

```rust
pub mod difficulty;
pub mod score;
pub mod player;
```

- [ ] **Step 6.2: Write the failing tests + implementation**

Create `src/game/player.rs`:

```rust
//! EDIE player: physics, state machine, hitbox. See spec §3.3.

use crate::time::DT;

// Physics constants (spec §3.3)
pub const GROUND_Y: f32 = 320.0;
pub const GRAVITY: f32 = 2400.0;
pub const JUMP_INITIAL_VY: f32 = -780.0;
pub const JUMP_HOLD_EXTRA_VY: f32 = -260.0;
pub const JUMP_HOLD_MAX_TIME: f32 = 0.120;
pub const COYOTE_TIME: f32 = 0.080;
pub const DUCK_HITBOX_SHRINK: f32 = 0.45;
pub const HITBOX_INSET: f32 = 4.0;

pub const PLAYER_W: f32 = 64.0;
pub const PLAYER_H: f32 = 80.0;
pub const PLAYER_X: f32 = 200.0; // fixed horizontal position

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Running,
    Jumping,
    Falling,
    Ducking,
    Hit,
}

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Aabb {
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }
}

#[derive(Debug)]
pub struct Player {
    pub y: f32,             // top of bounding box
    pub vy: f32,
    pub state: PlayerState,
    pub jump_hold_time: f32,
    pub time_since_grounded: f32,
    pub jump_held: bool,
    pub duck_held: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            y: GROUND_Y - PLAYER_H,
            vy: 0.0,
            state: PlayerState::Running,
            jump_hold_time: 0.0,
            time_since_grounded: 0.0,
            jump_held: false,
            duck_held: false,
        }
    }
}

impl Player {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_grounded(&self) -> bool {
        matches!(self.state, PlayerState::Running | PlayerState::Ducking)
    }

    /// Try to start a jump. Returns true if jump initiated.
    pub fn try_jump(&mut self) -> bool {
        if self.is_grounded() || self.time_since_grounded <= COYOTE_TIME {
            self.vy = JUMP_INITIAL_VY;
            self.state = PlayerState::Jumping;
            self.jump_hold_time = 0.0;
            self.jump_held = true;
            self.time_since_grounded = COYOTE_TIME + 1.0; // consumed
            true
        } else {
            false
        }
    }

    pub fn release_jump(&mut self) {
        self.jump_held = false;
    }

    pub fn try_duck(&mut self) {
        if self.is_grounded() {
            self.state = PlayerState::Ducking;
        }
        self.duck_held = true;
    }

    pub fn release_duck(&mut self) {
        self.duck_held = false;
        if matches!(self.state, PlayerState::Ducking) {
            self.state = PlayerState::Running;
        }
    }

    pub fn hit(&mut self) {
        self.state = PlayerState::Hit;
        self.vy = 0.0;
    }

    pub fn update(&mut self, dt: f32) {
        if matches!(self.state, PlayerState::Hit) {
            return;
        }

        // Variable jump height: while held within window, apply extra upward velocity
        if matches!(self.state, PlayerState::Jumping) && self.jump_held {
            if self.jump_hold_time < JUMP_HOLD_MAX_TIME {
                self.vy += JUMP_HOLD_EXTRA_VY * dt;
                self.jump_hold_time += dt;
            }
        }

        // Gravity (always when airborne)
        if !self.is_grounded() {
            self.vy += GRAVITY * dt;
            self.y += self.vy * dt;
            self.time_since_grounded += dt;
        }

        // Ground clamp
        if self.y + PLAYER_H >= GROUND_Y {
            self.y = GROUND_Y - PLAYER_H;
            self.vy = 0.0;
            self.time_since_grounded = 0.0;
            self.state = if self.duck_held {
                PlayerState::Ducking
            } else {
                PlayerState::Running
            };
        } else if matches!(self.state, PlayerState::Jumping) && self.vy >= 0.0 {
            self.state = PlayerState::Falling;
        }
    }

    pub fn hitbox(&self) -> Aabb {
        let mut h = PLAYER_H;
        let mut y = self.y;
        if matches!(self.state, PlayerState::Ducking) {
            let shrink = PLAYER_H * DUCK_HITBOX_SHRINK;
            h -= shrink;
            y += shrink;
        }
        Aabb {
            x: PLAYER_X + HITBOX_INSET,
            y: y + HITBOX_INSET,
            w: PLAYER_W - 2.0 * HITBOX_INSET,
            h: h - 2.0 * HITBOX_INSET,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(p: &mut Player, seconds: f32) {
        let n = (seconds / DT).round() as u32;
        for _ in 0..n {
            p.update(DT);
        }
    }

    #[test]
    fn starts_grounded_running() {
        let p = Player::new();
        assert!(p.is_grounded());
        assert_eq!(p.state, PlayerState::Running);
    }

    #[test]
    fn jump_apex_reaches_expected_height() {
        let mut p = Player::new();
        let start_y = p.y;
        assert!(p.try_jump());
        // tap jump (release immediately) — find apex
        p.release_jump();
        let mut min_y = p.y;
        for _ in 0..200 {
            p.update(DT);
            if p.y < min_y {
                min_y = p.y;
            }
        }
        let apex_height = start_y - min_y;
        // Tap-jump apex with -780 initial vy and 2400 gravity:
        // h = vy^2 / (2g) = 780^2 / 4800 = 126.75 px
        assert!(
            (apex_height - 126.75).abs() < 2.0,
            "apex {apex_height} px, expected ~126.75"
        );
    }

    #[test]
    fn variable_jump_height_held_higher_than_tapped() {
        // Tapped
        let mut tap = Player::new();
        let tap_start = tap.y;
        tap.try_jump();
        tap.release_jump();
        let mut tap_min = tap.y;
        for _ in 0..200 {
            tap.update(DT);
            tap_min = tap_min.min(tap.y);
        }
        let tap_h = tap_start - tap_min;

        // Held for full 120 ms
        let mut held = Player::new();
        let held_start = held.y;
        held.try_jump();
        // hold for 120 ms, then release
        let hold_steps = (JUMP_HOLD_MAX_TIME / DT).ceil() as u32;
        for _ in 0..hold_steps {
            held.update(DT);
        }
        held.release_jump();
        let mut held_min = held.y;
        for _ in 0..200 {
            held.update(DT);
            held_min = held_min.min(held.y);
        }
        let held_h = held_start - held_min;

        assert!(
            held_h > tap_h + 5.0,
            "held jump ({held_h}) should be meaningfully higher than tap ({tap_h})"
        );
    }

    #[test]
    fn duck_shrinks_hitbox_by_45_percent() {
        let mut p = Player::new();
        let normal = p.hitbox();
        p.try_duck();
        let ducked = p.hitbox();
        // Shrink applies to the inner (insets removed) box. The pre-inset
        // height is PLAYER_H = 80, shrink = 36. After insets (4 each side):
        // normal inner h = 72, ducked inner h = 80 - 36 - 8 = 36.
        let ratio = ducked.h / normal.h;
        assert!(
            (ratio - 0.5).abs() < 0.05,
            "duck hitbox ratio {ratio}, expected ~0.5 (45% shrink of 80px outer)"
        );
    }

    #[test]
    fn coyote_time_jump_within_window_succeeds() {
        let mut p = Player::new();
        // Force into Falling state by yanking it off the ground
        p.state = PlayerState::Falling;
        p.y -= 1.0;
        p.time_since_grounded = 0.0;
        // 79 ms after leaving ground — coyote should still allow jump
        step(&mut p, 0.079);
        assert!(p.try_jump(), "coyote jump within window should succeed");
    }

    #[test]
    fn coyote_time_jump_after_window_fails() {
        let mut p = Player::new();
        p.state = PlayerState::Falling;
        p.y -= 1.0;
        p.time_since_grounded = 0.0;
        step(&mut p, 0.100); // beyond 80 ms
        assert!(!p.try_jump(), "jump after coyote window must fail");
    }

    #[test]
    fn aabb_intersection() {
        let a = Aabb { x: 0.0, y: 0.0, w: 10.0, h: 10.0 };
        let b = Aabb { x: 5.0, y: 5.0, w: 10.0, h: 10.0 };
        let c = Aabb { x: 20.0, y: 20.0, w: 5.0, h: 5.0 };
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }
}
```

- [ ] **Step 6.3: Run tests**

```bash
cargo test --lib game::player::tests
```

Expected: 7 passed. If apex test fails by more than 2 px, the integration error from the fixed timestep is acceptable but worth noting — adjust the tolerance or revisit `update` order.

- [ ] **Step 6.4: Commit**

```bash
git add src/game/player.rs src/game/mod.rs
git commit -m "feat(player): physics, state machine, jump arc + coyote tests"
```

---

## Task 7: game::obstacles (types, spawn, collision)

**Files:**
- Create: `src/game/obstacles.rs`
- Modify: `src/game/mod.rs`

Spec §3.4 obstacle table.

- [ ] **Step 7.1: Add module declaration**

Edit `src/game/mod.rs`:

```rust
pub mod difficulty;
pub mod score;
pub mod player;
pub mod obstacles;
```

- [ ] **Step 7.2: Write the failing test + implementation**

Create `src/game/obstacles.rs`:

```rust
//! Obstacles: types, spawning, collision shapes. See spec §3.4.

use crate::game::difficulty::{tier_for_score, SPARK_BURST_UNLOCK_TIER};
use crate::game::player::{Aabb, GROUND_Y};
use rand::rngs::SmallRng;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleKind {
    CoiledCable,
    ChargingDock,
    ToolCart,
    SensorCone,
    QuadDrone,
    SparkBurst,
}

impl ObstacleKind {
    pub fn destroyable_by_dash(&self) -> bool {
        // §3.4: Charging Dock is too heavy. SparkBurst passes through naturally.
        !matches!(self, ObstacleKind::ChargingDock)
    }

    /// Bounding box dimensions in logical pixels.
    pub fn size(&self) -> (f32, f32) {
        match self {
            ObstacleKind::CoiledCable => (32.0, 32.0),
            ObstacleKind::ChargingDock => (40.0, 96.0),
            ObstacleKind::ToolCart => (128.0, 48.0),
            ObstacleKind::SensorCone => (24.0, 32.0),
            ObstacleKind::QuadDrone => (56.0, 32.0),
            ObstacleKind::SparkBurst => (24.0, 24.0),
        }
    }

    /// Y position of the top of the obstacle.
    pub fn y_for_kind(&self) -> f32 {
        let (_, h) = self.size();
        match self {
            ObstacleKind::QuadDrone => GROUND_Y - 96.0, // hover at duck height
            ObstacleKind::SparkBurst => GROUND_Y - 160.0, // high air
            _ => GROUND_Y - h,                          // ground-resting
        }
    }
}

#[derive(Debug, Clone)]
pub struct Obstacle {
    pub kind: ObstacleKind,
    pub x: f32,
    pub y: f32,
    pub alive: bool,
}

impl Obstacle {
    pub fn new(kind: ObstacleKind, x: f32) -> Self {
        let y = kind.y_for_kind();
        Self { kind, x, y, alive: true }
    }

    pub fn hitbox(&self) -> Aabb {
        let (w, h) = self.kind.size();
        Aabb { x: self.x, y: self.y, w, h }
    }
}

const SPAWN_X: f32 = 1400.0; // just off-screen right of 1280-wide viewport

pub struct ObstacleField {
    pub obstacles: Vec<Obstacle>,
    /// Distance scrolled since last spawn.
    pub scrolled_since_spawn: f32,
    /// Spacing required before next spawn — recomputed on each spawn.
    pub next_spawn_gap: f32,
}

impl Default for ObstacleField {
    fn default() -> Self {
        Self {
            obstacles: Vec::new(),
            scrolled_since_spawn: 0.0,
            next_spawn_gap: 0.0,
        }
    }
}

impl ObstacleField {
    pub fn new() -> Self {
        Self::default()
    }

    /// Minimum distance between consecutive obstacles, scaled by current speed
    /// so the player always has at least 1.0s of reaction time.
    pub fn min_gap(speed: f32) -> f32 {
        (speed * 1.0).max(180.0)
    }

    /// Pick a random kind given current tier and rng.
    fn random_kind(&self, score: u32, rng: &mut SmallRng) -> ObstacleKind {
        let tier = tier_for_score(score);
        // Build a small weighted list. Higher tiers shift toward harder.
        let mut pool: Vec<ObstacleKind> = vec![
            ObstacleKind::CoiledCable,
            ObstacleKind::CoiledCable,
            ObstacleKind::SensorCone,
            ObstacleKind::ToolCart,
        ];
        if tier >= 1 {
            pool.push(ObstacleKind::ChargingDock);
            pool.push(ObstacleKind::QuadDrone);
        }
        if tier >= 2 {
            pool.push(ObstacleKind::QuadDrone);
            pool.push(ObstacleKind::ChargingDock);
        }
        if tier >= SPARK_BURST_UNLOCK_TIER {
            pool.push(ObstacleKind::SparkBurst);
        }
        let idx = rng.gen_range(0..pool.len());
        pool[idx]
    }

    /// Advance one fixed step. Scrolls existing obstacles, spawns when allowed,
    /// and removes off-screen ones.
    pub fn update(&mut self, dt: f32, speed: f32, score: u32, rng: &mut SmallRng) {
        let dx = speed * dt;
        for o in &mut self.obstacles {
            o.x -= dx;
        }
        self.obstacles.retain(|o| o.alive && o.x + o.kind.size().0 > -50.0);

        self.scrolled_since_spawn += dx;
        if self.scrolled_since_spawn >= self.next_spawn_gap {
            let kind = self.random_kind(score, rng);
            self.obstacles.push(Obstacle::new(kind, SPAWN_X));
            self.scrolled_since_spawn = 0.0;
            // Random extra gap on top of minimum
            let extra = rng.gen_range(0.0..200.0);
            self.next_spawn_gap = Self::min_gap(speed) + extra;
        }
    }

    /// Returns the first obstacle whose hitbox intersects the given player box,
    /// if any.
    pub fn first_collision(&self, player: &Aabb) -> Option<usize> {
        self.obstacles
            .iter()
            .position(|o| o.alive && o.hitbox().intersects(player))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::difficulty::speed_for_score;
    use rand::SeedableRng;

    #[test]
    fn destroyable_flags() {
        assert!(!ObstacleKind::ChargingDock.destroyable_by_dash());
        assert!(ObstacleKind::CoiledCable.destroyable_by_dash());
        assert!(ObstacleKind::ToolCart.destroyable_by_dash());
        assert!(ObstacleKind::QuadDrone.destroyable_by_dash());
    }

    #[test]
    fn min_gap_grows_with_speed() {
        assert!(ObstacleField::min_gap(720.0) > ObstacleField::min_gap(320.0));
    }

    #[test]
    fn spawn_respects_min_spacing_at_every_tier() {
        // Simulate 60 seconds at each speed tier and assert no obstacle is
        // spawned with horizontal overlap to the previous one.
        for tier in 0..=8u32 {
            let score = tier * 500;
            let speed = speed_for_score(score);
            let mut field = ObstacleField::new();
            let mut rng = SmallRng::seed_from_u64(42 + tier as u64);
            // 60 seconds at fixed step
            let steps = (60.0 / crate::time::DT) as u32;
            for _ in 0..steps {
                field.update(crate::time::DT, speed, score, &mut rng);
            }
            // Sort by spawn X (current x positions reflect spawn order modulo scroll)
            let mut xs: Vec<f32> = field.obstacles.iter().map(|o| o.x).collect();
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            for w in xs.windows(2) {
                let gap = w[1] - w[0];
                assert!(
                    gap >= ObstacleField::min_gap(speed) - 1.0,
                    "tier {tier} speed {speed}: gap {gap} < min {}",
                    ObstacleField::min_gap(speed)
                );
            }
        }
    }

    #[test]
    fn spark_burst_only_at_tier_3_plus() {
        let mut rng = SmallRng::seed_from_u64(7);
        let field = ObstacleField::new();
        // tier 0 — should never produce spark
        for _ in 0..200 {
            let k = field.random_kind(0, &mut rng);
            assert_ne!(k, ObstacleKind::SparkBurst);
        }
        // tier 3 — should produce spark sometimes within 1000 draws
        let mut saw_spark = false;
        for _ in 0..1000 {
            if field.random_kind(SPARK_BURST_UNLOCK_TIER * 500, &mut rng)
                == ObstacleKind::SparkBurst
            {
                saw_spark = true;
                break;
            }
        }
        assert!(saw_spark);
    }
}
```

- [ ] **Step 7.3: Run tests**

```bash
cargo test --lib game::obstacles::tests
```

Expected: 4 passed.

- [ ] **Step 7.4: Commit**

```bash
git add src/game/obstacles.rs src/game/mod.rs
git commit -m "feat(obstacles): types, seeded spawn, min-spacing test"
```

---

## Task 8: game::pickups (Aurora Stones)

**Files:**
- Create: `src/game/pickups.rs`
- Modify: `src/game/mod.rs`

Spec §3.5: glowing orbs, two cosmetic colors, fill energy meter (max 3).

- [ ] **Step 8.1: Add module declaration**

Edit `src/game/mod.rs`:

```rust
pub mod difficulty;
pub mod score;
pub mod player;
pub mod obstacles;
pub mod pickups;
```

- [ ] **Step 8.2: Write the failing test + implementation**

Create `src/game/pickups.rs`:

```rust
//! Aurora Stones: collectible pickups that fill the dash energy meter.
//! See spec §3.5.

use crate::game::player::{Aabb, GROUND_Y};
use rand::rngs::SmallRng;
use rand::Rng;

pub const MAX_AURORA: u32 = 3;
pub const PICKUP_W: f32 = 28.0;
pub const PICKUP_H: f32 = 28.0;
const SPAWN_X: f32 = 1400.0;
const SPAWN_INTERVAL_MIN: f32 = 8.0;
const SPAWN_INTERVAL_MAX: f32 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuroraColor {
    Purple,
    Green,
}

#[derive(Debug, Clone)]
pub struct AuroraStone {
    pub x: f32,
    pub y: f32,
    pub color: AuroraColor,
    pub collected: bool,
}

impl AuroraStone {
    pub fn hitbox(&self) -> Aabb {
        Aabb { x: self.x, y: self.y, w: PICKUP_W, h: PICKUP_H }
    }
}

pub struct PickupField {
    pub stones: Vec<AuroraStone>,
    pub time_to_next: f32,
}

impl Default for PickupField {
    fn default() -> Self {
        Self { stones: Vec::new(), time_to_next: SPAWN_INTERVAL_MIN }
    }
}

impl PickupField {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, dt: f32, speed: f32, rng: &mut SmallRng) {
        let dx = speed * dt;
        for s in &mut self.stones {
            s.x -= dx;
        }
        self.stones.retain(|s| !s.collected && s.x + PICKUP_W > -50.0);

        self.time_to_next -= dt;
        if self.time_to_next <= 0.0 {
            // Random vertical placement: ground-level, mid-air, or high-air.
            let tier = rng.gen_range(0..3u32);
            let y = match tier {
                0 => GROUND_Y - PICKUP_H - 8.0,        // ground
                1 => GROUND_Y - 110.0,                  // mid (jump to grab)
                _ => GROUND_Y - 160.0,                  // high
            };
            let color = if rng.gen_bool(0.5) {
                AuroraColor::Purple
            } else {
                AuroraColor::Green
            };
            self.stones.push(AuroraStone { x: SPAWN_X, y, color, collected: false });
            self.time_to_next = rng.gen_range(SPAWN_INTERVAL_MIN..SPAWN_INTERVAL_MAX);
        }
    }

    /// Returns indices of stones colliding with the player box.
    pub fn collisions_with(&self, player: &Aabb) -> Vec<usize> {
        self.stones
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.collected && s.hitbox().intersects(player))
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn spawns_within_interval_window() {
        let mut field = PickupField::new();
        let mut rng = SmallRng::seed_from_u64(1);
        // Run 30s at speed 320 — expect ~3 spawns
        let dt = crate::time::DT;
        let steps = (30.0 / dt) as u32;
        let mut spawn_count = 0u32;
        let mut last_len = 0;
        for _ in 0..steps {
            field.update(dt, 320.0, &mut rng);
            if field.stones.len() > last_len {
                spawn_count += 1;
            }
            last_len = field.stones.len();
        }
        assert!(spawn_count >= 2 && spawn_count <= 5, "got {spawn_count} spawns");
    }

    #[test]
    fn collected_stones_pruned() {
        let mut field = PickupField::new();
        field.stones.push(AuroraStone {
            x: 100.0,
            y: 100.0,
            color: AuroraColor::Purple,
            collected: true,
        });
        let mut rng = SmallRng::seed_from_u64(0);
        field.update(0.001, 100.0, &mut rng);
        assert!(field.stones.iter().all(|s| !s.collected));
    }

    #[test]
    fn collisions_with_overlapping_player() {
        let mut field = PickupField::new();
        field.stones.push(AuroraStone {
            x: 50.0,
            y: 50.0,
            color: AuroraColor::Green,
            collected: false,
        });
        let player = Aabb { x: 60.0, y: 60.0, w: 10.0, h: 10.0 };
        assert_eq!(field.collisions_with(&player), vec![0]);
    }
}
```

- [ ] **Step 8.3: Run tests**

```bash
cargo test --lib game::pickups::tests
```

Expected: 3 passed.

- [ ] **Step 8.4: Commit**

```bash
git add src/game/pickups.rs src/game/mod.rs
git commit -m "feat(pickups): Aurora Stones spawn + collision"
```

---

## Task 9: game::dash (state, invulnerability, slow-mo)

**Files:**
- Create: `src/game/dash.rs`
- Modify: `src/game/mod.rs`

Spec §3.5: 280 ms invulnerable, +60% horizontal, costs 1 aurora, 400 ms cooldown, smashes destroyables, slow-mo on drone hit.

- [ ] **Step 9.1: Add module declaration**

Edit `src/game/mod.rs`:

```rust
pub mod difficulty;
pub mod score;
pub mod player;
pub mod obstacles;
pub mod pickups;
pub mod dash;
```

- [ ] **Step 9.2: Write the failing test + implementation**

Create `src/game/dash.rs`:

```rust
//! Dash: short invulnerable burst that smashes destroyable obstacles.
//! See spec §3.5.

pub const DASH_DURATION: f32 = 0.280;
pub const DASH_COOLDOWN: f32 = 0.400;
pub const DASH_SPEED_MULT: f32 = 1.60;
pub const DASH_COST: u32 = 1;
pub const SLOWMO_DURATION: f32 = 0.200;
pub const SLOWMO_SCALE: f32 = 0.60;

#[derive(Debug, Default)]
pub struct DashState {
    pub aurora: u32,
    pub active_remaining: f32,
    pub cooldown_remaining: f32,
    pub slowmo_remaining: f32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DashRequest {
    Started,
    Denied,
}

impl DashState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_aurora(&mut self, n: u32) {
        self.aurora = (self.aurora + n).min(crate::game::pickups::MAX_AURORA);
    }

    pub fn is_active(&self) -> bool {
        self.active_remaining > 0.0
    }

    pub fn is_invulnerable(&self) -> bool {
        self.is_active()
    }

    pub fn try_start(&mut self) -> DashRequest {
        if self.aurora < DASH_COST || self.cooldown_remaining > 0.0 || self.is_active() {
            DashRequest::Denied
        } else {
            self.aurora -= DASH_COST;
            self.active_remaining = DASH_DURATION;
            DashRequest::Started
        }
    }

    pub fn trigger_slowmo(&mut self) {
        self.slowmo_remaining = SLOWMO_DURATION;
    }

    /// Returns the simulated-time scale to apply to world.update.
    pub fn time_scale(&self) -> f32 {
        if self.slowmo_remaining > 0.0 {
            SLOWMO_SCALE
        } else {
            1.0
        }
    }

    /// Returns the horizontal speed multiplier (>=1.0 during dash).
    pub fn speed_mult(&self) -> f32 {
        if self.is_active() {
            DASH_SPEED_MULT
        } else {
            1.0
        }
    }

    /// Tick timers using REAL dt (not slow-mo-scaled). Slow-mo affects only
    /// world physics, not the dash/cooldown clocks themselves.
    pub fn update(&mut self, real_dt: f32) {
        if self.active_remaining > 0.0 {
            self.active_remaining -= real_dt;
            if self.active_remaining <= 0.0 {
                self.active_remaining = 0.0;
                self.cooldown_remaining = DASH_COOLDOWN;
            }
        } else if self.cooldown_remaining > 0.0 {
            self.cooldown_remaining = (self.cooldown_remaining - real_dt).max(0.0);
        }
        if self.slowmo_remaining > 0.0 {
            self.slowmo_remaining = (self.slowmo_remaining - real_dt).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::DT;

    #[test]
    fn cannot_dash_without_aurora() {
        let mut d = DashState::new();
        assert_eq!(d.try_start(), DashRequest::Denied);
    }

    #[test]
    fn dash_costs_one_aurora() {
        let mut d = DashState::new();
        d.add_aurora(2);
        assert_eq!(d.try_start(), DashRequest::Started);
        assert_eq!(d.aurora, 1);
    }

    #[test]
    fn dash_invulnerable_for_280ms_then_cooldown() {
        let mut d = DashState::new();
        d.add_aurora(1);
        d.try_start();
        assert!(d.is_invulnerable());

        // Tick for ~280 ms in DT increments
        let steps = (DASH_DURATION / DT).round() as u32;
        for _ in 0..steps {
            d.update(DT);
        }
        assert!(!d.is_invulnerable(), "dash should have ended");
        assert!(d.cooldown_remaining > 0.0, "should be on cooldown");
    }

    #[test]
    fn cannot_chain_dash_during_cooldown() {
        let mut d = DashState::new();
        d.add_aurora(2);
        d.try_start();
        // end dash
        let steps = (DASH_DURATION / DT).round() as u32;
        for _ in 0..steps {
            d.update(DT);
        }
        // still in cooldown
        assert_eq!(d.try_start(), DashRequest::Denied);
    }

    #[test]
    fn aurora_capped_at_max() {
        let mut d = DashState::new();
        d.add_aurora(10);
        assert_eq!(d.aurora, crate::game::pickups::MAX_AURORA);
    }

    #[test]
    fn slowmo_scales_time() {
        let mut d = DashState::new();
        assert_eq!(d.time_scale(), 1.0);
        d.trigger_slowmo();
        assert_eq!(d.time_scale(), SLOWMO_SCALE);
    }
}
```

- [ ] **Step 9.3: Run tests**

```bash
cargo test --lib game::dash::tests
```

Expected: 6 passed.

- [ ] **Step 9.4: Commit**

```bash
git add src/game/dash.rs src/game/mod.rs
git commit -m "feat(dash): aurora-cost dash with cooldown + slow-mo"
```

---

## Task 10: game::background (parallax scroll, greybox)

**Files:**
- Create: `src/game/background.rs`
- Modify: `src/game/mod.rs`

Greybox: just three solid color bands scrolling at different rates. Real art is Phase 2.

- [ ] **Step 10.1: Add module declaration**

Edit `src/game/mod.rs`:

```rust
pub mod difficulty;
pub mod score;
pub mod player;
pub mod obstacles;
pub mod pickups;
pub mod dash;
pub mod background;
```

- [ ] **Step 10.2: Implement (no test needed — pure visual placeholder)**

Create `src/game/background.rs`:

```rust
//! Parallax background scroll. Greybox uses three solid bands; Phase 2 swaps
//! these for tiled art.

#[derive(Debug, Default)]
pub struct Background {
    pub far_offset: f32,
    pub mid_offset: f32,
    pub floor_offset: f32,
}

impl Background {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, dt: f32, speed: f32) {
        // Far layer scrolls slowly, floor at full speed.
        self.far_offset = (self.far_offset + speed * 0.10 * dt) % 1280.0;
        self.mid_offset = (self.mid_offset + speed * 0.30 * dt) % 1280.0;
        self.floor_offset = (self.floor_offset + speed * 1.00 * dt) % 1280.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn far_lags_floor() {
        let mut bg = Background::new();
        bg.update(1.0, 320.0);
        assert!(bg.floor_offset > bg.mid_offset);
        assert!(bg.mid_offset > bg.far_offset);
    }
}
```

- [ ] **Step 10.3: Run tests**

```bash
cargo test --lib game::background::tests
```

Expected: 1 passed.

- [ ] **Step 10.4: Commit**

```bash
git add src/game/background.rs src/game/mod.rs
git commit -m "feat(background): three-layer parallax scroll (greybox)"
```

---

## Task 11: game::world composition

**Files:**
- Create: `src/game/world.rs`
- Modify: `src/game/mod.rs`

The single owner of all mutable game state. Composes player, obstacles, pickups, dash, background, score, rng. Spec §4.1, §4.3.

- [ ] **Step 11.1: Add module declaration**

Edit `src/game/mod.rs`:

```rust
pub mod difficulty;
pub mod score;
pub mod player;
pub mod obstacles;
pub mod pickups;
pub mod dash;
pub mod background;
pub mod world;
```

- [ ] **Step 11.2: Write the test + implementation**

Create `src/game/world.rs`:

```rust
//! Composite game state. Owns the only RNG.

use crate::game::background::Background;
use crate::game::dash::{DashRequest, DashState};
use crate::game::difficulty::speed_for_score;
use crate::game::obstacles::{ObstacleField, ObstacleKind};
use crate::game::pickups::PickupField;
use crate::game::player::{Player, PlayerState};
use crate::game::score::Score;
use crate::platform::input::Action;
use crate::platform::storage::Storage;
use rand::rngs::SmallRng;
use rand::SeedableRng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunOutcome {
    Continuing,
    Died,
}

pub struct World {
    pub player: Player,
    pub obstacles: ObstacleField,
    pub pickups: PickupField,
    pub dash: DashState,
    pub background: Background,
    pub score: Score,
    pub rng: SmallRng,
    pub elapsed: f32,
}

impl World {
    pub fn new<S: Storage>(seed: u64, storage: &S) -> Self {
        Self {
            player: Player::new(),
            obstacles: ObstacleField::new(),
            pickups: PickupField::new(),
            dash: DashState::new(),
            background: Background::new(),
            score: Score::load(storage),
            rng: SmallRng::seed_from_u64(seed),
            elapsed: 0.0,
        }
    }

    pub fn current_speed(&self) -> f32 {
        speed_for_score(self.score.current) * self.dash.speed_mult()
    }

    /// Apply an input action to the world.
    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::Jump => {
                self.player.try_jump();
            }
            Action::JumpRelease => self.player.release_jump(),
            Action::Duck => self.player.try_duck(),
            Action::DuckRelease => self.player.release_duck(),
            Action::Dash => {
                if let DashRequest::Started = self.dash.try_start() {
                    // accept
                }
            }
            Action::Confirm | Action::Pause => { /* handled by state machine */ }
        }
    }

    /// One fixed-step update. Returns the run outcome (so the state machine
    /// can transition to GameOver on death).
    pub fn update(&mut self, real_dt: f32) -> RunOutcome {
        if matches!(self.player.state, PlayerState::Hit) {
            return RunOutcome::Died;
        }

        let scale = self.dash.time_scale();
        let sim_dt = real_dt * scale;

        self.elapsed += sim_dt;
        self.dash.update(real_dt); // dash timers always real time
        self.player.update(sim_dt);

        let speed = self.current_speed();
        self.background.update(sim_dt, speed);
        self.obstacles
            .update(sim_dt, speed, self.score.current, &mut self.rng);
        self.pickups.update(sim_dt, speed, &mut self.rng);

        // Score: 1 point per pixel scrolled (will feel right at base speed)
        let dx = (speed * sim_dt) as u32;
        self.score.add(dx / 4); // ~80 pts/sec at base speed

        // Pickup collisions
        let player_box = self.player.hitbox();
        let collected = self.pickups.collisions_with(&player_box);
        for &i in &collected {
            self.pickups.stones[i].collected = true;
            self.dash.add_aurora(1);
            self.score.add(50);
        }

        // Obstacle collisions
        if let Some(idx) = self.obstacles.first_collision(&player_box) {
            let kind = self.obstacles.obstacles[idx].kind;
            if self.dash.is_invulnerable() && kind.destroyable_by_dash() {
                self.obstacles.obstacles[idx].alive = false;
                self.score.add(25);
                if matches!(kind, ObstacleKind::QuadDrone) {
                    self.dash.trigger_slowmo();
                }
            } else if !self.dash.is_invulnerable() {
                self.player.hit();
                return RunOutcome::Died;
            }
            // Invulnerable hit on a non-destroyable: pass through harmlessly.
        }

        RunOutcome::Continuing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::storage::InMemoryStorage;
    use crate::time::DT;

    fn fresh_world() -> World {
        let storage = InMemoryStorage::new();
        World::new(123, &storage)
    }

    #[test]
    fn world_starts_alive_and_running() {
        let w = fresh_world();
        assert_eq!(w.player.state, PlayerState::Running);
        assert_eq!(w.score.current, 0);
        assert_eq!(w.dash.aurora, 0);
    }

    #[test]
    fn jump_action_lifts_player() {
        let mut w = fresh_world();
        let start_y = w.player.y;
        w.apply_action(Action::Jump);
        // Tick a few frames
        for _ in 0..10 {
            w.update(DT);
        }
        assert!(w.player.y < start_y);
    }

    #[test]
    fn pickup_grants_aurora_and_score() {
        let mut w = fresh_world();
        // Hand-place a stone overlapping the player
        let pbox = w.player.hitbox();
        w.pickups.stones.push(crate::game::pickups::AuroraStone {
            x: pbox.x,
            y: pbox.y,
            color: crate::game::pickups::AuroraColor::Purple,
            collected: false,
        });
        w.update(DT);
        assert_eq!(w.dash.aurora, 1);
        assert!(w.score.current >= 50);
    }

    #[test]
    fn collision_kills_player_when_not_dashing() {
        let mut w = fresh_world();
        let pbox = w.player.hitbox();
        // Hand-place a coiled cable overlapping the player
        w.obstacles.obstacles.push(crate::game::obstacles::Obstacle::new(
            ObstacleKind::CoiledCable,
            pbox.x,
        ));
        // Force the obstacle Y to overlap the player's hitbox
        w.obstacles.obstacles[0].y = pbox.y;
        let outcome = w.update(DT);
        assert_eq!(outcome, RunOutcome::Died);
        assert_eq!(w.player.state, PlayerState::Hit);
    }

    #[test]
    fn dash_smashes_destroyable_obstacle() {
        let mut w = fresh_world();
        w.dash.add_aurora(1);
        w.dash.try_start();
        let pbox = w.player.hitbox();
        w.obstacles.obstacles.push(crate::game::obstacles::Obstacle::new(
            ObstacleKind::CoiledCable,
            pbox.x,
        ));
        w.obstacles.obstacles[0].y = pbox.y;
        let outcome = w.update(DT);
        assert_eq!(outcome, RunOutcome::Continuing);
        assert!(!w.obstacles.obstacles[0].alive);
    }

    #[test]
    fn dash_does_not_smash_charging_dock() {
        let mut w = fresh_world();
        w.dash.add_aurora(1);
        w.dash.try_start();
        let pbox = w.player.hitbox();
        w.obstacles.obstacles.push(crate::game::obstacles::Obstacle::new(
            ObstacleKind::ChargingDock,
            pbox.x,
        ));
        w.obstacles.obstacles[0].y = pbox.y;
        let outcome = w.update(DT);
        // Invulnerable, so passes through harmlessly — alive AND not died
        assert_eq!(outcome, RunOutcome::Continuing);
        assert!(w.obstacles.obstacles[0].alive);
    }
}
```

- [ ] **Step 11.3: Run tests**

```bash
cargo test --lib game::world::tests
```

Expected: 6 passed.

- [ ] **Step 11.4: Commit**

```bash
git add src/game/world.rs src/game/mod.rs
git commit -m "feat(world): composite state, action dispatch, collision resolution"
```

---

## Task 12: game::state (Title/Playing/Paused/GameOver state machine)

**Files:**
- Create: `src/game/state.rs`
- Modify: `src/game/mod.rs`

Spec §3.8.

- [ ] **Step 12.1: Add module declaration**

Edit `src/game/mod.rs`:

```rust
pub mod difficulty;
pub mod score;
pub mod player;
pub mod obstacles;
pub mod pickups;
pub mod dash;
pub mod background;
pub mod world;
pub mod state;
```

- [ ] **Step 12.2: Write the test + implementation**

Create `src/game/state.rs`:

```rust
//! Top-level game state machine. See spec §3.8.

use crate::game::world::{RunOutcome, World};
use crate::platform::input::Action;
use crate::platform::storage::Storage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Title,
    Playing,
    Paused,
    GameOver,
}

pub struct Game {
    pub state: GameState,
    pub world: World,
    pub seed_counter: u64,
}

impl Game {
    pub fn new<S: Storage>(seed: u64, storage: &S) -> Self {
        Self {
            state: GameState::Title,
            world: World::new(seed, storage),
            seed_counter: seed,
        }
    }

    /// Apply visibility events from the platform layer.
    pub fn on_visibility_change(&mut self, visible: bool) {
        if !visible && self.state == GameState::Playing {
            self.state = GameState::Paused;
        }
    }

    /// Forward an Action through the state machine.
    pub fn handle<S: Storage>(&mut self, action: Action, storage: &mut S) {
        match (self.state, action) {
            (GameState::Title, Action::Confirm) | (GameState::Title, Action::Jump) => {
                self.start_run(storage);
            }
            (GameState::Playing, Action::Pause) => {
                self.state = GameState::Paused;
            }
            (GameState::Paused, Action::Pause) | (GameState::Paused, Action::Confirm) => {
                self.state = GameState::Playing;
            }
            (GameState::GameOver, Action::Confirm) | (GameState::GameOver, Action::Jump) => {
                self.start_run(storage);
            }
            (GameState::Playing, _) => {
                self.world.apply_action(action);
            }
            _ => {}
        }
    }

    fn start_run<S: Storage>(&mut self, storage: &S) {
        self.seed_counter = self.seed_counter.wrapping_add(1);
        self.world = World::new(self.seed_counter, storage);
        self.state = GameState::Playing;
    }

    /// Run one fixed step. Persists high score on death.
    pub fn update<S: Storage>(&mut self, real_dt: f32, storage: &mut S) {
        if self.state != GameState::Playing {
            return;
        }
        match self.world.update(real_dt) {
            RunOutcome::Continuing => {}
            RunOutcome::Died => {
                self.state = GameState::GameOver;
                let _ = self.world.score.save_if_new_high(storage);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::storage::InMemoryStorage;
    use crate::time::DT;

    #[test]
    fn starts_in_title() {
        let s = InMemoryStorage::new();
        let g = Game::new(1, &s);
        assert_eq!(g.state, GameState::Title);
    }

    #[test]
    fn confirm_from_title_starts_run() {
        let mut s = InMemoryStorage::new();
        let mut g = Game::new(1, &s);
        g.handle(Action::Confirm, &mut s);
        assert_eq!(g.state, GameState::Playing);
    }

    #[test]
    fn visibility_loss_pauses_during_play() {
        let mut s = InMemoryStorage::new();
        let mut g = Game::new(1, &s);
        g.handle(Action::Confirm, &mut s);
        g.on_visibility_change(false);
        assert_eq!(g.state, GameState::Paused);
    }

    #[test]
    fn pause_action_toggles() {
        let mut s = InMemoryStorage::new();
        let mut g = Game::new(1, &s);
        g.handle(Action::Confirm, &mut s);
        g.handle(Action::Pause, &mut s);
        assert_eq!(g.state, GameState::Paused);
        g.handle(Action::Pause, &mut s);
        assert_eq!(g.state, GameState::Playing);
    }

    #[test]
    fn death_transitions_to_game_over_and_persists() {
        let mut s = InMemoryStorage::new();
        let mut g = Game::new(1, &s);
        g.handle(Action::Confirm, &mut s);
        // Hand-place an overlapping obstacle to force death next update
        let pbox = g.world.player.hitbox();
        let mut o = crate::game::obstacles::Obstacle::new(
            crate::game::obstacles::ObstacleKind::CoiledCable,
            pbox.x,
        );
        o.y = pbox.y;
        g.world.obstacles.obstacles.push(o);
        // Force a visible score so save_if_new_high triggers
        g.world.score.current = 999;
        g.update(DT, &mut s);
        assert_eq!(g.state, GameState::GameOver);
        assert!(s.get(crate::game::score::STORAGE_KEY).is_some());
    }

    #[test]
    fn confirm_from_game_over_restarts() {
        let mut s = InMemoryStorage::new();
        let mut g = Game::new(1, &s);
        g.state = GameState::GameOver;
        g.handle(Action::Confirm, &mut s);
        assert_eq!(g.state, GameState::Playing);
    }
}
```

- [ ] **Step 12.3: Run tests**

```bash
cargo test --lib game::state::tests
```

Expected: 6 passed.

- [ ] **Step 12.4: Run all unit tests**

```bash
cargo test --lib
```

Expected: ALL tests pass (~38–42 tests across all modules).

- [ ] **Step 12.5: Commit**

```bash
git add src/game/state.rs src/game/mod.rs
git commit -m "feat(state): top-level Title/Playing/Paused/GameOver state machine"
```

---

## Task 13: render::camera (logical→screen mapping)

**Files:**
- Create: `src/render/mod.rs`
- Create: `src/render/camera.rs`

This is the FIRST module that touches macroquad. Tests for this module are limited to pure math.

- [ ] **Step 13.1: Create render module**

Create `src/render/mod.rs`:

```rust
//! Rendering. The only modules in the codebase (besides platform impls) that
//! call macroquad draw functions live here.
pub mod camera;
pub mod sprites;
pub mod ui;
```

- [ ] **Step 13.2: Implement camera**

Create `src/render/camera.rs`:

```rust
//! Logical-to-screen coordinate mapping. Logical resolution is 1280×400
//! per spec §3.3. The screen is letterboxed to preserve aspect ratio.

pub const LOGICAL_W: f32 = 1280.0;
pub const LOGICAL_H: f32 = 400.0;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub screen_w: f32,
    pub screen_h: f32,
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Camera {
    pub fn new(screen_w: f32, screen_h: f32) -> Self {
        let scale_x = screen_w / LOGICAL_W;
        let scale_y = screen_h / LOGICAL_H;
        let scale = scale_x.min(scale_y);
        let used_w = LOGICAL_W * scale;
        let used_h = LOGICAL_H * scale;
        let offset_x = (screen_w - used_w) * 0.5;
        let offset_y = (screen_h - used_h) * 0.5;
        Self { screen_w, screen_h, scale, offset_x, offset_y }
    }

    pub fn to_screen(&self, lx: f32, ly: f32) -> (f32, f32) {
        (self.offset_x + lx * self.scale, self.offset_y + ly * self.scale)
    }

    pub fn scaled(&self, v: f32) -> f32 {
        v * self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_inside_wide_screen() {
        let c = Camera::new(1920.0, 600.0);
        // 1920/1280 = 1.5, 600/400 = 1.5 — exactly fits both
        assert_eq!(c.scale, 1.5);
    }

    #[test]
    fn letterboxes_too_wide() {
        let c = Camera::new(2000.0, 400.0);
        // height-bound: 400/400 = 1.0
        assert_eq!(c.scale, 1.0);
        assert!(c.offset_x > 0.0);
    }

    #[test]
    fn origin_maps_to_offset() {
        let c = Camera::new(1280.0, 400.0);
        let (x, y) = c.to_screen(0.0, 0.0);
        assert!((x - c.offset_x).abs() < 0.001);
        assert!((y - c.offset_y).abs() < 0.001);
    }
}
```

- [ ] **Step 13.3: Run tests**

```bash
cargo test --lib render::camera::tests
```

Expected: 3 passed.

- [ ] **Step 13.4: Commit**

```bash
git add src/render/mod.rs src/render/camera.rs
git commit -m "feat(camera): logical→screen letterbox mapping"
```

---

## Task 14: render::sprites (greybox = colored rectangles)

**Files:**
- Create: `src/render/sprites.rs`

Greybox draws solid colored rectangles for player, obstacles, pickups. Phase 2 replaces the bodies with `draw_texture`.

- [ ] **Step 14.1: Implement sprites**

Create `src/render/sprites.rs`:

```rust
//! Greybox sprite drawing — solid colored rectangles. Phase 2 swaps these
//! bodies for textured draws while keeping the same function signatures.

use crate::game::obstacles::{Obstacle, ObstacleKind};
use crate::game::pickups::{AuroraColor, AuroraStone};
use crate::game::player::{Player, PlayerState, PLAYER_H, PLAYER_W, PLAYER_X};
use crate::render::camera::Camera;
use macroquad::prelude::*;

pub fn draw_player(player: &Player, cam: &Camera) {
    let color = match player.state {
        PlayerState::Hit => RED,
        PlayerState::Ducking => Color::new(0.95, 0.55, 0.20, 1.0), // orange
        _ => Color::new(0.24, 0.24, 0.24, 1.0),                    // edie-body charcoal
    };
    let h = if matches!(player.state, PlayerState::Ducking) {
        PLAYER_H * 0.55
    } else {
        PLAYER_H
    };
    let y = if matches!(player.state, PlayerState::Ducking) {
        player.y + PLAYER_H * 0.45
    } else {
        player.y
    };
    let (sx, sy) = cam.to_screen(PLAYER_X, y);
    draw_rectangle(sx, sy, cam.scaled(PLAYER_W), cam.scaled(h), color);
}

pub fn draw_obstacle(o: &Obstacle, cam: &Camera) {
    let color = match o.kind {
        ObstacleKind::CoiledCable => Color::new(0.30, 0.30, 0.30, 1.0),
        ObstacleKind::ChargingDock => Color::new(0.55, 0.20, 0.20, 1.0),
        ObstacleKind::ToolCart => Color::new(0.40, 0.30, 0.20, 1.0),
        ObstacleKind::SensorCone => Color::new(0.91, 0.57, 0.24, 1.0),
        ObstacleKind::QuadDrone => Color::new(0.20, 0.30, 0.50, 1.0),
        ObstacleKind::SparkBurst => Color::new(0.95, 0.85, 0.30, 1.0),
    };
    let (w, h) = o.kind.size();
    let (sx, sy) = cam.to_screen(o.x, o.y);
    draw_rectangle(sx, sy, cam.scaled(w), cam.scaled(h), color);
}

pub fn draw_aurora(s: &AuroraStone, cam: &Camera) {
    let color = match s.color {
        AuroraColor::Purple => Color::new(0.62, 0.42, 1.00, 1.0),
        AuroraColor::Green => Color::new(0.36, 0.89, 0.66, 1.0),
    };
    let (sx, sy) = cam.to_screen(s.x, s.y);
    draw_rectangle(
        sx,
        sy,
        cam.scaled(crate::game::pickups::PICKUP_W),
        cam.scaled(crate::game::pickups::PICKUP_H),
        color,
    );
    // Halo: a slightly larger lighter rect behind
    let halo_inset = 4.0;
    draw_rectangle(
        sx - cam.scaled(halo_inset),
        sy - cam.scaled(halo_inset),
        cam.scaled(crate::game::pickups::PICKUP_W + 2.0 * halo_inset),
        cam.scaled(crate::game::pickups::PICKUP_H + 2.0 * halo_inset),
        Color::new(color.r, color.g, color.b, 0.25),
    );
}
```

- [ ] **Step 14.2: Verify it compiles**

```bash
cargo check
```

Expected: warnings about unused functions are OK; no errors.

- [ ] **Step 14.3: Commit**

```bash
git add src/render/sprites.rs
git commit -m "feat(sprites): greybox rectangle drawing for player/obstacles/aurora"
```

---

## Task 15: render::ui (background bands, HUD, overlays)

**Files:**
- Create: `src/render/ui.rs`

- [ ] **Step 15.1: Implement UI**

Create `src/render/ui.rs`:

```rust
//! UI: background bands, HUD, title/pause/game-over overlays.

use crate::game::background::Background;
use crate::game::dash::DashState;
use crate::game::pickups::MAX_AURORA;
use crate::game::score::Score;
use crate::game::state::GameState;
use crate::render::camera::{Camera, LOGICAL_H, LOGICAL_W};
use macroquad::prelude::*;

pub fn draw_background(bg: &Background, cam: &Camera) {
    // Sky
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(
        x0,
        y0,
        cam.scaled(LOGICAL_W),
        cam.scaled(LOGICAL_H),
        Color::new(0.96, 0.94, 0.89, 1.0), // bg-sky
    );

    // Far servers (parallax)
    let band_h = 100.0;
    let (fx, fy) = cam.to_screen(-bg.far_offset, 200.0);
    draw_rectangle(
        fx,
        fy,
        cam.scaled(LOGICAL_W * 2.0),
        cam.scaled(band_h),
        Color::new(0.79, 0.76, 0.70, 1.0), // bg-far
    );

    // Mid workbenches
    let (mx, my) = cam.to_screen(-bg.mid_offset, 280.0);
    draw_rectangle(
        mx,
        my,
        cam.scaled(LOGICAL_W * 2.0),
        cam.scaled(40.0),
        Color::new(0.56, 0.53, 0.46, 1.0), // bg-mid
    );

    // Floor
    let (flx, fly) = cam.to_screen(0.0, 320.0);
    draw_rectangle(
        flx,
        fly,
        cam.scaled(LOGICAL_W),
        cam.scaled(80.0),
        Color::new(0.29, 0.27, 0.22, 1.0), // floor
    );
    // Floor accent line
    let (lx, ly) = cam.to_screen(0.0, 320.0);
    draw_line(
        lx,
        ly,
        lx + cam.scaled(LOGICAL_W),
        ly,
        2.0 * cam.scale,
        Color::new(0.18, 0.16, 0.13, 1.0),
    );
}

pub fn draw_hud(score: &Score, dash: &DashState, cam: &Camera) {
    // Score top-right
    let score_text = format!("{:06}", score.current);
    let high_text = format!("HI {:06}", score.high);
    let font_size = 28.0 * cam.scale;
    let (sx, sy) = cam.to_screen(LOGICAL_W - 200.0, 30.0);
    draw_text(&score_text, sx, sy, font_size, BLACK);
    let (hx, hy) = cam.to_screen(LOGICAL_W - 200.0, 60.0);
    draw_text(&high_text, hx, hy, 20.0 * cam.scale, DARKGRAY);

    // Aurora meter top-left
    let icon_w = 28.0;
    let gap = 8.0;
    for i in 0..MAX_AURORA {
        let x = 20.0 + i as f32 * (icon_w + gap);
        let (sx, sy) = cam.to_screen(x, 20.0);
        let filled = i < dash.aurora;
        let color = if filled {
            Color::new(0.62, 0.42, 1.00, 1.0)
        } else {
            Color::new(0.62, 0.42, 1.00, 0.25)
        };
        draw_rectangle(sx, sy, cam.scaled(icon_w), cam.scaled(icon_w), color);
    }
}

pub fn draw_overlay(state: GameState, score: &Score, cam: &Camera) {
    let dim = match state {
        GameState::Title | GameState::Paused | GameState::GameOver => 0.45,
        _ => return,
    };
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(
        x0,
        y0,
        cam.scaled(LOGICAL_W),
        cam.scaled(LOGICAL_H),
        Color::new(0.0, 0.0, 0.0, dim),
    );

    let (cx, cy) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.4);
    let title = match state {
        GameState::Title => "EDIE RUNNER",
        GameState::Paused => "PAUSED",
        GameState::GameOver => "GAME OVER",
        _ => "",
    };
    let size = 64.0 * cam.scale;
    let dim_text = measure_text(title, None, size as u16, 1.0);
    draw_text(title, cx - dim_text.width * 0.5, cy, size, WHITE);

    let sub = match state {
        GameState::Title => "PRESS SPACE TO START".to_string(),
        GameState::Paused => "PRESS P OR SPACE TO RESUME".to_string(),
        GameState::GameOver => format!("SCORE {} | HI {} | SPACE TO RETRY", score.current, score.high),
    };
    let sub_size = 24.0 * cam.scale;
    let (sx, sy) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.6);
    let dim_sub = measure_text(&sub, None, sub_size as u16, 1.0);
    draw_text(&sub, sx - dim_sub.width * 0.5, sy, sub_size, WHITE);
}
```

- [ ] **Step 15.2: Verify it compiles**

```bash
cargo check
```

Expected: no errors.

- [ ] **Step 15.3: Commit**

```bash
git add src/render/ui.rs
git commit -m "feat(ui): background bands, HUD with aurora meter, state overlays"
```

---

## Task 16: lib.rs export render module

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 16.1: Add render and game modules to lib.rs**

Replace `src/lib.rs`:

```rust
//! EDIE Runner library entry point.

pub mod game;
pub mod platform;
pub mod render;
pub mod time;
```

- [ ] **Step 16.2: Verify cargo check passes**

```bash
cargo check
```

Expected: no errors.

- [ ] **Step 16.3: Commit**

```bash
git add src/lib.rs
git commit -m "chore: export render module from lib"
```

---

## Task 17: platform::storage QuadStorage impl

**Files:**
- Modify: `src/platform/storage.rs`

Production impl of `Storage` that wraps `quad-storage`. Gated to non-test builds because `quad-storage` requires the macroquad runtime.

- [ ] **Step 17.1: Append the impl**

Append to `src/platform/storage.rs` (after the existing tests module):

```rust
/// Production storage backed by quad-storage (browser localStorage).
pub struct QuadStorage;

impl QuadStorage {
    pub fn new() -> Self {
        Self
    }
}

impl Default for QuadStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage for QuadStorage {
    fn get(&self, key: &str) -> Option<String> {
        let storage = quad_storage::STORAGE.lock().unwrap();
        storage.get(key)
    }

    fn set(&mut self, key: &str, value: &str) {
        let mut storage = quad_storage::STORAGE.lock().unwrap();
        storage.set(key, value);
    }
}
```

- [ ] **Step 17.2: Verify build still passes for the wasm target**

```bash
cargo check --target wasm32-unknown-unknown
```

Expected: no errors. (May take longer the first time as macroquad downloads.)

- [ ] **Step 17.3: Commit**

```bash
git add src/platform/storage.rs
git commit -m "feat(storage): QuadStorage impl wrapping quad-storage"
```

---

## Task 18: platform::input MacroquadInput impl

**Files:**
- Modify: `src/platform/input.rs`

Polls macroquad keyboard each frame and emits `Action`s. Needs to track Jump/Duck press vs release transitions.

- [ ] **Step 18.1: Append the impl**

Append to `src/platform/input.rs` (after the existing tests module):

```rust
use macroquad::prelude::*;

/// Production input source: reads macroquad keyboard each frame.
pub struct MacroquadInput {
    jump_was_down: bool,
    duck_was_down: bool,
}

impl MacroquadInput {
    pub fn new() -> Self {
        Self { jump_was_down: false, duck_was_down: false }
    }
}

impl Default for MacroquadInput {
    fn default() -> Self {
        Self::new()
    }
}

impl InputSource for MacroquadInput {
    fn poll(&mut self) -> Vec<Action> {
        let mut out = Vec::new();
        let jump_now = is_key_down(KeyCode::Space) || is_key_down(KeyCode::Up);
        let duck_now = is_key_down(KeyCode::Down);

        if jump_now && !self.jump_was_down {
            out.push(Action::Jump);
            out.push(Action::Confirm);
        }
        if !jump_now && self.jump_was_down {
            out.push(Action::JumpRelease);
        }
        if duck_now && !self.duck_was_down {
            out.push(Action::Duck);
        }
        if !duck_now && self.duck_was_down {
            out.push(Action::DuckRelease);
        }
        if is_key_pressed(KeyCode::LeftShift) || is_key_pressed(KeyCode::RightShift) {
            out.push(Action::Dash);
        }
        if is_key_pressed(KeyCode::P) {
            out.push(Action::Pause);
        }

        self.jump_was_down = jump_now;
        self.duck_was_down = duck_now;
        out
    }
}
```

- [ ] **Step 18.2: Verify wasm build**

```bash
cargo check --target wasm32-unknown-unknown
```

Expected: no errors.

- [ ] **Step 18.3: Commit**

```bash
git add src/platform/input.rs
git commit -m "feat(input): MacroquadInput with press/release transitions"
```

---

## Task 19: platform::visibility (tab focus)

**Files:**
- Create: `src/platform/visibility.rs`
- Modify: `src/platform/mod.rs`

macroquad does not directly expose `visibilitychange`. We approximate by detecting when window focus is lost. For Phase 1 we treat any large frame_time as a focus-loss signal (already handled by the time clamp + a notification).

- [ ] **Step 19.1: Add module declaration**

Edit `src/platform/mod.rs`:

```rust
pub mod storage;
pub mod input;
pub mod visibility;
```

- [ ] **Step 19.2: Implement**

Create `src/platform/visibility.rs`:

```rust
//! Approximate visibility tracking. macroquad lacks a direct
//! visibilitychange hook, so we infer from frame-time spikes.

const SUSPICIOUSLY_LARGE_FRAME: f32 = 0.5;

#[derive(Default)]
pub struct VisibilityTracker {
    pub visible: bool,
}

impl VisibilityTracker {
    pub fn new() -> Self {
        Self { visible: true }
    }

    /// Returns Some(new_visibility) if the visibility changed this frame.
    pub fn observe(&mut self, frame_time: f32) -> Option<bool> {
        if frame_time > SUSPICIOUSLY_LARGE_FRAME && self.visible {
            self.visible = false;
            return Some(false);
        }
        if frame_time <= SUSPICIOUSLY_LARGE_FRAME && !self.visible {
            self.visible = true;
            return Some(true);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_frames_dont_change() {
        let mut v = VisibilityTracker::new();
        for _ in 0..100 {
            assert_eq!(v.observe(0.016), None);
        }
    }

    #[test]
    fn large_frame_marks_invisible() {
        let mut v = VisibilityTracker::new();
        assert_eq!(v.observe(2.0), Some(false));
        assert_eq!(v.observe(0.016), Some(true));
    }
}
```

- [ ] **Step 19.3: Run tests**

```bash
cargo test --lib platform::visibility::tests
```

Expected: 2 passed.

- [ ] **Step 19.4: Commit**

```bash
git add src/platform/mod.rs src/platform/visibility.rs
git commit -m "feat(visibility): frame-time spike detection for tab blur"
```

---

## Task 20: web shell (index.html)

**Files:**
- Create: `web/index.html`

Standard macroquad WASM HTML shell.

- [ ] **Step 20.1: Create web shell**

Create `web/index.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <title>EDIE Runner</title>
    <style>
        html, body, canvas {
            margin: 0;
            padding: 0;
            width: 100%;
            height: 100%;
            overflow: hidden;
            background: #F5EFE4;
            touch-action: none;
        }
        canvas:focus { outline: none; }
    </style>
</head>
<body>
    <canvas id="glcanvas" tabindex="1"></canvas>
    <script src="https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js"></script>
    <script>load("edie_runner.wasm");</script>
</body>
</html>
```

- [ ] **Step 20.2: Commit**

```bash
git add web/index.html
git commit -m "chore(web): static HTML shell for wasm load"
```

---

## Task 21: main.rs — full integration

**Files:**
- Modify: `src/main.rs`

The macroquad entry point. Wires everything together using fixed timestep.

- [ ] **Step 21.1: Replace main.rs**

Replace `src/main.rs`:

```rust
//! EDIE Runner — macroquad entry point. See spec §4.2 for the loop shape.

use edie_runner::game::state::Game;
use edie_runner::game::world::RunOutcome;
use edie_runner::platform::input::{InputSource, MacroquadInput};
use edie_runner::platform::storage::QuadStorage;
use edie_runner::platform::visibility::VisibilityTracker;
use edie_runner::render::camera::Camera;
use edie_runner::render::sprites::{draw_aurora, draw_obstacle, draw_player};
use edie_runner::render::ui::{draw_background, draw_hud, draw_overlay};
use edie_runner::time::{FixedStep, DT};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "EDIE Runner".to_string(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut storage = QuadStorage::new();
    let mut input = MacroquadInput::new();
    let mut visibility = VisibilityTracker::new();
    let mut step = FixedStep::new();
    let initial_seed = (get_time() * 1000.0) as u64;
    let mut game = Game::new(initial_seed, &storage);

    loop {
        let frame_time = get_frame_time();

        // Visibility
        if let Some(visible) = visibility.observe(frame_time) {
            game.on_visibility_change(visible);
        }

        // Input
        let actions = input.poll();
        for a in actions {
            game.handle(a, &mut storage);
        }

        // Fixed-step world updates
        let n = step.advance(frame_time);
        for _ in 0..n {
            game.update(DT, &mut storage);
        }

        // Render
        clear_background(Color::new(0.96, 0.94, 0.89, 1.0));
        let cam = Camera::new(screen_width(), screen_height());
        draw_background(&game.world.background, &cam);
        for o in &game.world.obstacles.obstacles {
            if o.alive {
                draw_obstacle(o, &cam);
            }
        }
        for s in &game.world.pickups.stones {
            if !s.collected {
                draw_aurora(s, &cam);
            }
        }
        draw_player(&game.world.player, &cam);
        draw_hud(&game.world.score, &game.world.dash, &cam);
        draw_overlay(game.state, &game.world.score, &cam);

        next_frame().await;
    }
}
```

- [ ] **Step 21.2: Verify host build (will fail because macroquad uses platform-specific OpenGL on host)**

```bash
cargo check --target wasm32-unknown-unknown
```

Expected: no errors. **Host `cargo check` may complain about OpenGL on Windows — we only target wasm for the binary.**

- [ ] **Step 21.3: Build the wasm binary**

```bash
cargo build --release --target wasm32-unknown-unknown --bin edie_runner
```

Expected: builds successfully. The artifact is at `target/wasm32-unknown-unknown/release/edie_runner.wasm`.

- [ ] **Step 21.4: Copy wasm into web/**

```bash
cp target/wasm32-unknown-unknown/release/edie_runner.wasm web/edie_runner.wasm
```

- [ ] **Step 21.5: Commit**

```bash
git add src/main.rs web/edie_runner.wasm
git commit -m "feat(main): macroquad entry with fixed-step loop and full wiring"
```

(Note: `web/edie_runner.wasm` is in `.gitignore`, so the second `git add` will be a no-op for the wasm file. That's OK.)

---

## Task 22: build scripts

**Files:**
- Create: `scripts/build.sh`
- Create: `scripts/build.ps1`
- Create: `README.md`

- [ ] **Step 22.1: Create build.sh**

Create `scripts/build.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown --bin edie_runner
cp target/wasm32-unknown-unknown/release/edie_runner.wasm web/edie_runner.wasm
echo
echo "Build complete. Serve the game with:"
echo "  cd web && python -m http.server 8080"
echo "Then open http://localhost:8080 in Chrome."
```

- [ ] **Step 22.2: Create build.ps1**

Create `scripts/build.ps1`:

```powershell
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
cargo build --release --target wasm32-unknown-unknown --bin edie_runner
Copy-Item target\wasm32-unknown-unknown\release\edie_runner.wasm web\edie_runner.wasm -Force
Write-Host ""
Write-Host "Build complete. Serve the game with:"
Write-Host "  cd web; python -m http.server 8080"
Write-Host "Then open http://localhost:8080 in Chrome."
```

- [ ] **Step 22.3: Create README.md**

Create `README.md`:

```markdown
# EDIE Runner

Endless runner starring EDIE, the ADrive robot. Runs in Chrome via WebAssembly.

## Phase 1 (Greybox)

Mechanically complete game using colored rectangles for art. Phase 2 adds bespoke art; Phase 3 adds juice and audio.

## Required tooling

- Rust 1.84 (auto-installed via `rust-toolchain.toml`)
- `rustup target add wasm32-unknown-unknown`
- Python 3 (for local serving) or any other static file server

## Build & run

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

## Controls

- **Space / ↑** — jump (hold for higher jump)
- **↓** — duck
- **Shift** — dash (costs 1 aurora)
- **P** — pause
- **Space** — confirm on Title / Game Over

## Run unit tests

```bash
cargo test --lib
```

All game logic is host-testable through trait seams in `src/platform/`.
```

- [ ] **Step 22.4: Make scripts executable (Linux/macOS)**

```bash
chmod +x scripts/build.sh
```

- [ ] **Step 22.5: Run a full build smoke**

```bash
./scripts/build.sh
```

Expected: builds and prints serve instructions.

- [ ] **Step 22.6: Commit**

```bash
git add scripts/build.sh scripts/build.ps1 README.md
git commit -m "chore: build scripts and README"
```

---

## Task 23: Final integration smoke

This task is **manual** — no code changes. It validates that Phase 1 is actually done.

- [ ] **Step 23.1: Run full unit-test sweep**

```bash
cargo test --lib
```

Expected: all tests pass. Count: ~42–48 tests across modules (time, storage, input, difficulty, score, player, obstacles, pickups, dash, background, world, state, camera, visibility).

- [ ] **Step 23.2: Build wasm**

```bash
./scripts/build.sh
```

Expected: success.

- [ ] **Step 23.3: Serve and open in Chrome**

```bash
cd web && python -m http.server 8080
```

Open http://localhost:8080 in Chrome.

- [ ] **Step 23.4: Manual smoke checklist**

Verify each:

- [ ] Title screen shows "EDIE RUNNER" and high-score line
- [ ] Pressing Space starts a run
- [ ] Player (charcoal rectangle) runs at fixed X
- [ ] Pressing Space jumps; holding gives a higher jump
- [ ] Pressing Down ducks; releasing returns to running
- [ ] Obstacles (colored rectangles) approach from the right
- [ ] Hitting an obstacle ends the run → "GAME OVER" overlay shows score
- [ ] Pressing Space from Game Over starts a fresh run
- [ ] Aurora Stones (purple/green squares) appear and increment the meter when collected
- [ ] Pressing Shift with ≥1 aurora performs a dash; the player passes through and smashes a destroyable obstacle
- [ ] Pressing P pauses; pressing P again resumes
- [ ] Switching to another tab and back resumes cleanly without a death-spiral
- [ ] Reloading the page preserves the high score

- [ ] **Step 23.5: Commit (if any tweaks were needed)**

If the smoke test surfaces a bug, fix it in the relevant module, re-run `cargo test --lib`, rebuild, and commit with `fix(<module>): <what>`.

If everything passes, tag Phase 1:

```bash
git tag phase1-greybox
```

---

## Phase 1 done. Next steps

- **Phase 2** plan (aurora-style art pass) — write next when this is merged.
- **Phase 3** plan (juice + audio) — write after Phase 2 plan.

Per spec §6 and the user's earlier decision, Phase 2 asset *authoring* can be parallelized with Phase 1 *coding*, but the **vertical-slice gate** (EDIE run + first obstacle + first floor tile, evaluated against the running greybox) must complete before the rest of Phase 2 art is produced. Phase 3 wiring depends on Phase 1 being complete.
