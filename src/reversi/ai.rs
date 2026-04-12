//! AI opponent for EDIE Battle Reverse.
//!
//! Three difficulty levels:
//!   Easy   — random valid move
//!   Normal — greedy (pick move that flips the most pieces)
//!   Hard   — minimax with alpha-beta pruning, depth 4,
//!            weighted position heuristic (corners = high)

use crate::reversi::board::{Board, Side, BOARD_SIZE};
use crate::reversi::game::GameMode;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

const POS_WEIGHT: [[i32; 8]; 8] = [
    [120, -20,  20,   5,   5,  20, -20, 120],
    [-20, -40,  -5,  -5,  -5,  -5, -40, -20],
    [ 20,  -5,  15,   3,   3,  15,  -5,  20],
    [  5,  -5,   3,   3,   3,   3,  -5,   5],
    [  5,  -5,   3,   3,   3,   3,  -5,   5],
    [ 20,  -5,  15,   3,   3,  15,  -5,  20],
    [-20, -40,  -5,  -5,  -5,  -5, -40, -20],
    [120, -20,  20,   5,   5,  20, -20, 120],
];

pub fn pick_move(board: &Board, mode: GameMode, seed: u64) -> Option<(usize, usize)> {
    let moves = board.valid_moves(board.turn);
    if moves.is_empty() { return None; }
    match mode {
        GameMode::VsAiEasy => {
            let mut rng = SmallRng::seed_from_u64(seed);
            Some(moves[rng.gen_range(0..moves.len())])
        }
        GameMode::VsAiNormal => {
            let mut best = moves[0];
            let mut best_count = 0usize;
            for &(r, c) in &moves {
                let n = board.flips_for_move(r, c, board.turn).len();
                if n > best_count { best_count = n; best = (r, c); }
            }
            Some(best)
        }
        GameMode::VsAiHard => {
            let depth = 4;
            let mut best_move = moves[0];
            let mut best_score = i32::MIN;
            for &(r, c) in &moves {
                let mut clone = board.clone();
                clone.apply_move(r, c);
                let score = minimax(&clone, depth - 1, i32::MIN, i32::MAX, false, board.turn);
                if score > best_score { best_score = score; best_move = (r, c); }
            }
            Some(best_move)
        }
        GameMode::VsLocal => None,
    }
}

fn minimax(board: &Board, depth: u32, mut alpha: i32, mut beta: i32, maximizing: bool, ai_side: Side) -> i32 {
    if depth == 0 || board.is_game_over() { return evaluate(board, ai_side); }
    let moves = board.valid_moves(board.turn);
    if moves.is_empty() { return evaluate(board, ai_side); }
    if maximizing {
        let mut max_eval = i32::MIN;
        for &(r, c) in &moves {
            let mut clone = board.clone();
            clone.apply_move(r, c);
            let eval = minimax(&clone, depth - 1, alpha, beta, false, ai_side);
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha { break; }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for &(r, c) in &moves {
            let mut clone = board.clone();
            clone.apply_move(r, c);
            let eval = minimax(&clone, depth - 1, alpha, beta, true, ai_side);
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha { break; }
        }
        min_eval
    }
}

fn evaluate(board: &Board, ai_side: Side) -> i32 {
    let opp = ai_side.opponent();
    let mut score: i32 = 0;
    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            if let crate::reversi::board::Cell::Piece(s) = board.cells[r][c] {
                let w = POS_WEIGHT[r][c];
                if s == ai_side { score += w; } else { score -= w; }
            }
        }
    }
    let ai_moves = board.valid_moves(ai_side).len() as i32;
    let opp_moves = board.valid_moves(opp).len() as i32;
    score += (ai_moves - opp_moves) * 10;
    score += (board.hp(ai_side) - board.hp(opp)) / 50;
    score += (board.piece_count(ai_side) as i32 - board.piece_count(opp) as i32) * 2;
    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reversi::board::Board;

    #[test]
    fn easy_ai_returns_valid_move() {
        let b = Board::new(42);
        let m = pick_move(&b, GameMode::VsAiEasy, 123);
        assert!(m.is_some());
        let (r, c) = m.unwrap();
        assert!(b.is_valid_move(r, c, b.turn));
    }

    #[test]
    fn normal_ai_returns_valid_move() {
        let b = Board::new(42);
        let m = pick_move(&b, GameMode::VsAiNormal, 0);
        assert!(m.is_some());
        let (r, c) = m.unwrap();
        assert!(b.is_valid_move(r, c, b.turn));
    }

    #[test]
    fn hard_ai_returns_valid_move() {
        let b = Board::new(42);
        let m = pick_move(&b, GameMode::VsAiHard, 0);
        assert!(m.is_some());
        let (r, c) = m.unwrap();
        assert!(b.is_valid_move(r, c, b.turn));
    }

    #[test]
    fn hard_ai_prefers_corner() {
        let mut b = Board::new(42);
        b.cells = [[crate::reversi::board::Cell::Empty; 8]; 8];
        b.cells[0][1] = crate::reversi::board::Cell::Piece(Side::Alice);
        b.cells[0][2] = crate::reversi::board::Cell::Piece(Side::Edie);
        b.turn = Side::Edie;
        if b.is_valid_move(0, 0, Side::Edie) {
            let m = pick_move(&b, GameMode::VsAiHard, 0);
            assert_eq!(m, Some((0, 0)));
        }
    }
}
