//! Boss mode: green corona virus rain. Triggered at score >= BOSS_TRIGGER_SCORE.
//!
//! Mechanics:
//! - Map "breaks" and the normal obstacle stream stops
//! - Viruses fall vertically from above at random x positions
//! - Player moves horizontally (Left/Right) across the screen bottom
//! - Survive 60 seconds -> Ending
//! - Hit a virus -> death

use crate::game::player::{Aabb, GROUND_Y, PLAYER_H, PLAYER_W};
use rand::rngs::SmallRng;
use rand::Rng;

pub const BOSS_DURATION: f32 = 60.0;
pub const VIRUS_W: f32 = 40.0;
pub const VIRUS_H: f32 = 40.0;
pub const PLAYER_SIDE_SPEED: f32 = 420.0;
/// Left/right bounds for the player inside a 1280 logical window.
pub const PLAYER_MIN_X: f32 = 40.0;
pub const PLAYER_MAX_X: f32 = 1280.0 - PLAYER_W - 40.0;

#[derive(Debug, Clone)]
pub struct Virus {
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub alive: bool,
}

impl Virus {
    pub fn hitbox(&self) -> Aabb {
        Aabb { x: self.x + 4.0, y: self.y + 4.0, w: VIRUS_W - 8.0, h: VIRUS_H - 8.0 }
    }
}

pub struct BossWorld {
    pub remaining: f32,
    pub player_x: f32,
    pub viruses: Vec<Virus>,
    pub spawn_timer: f32,
    pub elapsed: f32,
}

impl BossWorld {
    pub fn new() -> Self {
        Self {
            remaining: BOSS_DURATION,
            player_x: 640.0 - PLAYER_W * 0.5,
            viruses: Vec::new(),
            spawn_timer: 0.3,
            elapsed: 0.0,
        }
    }

    /// Returns true if time ran out (player survived -> Ending).
    pub fn update(
        &mut self,
        dt: f32,
        input_dx: f32, // -1, 0, or +1
        rng: &mut SmallRng,
    ) -> BossOutcome {
        self.elapsed += dt;
        self.remaining -= dt;

        // Player horizontal movement
        self.player_x += input_dx * PLAYER_SIDE_SPEED * dt;
        self.player_x = self.player_x.clamp(PLAYER_MIN_X, PLAYER_MAX_X);

        // Spawn viruses at increasing density over time
        self.spawn_timer -= dt;
        let spawn_interval = {
            let progress = (self.elapsed / BOSS_DURATION).clamp(0.0, 1.0);
            0.55 - progress * 0.38 // 0.55s at start, 0.17s at end
        };
        if self.spawn_timer <= 0.0 {
            let x = rng.gen_range(0.0..=(1280.0 - VIRUS_W));
            let vy = rng.gen_range(160.0..260.0) + self.elapsed * 3.0;
            self.viruses.push(Virus { x, y: -VIRUS_H, vy, alive: true });
            self.spawn_timer = spawn_interval;
        }

        // Advance viruses
        for v in &mut self.viruses {
            v.y += v.vy * dt;
        }
        self.viruses.retain(|v| v.alive && v.y < GROUND_Y + 20.0);

        // Collision: player occupies [player_x, player_x + PLAYER_W] x bottom band
        let player_box = Aabb {
            x: self.player_x + 8.0,
            y: GROUND_Y - PLAYER_H + 8.0,
            w: PLAYER_W - 16.0,
            h: PLAYER_H - 16.0,
        };
        for v in &self.viruses {
            if v.alive && v.hitbox().intersects(&player_box) {
                return BossOutcome::Hit;
            }
        }

        if self.remaining <= 0.0 {
            return BossOutcome::Survived;
        }
        BossOutcome::Continuing
    }
}

impl Default for BossWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BossOutcome {
    Continuing,
    Hit,
    Survived,
}
