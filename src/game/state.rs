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
    Help,
    Story,
    BossFight,
    Ending,
}

pub const RUN_HISTORY_LEN: usize = 5;
pub const STORY_DURATION: f32 = 52.0;
pub const COUNTDOWN_DURATION: f32 = 2.5;

pub struct Game {
    pub state: GameState,
    pub world: World,
    pub seed_counter: u64,
    pub run_history: Vec<u32>,
    pub last_run_rank: Option<usize>,
    pub story_start_time: f32,
    pub countdown_remaining: f32,
    /// Active boss fight state (present when state == BossFight).
    pub boss: Option<crate::game::boss::BossWorld>,
    pub boss_input_dx: f32,
    /// When boss break-in cinematic is playing (score hit trigger, before Boss state).
    pub boss_intro_remaining: f32,
    /// True if the last completed boss fight went all the way through phase 2.
    pub last_ending_true: bool,
}

impl Game {
    pub fn new<S: Storage>(seed: u64, storage: &S) -> Self {
        Self {
            state: GameState::Title,
            world: World::new(seed, storage),
            seed_counter: seed,
            run_history: Vec::new(),
            last_run_rank: None,
            story_start_time: 0.0,
            countdown_remaining: 0.0,
            boss: None,
            boss_input_dx: 0.0,
            boss_intro_remaining: 0.0,
            last_ending_true: false,
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
        // Boss fight: track left/right input state separately.
        if matches!(self.state, GameState::BossFight) {
            match action {
                Action::MoveLeft => self.boss_input_dx = -1.0,
                Action::MoveRight => self.boss_input_dx = 1.0,
                Action::MoveLeftRelease => {
                    if self.boss_input_dx < 0.0 {
                        self.boss_input_dx = 0.0;
                    }
                }
                Action::MoveRightRelease => {
                    if self.boss_input_dx > 0.0 {
                        self.boss_input_dx = 0.0;
                    }
                }
                _ => {}
            }
            return;
        }
        if matches!(self.state, GameState::Ending) {
            if matches!(action, Action::Confirm | Action::Jump) {
                self.state = GameState::Title;
            }
            return;
        }
        match (self.state, action) {
            (GameState::Title, Action::Confirm) | (GameState::Title, Action::Jump) => {
                self.start_run(storage);
            }
            (GameState::Title, Action::OpenHelp) => {
                self.state = GameState::Help;
            }
            (GameState::Title, Action::DebugBoss)
            | (GameState::GameOver, Action::DebugBoss) => {
                // Dev shortcut: skip straight to the Mungchi boss fight.
                self.seed_counter = self.seed_counter.wrapping_add(1);
                self.world = World::new(self.seed_counter, storage);
                self.world.score.current = crate::game::difficulty::BOSS_TRIGGER_SCORE;
                self.state = GameState::BossFight;
                self.boss = Some(crate::game::boss::BossWorld::new());
                self.countdown_remaining = 0.0;
            }
            (GameState::Title, Action::OpenStory) => {
                self.state = GameState::Story;
                // story_start_time is set by main.rs from wall clock
            }
            (GameState::Help, _) => {
                // Any key returns to title
                self.state = GameState::Title;
            }
            (GameState::Story, _) => {
                // Any key skips back to title
                self.state = GameState::Title;
            }
            (GameState::Playing, Action::Pause) | (GameState::Playing, Action::Back) => {
                self.state = GameState::Paused;
            }
            (GameState::Paused, Action::Pause) | (GameState::Paused, Action::Jump) => {
                self.state = GameState::Playing;
            }
            (GameState::Paused, Action::Back) | (GameState::Paused, Action::Confirm) => {
                // ESC / Q from pause returns to the title screen, abandoning
                // the current run.
                self.state = GameState::Title;
                self.countdown_remaining = 0.0;
            }
            (GameState::GameOver, Action::Confirm) | (GameState::GameOver, Action::Jump) => {
                self.start_run(storage);
            }
            (GameState::GameOver, Action::OpenHelp) => {
                self.state = GameState::Help;
            }
            (GameState::GameOver, Action::OpenStory) => {
                self.state = GameState::Story;
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
        self.countdown_remaining = COUNTDOWN_DURATION;
        self.boss = None;
        self.boss_input_dx = 0.0;
        self.boss_intro_remaining = 0.0;
    }

    pub fn update<S: Storage>(&mut self, real_dt: f32, storage: &mut S) {
        self.world.effects.update(real_dt);

        // Boss intro cinematic: brief flash + shake before entering BossFight.
        if self.boss_intro_remaining > 0.0 {
            self.boss_intro_remaining = (self.boss_intro_remaining - real_dt).max(0.0);
            if self.boss_intro_remaining <= 0.0 {
                self.state = GameState::BossFight;
                self.boss = Some(crate::game::boss::BossWorld::new());
            }
            return;
        }

        // Boss fight update
        if matches!(self.state, GameState::BossFight) {
            if let Some(b) = self.boss.as_mut() {
                use crate::game::boss::BossOutcome;
                match b.update(real_dt, self.boss_input_dx, &mut self.world.rng) {
                    BossOutcome::Continuing => {}
                    BossOutcome::Hit => {
                        self.state = GameState::GameOver;
                        let final_score = self.world.score.current;
                        let _ = self.world.score.save_if_new_high(storage);
                        self.run_history.insert(0, final_score);
                        if self.run_history.len() > RUN_HISTORY_LEN {
                            self.run_history.truncate(RUN_HISTORY_LEN);
                        }
                        let best = self.best_runs();
                        self.last_run_rank = best
                            .iter()
                            .position(|s| *s == final_score)
                            .map(|i| i + 1);
                    }
                    BossOutcome::Survived => {
                        // Phase 2 survived means the TRUE ending; phase 1 is
                        // never Survived now (we interlude into phase 2),
                        // but keep the fallback defensively.
                        let phase = self.boss.as_ref().map(|b| b.phase).unwrap_or(1);
                        self.last_ending_true = phase >= 2;
                        self.state = GameState::Ending;
                        let _ = self.world.score.save_if_new_high(storage);
                    }
                }
            }
            return;
        }

        if self.state != GameState::Playing {
            return;
        }
        if self.countdown_remaining > 0.0 {
            self.countdown_remaining = (self.countdown_remaining - real_dt).max(0.0);
            return;
        }

        // Check boss trigger BEFORE world update.
        if self.world.score.current >= crate::game::difficulty::BOSS_TRIGGER_SCORE {
            self.boss_intro_remaining = crate::game::boss::BOSS_INTRO_DURATION;
            return;
        }

        match self.world.update(real_dt) {
            RunOutcome::Continuing => {}
            RunOutcome::Died => {
                self.state = GameState::GameOver;
                let _ = self.world.score.save_if_new_high(storage);

                let final_score = self.world.score.current;
                self.run_history.insert(0, final_score);
                if self.run_history.len() > RUN_HISTORY_LEN {
                    self.run_history.truncate(RUN_HISTORY_LEN);
                }
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
        // Skip countdown for the test
        g.countdown_remaining = 0.0;
        let pbox = g.world.player.hitbox();
        let mut o = crate::game::obstacles::Obstacle::new(
            crate::game::obstacles::ObstacleKind::CoffeeCup,
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
