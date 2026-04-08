//! Visual effects — particles, score popups, screen shake, hit flash.
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

#[derive(Debug, Default)]
pub struct Effects {
    pub particles: Vec<Particle>,
    pub popups: Vec<ScorePopup>,
    pub shake_remaining: f32,
    pub shake_intensity: f32,
    pub hit_flash: f32,
    pub flash_max: f32,
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
        if self.shake_remaining <= 0.0 {
            return (0.0, 0.0);
        }
        let amp = self.shake_intensity * (self.shake_remaining / 0.25).min(1.0);
        let ox = (seed * 47.0).sin() * amp;
        let oy = (seed * 53.0).cos() * amp;
        (ox, oy)
    }
}
