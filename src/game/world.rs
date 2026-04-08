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
    /// Fractional score accumulator (px scrolled / 4) — flushed when ≥1.
    score_accum: f32,
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
            score_accum: 0.0,
        }
    }

    pub fn current_speed(&self) -> f32 {
        speed_for_score(self.score.current) * self.dash.speed_mult()
    }

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

    pub fn update(&mut self, real_dt: f32) -> RunOutcome {
        if matches!(self.player.state, PlayerState::Hit) {
            return RunOutcome::Died;
        }

        let scale = self.dash.time_scale();
        let sim_dt = real_dt * scale;

        self.elapsed += sim_dt;
        self.dash.update(real_dt);
        self.player.update(sim_dt);

        let speed = self.current_speed();
        self.background.update(sim_dt, speed);
        self.obstacles
            .update(sim_dt, speed, self.score.current, &mut self.rng);
        self.pickups.update(sim_dt, speed, &mut self.rng);

        // Accumulate fractional score (1 point per 4 px scrolled).
        self.score_accum += speed * sim_dt / 4.0;
        if self.score_accum >= 1.0 {
            let whole = self.score_accum.floor() as u32;
            self.score.add(whole);
            self.score_accum -= whole as f32;
        }

        let player_box = self.player.hitbox();
        let collected = self.pickups.collisions_with(&player_box);
        for &i in &collected {
            self.pickups.stones[i].collected = true;
            self.dash.add_aurora(1);
            self.score.add(50);
        }

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
        for _ in 0..10 {
            w.update(DT);
        }
        assert!(w.player.y < start_y);
    }

    #[test]
    fn pickup_grants_aurora_and_score() {
        let mut w = fresh_world();
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
        let mut o = crate::game::obstacles::Obstacle::new(
            ObstacleKind::CoiledCable,
            pbox.x,
        );
        o.y = pbox.y;
        w.obstacles.obstacles.push(o);
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
        let mut o = crate::game::obstacles::Obstacle::new(
            ObstacleKind::CoiledCable,
            pbox.x,
        );
        o.y = pbox.y;
        w.obstacles.obstacles.push(o);
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
        let mut o = crate::game::obstacles::Obstacle::new(
            ObstacleKind::ChargingDock,
            pbox.x,
        );
        o.y = pbox.y;
        w.obstacles.obstacles.push(o);
        let outcome = w.update(DT);
        assert_eq!(outcome, RunOutcome::Continuing);
        assert!(w.obstacles.obstacles[0].alive);
    }
}
