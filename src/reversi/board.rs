//! Core Othello / Reversi board logic for EDIE Battle Reverse.
//!
//! Pure game logic — no rendering, no IO. The board is an 8×8 grid with
//! two piece types (EDIE / Alice), virus-blocked cells, and an HP-based
//! damage system per the design spec §2.

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

// ====================================================================
// Types
// ====================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Piece(Side),
    /// Mungchi virus — blocks placement and flip paths.
    Virus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Edie,
    Alice,
}

impl Side {
    pub fn opponent(self) -> Side {
        match self {
            Side::Edie => Side::Alice,
            Side::Alice => Side::Edie,
        }
    }
}

/// Result of applying a move to the board.
#[derive(Debug, Clone)]
pub struct MoveResult {
    /// Positions of cells that were flipped (not including the placed cell).
    pub flipped: Vec<(usize, usize)>,
    /// HP damage dealt to the opponent.
    pub damage: i32,
    /// True if 6+ pieces were flipped (triggers Mungchi Alert bonus).
    pub mungchi_alert: bool,
    /// True if the game ended because opponent HP hit 0.
    pub knockout: bool,
}

// ====================================================================
// Constants
// ====================================================================

pub const BOARD_SIZE: usize = 8;
pub const INITIAL_HP: i32 = 10_000;
pub const NUM_VIRUSES: usize = 5;
pub const MUNGCHI_THRESHOLD: usize = 6;
pub const MUNGCHI_BONUS: i32 = 500;

const DIRS: [(i32, i32); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    ( 0, -1),          ( 0, 1),
    ( 1, -1), ( 1, 0), ( 1, 1),
];

// ====================================================================
// Board
// ====================================================================

#[derive(Debug, Clone)]
pub struct Board {
    pub cells: [[Cell; BOARD_SIZE]; BOARD_SIZE],
    pub turn: Side,
    pub edie_hp: i32,
    pub alice_hp: i32,
    pub turn_count: u32,
}

impl Board {
    pub fn new(seed: u64) -> Self {
        let mut board = Board {
            cells: [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE],
            turn: Side::Edie,
            edie_hp: INITIAL_HP,
            alice_hp: INITIAL_HP,
            turn_count: 0,
        };
        board.cells[3][3] = Cell::Piece(Side::Alice);
        board.cells[3][4] = Cell::Piece(Side::Edie);
        board.cells[4][3] = Cell::Piece(Side::Edie);
        board.cells[4][4] = Cell::Piece(Side::Alice);
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut placed = 0;
        while placed < NUM_VIRUSES {
            let r = rng.gen_range(0..BOARD_SIZE);
            let c = rng.gen_range(0..BOARD_SIZE);
            if board.cells[r][c] == Cell::Empty {
                board.cells[r][c] = Cell::Virus;
                placed += 1;
            }
        }
        board
    }

    pub fn get(&self, row: usize, col: usize) -> Cell {
        self.cells[row][col]
    }

    pub fn hp(&self, side: Side) -> i32 {
        match side {
            Side::Edie => self.edie_hp,
            Side::Alice => self.alice_hp,
        }
    }

    pub fn piece_count(&self, side: Side) -> u32 {
        let target = Cell::Piece(side);
        self.cells.iter().flat_map(|row| row.iter()).filter(|c| **c == target).count() as u32
    }

    fn flips_in_dir(&self, row: usize, col: usize, dr: i32, dc: i32, side: Side) -> Vec<(usize, usize)> {
        let opp = Cell::Piece(side.opponent());
        let own = Cell::Piece(side);
        let mut candidates = Vec::new();
        let mut r = row as i32 + dr;
        let mut c = col as i32 + dc;
        while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
            let cell = self.cells[r as usize][c as usize];
            if cell == opp {
                candidates.push((r as usize, c as usize));
            } else if cell == own {
                return candidates;
            } else {
                return Vec::new();
            }
            r += dr;
            c += dc;
        }
        Vec::new()
    }

    pub fn flips_for_move(&self, row: usize, col: usize, side: Side) -> Vec<(usize, usize)> {
        let mut all = Vec::new();
        for &(dr, dc) in &DIRS {
            all.extend(self.flips_in_dir(row, col, dr, dc, side));
        }
        all
    }

    pub fn is_valid_move(&self, row: usize, col: usize, side: Side) -> bool {
        if self.cells[row][col] != Cell::Empty { return false; }
        for &(dr, dc) in &DIRS {
            if !self.flips_in_dir(row, col, dr, dc, side).is_empty() { return true; }
        }
        false
    }

    pub fn valid_moves(&self, side: Side) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for r in 0..BOARD_SIZE {
            for c in 0..BOARD_SIZE {
                if self.is_valid_move(r, c, side) { moves.push((r, c)); }
            }
        }
        moves
    }

    pub fn apply_move(&mut self, row: usize, col: usize) -> MoveResult {
        let side = self.turn;
        assert!(self.is_valid_move(row, col, side), "invalid move ({}, {}) for {:?}", row, col, side);
        self.cells[row][col] = Cell::Piece(side);
        let flipped = self.flips_for_move(row, col, side);
        for &(fr, fc) in &flipped { self.cells[fr][fc] = Cell::Piece(side); }
        let n = flipped.len() as u32;
        let damage = damage_for_flips(n);
        let mungchi_alert = flipped.len() >= MUNGCHI_THRESHOLD;
        match side {
            Side::Edie => self.alice_hp -= damage,
            Side::Alice => self.edie_hp -= damage,
        }
        if mungchi_alert {
            match side {
                Side::Edie => self.edie_hp = (self.edie_hp + MUNGCHI_BONUS).min(INITIAL_HP),
                Side::Alice => self.alice_hp = (self.alice_hp + MUNGCHI_BONUS).min(INITIAL_HP),
            }
        }
        let knockout = self.hp(side.opponent()) <= 0;
        self.turn_count += 1;
        self.turn = side.opponent();
        if !knockout && self.valid_moves(self.turn).is_empty() {
            self.turn = side;
        }
        MoveResult { flipped, damage, mungchi_alert, knockout }
    }

    pub fn is_game_over(&self) -> bool {
        self.edie_hp <= 0 || self.alice_hp <= 0
            || (self.valid_moves(Side::Edie).is_empty() && self.valid_moves(Side::Alice).is_empty())
    }

    pub fn winner(&self) -> Option<Side> {
        if self.edie_hp <= 0 { return Some(Side::Alice); }
        if self.alice_hp <= 0 { return Some(Side::Edie); }
        if self.edie_hp > self.alice_hp { Some(Side::Edie) }
        else if self.alice_hp > self.edie_hp { Some(Side::Alice) }
        else {
            let ec = self.piece_count(Side::Edie);
            let ac = self.piece_count(Side::Alice);
            if ec > ac { Some(Side::Edie) } else if ac > ec { Some(Side::Alice) } else { None }
        }
    }
}

pub fn damage_for_flips(n: u32) -> i32 {
    match n {
        0 => 0, 1 => 100, 2 => 220, 3 => 360, 4 => 500, 5 => 680,
        n => (140 * n) as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_board() -> Board { Board::new(42) }

    #[test]
    fn initial_board_has_4_pieces() {
        let b = test_board();
        assert_eq!(b.piece_count(Side::Edie) + b.piece_count(Side::Alice), 4);
    }

    #[test]
    fn initial_board_has_5_viruses() {
        let b = test_board();
        let v: usize = b.cells.iter().flat_map(|r| r.iter()).filter(|c| **c == Cell::Virus).count();
        assert_eq!(v, NUM_VIRUSES);
    }

    #[test]
    fn virus_cells_block_placement() {
        let b = test_board();
        for r in 0..BOARD_SIZE { for c in 0..BOARD_SIZE {
            if b.cells[r][c] == Cell::Virus {
                assert!(!b.is_valid_move(r, c, Side::Edie));
            }
        }}
    }

    #[test]
    fn valid_moves_initial_edie() {
        let b = test_board();
        let moves = b.valid_moves(Side::Edie);
        assert!(!moves.is_empty() && moves.len() <= 4);
    }

    #[test]
    fn apply_move_flips_correctly() {
        let mut b = test_board();
        let moves = b.valid_moves(Side::Edie);
        let (r, c) = moves[0];
        let result = b.apply_move(r, c);
        assert!(!result.flipped.is_empty());
        assert_eq!(b.cells[r][c], Cell::Piece(Side::Edie));
        for &(fr, fc) in &result.flipped { assert_eq!(b.cells[fr][fc], Cell::Piece(Side::Edie)); }
    }

    #[test]
    fn apply_move_deals_damage() {
        let mut b = test_board();
        let moves = b.valid_moves(Side::Edie);
        let hp_before = b.alice_hp;
        let result = b.apply_move(moves[0].0, moves[0].1);
        assert!(result.damage > 0);
        assert_eq!(b.alice_hp, hp_before - result.damage);
    }

    #[test]
    fn turn_switches_after_move() {
        let mut b = test_board();
        assert_eq!(b.turn, Side::Edie);
        let moves = b.valid_moves(Side::Edie);
        b.apply_move(moves[0].0, moves[0].1);
        assert_eq!(b.turn, Side::Alice);
    }

    #[test]
    fn damage_table_scales_up() {
        assert_eq!(damage_for_flips(1), 100);
        assert_eq!(damage_for_flips(5), 680);
        assert_eq!(damage_for_flips(8), 1120);
    }

    #[test]
    fn mungchi_alert_at_threshold() {
        let mut b = Board { cells: [[Cell::Empty; 8]; 8], turn: Side::Edie, edie_hp: INITIAL_HP, alice_hp: INITIAL_HP, turn_count: 0 };
        for c in 1..7 { b.cells[3][c] = Cell::Piece(Side::Alice); }
        b.cells[3][7] = Cell::Piece(Side::Edie);
        let result = b.apply_move(3, 0);
        assert_eq!(result.flipped.len(), 6);
        assert!(result.mungchi_alert);
    }

    #[test]
    fn game_over_on_hp_knockout() {
        let mut b = test_board();
        b.alice_hp = 50;
        let moves = b.valid_moves(Side::Edie);
        if !moves.is_empty() {
            let result = b.apply_move(moves[0].0, moves[0].1);
            if result.damage >= 50 { assert!(b.is_game_over()); }
        }
    }

    #[test]
    fn full_game_terminates() {
        let mut b = test_board();
        let mut rng = SmallRng::seed_from_u64(99);
        for _ in 0..60 {
            if b.is_game_over() { break; }
            let moves = b.valid_moves(b.turn);
            if moves.is_empty() { break; }
            let idx = rng.gen_range(0..moves.len());
            b.apply_move(moves[idx].0, moves[idx].1);
        }
        assert!(b.turn_count > 0);
    }

    #[test]
    fn winner_by_hp() {
        let mut b = test_board();
        b.edie_hp = 5000; b.alice_hp = 3000;
        b.cells = [[Cell::Piece(Side::Edie); 8]; 8];
        b.cells[0][0] = Cell::Piece(Side::Alice);
        assert!(b.is_game_over());
        assert_eq!(b.winner(), Some(Side::Edie));
    }
}
