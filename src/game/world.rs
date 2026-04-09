//! Composite game state. Owns the only RNG.

use crate::game::background::Background;
use crate::game::dash::{DashRequest, DashState};
use crate::game::difficulty::{speed_for_score, stage_for_tier, tier_for_score, Stage};
use crate::game::effects::{Effects, SfxCue};
use crate::game::obstacles::{ObstacleField, ObstacleKind};
use crate::game::pickups::PickupField;
use crate::game::player::{Player, PlayerState, GROUND_Y, PLAYER_H, PLAYER_W, PLAYER_X};
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

pub const MAX_HP: u32 = 3;
pub const HP_INVULN_TIME: f32 = 1.0;

fn tier_banner_label(tier: u32) -> String {
    match tier {
        1 => "PANGYO STREET - SIDEWALK PATROL".to_string(),
        2 => "PANGYO STREET - VACUUM BOTS".to_string(),
        3 => "HIGHWAY - DEER SEASON".to_string(),
        4 => "HIGHWAY - INCOMING TRAFFIC".to_string(),
        5 => "HANYANG ERICA - ALICE3 ONLINE".to_string(),
        6 => "HANYANG ERICA - ALICE4 ENGAGED".to_string(),
        7 => "AEIROBOT OFFICE - HOME STRETCH".to_string(),
        8 => "AEIROBOT FACTORY - INFECTED".to_string(),
        9 => "FACTORY - CRITICAL INFECTION".to_string(),
        _ => format!("TIER {}", tier),
    }
}

pub struct World {
    pub player: Player,
    pub obstacles: ObstacleField,
    pub pickups: PickupField,
    pub dash: DashState,
    pub background: Background,
    pub score: Score,
    pub effects: Effects,
    pub rng: SmallRng,
    pub elapsed: f32,
    /// Fractional score accumulator (px scrolled / 4) - flushed when >= 1.
    score_accum: f32,
    pub hp: u32,
    pub hp_invuln: f32,
    /// Tracks whether player was airborne on previous tick, for landing detection.
    was_airborne: bool,
    /// Last observed difficulty tier - used to trigger tier banners on change.
    last_tier: u32,
    last_stage: Stage,
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
            hp: 1,
            hp_invuln: 0.0,
            effects: Effects::new(),
            was_airborne: false,
            last_tier: 0,
            last_stage: Stage::DepartmentStore,
        }
    }

    pub fn is_hp_invuln(&self) -> bool {
        self.hp_invuln > 0.0
    }

    pub fn current_speed(&self) -> f32 {
        speed_for_score(self.score.current) * self.dash.speed_mult()
    }

    pub fn current_stage(&self) -> Stage {
        stage_for_tier(tier_for_score(self.score.current))
    }

    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::Jump => {
                if self.player.try_jump() {
                    self.effects.sfx(SfxCue::Jump);
                }
            }
            Action::JumpRelease => self.player.release_jump(),
            Action::Duck => self.player.try_duck(),
            Action::DuckRelease => self.player.release_duck(),
            Action::Dash => {
                if let DashRequest::Started = self.dash.try_start() {
                    self.effects.sfx(SfxCue::Dash);
                }
            }
            Action::DashRelease => {
                self.dash.release();
            }
            Action::Confirm
            | Action::Pause
            | Action::OpenHelp
            | Action::OpenStory
            | Action::Back
            | Action::MoveLeft
            | Action::MoveRight
            | Action::MoveLeftRelease
            | Action::MoveRightRelease
            | Action::DebugBoss => { /* handled by state machine */ }
            _ => {}
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
        // effects.update is driven from Game::update so it continues to
        // advance even after death (for the 2-pulse shake).
        self.player.update(sim_dt);
        if self.hp_invuln > 0.0 {
            self.hp_invuln = (self.hp_invuln - real_dt).max(0.0);
        }

        // Landing detection: airborne last tick, grounded now → dust burst
        let is_airborne = !self.player.is_grounded();
        if self.was_airborne && !is_airborne {
            let foot_x = PLAYER_X + PLAYER_W * 0.5;
            let foot_y = GROUND_Y - 2.0;
            self.effects.dust_burst(foot_x, foot_y, 8);
        }
        self.was_airborne = is_airborne;

        let speed = self.current_speed();
        self.background.update(sim_dt, speed);
        // Stage boundary handling.
        let wipe_active = self.effects.is_stage_wiping();
        const PREWIPE_WINDOW: u32 = 400;
        let score = self.score.current;
        let tier_now = tier_for_score(score);
        let stage_now = crate::game::difficulty::stage_for_tier(tier_now);
        let into_tier = score % crate::game::difficulty::SCORE_PER_TIER;
        let distance_to_next = crate::game::difficulty::SCORE_PER_TIER - into_tier;
        let next_tier = (tier_now + 1).min(crate::game::difficulty::MAX_TIER);
        let next_stage = crate::game::difficulty::stage_for_tier(next_tier);
        let crossing_stage = next_stage != stage_now;
        let prewipe_block = crossing_stage && distance_to_next <= PREWIPE_WINDOW;

        if wipe_active {
            // Let existing obstacles scroll off. Keep updating their motion
            // but do not spawn new ones. Collisions still work; the player
            // handles any stragglers.
            self.obstacles
                .update(sim_dt, speed, score, &mut self.rng);
            self.pickups
                .update(sim_dt, speed, &mut self.rng, &self.obstacles);
            // Drain obstacles that have passed the player so they can't
            // build up off-screen.
            self.obstacles
                .obstacles
                .retain(|o| o.x + o.kind.size().0 > -80.0);
        } else if prewipe_block {
            // Pre-wipe window: advance existing obstacles but block new
            // spawns. Call the obstacle update with a "spawn suppressed"
            // signal by temporarily forcing next_spawn_gap very high.
            let saved_gap = self.obstacles.next_spawn_gap;
            self.obstacles.next_spawn_gap = f32::INFINITY;
            self.obstacles
                .update(sim_dt, speed, score, &mut self.rng);
            // Restore only if no spawn was triggered (spawn trigger resets
            // the gap internally; we always set to a finite value for
            // post-wipe resumption).
            if self.obstacles.next_spawn_gap.is_infinite() {
                self.obstacles.next_spawn_gap = saved_gap;
            }
            // Also suppress pickup spawns in the pre-wipe window.
            let saved_heart = self.pickups.time_to_next_heart;
            let saved_aurora = self.pickups.time_to_next;
            self.pickups.time_to_next_heart = f32::INFINITY;
            self.pickups.time_to_next = f32::INFINITY;
            self.pickups
                .update(sim_dt, speed, &mut self.rng, &self.obstacles);
            if self.pickups.time_to_next_heart.is_infinite() {
                self.pickups.time_to_next_heart = saved_heart;
            }
            if self.pickups.time_to_next.is_infinite() {
                self.pickups.time_to_next = saved_aurora;
            }
        } else {
            self.obstacles
                .update(sim_dt, speed, score, &mut self.rng);
            self.pickups
                .update(sim_dt, speed, &mut self.rng, &self.obstacles);
        }

        // Accumulate fractional score (1 point per 4 px scrolled).
        self.score_accum += speed * sim_dt / 4.0;
        if self.score_accum >= 1.0 {
            let whole = self.score_accum.floor() as u32;
            self.score.add(whole);
            self.score_accum -= whole as f32;
        }

        // Tier change banner - triggers on crossing a difficulty threshold.
        let current_tier = tier_for_score(self.score.current);
        if current_tier > self.last_tier {
            let label = tier_banner_label(current_tier);
            self.effects.push_tier_banner(label, 2.0);
            self.last_tier = current_tier;
        }

        // Stage change wipe - Metal Slug style transition.
        // We trigger the wipe the moment we enter a new stage, AND we
        // suppress new obstacle spawns for a pre-wipe window so the stage
        // boundary feels smooth (no pop-out of obstacles mid-wipe).
        let current_stage = crate::game::difficulty::stage_for_tier(current_tier);
        if current_stage != self.last_stage {
            let name = crate::game::difficulty::stage_name(current_stage).to_string();
            self.effects.start_stage_wipe(name, 2.6);
            self.last_stage = current_stage;
        }

        let player_box = self.player.hitbox();
        let collected = self.pickups.collisions_with(&player_box);
        for &i in &collected {
            self.pickups.stones[i].collected = true;
            self.dash.add_aurora(1);
            self.score.add(50);
        }

        // Score popup spawns for collected auroras
        for &i in &collected {
            let s = &self.pickups.stones[i];
            self.effects.score_popup(s.x, s.y, 50, (0.62, 0.42, 1.00));
        }
        if !collected.is_empty() {
            self.effects.sfx(SfxCue::Pickup);
        }

        // Heart pickups
        let heart_indices = self.pickups.heart_collisions_with(&player_box);
        for &i in &heart_indices {
            let h = self.pickups.hearts[i].clone();
            self.pickups.hearts[i].collected = true;
            if self.hp < MAX_HP {
                self.hp += 1;
            }
            self.score.add(75);
            self.effects.score_popup(h.x, h.y, 75, (0.95, 0.3, 0.35));
        }
        if !heart_indices.is_empty() {
            self.effects.sfx(SfxCue::Heart);
        }

        if let Some(idx) = self.obstacles.first_collision(&player_box) {
            let kind = self.obstacles.obstacles[idx].kind;
            let ox = self.obstacles.obstacles[idx].x;
            let oy = self.obstacles.obstacles[idx].y;
            if self.dash.is_invulnerable() && kind.destroyable_by_dash() {
                self.obstacles.obstacles[idx].alive = false;
                self.score.add(25);
                self.effects.smash_burst(ox + 16.0, oy + 16.0);
                self.effects.score_popup(ox, oy, 25, (1.0, 0.82, 0.2));
                self.effects.shake(4.0, 0.12);
                self.effects.sfx(SfxCue::Smash);
                if matches!(kind, ObstacleKind::Amy) {
                    self.dash.trigger_slowmo();
                }
            } else if !self.dash.is_invulnerable() && !self.is_hp_invuln() {
                if self.hp > 1 {
                    self.hp -= 1;
                    self.hp_invuln = HP_INVULN_TIME;
                    self.obstacles.obstacles[idx].alive = false;
                    self.effects.hit_burst(ox + 16.0, oy + 16.0);
                    self.effects.shake(8.0, 0.2);
                    self.effects.flash(0.35, 0.3);
                    self.effects.sfx(SfxCue::Hit);
                } else {
                    self.player.hit();
                    self.effects.hit_burst(PLAYER_X + PLAYER_W * 0.5, self.player.y + PLAYER_H * 0.5);
                    // Death: two distinct punches, no prolonged jitter.
                    self.effects.trigger_death_shake();
                    self.effects.flash(0.5, 0.5);
                    self.effects.sfx(SfxCue::Hit);
                    return RunOutcome::Died;
                }
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
            ObstacleKind::CoffeeCup,
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
            ObstacleKind::CoffeeCup,
            pbox.x,
        );
        o.y = pbox.y;
        w.obstacles.obstacles.push(o);
        let outcome = w.update(DT);
        assert_eq!(outcome, RunOutcome::Continuing);
        assert!(!w.obstacles.obstacles[0].alive);
    }

    #[test]
    fn dash_smashes_heavy_humanoids() {
        // Post-update: dash smashes everything, including Alice3/Alice4.
        let mut w = fresh_world();
        w.dash.add_aurora(1);
        w.dash.try_start();
        let pbox = w.player.hitbox();
        let mut o = crate::game::obstacles::Obstacle::new(
            ObstacleKind::Alice3,
            pbox.x,
        );
        o.y = pbox.y;
        w.obstacles.obstacles.push(o);
        let outcome = w.update(DT);
        assert_eq!(outcome, RunOutcome::Continuing);
        assert!(!w.obstacles.obstacles[0].alive);
    }
}
