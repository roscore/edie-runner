//! Top-level game state machine. See spec §3.8.

use crate::game::world::{RunOutcome, World};
use crate::platform::input::Action;
use crate::platform::storage::Storage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Title,
    Playing,
    Paused,
    GameOver,
}

pub const RUN_HISTORY_LEN: usize = 5;

pub struct Game {
    pub state: GameState,
    pub world: World,
    pub seed_counter: u64,
    /// Most recent run scores, newest first. Capped at RUN_HISTORY_LEN.
    pub run_history: Vec<u32>,
    /// Position (1-indexed) of the most recently completed run within the
    /// best-runs leaderboard, if it qualified. Used for the "NEW #N" badge.
    pub last_run_rank: Option<usize>,
}

impl Game {
    pub fn new<S: Storage>(seed: u64, storage: &S) -> Self {
        Self {
            state: GameState::Title,
            world: World::new(seed, storage),
            seed_counter: seed,
            run_history: Vec::new(),
            last_run_rank: None,
        }
    }

    /// Top scores in the session, sorted high→low.
    pub fn best_runs(&self) -> Vec<u32> {
        let mut sorted = self.run_history.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        sorted
    }

    pub fn on_visibility_change(&mut self, visible: bool) {
        if !visible && self.state == GameState::Playing {
            self.state = GameState::Paused;
        }
    }

    pub fn handle<S: Storage>(&mut self, action: Action, storage: &mut S) {
        match (self.state, action) {
            (GameState::Title, Action::Confirm) | (GameState::Title, Action::Jump) => {
                self.start_run(storage);
            }
            (GameState::Playing, Action::Pause) => {
                self.state = GameState::Paused;
            }
            (GameState::Paused, Action::Pause) | (GameState::Paused, Action::Confirm) => {
                self.state = GameState::Playing;
            }
            (GameState::GameOver, Action::Confirm) | (GameState::GameOver, Action::Jump) => {
                self.start_run(storage);
            }
            (GameState::Playing, _) => {
                self.world.apply_action(action);
            }
            _ => {}
        }
    }

    fn start_run<S: Storage>(&mut self, storage: &S) {
        self.seed_counter = self.seed_counter.wrapping_add(1);
        self.world = World::new(self.seed_counter, storage);
        self.state = GameState::Playing;
    }

    pub fn update<S: Storage>(&mut self, real_dt: f32, storage: &mut S) {
        if self.state != GameState::Playing {
            return;
        }
        match self.world.update(real_dt) {
            RunOutcome::Continuing => {}
            RunOutcome::Died => {
                self.state = GameState::GameOver;
                let _ = self.world.score.save_if_new_high(storage);

                let final_score = self.world.score.current;
                // Insert into history (newest first)
                self.run_history.insert(0, final_score);
                if self.run_history.len() > RUN_HISTORY_LEN {
                    self.run_history.truncate(RUN_HISTORY_LEN);
                }
                // Compute rank within best_runs (1-indexed)
                let best = self.best_runs();
                self.last_run_rank = best
                    .iter()
                    .position(|s| *s == final_score)
                    .map(|i| i + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::storage::InMemoryStorage;
    use crate::time::DT;

    #[test]
    fn starts_in_title() {
        let s = InMemoryStorage::new();
        let g = Game::new(1, &s);
        assert_eq!(g.state, GameState::Title);
    }

    #[test]
    fn confirm_from_title_starts_run() {
        let mut s = InMemoryStorage::new();
        let mut g = Game::new(1, &s);
        g.handle(Action::Confirm, &mut s);
        assert_eq!(g.state, GameState::Playing);
    }

    #[test]
    fn visibility_loss_pauses_during_play() {
        let mut s = InMemoryStorage::new();
        let mut g = Game::new(1, &s);
        g.handle(Action::Confirm, &mut s);
        g.on_visibility_change(false);
        assert_eq!(g.state, GameState::Paused);
    }

    #[test]
    fn pause_action_toggles() {
        let mut s = InMemoryStorage::new();
        let mut g = Game::new(1, &s);
        g.handle(Action::Confirm, &mut s);
        g.handle(Action::Pause, &mut s);
        assert_eq!(g.state, GameState::Paused);
        g.handle(Action::Pause, &mut s);
        assert_eq!(g.state, GameState::Playing);
    }

    #[test]
    fn death_transitions_to_game_over_and_persists() {
        let mut s = InMemoryStorage::new();
        let mut g = Game::new(1, &s);
        g.handle(Action::Confirm, &mut s);
        let pbox = g.world.player.hitbox();
        let mut o = crate::game::obstacles::Obstacle::new(
            crate::game::obstacles::ObstacleKind::CoiledCable,
            pbox.x,
        );
        o.y = pbox.y;
        g.world.obstacles.obstacles.push(o);
        g.world.score.current = 999;
        g.update(DT, &mut s);
        assert_eq!(g.state, GameState::GameOver);
        assert!(s.get(crate::game::score::STORAGE_KEY).is_some());
    }

    #[test]
    fn confirm_from_game_over_restarts() {
        let mut s = InMemoryStorage::new();
        let mut g = Game::new(1, &s);
        g.state = GameState::GameOver;
        g.handle(Action::Confirm, &mut s);
        assert_eq!(g.state, GameState::Playing);
    }
}
