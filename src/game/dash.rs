//! Dash: short invulnerable burst that smashes destroyable obstacles.
//! See spec §3.5.

/// Maximum time one aurora stone can fuel a dash. Held past this it auto-ends.
pub const DASH_MAX_DURATION: f32 = 0.850;
/// Minimum dash even if released immediately -- gives a usable invincibility
/// window on a single tap.
pub const DASH_MIN_DURATION: f32 = 0.220;
/// Kept for compatibility with older code paths that reference
/// `DASH_DURATION` directly. Represents a "typical" mid-length dash.
pub const DASH_DURATION: f32 = 0.450;
pub const DASH_COOLDOWN: f32 = 0.400;
pub const DASH_SPEED_MULT: f32 = 1.45;
pub const DASH_COST: u32 = 1;
pub const SLOWMO_DURATION: f32 = 0.200;
pub const SLOWMO_SCALE: f32 = 0.60;

#[derive(Debug, Default, Clone)]
pub struct DashState {
    pub aurora: u32,
    /// Fuel remaining for the active dash -- counts down whenever dash is
    /// active. When it hits zero the dash auto-ends.
    pub active_remaining: f32,
    pub cooldown_remaining: f32,
    pub slowmo_remaining: f32,
    /// True while the dash key is still being held (or touch button held).
    pub holding: bool,
    /// Time elapsed since the current dash started. Used to enforce
    /// DASH_MIN_DURATION so that a tap still yields a short usable dash.
    pub time_since_start: f32,
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
            self.active_remaining = DASH_MAX_DURATION;
            self.time_since_start = 0.0;
            self.holding = true;
            DashRequest::Started
        }
    }

    /// Call when the dash key/button is released. Schedules the dash to end
    /// once at least DASH_MIN_DURATION has elapsed.
    pub fn release(&mut self) {
        self.holding = false;
    }

    pub fn trigger_slowmo(&mut self) {
        self.slowmo_remaining = SLOWMO_DURATION;
    }

    pub fn time_scale(&self) -> f32 {
        if self.slowmo_remaining > 0.0 {
            SLOWMO_SCALE
        } else {
            1.0
        }
    }

    pub fn speed_mult(&self) -> f32 {
        if self.is_active() {
            DASH_SPEED_MULT
        } else {
            1.0
        }
    }

    pub fn update(&mut self, real_dt: f32) {
        if self.active_remaining > 0.0 {
            self.active_remaining -= real_dt;
            self.time_since_start += real_dt;
            // If the player has released and we've crossed the minimum
            // dash window, end now instead of draining the full max.
            let released_past_min =
                !self.holding && self.time_since_start >= DASH_MIN_DURATION;
            if self.active_remaining <= 0.0 || released_past_min {
                self.active_remaining = 0.0;
                self.cooldown_remaining = DASH_COOLDOWN;
                self.holding = false;
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
    fn held_dash_runs_for_max_duration() {
        let mut d = DashState::new();
        d.add_aurora(1);
        d.try_start();
        assert!(d.is_invulnerable());
        // Hold -> runs the full max window.
        let steps = ((DASH_MAX_DURATION / DT).round() as u32) + 2;
        for _ in 0..steps {
            d.update(DT);
        }
        assert!(!d.is_invulnerable(), "dash should have ended");
        assert!(d.cooldown_remaining > 0.0);
    }

    #[test]
    fn released_tap_respects_min_duration() {
        let mut d = DashState::new();
        d.add_aurora(1);
        d.try_start();
        d.release();
        // Before min duration -> still active
        let half = ((DASH_MIN_DURATION * 0.4 / DT).round() as u32).max(1);
        for _ in 0..half {
            d.update(DT);
        }
        assert!(d.is_invulnerable());
        // After min duration -> ends
        let rest = ((DASH_MIN_DURATION / DT).round() as u32) + 2;
        for _ in 0..rest {
            d.update(DT);
        }
        assert!(!d.is_invulnerable());
    }

    #[test]
    fn cannot_chain_dash_during_cooldown() {
        let mut d = DashState::new();
        d.add_aurora(2);
        d.try_start();
        d.release();
        let steps = ((DASH_MIN_DURATION / DT).round() as u32) + 4;
        for _ in 0..steps {
            d.update(DT);
        }
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
