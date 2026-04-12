//! Yut Nori game state machine.

use crate::yut::board::{resolve_move, is_shortcut_corner, EXITED, HOME};
use crate::yut::throw::{throw_yut, YutResult};
use crate::yut::powers::{Power, grant_random_power, apply_immediate, POWER_GRANT_INTERVAL, MAX_HELD_POWERS};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

pub const MAX_PLAYERS: usize = 4;
pub const PIECES_PER_PLAYER: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Menu,
    Throwing,
    SelectPiece,
    /// Player must choose: take shortcut or stay on outer ring.
    SelectPath,
    Moving,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub pos: usize,
    /// How many pieces are stacked here (1 = just this piece).
    pub stack: u8,
    /// Shield turns remaining (0 = no shield).
    pub shield: u8,
}

impl Piece {
    pub fn new() -> Self {
        Self { pos: HOME, stack: 1, shield: 0 }
    }

    pub fn is_home(&self) -> bool { self.pos == HOME }
    pub fn is_exited(&self) -> bool { self.pos == EXITED }
    pub fn is_on_board(&self) -> bool { !self.is_home() && !self.is_exited() }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub pieces: [Piece; PIECES_PER_PLAYER],
    pub finished: u8,
}

impl Player {
    pub fn new() -> Self {
        Self {
            pieces: [Piece::new(); PIECES_PER_PLAYER],
            finished: 0,
        }
    }

    pub fn all_exited(&self) -> bool {
        self.pieces.iter().all(|p| p.is_exited())
    }

    /// Pieces that can be moved (on board or at home).
    pub fn movable_pieces(&self) -> Vec<usize> {
        self.pieces.iter().enumerate()
            .filter(|(_, p)| !p.is_exited())
            .map(|(i, _)| i)
            .collect()
    }
}

pub struct YutGame {
    pub players: Vec<Player>,
    pub num_players: usize,
    pub current_player: usize,
    pub phase: Phase,
    pub last_throw: Option<YutResult>,
    pub last_sticks: Option<[bool; 4]>,
    pub rng: SmallRng,
    pub selected_piece: Option<usize>,
    pub winner: Option<usize>,
    pub bonus_turns: u32,
    pub turn_count: u32,
    pub toast: Option<(String, f32)>,
    // Superpower system
    pub power_cards: Vec<Vec<Power>>,
    pub midas_active: bool,
    pub aurora_bonus: usize,
    pub blocked_cells: Vec<(usize, u8)>,  // (position, turns_remaining)
    pub traps: Vec<(usize, usize)>,       // (position, owner_player)
}

impl YutGame {
    pub fn new(seed: u64) -> Self {
        Self {
            players: Vec::new(),
            num_players: 2,
            current_player: 0,
            phase: Phase::Menu,
            last_throw: None,
            last_sticks: None,
            rng: SmallRng::seed_from_u64(seed),
            selected_piece: None,
            winner: None,
            bonus_turns: 0,
            turn_count: 0,
            toast: None,
            power_cards: Vec::new(),
            midas_active: false,
            aurora_bonus: 0,
            blocked_cells: Vec::new(),
            traps: Vec::new(),
        }
    }

    pub fn start_game(&mut self, num_players: usize) {
        self.num_players = num_players.clamp(2, MAX_PLAYERS);
        self.players = (0..self.num_players).map(|_| Player::new()).collect();
        self.current_player = 0;
        self.phase = Phase::Throwing;
        self.last_throw = None;
        self.last_sticks = None;
        self.selected_piece = None;
        self.winner = None;
        self.bonus_turns = 0;
        self.turn_count = 0;
        self.toast = None;
        self.power_cards = (0..self.num_players).map(|_| Vec::new()).collect();
        self.midas_active = false;
        self.aurora_bonus = 0;
        self.blocked_cells = Vec::new();
        self.traps = Vec::new();
    }

    /// Use a power card from the current player's hand.
    pub fn use_power(&mut self, card_idx: usize) {
        if self.phase != Phase::Throwing && self.phase != Phase::SelectPiece { return; }
        let pi = self.current_player;
        if card_idx >= self.power_cards[pi].len() { return; }
        let power = self.power_cards[pi][card_idx];
        if apply_immediate(self, power) {
            self.power_cards[pi].remove(card_idx);
        } else {
            // Powers needing target selection: simplified — apply with defaults
            match power {
                Power::ForceReturn => {
                    let next = (pi + 1) % self.num_players;
                    for i in 0..PIECES_PER_PLAYER {
                        if self.players[next].pieces[i].is_on_board() {
                            crate::yut::powers::apply_force_return(self, next, i);
                            self.power_cards[pi].remove(card_idx);
                            return;
                        }
                    }
                    self.toast = Some(("대상 없음".into(), 1.0));
                }
                Power::Shield => {
                    for i in 0..PIECES_PER_PLAYER {
                        if self.players[pi].pieces[i].is_on_board() {
                            crate::yut::powers::apply_shield(self, i);
                            self.power_cards[pi].remove(card_idx);
                            return;
                        }
                    }
                }
                Power::VirusTrap => {
                    // Place trap on a random empty position
                    let pos = self.rng.gen_range(0..crate::yut::board::NUM_POSITIONS);
                    self.traps.push((pos, pi));
                    self.toast = Some(("함정 설치!".into(), 1.5));
                    self.power_cards[pi].remove(card_idx);
                }
                Power::MungchiBlock => {
                    let pos = self.rng.gen_range(0..crate::yut::board::NUM_POSITIONS);
                    self.blocked_cells.push((pos, 3));
                    self.toast = Some(("칸 봉쇄!".into(), 1.5));
                    self.power_cards[pi].remove(card_idx);
                }
                Power::Split => {
                    let next = (pi + 1) % self.num_players;
                    for i in 0..PIECES_PER_PLAYER {
                        if self.players[next].pieces[i].stack > 1 {
                            self.players[next].pieces[i].stack = 1;
                            self.toast = Some(("분열!".into(), 1.5));
                            self.power_cards[pi].remove(card_idx);
                            return;
                        }
                    }
                    self.toast = Some(("대상 없음".into(), 1.0));
                }
                Power::EnergyDrain => {
                    let next = (pi + 1) % self.num_players;
                    if !self.power_cards[next].is_empty() {
                        let stolen = self.power_cards[next].remove(0);
                        if self.power_cards[pi].len() < MAX_HELD_POWERS {
                            self.power_cards[pi].push(stolen); // don't remove card_idx here, different card
                        }
                        // Remove the used EnergyDrain card
                        if let Some(pos) = self.power_cards[pi].iter().position(|&p| p == Power::EnergyDrain) {
                            self.power_cards[pi].remove(pos);
                        }
                        self.toast = Some((format!("{} 뺏기!", stolen.name()), 1.5));
                    } else {
                        self.toast = Some(("상대 카드 없음".into(), 1.0));
                    }
                }
                Power::Teleport => {
                    for i in 0..PIECES_PER_PLAYER {
                        if self.players[pi].pieces[i].is_on_board() {
                            let dest = self.rng.gen_range(0..crate::yut::board::NUM_POSITIONS);
                            crate::yut::powers::apply_teleport(self, i, dest);
                            self.power_cards[pi].remove(card_idx);
                            return;
                        }
                    }
                }
                Power::BoardFlip => {
                    let next = (pi + 1) % self.num_players;
                    let own = (0..PIECES_PER_PLAYER).find(|&i| self.players[pi].pieces[i].is_on_board());
                    let opp = (0..PIECES_PER_PLAYER).find(|&i| self.players[next].pieces[i].is_on_board());
                    if let (Some(o), Some(t)) = (own, opp) {
                        crate::yut::powers::apply_board_flip(self, o, next, t);
                        self.power_cards[pi].remove(card_idx);
                    } else {
                        self.toast = Some(("대상 없음".into(), 1.0));
                    }
                }
                Power::TimeRewind => {
                    let next = (pi + 1) % self.num_players;
                    for i in 0..PIECES_PER_PLAYER {
                        if self.players[next].pieces[i].is_on_board() {
                            self.players[next].pieces[i].pos = HOME;
                            self.players[next].pieces[i].stack = 1;
                            self.toast = Some(("시간 역행!".into(), 1.5));
                            self.power_cards[pi].remove(card_idx);
                            return;
                        }
                    }
                }
                Power::MergeCall => {
                    let positions: Vec<(usize, usize)> = (0..PIECES_PER_PLAYER)
                        .filter(|&i| self.players[pi].pieces[i].is_on_board())
                        .map(|i| (i, self.players[pi].pieces[i].pos))
                        .collect();
                    if positions.len() >= 2 {
                        let dest = positions[0].1;
                        self.players[pi].pieces[positions[1].0].pos = dest;
                        self.players[pi].pieces[positions[0].0].stack += self.players[pi].pieces[positions[1].0].stack;
                        self.players[pi].pieces[positions[1].0].stack = 0;
                        self.toast = Some(("합체!".into(), 1.5));
                        self.power_cards[pi].remove(card_idx);
                    }
                }
                Power::YutControl => {
                    // Give a Yut (4 steps + bonus) as controlled throw
                    self.last_throw = Some(YutResult::Yut);
                    self.last_sticks = Some([true, true, true, true]);
                    self.bonus_turns += 1;
                    self.toast = Some(("윷 조작: 윷!".into(), 1.5));
                    self.power_cards[pi].remove(card_idx);
                    self.phase = Phase::SelectPiece;
                }
                _ => {}
            }
        }
    }

    /// Perform the yut throw for the current player.
    pub fn do_throw(&mut self) {
        if self.phase != Phase::Throwing { return; }
        let (mut result, sticks) = throw_yut(&mut self.rng);
        // Midas Touch: double the throw
        if self.midas_active {
            self.midas_active = false;
            // Can't double beyond Mo(5), so cap at 5
            result = match result {
                YutResult::Do => YutResult::Gae,
                YutResult::Gae => YutResult::Yut,
                YutResult::Geol => YutResult::Mo, // 3*2=6 → cap at Mo(5)
                r => r, // Yut/Mo already max
            };
        }
        self.last_throw = Some(result);
        self.last_sticks = Some(sticks);
        if result.grants_bonus() {
            self.bonus_turns += 1;
        }
        // Grant power card every N turns
        let pi = self.current_player;
        if self.turn_count > 0 && self.turn_count % POWER_GRANT_INTERVAL == 0 {
            if self.power_cards[pi].len() < MAX_HELD_POWERS {
                let pw = grant_random_power(&mut self.rng);
                self.power_cards[pi].push(pw);
                self.toast = Some((format!("{}! + 초능력: {}", result.name_ko(), pw.name()), 2.0));
            } else {
                self.toast = Some((format!("{}! ({}칸)", result.name_ko(), result.steps()), 1.5));
            }
        } else {
            self.toast = Some((format!("{}! ({}칸)", result.name_ko(), result.steps()), 1.5));
        }
        // Check if the player has any movable pieces
        let player = &self.players[self.current_player];
        let movable = player.movable_pieces();
        if movable.is_empty() {
            self.advance_turn();
        } else if movable.len() == 1 {
            // Auto-select the only movable piece
            self.selected_piece = Some(movable[0]);
            self.try_move_selected();
        } else {
            self.phase = Phase::SelectPiece;
        }
    }

    /// Player selects which piece to move.
    pub fn select_piece(&mut self, piece_idx: usize) {
        if self.phase != Phase::SelectPiece { return; }
        let player = &self.players[self.current_player];
        if piece_idx >= PIECES_PER_PLAYER { return; }
        if player.pieces[piece_idx].is_exited() { return; }
        self.selected_piece = Some(piece_idx);
        self.try_move_selected();
    }

    fn try_move_selected(&mut self) {
        let piece_idx = match self.selected_piece {
            Some(i) => i,
            None => return,
        };
        let pos = self.players[self.current_player].pieces[piece_idx].pos;
        let steps = self.last_throw.map(|t| t.steps()).unwrap_or(0);

        // If on a shortcut corner, ask the player to choose path
        if is_shortcut_corner(pos) && steps > 0 {
            self.phase = Phase::SelectPath;
            return;
        }

        self.execute_move(false);
    }

    /// Player chooses whether to take the shortcut (true) or stay outer (false).
    pub fn choose_path(&mut self, take_shortcut: bool) {
        if self.phase != Phase::SelectPath { return; }
        self.execute_move(take_shortcut);
    }

    fn execute_move(&mut self, take_shortcut: bool) {
        let piece_idx = match self.selected_piece {
            Some(i) => i,
            None => return,
        };
        let base_steps = self.last_throw.map(|t| t.steps()).unwrap_or(0);
        let steps = base_steps + self.aurora_bonus;
        self.aurora_bonus = 0;
        let pos = self.players[self.current_player].pieces[piece_idx].pos;
        let dest = resolve_move(pos, steps, take_shortcut);

        // Move the piece
        self.players[self.current_player].pieces[piece_idx].pos = dest;

        if dest == EXITED {
            self.players[self.current_player].pieces[piece_idx].stack = 0;
            // Check win
            if self.players[self.current_player].all_exited() {
                self.winner = Some(self.current_player);
                self.phase = Phase::GameOver;
                return;
            }
        } else {
            // Check for capture or stacking at destination
            self.resolve_landing(dest, piece_idx);
        }

        self.selected_piece = None;
        self.advance_turn();
    }

    fn resolve_landing(&mut self, dest: usize, piece_idx: usize) {
        let current = self.current_player;

        // Check traps
        if let Some(trap_idx) = self.traps.iter().position(|&(pos, owner)| pos == dest && owner != current) {
            self.traps.remove(trap_idx);
            self.players[current].pieces[piece_idx].pos = HOME;
            self.players[current].pieces[piece_idx].stack = 1;
            self.toast = Some(("함정에 걸림!".into(), 1.5));
            return;
        }

        // Check other players' pieces at this position
        for p in 0..self.num_players {
            if p == current { continue; }
            for i in 0..PIECES_PER_PLAYER {
                if self.players[p].pieces[i].pos == dest {
                    if self.players[p].pieces[i].shield > 0 {
                        self.toast = Some(("Shield blocked capture!".into(), 1.5));
                        continue;
                    }
                    // Capture! Send opponent piece(s) home
                    let stack = self.players[p].pieces[i].stack;
                    self.players[p].pieces[i].pos = HOME;
                    self.players[p].pieces[i].stack = 1;
                    // If stacked, also send home any other pieces at same pos
                    if stack > 1 {
                        for j in 0..PIECES_PER_PLAYER {
                            if j != i && self.players[p].pieces[j].pos == dest {
                                self.players[p].pieces[j].pos = HOME;
                                self.players[p].pieces[j].stack = 1;
                            }
                        }
                    }
                    self.toast = Some(("Captured!".into(), 1.5));
                    self.bonus_turns += 1; // Capture grants bonus turn
                }
            }
        }

        // Check for stacking with own pieces
        for i in 0..PIECES_PER_PLAYER {
            if i == piece_idx { continue; }
            if self.players[current].pieces[i].pos == dest && self.players[current].pieces[i].is_on_board() {
                // Stack: merge pieces
                let combined = self.players[current].pieces[piece_idx].stack
                    + self.players[current].pieces[i].stack;
                self.players[current].pieces[piece_idx].stack = combined;
                // Mark the other piece as riding along (same pos, stack=0 sentinel)
                // Actually, let's just track stack count on the "primary" piece
                self.players[current].pieces[i].stack = 0; // mark as stacked-onto
                self.toast = Some(("Stacked!".into(), 1.0));
            }
        }
    }

    fn advance_turn(&mut self) {
        // Decrement shields
        for p in &mut self.players {
            for piece in &mut p.pieces {
                if piece.shield > 0 { piece.shield -= 1; }
            }
        }
        // Decrement blocked cells
        self.blocked_cells.retain_mut(|(_pos, turns)| {
            *turns -= 1;
            *turns > 0
        });

        if self.bonus_turns > 0 {
            self.bonus_turns -= 1;
            self.phase = Phase::Throwing;
            return;
        }
        self.current_player = (self.current_player + 1) % self.num_players;
        self.turn_count += 1;
        self.phase = Phase::Throwing;
        self.last_throw = None;
    }

    pub fn update(&mut self, dt: f32) {
        if let Some((_, ref mut t)) = self.toast {
            *t -= dt;
            if *t <= 0.0 { self.toast = None; }
        }
    }

    pub fn current_player_name(&self) -> &'static str {
        match self.current_player {
            0 => "EDIE",
            1 => "ALICE",
            2 => "AMY",
            3 => "BOXBOT",
            _ => "???",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_game(n: usize) -> YutGame {
        let mut g = YutGame::new(42);
        g.start_game(n);
        g
    }

    #[test]
    fn game_starts_in_throwing_phase() {
        let g = new_game(2);
        assert_eq!(g.phase, Phase::Throwing);
        assert_eq!(g.current_player, 0);
        assert_eq!(g.players.len(), 2);
    }

    #[test]
    fn all_pieces_start_at_home() {
        let g = new_game(4);
        for p in &g.players {
            for piece in &p.pieces {
                assert_eq!(piece.pos, HOME);
            }
        }
    }

    #[test]
    fn throw_transitions_to_select_piece() {
        let mut g = new_game(2);
        g.do_throw();
        // Should be in SelectPiece (all 4 pieces are movable from HOME)
        assert!(matches!(g.phase, Phase::SelectPiece));
        assert!(g.last_throw.is_some());
    }

    #[test]
    fn select_piece_and_move() {
        let mut g = new_game(2);
        g.do_throw();
        let steps = g.last_throw.unwrap().steps();
        let bonus = g.last_throw.unwrap().grants_bonus();
        g.select_piece(0);
        // Piece should have moved
        if g.phase != Phase::SelectPath {
            // If not on a shortcut corner, piece should be on board
            let expected_pos = steps - 1; // HOME → 0 → steps-1
            assert_eq!(g.players[0].pieces[0].pos, expected_pos);
        }
        // Turn should advance (unless bonus)
        if !bonus && g.phase != Phase::SelectPath {
            assert_eq!(g.current_player, 1);
        }
    }

    #[test]
    fn capture_sends_home_and_grants_bonus() {
        let mut g = new_game(2);
        // Place P1 piece at pos 3
        g.players[1].pieces[0].pos = 3;
        // Move P0 piece to pos 3
        g.players[0].pieces[0].pos = HOME;
        g.last_throw = Some(YutResult::Yut); // 4 steps: HOME→0→1→2→3
        g.phase = Phase::SelectPiece;
        g.select_piece(0);
        // P1's piece should be back home
        assert_eq!(g.players[1].pieces[0].pos, HOME);
    }

    #[test]
    fn stacking_merges_pieces() {
        let mut g = new_game(2);
        g.players[0].pieces[0].pos = 5;
        g.players[0].pieces[1].pos = 3;
        g.last_throw = Some(YutResult::Gae); // 2 steps
        g.phase = Phase::SelectPiece;
        g.select_piece(1); // Move piece 1 from 3 to 5
        if g.phase != Phase::SelectPath {
            // Should be stacked
            assert_eq!(g.players[0].pieces[0].pos, 5);
            assert!(g.players[0].pieces[0].stack >= 2 || g.players[0].pieces[1].stack >= 2);
        }
    }

    #[test]
    fn win_when_all_exited() {
        let mut g = new_game(2);
        // Set 3 pieces as exited, 1 near exit
        for i in 0..3 {
            g.players[0].pieces[i].pos = EXITED;
        }
        g.players[0].pieces[3].pos = 18;
        g.last_throw = Some(YutResult::Gae); // 2 steps: 18→19→EXIT
        g.phase = Phase::SelectPiece;
        g.select_piece(3);
        assert_eq!(g.phase, Phase::GameOver);
        assert_eq!(g.winner, Some(0));
    }

    #[test]
    fn turn_cycles_through_players() {
        let mut g = new_game(3);
        // Force Do (no bonus) and auto-move
        g.players[0].pieces[0].pos = 0;
        g.players[0].pieces[1].pos = EXITED;
        g.players[0].pieces[2].pos = EXITED;
        g.players[0].pieces[3].pos = EXITED;
        g.last_throw = Some(YutResult::Do);
        g.phase = Phase::SelectPiece;
        g.select_piece(0);
        // Should advance to player 1
        if g.phase == Phase::Throwing {
            assert_eq!(g.current_player, 1);
        }
    }

    #[test]
    fn four_player_game_works() {
        let g = new_game(4);
        assert_eq!(g.players.len(), 4);
        assert_eq!(g.num_players, 4);
    }
}
