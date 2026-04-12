//! Game state machine for EDIE Battle Reverse.

use crate::reversi::board::{Board, MoveResult, Powerup, Side, AURORA_INTERVAL};
use rand::rngs::SmallRng;
use rand::SeedableRng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    VsLocal,
    VsAiEasy,    // Amy
    VsAiNormal,  // Alice3
    VsAiHard,    // AliceM1
    VsAiInsane,  // Alice4
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Menu,
    Playing,
    Animating,
    /// Player is selecting a target for their powerup (VirusCure or ForceFlip).
    UsingPowerup,
    GameOver,
}

#[derive(Debug, Clone)]
pub struct FlipAnim {
    pub cells: Vec<(usize, usize)>,
    pub placed: (usize, usize),
    pub side: Side,
    pub elapsed: f32,
    pub total: f32,
    pub result: MoveResult,
}

pub struct ReversiGame {
    pub board: Board,
    pub phase: Phase,
    pub mode: GameMode,
    pub flip_anim: Option<FlipAnim>,
    pub hover: Option<(usize, usize)>,
    pub seed: u64,
    pub rng: SmallRng,
    /// Which powerup is being targeted (during UsingPowerup phase).
    pub targeting_powerup: Option<Powerup>,
    /// Toast message shown briefly after powerup events.
    pub toast: Option<(String, f32)>,
    /// Turn timer: remaining seconds for current turn.
    pub turn_timer: f32,
    /// Turn timer: max seconds per turn.
    pub turn_timer_max: f32,
    /// Easter egg: theme index (0=EDIE, 1=Amy, 2=AliceM1, 3=Alice3, 4=Alice4).
    pub theme_index: usize,
}

impl ReversiGame {
    pub fn new(seed: u64) -> Self {
        Self {
            board: Board::new(seed),
            phase: Phase::Menu,
            mode: GameMode::VsLocal,
            flip_anim: None,
            hover: None,
            seed,
            rng: SmallRng::seed_from_u64(seed.wrapping_add(777)),
            targeting_powerup: None,
            toast: None,
            turn_timer: 15.0,
            turn_timer_max: 15.0,
            theme_index: 0,
        }
    }

    fn turn_time_for_mode(mode: GameMode) -> f32 {
        match mode {
            GameMode::VsLocal => 30.0,
            GameMode::VsAiEasy => 20.0,
            GameMode::VsAiNormal => 15.0,
            GameMode::VsAiHard => 12.0,
            GameMode::VsAiInsane => 8.0,
        }
    }

    pub fn reset_turn_timer(&mut self) {
        self.turn_timer = self.turn_timer_max;
    }

    pub fn start_game(&mut self, mode: GameMode) {
        self.seed = self.seed.wrapping_add(1);
        self.board = Board::new(self.seed);
        self.rng = SmallRng::seed_from_u64(self.seed.wrapping_add(777));
        self.mode = mode;
        self.phase = Phase::Playing;
        self.flip_anim = None;
        self.hover = None;
        self.targeting_powerup = None;
        self.toast = None;
        self.turn_timer_max = Self::turn_time_for_mode(mode);
        self.turn_timer = self.turn_timer_max;
    }

    pub fn on_cell_click(&mut self, row: usize, col: usize) {
        if self.phase == Phase::UsingPowerup {
            self.on_powerup_target(row, col);
            return;
        }
        if self.phase != Phase::Playing { return; }
        if !self.board.is_valid_move(row, col, self.board.turn) { return; }
        let side = self.board.turn;
        let result = self.board.apply_move(row, col);
        if let Some(pw) = result.powerup_gained {
            let name = match pw {
                Powerup::DoubleStrike => "DOUBLE STRIKE",
                Powerup::VirusCure => "VIRUS CURE",
                Powerup::ForceFlip => "FORCE FLIP",
            };
            self.toast = Some((format!("{} got {}!", if side == Side::Edie { "EDIE" } else { "ALICE" }, name), 2.0));
        }
        self.flip_anim = Some(FlipAnim {
            cells: result.flipped.clone(),
            placed: (row, col),
            side,
            elapsed: 0.0,
            total: 0.35,
            result,
        });
        self.phase = Phase::Animating;
        self.reset_turn_timer();
    }

    /// Activate the current player's powerup (VirusCure or ForceFlip).
    pub fn activate_powerup(&mut self) {
        if self.phase != Phase::Playing { return; }
        let side = self.board.turn;
        match self.board.powerup(side) {
            Some(Powerup::VirusCure) | Some(Powerup::ForceFlip) => {
                self.targeting_powerup = self.board.powerup(side);
                self.phase = Phase::UsingPowerup;
            }
            _ => {} // DoubleStrike is auto-applied, nothing to activate
        }
    }

    /// Cancel powerup targeting — return to normal Playing.
    pub fn cancel_powerup(&mut self) {
        if self.phase == Phase::UsingPowerup {
            self.targeting_powerup = None;
            self.phase = Phase::Playing;
        }
    }

    fn on_powerup_target(&mut self, row: usize, col: usize) {
        match self.targeting_powerup {
            Some(Powerup::VirusCure) => {
                if self.board.use_virus_cure(row, col) {
                    self.toast = Some(("Virus cured!".into(), 1.5));
                    self.targeting_powerup = None;
                    self.phase = Phase::Playing;
                }
            }
            Some(Powerup::ForceFlip) => {
                if self.board.use_force_flip(row, col) {
                    self.toast = Some(("Force flip!".into(), 1.5));
                    self.targeting_powerup = None;
                    if self.board.is_game_over() {
                        self.phase = Phase::GameOver;
                    } else {
                        self.phase = Phase::Playing;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Tick toast timer
        if let Some((_, ref mut t)) = self.toast {
            *t -= dt;
            if *t <= 0.0 { self.toast = None; }
        }
        // Turn timer: count down during Playing/UsingPowerup
        if matches!(self.phase, Phase::Playing | Phase::UsingPowerup) {
            self.turn_timer -= dt;
            if self.turn_timer <= 0.0 {
                // Time's up: play a random valid move for the current player
                self.turn_timer = 0.0;
                let moves = self.board.valid_moves(self.board.turn);
                if !moves.is_empty() {
                    use rand::Rng as _;
                    let idx = self.rng.gen_range(0..moves.len());
                    let (r, c) = moves[idx];
                    self.targeting_powerup = None;
                    let side = self.board.turn;
                    let result = self.board.apply_move(r, c);
                    self.toast = Some(("TIME OUT!".into(), 1.5));
                    self.flip_anim = Some(FlipAnim {
                        cells: result.flipped.clone(),
                        placed: (r, c),
                        side,
                        elapsed: 0.0,
                        total: 0.35,
                        result,
                    });
                    self.phase = Phase::Animating;
                    self.reset_turn_timer();
                }
            }
        }
        if let Some(anim) = &mut self.flip_anim {
            anim.elapsed += dt;
            if anim.elapsed >= anim.total {
                // Spawn aurora after animation finishes
                if self.board.turn_count > 0 && self.board.turn_count % AURORA_INTERVAL == 0 {
                    self.board.spawn_aurora(&mut self.rng);
                }
                self.flip_anim = None;
                if self.board.is_game_over() {
                    self.phase = Phase::GameOver;
                } else {
                    self.phase = Phase::Playing;
                    self.reset_turn_timer();
                }
            }
        }
    }
}
