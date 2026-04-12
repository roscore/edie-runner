//! Yut Nori board: 29 positions in a cross-shaped graph.
//!
//! Outer ring: 0–19 (counterclockwise from start).
//! Diagonal A (from corner 5): 20→21→24(center)→27→28→EXIT
//! Diagonal B (from corner 10): 22→23→24(center)→25→26→EXIT
//! A piece completing the outer ring past 19 also exits.

/// Total named positions on the board.
pub const NUM_POSITIONS: usize = 29;
/// Sentinel value: piece has completed the circuit.
pub const EXITED: usize = 99;
/// Sentinel value: piece is waiting to enter the board.
pub const HOME: usize = 100;

/// Corner positions where diagonal shortcuts are available.
pub const CORNER_NE: usize = 5;
pub const CORNER_NW: usize = 10;

/// The center position shared by both diagonals.
pub const CENTER: usize = 24;

/// Resolve the next position along the OUTER ring. Returns EXITED if
/// the piece crosses past position 19.
pub fn next_outer(pos: usize, steps: usize) -> usize {
    let target = pos + steps;
    if target >= 20 { EXITED } else { target }
}

/// Resolve movement along diagonal A (from corner 5 inward).
/// Path: 5 → 20 → 21 → 24(center) → 27 → 28 → EXIT
const DIAG_A: [usize; 6] = [20, 21, 24, 27, 28, EXITED];

/// Resolve movement along diagonal B (from corner 10 inward).
/// Path: 10 → 22 → 23 → 24(center) → 25 → 26 → EXIT
const DIAG_B: [usize; 6] = [22, 23, 24, 25, 26, EXITED];

/// From-center path (used when a piece is already at center 24).
/// Path: 24 → 25 → 26 → EXIT  (or 24 → 27 → 28 → EXIT via diag A)
/// We use the shorter "toward exit" path: 25 → 26 → EXIT.
const FROM_CENTER: [usize; 3] = [25, 26, EXITED];

/// Index of `pos` within a diagonal path, or None.
fn diag_index(path: &[usize], pos: usize) -> Option<usize> {
    path.iter().position(|&p| p == pos)
}

/// Resolve the destination after moving `steps` from `pos`.
/// `take_shortcut`: if the piece is exactly on a corner, whether to
/// enter the diagonal.  For positions already on a diagonal the piece
/// continues along that diagonal automatically.
pub fn resolve_move(pos: usize, steps: usize, take_shortcut: bool) -> usize {
    if pos == HOME {
        // Entering the board: start at position 0 then move steps-1 more.
        if steps <= 1 { return 0; }
        return resolve_move(0, steps - 1, false);
    }

    // Center always exits via FROM_CENTER path (must check before diagonals
    // because CENTER appears in both DIAG_A and DIAG_B).
    if pos == CENTER {
        let target = steps;
        return if target > FROM_CENTER.len() { EXITED }
        else if target == 0 { CENTER }
        else { FROM_CENTER[target - 1] };
    }

    // Already on diagonal A?
    if let Some(idx) = diag_index(&DIAG_A, pos) {
        let target = idx + steps;
        return if target >= DIAG_A.len() { EXITED } else { DIAG_A[target] };
    }
    // Already on diagonal B?
    if let Some(idx) = diag_index(&DIAG_B, pos) {
        let target = idx + steps;
        return if target >= DIAG_B.len() { EXITED } else { DIAG_B[target] };
    }

    // On the outer ring.
    if take_shortcut && pos == CORNER_NE {
        // Enter diagonal A
        return if steps > DIAG_A.len() { EXITED }
        else if steps == 0 { CORNER_NE }
        else { DIAG_A[steps - 1] };
    }
    if take_shortcut && pos == CORNER_NW {
        // Enter diagonal B
        return if steps > DIAG_B.len() { EXITED }
        else if steps == 0 { CORNER_NW }
        else { DIAG_B[steps - 1] };
    }

    next_outer(pos, steps)
}

/// All positions reachable from a given position. Used for rendering
/// valid move highlights.
pub fn is_on_diagonal(pos: usize) -> bool {
    matches!(pos, 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28)
}

/// True if position is a corner where a shortcut decision is available.
pub fn is_shortcut_corner(pos: usize) -> bool {
    pos == CORNER_NE || pos == CORNER_NW
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_to_board() {
        assert_eq!(resolve_move(HOME, 1, false), 0);
        assert_eq!(resolve_move(HOME, 3, false), 2);
        assert_eq!(resolve_move(HOME, 5, false), 4);
    }

    #[test]
    fn outer_ring_movement() {
        assert_eq!(resolve_move(0, 3, false), 3);
        assert_eq!(resolve_move(5, 2, false), 7);
        assert_eq!(resolve_move(17, 2, false), 19);
    }

    #[test]
    fn outer_ring_exit() {
        assert_eq!(resolve_move(18, 3, false), EXITED);
        assert_eq!(resolve_move(19, 1, false), EXITED);
    }

    #[test]
    fn shortcut_from_corner_ne() {
        // Corner 5 → diagonal A
        assert_eq!(resolve_move(5, 1, true), 20);
        assert_eq!(resolve_move(5, 2, true), 21);
        assert_eq!(resolve_move(5, 3, true), 24); // center
        assert_eq!(resolve_move(5, 4, true), 27);
        assert_eq!(resolve_move(5, 5, true), 28);
        assert_eq!(resolve_move(5, 6, true), EXITED);
    }

    #[test]
    fn shortcut_from_corner_nw() {
        // Corner 10 → diagonal B
        assert_eq!(resolve_move(10, 1, true), 22);
        assert_eq!(resolve_move(10, 3, true), 24); // center
        assert_eq!(resolve_move(10, 5, true), 26);
        assert_eq!(resolve_move(10, 6, true), EXITED);
    }

    #[test]
    fn no_shortcut_stays_outer() {
        assert_eq!(resolve_move(5, 2, false), 7);
        assert_eq!(resolve_move(10, 3, false), 13);
    }

    #[test]
    fn diagonal_continuation() {
        // Already on diagonal A at pos 20
        assert_eq!(resolve_move(20, 1, false), 21);
        assert_eq!(resolve_move(20, 2, false), 24); // center
        assert_eq!(resolve_move(21, 3, false), 28);
    }

    #[test]
    fn diagonal_exit() {
        assert_eq!(resolve_move(28, 1, false), EXITED);
        assert_eq!(resolve_move(26, 1, false), EXITED);
    }

    #[test]
    fn center_continues_toward_exit() {
        assert_eq!(resolve_move(24, 1, false), 25);
        assert_eq!(resolve_move(24, 2, false), 26);
        assert_eq!(resolve_move(24, 3, false), EXITED);
    }

    #[test]
    fn home_large_throw_exits() {
        // Mo (5) from home: 0→1→2→3→4 = pos 4
        assert_eq!(resolve_move(HOME, 5, false), 4);
    }
}
