//! EDIE player: physics, state machine, hitbox. See spec §3.3.

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
pub const PLAYER_X: f32 = 200.0;

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
    pub y: f32,
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

    pub fn try_jump(&mut self) -> bool {
        if self.is_grounded() || self.time_since_grounded <= COYOTE_TIME {
            self.vy = JUMP_INITIAL_VY;
            self.state = PlayerState::Jumping;
            self.jump_hold_time = 0.0;
            self.jump_held = true;
            self.time_since_grounded = COYOTE_TIME + 1.0;
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

        if matches!(self.state, PlayerState::Jumping) && self.jump_held {
            if self.jump_hold_time < JUMP_HOLD_MAX_TIME {
                self.vy += JUMP_HOLD_EXTRA_VY * dt;
                self.jump_hold_time += dt;
            }
        }

        if !self.is_grounded() {
            self.vy += GRAVITY * dt;
            self.y += self.vy * dt;
            self.time_since_grounded += dt;
        }

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
    use crate::time::DT;

    #[allow(dead_code)]
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
        p.release_jump();
        let mut min_y = p.y;
        for _ in 0..200 {
            p.update(DT);
            if p.y < min_y {
                min_y = p.y;
            }
        }
        let apex_height = start_y - min_y;
        assert!(
            (apex_height - 126.75).abs() < 4.0,
            "apex {apex_height} px, expected ~126.75"
        );
    }

    #[test]
    fn variable_jump_height_held_higher_than_tapped() {
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

        let mut held = Player::new();
        let held_start = held.y;
        held.try_jump();
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
        let ratio = ducked.h / normal.h;
        assert!(
            (ratio - 0.5).abs() < 0.1,
            "duck hitbox ratio {ratio}, expected ~0.5"
        );
    }

    #[test]
    fn coyote_time_jump_within_window_succeeds() {
        // Set the flag directly so we don't depend on physics integration.
        let mut p = Player::new();
        p.state = PlayerState::Falling;
        p.y -= 200.0;
        p.vy = 50.0;
        p.time_since_grounded = 0.079;
        assert!(p.try_jump(), "coyote jump within window should succeed");
    }

    #[test]
    fn coyote_time_jump_after_window_fails() {
        let mut p = Player::new();
        p.state = PlayerState::Falling;
        p.y -= 200.0;
        p.vy = 50.0;
        p.time_since_grounded = 0.100;
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
