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
    /// Name entry for a qualifying leaderboard score. Only reached after
    /// GameOver when `leaderboard.qualifies(final_score)` is true.
    NameEntry,
}

pub const RUN_HISTORY_LEN: usize = 5;

fn next_name_char(c: char) -> char {
    // Cycle: A..Z, then back to A. Anything else -> 'A'.
    if ('A'..='Z').contains(&c) {
        if c == 'Z' {
            'A'
        } else {
            (c as u8 + 1) as char
        }
    } else {
        'A'
    }
}

fn prev_name_char(c: char) -> char {
    if ('A'..='Z').contains(&c) {
        if c == 'A' {
            'Z'
        } else {
            (c as u8 - 1) as char
        }
    } else {
        'A'
    }
}

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
    pub debug_run: bool,
    pub leaderboard: crate::game::leaderboard::Leaderboard,
    /// Scratch pad for the name entry screen: 3 chars, one active cursor.
    pub name_buf: [char; 3],
    pub name_cursor: usize,
    /// Score being entered (snapshotted at GameOver transition).
    pub pending_score: u32,
    /// Number of consecutive B presses on the Title screen -- three in a
    /// row jumps straight to the boss intro cinematic.
    pub title_b_presses: u32,
}

impl Game {
    pub fn new<S: Storage>(seed: u64, storage: &S) -> Self {
        let leaderboard = crate::game::leaderboard::Leaderboard::load(storage);
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
            debug_run: false,
            leaderboard,
            name_buf: ['A', 'A', 'A'],
            name_cursor: 0,
            pending_score: 0,
            title_b_presses: 0,
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
        // Name entry takes over input while active.
        if matches!(self.state, GameState::NameEntry) {
            match action {
                Action::NameUp | Action::Jump => {
                    let c = self.name_buf[self.name_cursor];
                    self.name_buf[self.name_cursor] = next_name_char(c);
                }
                Action::NameDown | Action::Duck => {
                    let c = self.name_buf[self.name_cursor];
                    self.name_buf[self.name_cursor] = prev_name_char(c);
                }
                Action::NameNext | Action::MoveRight | Action::Dash => {
                    if self.name_cursor + 1 < self.name_buf.len() {
                        self.name_cursor += 1;
                    } else {
                        self.commit_name_entry(storage);
                    }
                }
                Action::NamePrev | Action::MoveLeft => {
                    if self.name_cursor > 0 {
                        self.name_cursor -= 1;
                    }
                }
                Action::NameCommit | Action::Confirm => {
                    self.commit_name_entry(storage);
                }
                Action::Back => {
                    // Skip name entry entirely -> back to GameOver screen
                    self.state = GameState::GameOver;
                }
                _ => {}
            }
            return;
        }

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
        // Any non-B input on the Title screen resets the hidden
        // triple-B debug counter so it only matches consecutive presses.
        if matches!(self.state, GameState::Title) && !matches!(action, Action::DebugBoss) {
            self.title_b_presses = 0;
        }
        match (self.state, action) {
            (GameState::Title, Action::Confirm) | (GameState::Title, Action::Jump) => {
                self.start_run(storage);
            }
            (GameState::Title, Action::OpenHelp) => {
                self.state = GameState::Help;
            }
            (GameState::Title, Action::DebugBoss) => {
                // Dev shortcut, hidden on purpose: press B three times in
                // a row from the Title screen to drop straight into the
                // boss intro cinematic (not the BossFight state -- the
                // player should see the break-in animation first).
                self.title_b_presses += 1;
                if self.title_b_presses >= 3 {
                    self.title_b_presses = 0;
                    self.seed_counter = self.seed_counter.wrapping_add(1);
                    self.world = World::new(self.seed_counter, storage);
                    self.state = GameState::Playing;
                    self.countdown_remaining = 0.0;
                    // Kick off the break-in animation. The Playing-state
                    // update path will flip to BossFight once the intro
                    // timer reaches zero.
                    self.boss_intro_remaining =
                        crate::game::boss::BOSS_INTRO_DURATION;
                    self.debug_run = true;
                }
            }
            (GameState::GameOver, Action::DebugBoss) => {
                // From GameOver, a single B press still goes straight in
                // (so developers can cycle attempts quickly).
                self.seed_counter = self.seed_counter.wrapping_add(1);
                self.world = World::new(self.seed_counter, storage);
                self.state = GameState::Playing;
                self.countdown_remaining = 0.0;
                self.boss_intro_remaining =
                    crate::game::boss::BOSS_INTRO_DURATION;
                self.debug_run = true;
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
                // During the 3-2-1-GO countdown, discard world actions so
                // held keys from the title screen / gameover screen don't
                // jump or duck EDIE the instant the run begins.
                if self.countdown_remaining > 0.0 {
                    return;
                }
                self.world.apply_action(action);
            }
            _ => {}
        }
    }

    fn commit_name_entry<S: Storage>(&mut self, storage: &mut S) {
        let name: String = self.name_buf.iter().collect();
        let entry = crate::game::leaderboard::Entry {
            name,
            score: self.pending_score,
            ts: 0,
        };
        self.leaderboard.insert(storage, entry);
        self.state = GameState::GameOver;
    }

    /// Called after any run ends (death, boss hit) with the final score.
    /// Returns true if the run qualified and we transitioned to NameEntry.
    fn try_enter_name_entry(&mut self, final_score: u32) -> bool {
        if self.debug_run || final_score == 0 {
            return false;
        }
        if !self.leaderboard.qualifies(final_score) {
            return false;
        }
        self.pending_score = final_score;
        self.name_buf = ['A', 'A', 'A'];
        self.name_cursor = 0;
        self.state = GameState::NameEntry;
        true
    }

    fn start_run<S: Storage>(&mut self, storage: &S) {
        self.seed_counter = self.seed_counter.wrapping_add(1);
        self.world = World::new(self.seed_counter, storage);
        self.state = GameState::Playing;
        self.countdown_remaining = COUNTDOWN_DURATION;
        self.boss = None;
        self.boss_input_dx = 0.0;
        self.boss_intro_remaining = 0.0;
        self.debug_run = false;
        // Explicit clean slate: player is grounded, not holding jump.
        self.world.player.jump_held = false;
        self.world.player.duck_held = false;
        self.world.player.vy = 0.0;
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
                        let final_score = self.world.score.current;
                        if !self.debug_run {
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
                        } else {
                            self.last_run_rank = None;
                        }
                        self.state = GameState::GameOver;
                        self.try_enter_name_entry(final_score);
                    }
                    BossOutcome::Survived => {
                        let phase = self.boss.as_ref().map(|b| b.phase).unwrap_or(1);
                        self.last_ending_true = phase >= 2;
                        self.state = GameState::Ending;
                        if !self.debug_run {
                            let _ = self.world.score.save_if_new_high(storage);
                            let final_score = self.world.score.current;
                            // Ending also qualifies for leaderboard, but
                            // we skip the name entry while the ending
                            // cinematic plays. Auto-insert as "WIN" if it
                            // qualifies so the board reflects the clear.
                            if self.leaderboard.qualifies(final_score) {
                                let entry = crate::game::leaderboard::Entry {
                                    name: if phase >= 2 { "DR!".to_string() } else { "WIN".to_string() },
                                    score: final_score,
                                    ts: 0,
                                };
                                self.leaderboard.insert(storage, entry);
                            }
                        }
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
                let final_score = self.world.score.current;
                if !self.debug_run {
                    // Persist the high score so reloads keep it.
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
                } else {
                    self.last_run_rank = None;
                }
                self.state = GameState::GameOver;
                self.try_enter_name_entry(final_score);
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
        // Any qualifying score enters NameEntry first; GameOver otherwise.
        assert!(matches!(
            g.state,
            GameState::GameOver | GameState::NameEntry
        ));
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
