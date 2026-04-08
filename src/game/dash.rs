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
        let steps = (DASH_DURATION / DT).round() as u32;
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
