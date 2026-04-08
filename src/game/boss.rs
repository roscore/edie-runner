//! Boss mode: central corona boss + falling virus rain + laser attacks.
//! Triggered at score >= BOSS_TRIGGER_SCORE.

use crate::game::player::{Aabb, GROUND_Y};
use rand::rngs::SmallRng;
use rand::Rng;

pub const BOSS_DURATION: f32 = 60.0;
pub const BOSS_INTRO_DURATION: f32 = 3.5;
pub const VIRUS_W: f32 = 60.0;
pub const VIRUS_H: f32 = 60.0;
pub const PLAYER_SIDE_SPEED: f32 = 520.0;

// Visual EDIE size in boss mode (must match draw_boss_mode).
pub const BOSS_EDIE_W: f32 = 56.0;
pub const BOSS_EDIE_H: f32 = 48.0;
pub const BOSS_EDIE_BOTTOM_INSET: f32 = 16.0;

pub const PLAYER_MIN_X: f32 = 40.0;
pub const PLAYER_MAX_X: f32 = 1280.0 - BOSS_EDIE_W - 40.0;

// Central boss
pub const BOSS_X: f32 = 640.0;
pub const BOSS_Y_BASE: f32 = 110.0;
pub const BOSS_SIZE: f32 = 180.0;

// Laser
pub const LASER_COOLDOWN: f32 = 4.0;
pub const LASER_WARN: f32 = 1.0;
pub const LASER_FIRE: f32 = 0.7;
pub const LASER_WIDTH: f32 = 96.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirusColor {
    Green,
    Purple,
}

#[derive(Debug, Clone)]
pub struct Virus {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub color: VirusColor,
    pub alive: bool,
}

impl Virus {
    pub fn hitbox(&self) -> Aabb {
        Aabb { x: self.x + 8.0, y: self.y + 8.0, w: VIRUS_W - 16.0, h: VIRUS_H - 16.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaserPhase {
    Warn,
    Firing,
}

#[derive(Debug, Clone)]
pub struct Laser {
    pub target_x: f32,
    pub phase: LaserPhase,
    pub remaining: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    Left,
    Right,
}

pub struct BossWorld {
    pub remaining: f32,
    pub player_x: f32,
    pub player_facing: Facing,
    pub viruses: Vec<Virus>,
    pub spawn_timer: f32,
    pub elapsed: f32,
    pub boss_bob_t: f32,
    pub laser: Option<Laser>,
    pub laser_cooldown: f32,
}

impl BossWorld {
    pub fn new() -> Self {
        Self {
            remaining: BOSS_DURATION,
            player_x: 640.0 - BOSS_EDIE_W * 0.5,
            player_facing: Facing::Right,
            viruses: Vec::new(),
            spawn_timer: 0.25,
            elapsed: 0.0,
            boss_bob_t: 0.0,
            laser: None,
            laser_cooldown: 3.0, // first laser ~3s in
        }
    }

    pub fn boss_center(&self) -> (f32, f32) {
        let bob = (self.boss_bob_t * 2.2).sin() * 6.0;
        (BOSS_X, BOSS_Y_BASE + bob)
    }

    pub fn update(&mut self, dt: f32, input_dx: f32, rng: &mut SmallRng) -> BossOutcome {
        self.elapsed += dt;
        self.remaining -= dt;
        self.boss_bob_t += dt;

        // Player horizontal movement + facing
        if input_dx < 0.0 {
            self.player_facing = Facing::Left;
        } else if input_dx > 0.0 {
            self.player_facing = Facing::Right;
        }
        self.player_x += input_dx * PLAYER_SIDE_SPEED * dt;
        self.player_x = self.player_x.clamp(PLAYER_MIN_X, PLAYER_MAX_X);

        let progress = (self.elapsed / BOSS_DURATION).clamp(0.0, 1.0);

        // Virus spawn — much denser, bigger, faster
        self.spawn_timer -= dt;
        let spawn_interval = 0.30 - progress * 0.24; // 0.30s -> 0.06s
        if self.spawn_timer <= 0.0 {
            let count = if progress > 0.66 {
                5
            } else if progress > 0.33 {
                4
            } else {
                2
            };
            for _ in 0..count {
                let x = rng.gen_range(0.0..=(1280.0 - VIRUS_W));
                // Faster fall speeds, escalate over time
                let vy = rng.gen_range(320.0..480.0) + self.elapsed * 9.0;
                let vx = rng.gen_range(-60.0..60.0);
                let color = if rng.gen_bool(0.5) {
                    VirusColor::Green
                } else {
                    VirusColor::Purple
                };
                self.viruses.push(Virus { x, y: -VIRUS_H, vy, vx, color, alive: true });
            }
            self.spawn_timer = spawn_interval.max(0.04);
        }

        // Advance viruses
        for v in &mut self.viruses {
            v.x += v.vx * dt;
            v.y += v.vy * dt;
        }
        self.viruses
            .retain(|v| v.alive && v.y < GROUND_Y + 40.0 && v.x > -80.0 && v.x < 1360.0);

        // Laser update
        if let Some(laser) = &mut self.laser {
            laser.remaining -= dt;
            if laser.remaining <= 0.0 {
                match laser.phase {
                    LaserPhase::Warn => {
                        laser.phase = LaserPhase::Firing;
                        laser.remaining = LASER_FIRE;
                    }
                    LaserPhase::Firing => {
                        self.laser = None;
                        // Tighter cooldown as fight progresses
                        self.laser_cooldown = LASER_COOLDOWN - progress * 1.5;
                    }
                }
            }
        } else {
            self.laser_cooldown -= dt;
            if self.laser_cooldown <= 0.0 {
                // Target where the player currently is
                let target = self.player_x + BOSS_EDIE_W * 0.5;
                self.laser = Some(Laser {
                    target_x: target.clamp(60.0, 1220.0),
                    phase: LaserPhase::Warn,
                    remaining: LASER_WARN,
                });
            }
        }

        // Player hitbox -- matches the visual sprite EXACTLY (this used to
        // be referencing PLAYER_H/GROUND_Y from the running game which made
        // the hitbox float ~80 px above the rendered EDIE).
        // EDIE is drawn at y = 400 - BOSS_EDIE_H - BOSS_EDIE_BOTTOM_INSET = 336,
        // size 56x48, with the player_x being the left edge of the visual.
        let edie_top = 400.0 - BOSS_EDIE_H - BOSS_EDIE_BOTTOM_INSET;
        let inset = 8.0;
        let player_box = Aabb {
            x: self.player_x + inset,
            y: edie_top + inset,
            w: BOSS_EDIE_W - 2.0 * inset,
            h: BOSS_EDIE_H - 2.0 * inset,
        };

        // Virus collision
        for v in &self.viruses {
            if v.alive && v.hitbox().intersects(&player_box) {
                return BossOutcome::Hit;
            }
        }

        // Laser collision (only during Firing)
        if let Some(laser) = &self.laser {
            if matches!(laser.phase, LaserPhase::Firing) {
                let lx_min = laser.target_x - LASER_WIDTH * 0.5;
                let lx_max = laser.target_x + LASER_WIDTH * 0.5;
                if player_box.x + player_box.w > lx_min && player_box.x < lx_max {
                    return BossOutcome::Hit;
                }
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
