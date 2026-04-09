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
        // Tighter than the 48x48 sprite: only the solid core counts.
        Aabb {
            x: self.x + 12.0,
            y: self.y + 12.0,
            w: VIRUS_W - 24.0,
            h: VIRUS_H - 24.0,
        }
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
    // ---- Phase 1 (green) patterns ----
    Rain,
    DiagonalVolley,
    Spiral,
    /// Boss telegraphs a wide vertical "danger band" then rains viruses
    /// everywhere EXCEPT inside a single "safe lane". Player moves into
    /// the safe lane to survive. Clear counter-play, no unblockable.
    SafeLaneBurst,
    // ---- Phase 2 (purple hardcore) patterns ----
    /// Horizontal bullets fired from both screen edges at multiple heights.
    /// Player dodges by stepping into gaps between the horizontal lanes.
    Crossfire,
    /// A row of bullets fired straight down at once, with exactly one or
    /// two slots missing. Player slides into the gap.
    PincerGrid,
    /// Boss paints a short 0.4s warning crosshair at the player's current
    /// x, then fires a fast single bolt. Repeats 3 times. Move to cancel.
    HunterBolts,
    /// Expanding concentric rings from boss center. Player stands in the
    /// gap between rings.
    RingPulse,
}

/// Bounds of the currently telegraphed safe lane in SafeLaneBurst.
#[derive(Debug, Clone, Copy)]
pub struct SafeLane {
    pub min_x: f32,
    pub max_x: f32,
    pub warn_remaining: f32,
    pub fire_remaining: f32,
}

/// Phase 2 Hunter Bolts state: telegraphs then fires fast bolts.
#[derive(Debug, Clone, Copy)]
pub struct HunterShot {
    pub target_x: f32,
    pub warn_remaining: f32, // >0 during telegraph; 0 = fired
    pub fired: bool,
}

/// Phase 2 Pincer Grid: a row of columns with a single gap index.
#[derive(Debug, Clone, Copy)]
pub struct PincerWave {
    pub gap_col: u32,
    pub cols: u32,
    pub warn_remaining: f32,
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
    pub safe_lane: Option<SafeLane>,
    pub phase: u8,
    pub interlude_remaining: f32,
    // Phase 2 specific state
    pub hunter_shots: Vec<HunterShot>,
    pub hunter_next: f32,
    pub hunter_fired_count: u32,
    pub pincer_wave: Option<PincerWave>,
    pub ring_next: f32,
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
            safe_lane: None,
            phase: 1,
            interlude_remaining: 0.0,
            hunter_shots: Vec::new(),
            hunter_next: 0.0,
            hunter_fired_count: 0,
            pincer_wave: None,
            ring_next: 0.0,
        }
    }

    pub fn boss_center(&self) -> (f32, f32) {
        let bob = (self.boss_bob_t * 2.2).sin() * 6.0;
        (BOSS_X, BOSS_Y_BASE + bob)
    }

    pub fn update(&mut self, dt: f32, input_dx: f32, rng: &mut SmallRng) -> BossOutcome {
        use rand::Rng as _;
        self.elapsed += dt;
        self.boss_bob_t += dt;

        // Interlude between phases: world is frozen aside from the player.
        if self.interlude_remaining > 0.0 {
            self.interlude_remaining -= dt;
            self.player_x += input_dx * PLAYER_SIDE_SPEED * dt;
            self.player_x = self.player_x.clamp(PLAYER_MIN_X, PLAYER_MAX_X);
            if self.interlude_remaining <= 0.0 {
                // Enter phase 2: hardcore purple boss with NEW patterns.
                self.phase = 2;
                self.remaining = BOSS_PHASE2_DURATION;
                self.viruses.clear();
                self.laser = None;
                self.safe_lane = None;
                self.hunter_shots.clear();
                self.hunter_next = 0.0;
                self.hunter_fired_count = 0;
                self.pincer_wave = None;
                self.ring_next = 0.0;
                self.pattern = BossPattern::Crossfire;
                self.pattern_timer = 5.0;
                self.spawn_timer = 0.2;
                self.laser_cooldown = 4.5;
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

        // Rotate attack pattern. Phase 1 and phase 2 have entirely
        // separate rotations so they feel like different fights.
        self.pattern_timer -= dt;
        if self.pattern_timer <= 0.0 {
            if self.phase == 2 {
                self.pattern = match self.pattern {
                    BossPattern::Crossfire => BossPattern::PincerGrid,
                    BossPattern::PincerGrid => BossPattern::HunterBolts,
                    BossPattern::HunterBolts => BossPattern::RingPulse,
                    BossPattern::RingPulse => BossPattern::Crossfire,
                    // Safety: if somehow a phase-1 pattern leaked in, jump to Crossfire.
                    _ => BossPattern::Crossfire,
                };
                self.pattern_timer = 5.0;
            } else {
                self.pattern = match self.pattern {
                    BossPattern::Rain => BossPattern::DiagonalVolley,
                    BossPattern::DiagonalVolley => BossPattern::Spiral,
                    BossPattern::Spiral => BossPattern::SafeLaneBurst,
                    BossPattern::SafeLaneBurst => BossPattern::Rain,
                    _ => BossPattern::Rain,
                };
                self.pattern_timer = 8.0;
            }
            // Clear pattern-specific state at rotation boundary
            self.safe_lane = None;
            self.hunter_shots.clear();
            self.hunter_next = 0.0;
            self.hunter_fired_count = 0;
            self.pincer_wave = None;
            self.ring_next = 0.0;
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
            BossPattern::SafeLaneBurst => {
                // Carve a telegraphed safe corridor on the ground, then
                // blanket everything outside it with viruses. Player reads
                // the warning and steps into the safe lane.
                if self.safe_lane.is_none() {
                    // Pick a lane centered somewhere, width depends on phase.
                    // Wider lanes + a longer telegraph so the player has
                    // enough wall-clock time to actually traverse the
                    // screen at PLAYER_SIDE_SPEED (520 px/s). The screen
                    // is 1280 px wide; even from corner-to-corner the
                    // worst case is ~2.5s. Phase-1 gets 2.4s warn, phase-2
                    // gets 1.8s -- both comfortably reachable.
                    let lane_w = if p2 { 240.0 } else { 320.0 };
                    let cx = rng.gen_range((lane_w * 0.5 + 40.0)..(1240.0 - lane_w * 0.5));
                    self.safe_lane = Some(SafeLane {
                        min_x: cx - lane_w * 0.5,
                        max_x: cx + lane_w * 0.5,
                        warn_remaining: if p2 { 1.8 } else { 2.4 },
                        fire_remaining: 0.0,
                    });
                }
                if let Some(lane) = &mut self.safe_lane {
                    if lane.warn_remaining > 0.0 {
                        lane.warn_remaining -= dt;
                        if lane.warn_remaining <= 0.0 {
                            // Kick off the volley
                            lane.fire_remaining = if p2 { 1.0 } else { 0.8 };
                            // Blast a wave of viruses: 20 drops spaced across
                            // the screen, SKIPPING the safe lane.
                            let step = 1280.0 / 20.0;
                            for i in 0..20u32 {
                                let cx = step * (i as f32 + 0.5);
                                if cx >= lane.min_x && cx <= lane.max_x {
                                    continue;
                                }
                                let color = if p2 {
                                    VirusColor::Purple
                                } else {
                                    VirusColor::Green
                                };
                                self.viruses.push(Virus {
                                    x: cx - VIRUS_W * 0.5,
                                    y: -VIRUS_H,
                                    vy: 380.0 * ps,
                                    vx: 0.0,
                                    color,
                                    alive: true,
                                });
                            }
                        }
                    } else if lane.fire_remaining > 0.0 {
                        lane.fire_remaining -= dt;
                        // Keep the lane visual on screen until the wave has
                        // fallen past the player.
                        if lane.fire_remaining <= 0.0 {
                            self.safe_lane = None;
                        }
                    }
                }
                // Small rain around the edges for flavor while lane is active
                self.spawn_timer -= dt;
                if self.spawn_timer <= 0.0 {
                    let x = rng.gen_range(0.0..=(1280.0 - VIRUS_W));
                    let vy = rng.gen_range(220.0..280.0) * ps;
                    self.viruses.push(Virus {
                        x,
                        y: -VIRUS_H,
                        vy,
                        vx: 0.0,
                        color: if p2 { VirusColor::Purple } else { VirusColor::Green },
                        alive: true,
                    });
                    self.spawn_timer = if p2 { 0.5 } else { 0.8 };
                }
            }

            // ================================================================
            // Phase 2 exclusive patterns
            // ================================================================
            BossPattern::Crossfire => {
                // Horizontal bullets from left and right edges at fixed
                // lanes. Gaps between lanes let the player survive with
                // careful left/right placement.
                self.spawn_timer -= dt;
                if self.spawn_timer <= 0.0 {
                    // Fire from a random vertical band near player height
                    let lanes = [260.0, 295.0, 330.0, 360.0];
                    let lane_y = lanes[rng.gen_range(0..lanes.len())];
                    let from_left = rng.gen_bool(0.5);
                    let count = 3u32;
                    for i in 0..count {
                        let x = if from_left {
                            -40.0 - (i as f32) * 40.0
                        } else {
                            1280.0 + (i as f32) * 40.0
                        };
                        let vx = if from_left { 500.0 } else { -500.0 };
                        self.viruses.push(Virus {
                            x,
                            y: lane_y,
                            vy: 0.0,
                            vx,
                            color: VirusColor::Purple,
                            alive: true,
                        });
                    }
                    self.spawn_timer = 0.55;
                }
            }
            BossPattern::PincerGrid => {
                // Periodically telegraph a vertical-drop grid with a gap.
                if self.pincer_wave.is_none() {
                    let cols = 9u32;
                    let gap = rng.gen_range(0..cols);
                    self.pincer_wave = Some(PincerWave {
                        cols,
                        gap_col: gap,
                        // 1.1s telegraph so the player can actually slide
                        // into the safe column from anywhere on screen.
                        warn_remaining: 1.1,
                    });
                }
                if let Some(wave) = &mut self.pincer_wave {
                    if wave.warn_remaining > 0.0 {
                        wave.warn_remaining -= dt;
                        if wave.warn_remaining <= 0.0 {
                            // Drop a bullet from every column except the gap
                            let step = 1280.0 / (wave.cols as f32);
                            for i in 0..wave.cols {
                                if i == wave.gap_col {
                                    continue;
                                }
                                let cx = step * (i as f32 + 0.5);
                                self.viruses.push(Virus {
                                    x: cx - VIRUS_W * 0.5,
                                    y: -VIRUS_H,
                                    vy: 460.0,
                                    vx: 0.0,
                                    color: VirusColor::Purple,
                                    alive: true,
                                });
                            }
                            self.pincer_wave = None;
                        }
                    }
                }
            }
            BossPattern::HunterBolts => {
                // Fire 3 crosshair-telegraphed bolts that lock onto the
                // player's x at the moment of telegraph start. Moving the
                // moment the crosshair appears guarantees a dodge.
                self.hunter_next -= dt;
                if self.hunter_next <= 0.0 && self.hunter_fired_count < 6 {
                    let tx = self.player_x + BOSS_EDIE_W * 0.5;
                    self.hunter_shots.push(HunterShot {
                        target_x: tx.clamp(60.0, 1220.0),
                        warn_remaining: 0.45,
                        fired: false,
                    });
                    self.hunter_next = 0.75;
                    self.hunter_fired_count += 1;
                }
                // Advance existing shots
                let mut to_fire: Vec<f32> = Vec::new();
                for shot in &mut self.hunter_shots {
                    if !shot.fired {
                        shot.warn_remaining -= dt;
                        if shot.warn_remaining <= 0.0 {
                            shot.fired = true;
                            to_fire.push(shot.target_x);
                        }
                    }
                }
                for tx in to_fire {
                    self.viruses.push(Virus {
                        x: tx - VIRUS_W * 0.5,
                        y: 160.0,
                        vy: 620.0,
                        vx: 0.0,
                        color: VirusColor::Purple,
                        alive: true,
                    });
                }
                self.hunter_shots.retain(|s| !s.fired || s.warn_remaining > -0.2);
            }
            BossPattern::RingPulse => {
                // Spawn an expanding 16-arm ring from boss center on a
                // cadence. Player stands between rings.
                self.ring_next -= dt;
                if self.ring_next <= 0.0 {
                    let (cx, cy) = self.boss_center();
                    let arms = 16u32;
                    let speed = 340.0;
                    for i in 0..arms {
                        let a = (i as f32) * std::f32::consts::TAU / (arms as f32);
                        let vx = a.cos() * speed;
                        let vy = a.sin() * speed;
                        self.viruses.push(Virus {
                            x: cx - VIRUS_W * 0.5,
                            y: cy - VIRUS_H * 0.5,
                            vy,
                            vx,
                            color: VirusColor::Purple,
                            alive: true,
                        });
                    }
                    self.ring_next = 1.2;
                }
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
        //
        // Generous insets so only EDIE's round body collides, not the
        // empty pixels around her. With 56x48 sprite and 14 px inset we
        // get a 28x20 hitbox that matches the visible blob.
        let edie_top = 400.0 - BOSS_EDIE_H - BOSS_EDIE_BOTTOM_INSET;
        let inset_x = 14.0;
        let inset_y = 14.0;
        let player_box = Aabb {
            x: self.player_x + inset_x,
            y: edie_top + inset_y,
            w: BOSS_EDIE_W - 2.0 * inset_x,
            h: BOSS_EDIE_H - 2.0 * inset_y,
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
            if self.phase == 1 {
                // Green boss down — start interlude into hardcore phase 2.
                self.interlude_remaining = 1.5;
                self.viruses.clear();
                self.laser = None;
                self.safe_lane = None;
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
