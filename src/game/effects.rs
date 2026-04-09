//! Visual effects - particles, score popups, screen shake, hit flash.
//! Purely visual, no gameplay impact.

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Debug, Clone)]
pub struct ScorePopup {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub life: f32,
    pub max_life: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Debug, Default, Clone)]
pub struct Effects {
    pub particles: Vec<Particle>,
    pub popups: Vec<ScorePopup>,
    pub shake_remaining: f32,
    pub shake_intensity: f32,
    pub hit_flash: f32,
    pub flash_max: f32,
    pub tier_banner: Option<TierBanner>,
    /// Queue of SFX cues drained by the main loop each frame.
    pub sfx_queue: Vec<SfxCue>,
    /// Two-pulse death shake, separate from continuous shake.
    pub death_shake: Option<DeathShake>,
    /// Metal-Slug-style stage transition wipe.
    pub stage_wipe: Option<StageWipe>,
}

#[derive(Debug, Clone)]
pub struct StageWipe {
    pub remaining: f32,
    pub total: f32,
    pub new_stage_name: String,
}

#[derive(Debug, Clone, Copy)]
pub struct DeathShake {
    pub remaining: f32,
    pub total: f32,
    pub intensity: f32,
}

const DEATH_SHAKE_PULSE: f32 = 0.10;
const DEATH_SHAKE_GAP: f32 = 0.08;
const DEATH_SHAKE_TOTAL: f32 = DEATH_SHAKE_PULSE * 2.0 + DEATH_SHAKE_GAP; // 0.28s

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfxCue {
    Jump,
    Hit,
    Pickup,
    Dash,
    Smash,
    Heart,
}

#[derive(Debug, Clone)]
pub struct TierBanner {
    pub text: String,
    pub remaining: f32,
    pub total: f32,
}

impl Effects {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, dt: f32) {
        // Advance particles
        for p in &mut self.particles {
            p.vy += 600.0 * dt; // gravity
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);

        // Advance popups (float up, fade)
        for pop in &mut self.popups {
            pop.y -= 50.0 * dt;
            pop.life -= dt;
        }
        self.popups.retain(|p| p.life > 0.0);

        // Shake decay
        if self.shake_remaining > 0.0 {
            self.shake_remaining = (self.shake_remaining - dt).max(0.0);
        }

        // Flash decay
        if self.hit_flash > 0.0 {
            self.hit_flash = (self.hit_flash - dt).max(0.0);
        }

        // Tier banner decay
        if let Some(b) = &mut self.tier_banner {
            b.remaining -= dt;
            if b.remaining <= 0.0 {
                self.tier_banner = None;
            }
        }

        // Death shake decay
        if let Some(ds) = &mut self.death_shake {
            ds.remaining -= dt;
            if ds.remaining <= 0.0 {
                self.death_shake = None;
            }
        }

        // Stage wipe decay
        if let Some(sw) = &mut self.stage_wipe {
            sw.remaining -= dt;
            if sw.remaining <= 0.0 {
                self.stage_wipe = None;
            }
        }
    }

    pub fn start_stage_wipe(&mut self, name: String, duration: f32) {
        self.stage_wipe = Some(StageWipe {
            remaining: duration,
            total: duration,
            new_stage_name: name,
        });
    }

    pub fn is_stage_wiping(&self) -> bool {
        self.stage_wipe.is_some()
    }

    pub fn trigger_death_shake(&mut self) {
        self.death_shake = Some(DeathShake {
            remaining: DEATH_SHAKE_TOTAL,
            total: DEATH_SHAKE_TOTAL,
            intensity: 12.0,
        });
        // Cancel any in-progress continuous shake so it doesn't blend.
        self.shake_remaining = 0.0;
    }

    pub fn push_tier_banner(&mut self, text: String, duration: f32) {
        self.tier_banner = Some(TierBanner {
            text,
            remaining: duration,
            total: duration,
        });
    }

    pub fn sfx(&mut self, cue: SfxCue) {
        self.sfx_queue.push(cue);
    }

    /// Spawn a burst of dust particles at the given position.
    pub fn dust_burst(&mut self, x: f32, y: f32, count: u32) {
        for i in 0..count {
            let angle = -std::f32::consts::PI * (0.2 + 0.6 * (i as f32 / count as f32));
            let speed = 60.0 + 40.0 * (i as f32 * 0.37).sin().abs();
            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: 0.4,
                max_life: 0.4,
                size: 3.0,
                r: 0.85,
                g: 0.82,
                b: 0.75,
            });
        }
    }

    /// Debris when dashing through an obstacle.
    pub fn smash_burst(&mut self, x: f32, y: f32) {
        for i in 0..10u32 {
            let angle = (i as f32 / 10.0) * std::f32::consts::PI * 2.0;
            let speed = 140.0 + (i % 3) as f32 * 30.0;
            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed - 80.0,
                life: 0.45,
                max_life: 0.45,
                size: 4.0,
                r: 0.9,
                g: 0.6,
                b: 0.2,
            });
        }
    }

    pub fn hit_burst(&mut self, x: f32, y: f32) {
        for i in 0..14u32 {
            let angle = (i as f32 / 14.0) * std::f32::consts::PI * 2.0;
            let speed = 180.0;
            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: 0.6,
                max_life: 0.6,
                size: 5.0,
                r: 0.9,
                g: 0.15,
                b: 0.2,
            });
        }
    }

    pub fn score_popup(&mut self, x: f32, y: f32, value: u32, color: (f32, f32, f32)) {
        self.popups.push(ScorePopup {
            x,
            y,
            text: format!("+{}", value),
            life: 0.9,
            max_life: 0.9,
            r: color.0,
            g: color.1,
            b: color.2,
        });
    }

    pub fn shake(&mut self, intensity: f32, duration: f32) {
        self.shake_intensity = self.shake_intensity.max(intensity);
        self.shake_remaining = self.shake_remaining.max(duration);
    }

    pub fn flash(&mut self, intensity: f32, duration: f32) {
        self.hit_flash = self.hit_flash.max(duration);
        self.flash_max = self.flash_max.max(intensity);
    }

    /// Returns (offset_x, offset_y) to add to camera during shake.
    pub fn shake_offset(&self, seed: f32) -> (f32, f32) {
        // Death shake: two distinct decaying pulses, no jitter in the gap.
        if let Some(ds) = &self.death_shake {
            let elapsed = ds.total - ds.remaining;
            // Figure out which pulse window we're in
            let in_first = elapsed < DEATH_SHAKE_PULSE;
            let in_second = elapsed >= DEATH_SHAKE_PULSE + DEATH_SHAKE_GAP
                && elapsed < DEATH_SHAKE_PULSE * 2.0 + DEATH_SHAKE_GAP;
            if in_first || in_second {
                // Local progress within the pulse (0..1), decaying
                let local = if in_first {
                    elapsed / DEATH_SHAKE_PULSE
                } else {
                    (elapsed - DEATH_SHAKE_PULSE - DEATH_SHAKE_GAP) / DEATH_SHAKE_PULSE
                };
                let decay = (1.0 - local).powi(2);
                // Direction: first pulse shoves one way, second pulse the other
                let dir = if in_first { 1.0 } else { -1.0 };
                let amp = ds.intensity * decay;
                return (dir * amp, (-0.4) * dir * amp);
            }
            return (0.0, 0.0);
        }

        // Continuous shake for non-death hits
        if self.shake_remaining <= 0.0 {
            return (0.0, 0.0);
        }
        let amp = self.shake_intensity * (self.shake_remaining / 0.25).min(1.0);
        let ox = (seed * 47.0).sin() * amp;
        let oy = (seed * 53.0).cos() * amp;
        (ox, oy)
    }
}
