//! Boss mode: central Mungchi boss + falling virus rain + laser attacks.
//! Triggered at score >= BOSS_TRIGGER_SCORE.

use crate::game::player::{Aabb, GROUND_Y};
use rand::rngs::SmallRng;
use rand::Rng;

pub const BOSS_DURATION: f32 = 60.0;
pub const BOSS_PHASE2_DURATION: f32 = 30.0;
pub const BOSS_INTRO_DURATION: f32 = 3.5;
pub const VIRUS_W: f32 = 48.0;
pub const VIRUS_H: f32 = 48.0;
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

// Laser — rarer, longer warn, narrower beam.
pub const LASER_COOLDOWN: f32 = 7.0;
pub const LASER_WARN: f32 = 1.8;
pub const LASER_FIRE: f32 = 0.5;
pub const LASER_WIDTH: f32 = 70.0;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossPattern {
    Rain,
    DiagonalVolley,
    Spiral,
    SweepLaser,
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
    pub pattern: BossPattern,
    pub pattern_timer: f32,
    pub sweep_laser_x: f32,
    pub sweep_laser_active: bool,
    pub sweep_laser_dir: f32,
    /// 1 = green Mungchi boss (60s), 2 = hardcore purple boss (30s)
    pub phase: u8,
    /// Brief interlude (e.g. 1.5s) between phases where player sees boss
    /// "shatter" before phase 2 spawns. Zero outside of interlude.
    pub interlude_remaining: f32,
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
            laser_cooldown: 3.0,
            pattern: BossPattern::Rain,
            pattern_timer: 8.0,
            sweep_laser_x: 0.0,
            sweep_laser_active: false,
            sweep_laser_dir: 1.0,
            phase: 1,
            interlude_remaining: 0.0,
        }
    }

    pub fn boss_center(&self) -> (f32, f32) {
        let bob = (self.boss_bob_t * 2.2).sin() * 6.0;
        (BOSS_X, BOSS_Y_BASE + bob)
    }

    pub fn update(&mut self, dt: f32, input_dx: f32, rng: &mut SmallRng) -> BossOutcome {
        self.elapsed += dt;
        self.boss_bob_t += dt;

        // Interlude between phases: world is frozen aside from the player.
        if self.interlude_remaining > 0.0 {
            self.interlude_remaining -= dt;
            self.player_x += input_dx * PLAYER_SIDE_SPEED * dt;
            self.player_x = self.player_x.clamp(PLAYER_MIN_X, PLAYER_MAX_X);
            if self.interlude_remaining <= 0.0 {
                // Enter phase 2: hardcore purple boss.
                self.phase = 2;
                self.remaining = BOSS_PHASE2_DURATION;
                self.viruses.clear();
                self.laser = None;
                self.sweep_laser_active = false;
                self.pattern = BossPattern::Spiral;
                self.pattern_timer = 4.0;
                self.spawn_timer = 0.2;
                self.laser_cooldown = 3.0;
                self.elapsed = 0.0;
            }
            return BossOutcome::Continuing;
        }

        self.remaining -= dt;

        // Player horizontal movement + facing
        if input_dx < 0.0 {
            self.player_facing = Facing::Left;
        } else if input_dx > 0.0 {
            self.player_facing = Facing::Right;
        }
        self.player_x += input_dx * PLAYER_SIDE_SPEED * dt;
        self.player_x = self.player_x.clamp(PLAYER_MIN_X, PLAYER_MAX_X);

        let progress = (self.elapsed / BOSS_DURATION).clamp(0.0, 1.0);

        // Rotate attack pattern. Phase 1 = 8s per pattern, phase 2 = 4s.
        self.pattern_timer -= dt;
        if self.pattern_timer <= 0.0 {
            self.pattern = match self.pattern {
                BossPattern::Rain => BossPattern::DiagonalVolley,
                BossPattern::DiagonalVolley => BossPattern::Spiral,
                BossPattern::Spiral => BossPattern::SweepLaser,
                BossPattern::SweepLaser => BossPattern::Rain,
            };
            self.pattern_timer = if self.phase == 2 { 4.0 } else { 8.0 };
            self.sweep_laser_active = false;
        }

        let boss_color = if self.phase == 2 {
            VirusColor::Purple
        } else {
            VirusColor::Green
        };
        let p2 = self.phase == 2;
        let ps = self.phase_scale();

        // Virus spawn — pattern-aware, scaled by phase.
        self.spawn_timer -= dt;
        match self.pattern {
            BossPattern::Rain => {
                let spawn_interval = (0.38 - progress * 0.18) / ps; // faster in p2
                if self.spawn_timer <= 0.0 {
                    let count = if p2 {
                        3
                    } else if progress > 0.6 {
                        2
                    } else {
                        1
                    };
                    for _ in 0..count {
                        let x = rng.gen_range(0.0..=(1280.0 - VIRUS_W));
                        let vy = (rng.gen_range(260.0..360.0) + self.elapsed * 4.0) * ps;
                        let vx = rng.gen_range(-30.0..30.0) * ps;
                        let color = if p2 {
                            boss_color
                        } else if rng.gen_bool(0.5) {
                            VirusColor::Green
                        } else {
                            VirusColor::Purple
                        };
                        self.viruses.push(Virus { x, y: -VIRUS_H, vy, vx, color, alive: true });
                    }
                    self.spawn_timer = spawn_interval.max(0.08);
                }
            }
            BossPattern::DiagonalVolley => {
                if self.spawn_timer <= 0.0 {
                    let from_left = ((self.elapsed * 0.8) as u32) % 2 == 0;
                    // Phase 2: dual-stream (both sides simultaneously)
                    let sides: &[bool] = if p2 { &[true, false] } else { &[from_left] };
                    let per = if p2 { 5u32 } else { 4u32 };
                    for &left in sides {
                        for i in 0..per {
                            let x = if left {
                                -40.0 - (i as f32) * 20.0
                            } else {
                                1280.0 + (i as f32) * 20.0
                            };
                            let y = -40.0 - (i as f32) * 30.0;
                            let base_vx = 260.0 * ps;
                            let vx = if left { base_vx } else { -base_vx };
                            let vy = 260.0 * ps;
                            let color = if p2 {
                                boss_color
                            } else if rng.gen_bool(0.5) {
                                VirusColor::Green
                            } else {
                                VirusColor::Purple
                            };
                            self.viruses.push(Virus { x, y, vy, vx, color, alive: true });
                        }
                    }
                    self.spawn_timer = if p2 { 0.55 } else { 0.9 };
                }
            }
            BossPattern::Spiral => {
                if self.spawn_timer <= 0.0 {
                    let (cx, cy) = self.boss_center();
                    let base_angle = self.elapsed * if p2 { 4.2 } else { 2.8 };
                    let arms: u32 = if p2 { 12 } else { 8 };
                    let speed = 280.0 * ps;
                    for i in 0..arms {
                        let a = base_angle + (i as f32) * std::f32::consts::TAU / (arms as f32);
                        let vx = a.cos() * speed;
                        let vy = a.sin() * speed;
                        let color = if p2 {
                            boss_color
                        } else if i % 2 == 0 {
                            VirusColor::Green
                        } else {
                            VirusColor::Purple
                        };
                        self.viruses.push(Virus {
                            x: cx - VIRUS_W * 0.5,
                            y: cy - VIRUS_H * 0.5,
                            vy,
                            vx,
                            color,
                            alive: true,
                        });
                    }
                    self.spawn_timer = if p2 { 0.22 } else { 0.35 };
                }
            }
            BossPattern::SweepLaser => {
                if self.spawn_timer <= 0.0 {
                    let count = if p2 { 3 } else { 1 };
                    for _ in 0..count {
                        let x = rng.gen_range(0.0..=(1280.0 - VIRUS_W));
                        let vy = rng.gen_range(220.0..320.0) * ps;
                        self.viruses.push(Virus {
                            x,
                            y: -VIRUS_H,
                            vy,
                            vx: 0.0,
                            color: if p2 { VirusColor::Purple } else { VirusColor::Green },
                            alive: true,
                        });
                    }
                    self.spawn_timer = if p2 { 0.35 } else { 0.7 };
                }
                if !self.sweep_laser_active {
                    self.sweep_laser_active = true;
                    self.sweep_laser_x = 100.0;
                    self.sweep_laser_dir = 1.0;
                }
            }
        }

        // Sweep laser movement (phase 2 scans faster)
        if self.sweep_laser_active {
            let scan_speed = if p2 { 520.0 } else { 320.0 };
            self.sweep_laser_x += self.sweep_laser_dir * scan_speed * dt;
            if self.sweep_laser_x > 1180.0 {
                self.sweep_laser_dir = -1.0;
            }
            if self.sweep_laser_x < 100.0 {
                self.sweep_laser_dir = 1.0;
            }
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

        // Sweep laser collision
        if self.sweep_laser_active {
            let lx_min = self.sweep_laser_x - 30.0;
            let lx_max = self.sweep_laser_x + 30.0;
            if player_box.x + player_box.w > lx_min && player_box.x < lx_max {
                return BossOutcome::Hit;
            }
        }

        if self.remaining <= 0.0 {
            if self.phase == 1 {
                // Green boss down — start interlude into hardcore phase 2.
                self.interlude_remaining = 1.5;
                self.viruses.clear();
                self.laser = None;
                self.sweep_laser_active = false;
                return BossOutcome::Continuing;
            }
            return BossOutcome::Survived;
        }
        BossOutcome::Continuing
    }

    /// Hardcore phase multiplier for the active pattern code paths below.
    fn phase_scale(&self) -> f32 {
        if self.phase == 2 {
            1.7
        } else {
            1.0
        }
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
