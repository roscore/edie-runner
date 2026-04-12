//! Game state machine for EDIE Battle Reverse.

use crate::reversi::board::{Board, MoveResult, Powerup, Side, AURORA_INTERVAL};
use rand::rngs::SmallRng;
use rand::SeedableRng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    VsLocal,
    VsAiEasy,
    VsAiNormal,
    VsAiHard,
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
        }
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
                }
            }
        }
    }
}
