//! Game state machine for EDIE Battle Reverse.

use crate::reversi::board::{Board, MoveResult, Side};

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
        }
    }

    pub fn start_game(&mut self, mode: GameMode) {
        self.seed = self.seed.wrapping_add(1);
        self.board = Board::new(self.seed);
        self.mode = mode;
        self.phase = Phase::Playing;
        self.flip_anim = None;
        self.hover = None;
    }

    pub fn on_cell_click(&mut self, row: usize, col: usize) {
        if self.phase != Phase::Playing { return; }
        if !self.board.is_valid_move(row, col, self.board.turn) { return; }
        let side = self.board.turn;
        let result = self.board.apply_move(row, col);
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

    pub fn update(&mut self, dt: f32) {
        if let Some(anim) = &mut self.flip_anim {
            anim.elapsed += dt;
            if anim.elapsed >= anim.total {
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
