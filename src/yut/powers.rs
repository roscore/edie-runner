//! Superpower card system for EDIE Yut Nori.
//! 16 unique abilities that add strategic depth to the traditional game.

use crate::yut::board::{resolve_move, is_shortcut_corner, EXITED, HOME, NUM_POSITIONS};
use crate::yut::game::{YutGame, Phase, PIECES_PER_PLAYER};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

/// Every N turns, a player gains a random superpower card.
pub const POWER_GRANT_INTERVAL: u32 = 3;
/// Max cards a player can hold.
pub const MAX_HELD_POWERS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Power {
    AuroraDash,     // +2 extra steps this move
    VirusTrap,      // Place trap on empty cell (landing sends piece home)
    ForceReturn,    // Send any opponent piece home
    Teleport,       // Move own piece to any empty cell
    Shield,         // Protect piece from capture for 2 turns
    BoardFlip,      // Swap own piece with opponent piece
    MidasTouch,     // Double next yut throw result
    TimeRewind,     // Undo opponent's last move (simplified: send their last-moved piece home)
    MergeCall,      // Stack two own pieces regardless of position
    Split,          // Unstack opponent's stacked piece
    MungchiBlock,   // Block a cell for 3 turns
    EnergyDrain,    // Steal one of opponent's power cards
    YutControl,     // Choose next throw result
    ExtraTurn,      // Get an immediate bonus turn
    Revive,         // A captured piece re-enters at start
    Scout,          // Reveal opponent's held power cards (toast message)
}

impl Power {
    pub fn name(self) -> &'static str {
        match self {
            Power::AuroraDash => "오로라 대시",
            Power::VirusTrap => "바이러스 트랩",
            Power::ForceReturn => "강제 귀가",
            Power::Teleport => "텔레포트",
            Power::Shield => "방어막",
            Power::BoardFlip => "밥상 뒤집기",
            Power::MidasTouch => "마이더스의 손",
            Power::TimeRewind => "시간 역행",
            Power::MergeCall => "합체 소환",
            Power::Split => "분열",
            Power::MungchiBlock => "몽치 소환",
            Power::EnergyDrain => "에너지 드레인",
            Power::YutControl => "윷 조작",
            Power::ExtraTurn => "연속 턴",
            Power::Revive => "부활",
            Power::Scout => "정찰",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Power::AuroraDash => "+2칸 추가 이동",
            Power::VirusTrap => "빈 칸에 함정 설치",
            Power::ForceReturn => "상대 말 1개 귀환",
            Power::Teleport => "내 말 아무 곳으로",
            Power::Shield => "2턴간 잡기 방지",
            Power::BoardFlip => "말 위치 교환",
            Power::MidasTouch => "다음 윷 x2",
            Power::TimeRewind => "상대 마지막 수 취소",
            Power::MergeCall => "내 말 2개 합체",
            Power::Split => "상대 업힌 말 분리",
            Power::MungchiBlock => "칸 3턴 봉쇄",
            Power::EnergyDrain => "상대 카드 뺏기",
            Power::YutControl => "윷 결과 선택",
            Power::ExtraTurn => "즉시 추가 턴",
            Power::Revive => "잡힌 말 부활",
            Power::Scout => "상대 카드 공개",
        }
    }

    pub fn all() -> [Power; 16] {
        [
            Power::AuroraDash, Power::VirusTrap, Power::ForceReturn, Power::Teleport,
            Power::Shield, Power::BoardFlip, Power::MidasTouch, Power::TimeRewind,
            Power::MergeCall, Power::Split, Power::MungchiBlock, Power::EnergyDrain,
            Power::YutControl, Power::ExtraTurn, Power::Revive, Power::Scout,
        ]
    }
}

/// Grant a random power card to a player.
pub fn grant_random_power(rng: &mut SmallRng) -> Power {
    let all = Power::all();
    all[rng.gen_range(0..all.len())]
}

/// Apply a power that needs no target selection (immediate effect).
/// Returns true if the power was used, false if it needs target selection.
pub fn apply_immediate(game: &mut YutGame, power: Power) -> bool {
    let pi = game.current_player;
    match power {
        Power::ExtraTurn => {
            game.bonus_turns += 1;
            game.toast = Some((format!("{}: 추가 턴!", power.name()), 1.5));
            true
        }
        Power::MidasTouch => {
            game.midas_active = true;
            game.toast = Some((format!("{}: 다음 윷 x2!", power.name()), 1.5));
            true
        }
        Power::Scout => {
            // Reveal next player's cards
            let next = (pi + 1) % game.num_players;
            let cards: Vec<String> = game.power_cards[next].iter().map(|p| p.name().to_string()).collect();
            let msg = if cards.is_empty() {
                format!("P{} has no cards", next + 1)
            } else {
                format!("P{}: {}", next + 1, cards.join(", "))
            };
            game.toast = Some((msg, 3.0));
            true
        }
        Power::Revive => {
            // Find a home piece and put it at position 0
            for i in 0..PIECES_PER_PLAYER {
                if game.players[pi].pieces[i].is_home() {
                    game.players[pi].pieces[i].pos = 0;
                    game.toast = Some((format!("{}: 말 부활!", power.name()), 1.5));
                    return true;
                }
            }
            game.toast = Some(("부활할 말이 없습니다".into(), 1.5));
            false
        }
        Power::AuroraDash => {
            game.aurora_bonus = 2;
            game.toast = Some((format!("{}: +2칸!", power.name()), 1.5));
            true
        }
        // Powers that need target selection
        _ => false,
    }
}

/// Apply ForceReturn: send target opponent piece home.
pub fn apply_force_return(game: &mut YutGame, target_player: usize, target_piece: usize) -> bool {
    if target_player == game.current_player { return false; }
    if target_piece >= PIECES_PER_PLAYER { return false; }
    let piece = &game.players[target_player].pieces[target_piece];
    if !piece.is_on_board() { return false; }
    if piece.shield > 0 {
        game.toast = Some(("방어막으로 막힘!".into(), 1.5));
        return false;
    }
    game.players[target_player].pieces[target_piece].pos = HOME;
    game.players[target_player].pieces[target_piece].stack = 1;
    game.toast = Some(("강제 귀가!".into(), 1.5));
    true
}

/// Apply Shield to own piece.
pub fn apply_shield(game: &mut YutGame, piece_idx: usize) -> bool {
    let pi = game.current_player;
    if piece_idx >= PIECES_PER_PLAYER { return false; }
    if !game.players[pi].pieces[piece_idx].is_on_board() { return false; }
    game.players[pi].pieces[piece_idx].shield = 2;
    game.toast = Some(("방어막 활성화!".into(), 1.5));
    true
}

/// Apply Teleport: move own piece to any position.
pub fn apply_teleport(game: &mut YutGame, piece_idx: usize, dest_pos: usize) -> bool {
    let pi = game.current_player;
    if piece_idx >= PIECES_PER_PLAYER { return false; }
    if dest_pos >= NUM_POSITIONS { return false; }
    let piece = &game.players[pi].pieces[piece_idx];
    if piece.is_exited() { return false; }
    game.players[pi].pieces[piece_idx].pos = dest_pos;
    game.toast = Some(("텔레포트!".into(), 1.5));
    true
}

/// Apply BoardFlip: swap own piece position with opponent piece.
pub fn apply_board_flip(game: &mut YutGame, own_piece: usize, target_player: usize, target_piece: usize) -> bool {
    let pi = game.current_player;
    if target_player == pi { return false; }
    let own_pos = game.players[pi].pieces[own_piece].pos;
    let opp_pos = game.players[target_player].pieces[target_piece].pos;
    if !game.players[pi].pieces[own_piece].is_on_board() { return false; }
    if !game.players[target_player].pieces[target_piece].is_on_board() { return false; }
    game.players[pi].pieces[own_piece].pos = opp_pos;
    game.players[target_player].pieces[target_piece].pos = own_pos;
    game.toast = Some(("밥상 뒤집기!".into(), 1.5));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yut::game::YutGame;
    use crate::yut::board::HOME;

    fn setup_game() -> YutGame {
        let mut g = YutGame::new(42);
        g.start_game(2);
        // Give player 0 a piece on the board
        g.players[0].pieces[0].pos = 5;
        g.players[1].pieces[0].pos = 10;
        g
    }

    #[test]
    fn extra_turn_grants_bonus() {
        let mut g = setup_game();
        let before = g.bonus_turns;
        apply_immediate(&mut g, Power::ExtraTurn);
        assert_eq!(g.bonus_turns, before + 1);
    }

    #[test]
    fn force_return_sends_home() {
        let mut g = setup_game();
        assert_eq!(g.players[1].pieces[0].pos, 10);
        apply_force_return(&mut g, 1, 0);
        assert_eq!(g.players[1].pieces[0].pos, HOME);
    }

    #[test]
    fn shield_blocks_force_return() {
        let mut g = setup_game();
        g.players[1].pieces[0].shield = 2;
        let result = apply_force_return(&mut g, 1, 0);
        assert!(!result);
        assert_eq!(g.players[1].pieces[0].pos, 10);
    }

    #[test]
    fn teleport_moves_piece() {
        let mut g = setup_game();
        apply_teleport(&mut g, 0, 15);
        assert_eq!(g.players[0].pieces[0].pos, 15);
    }

    #[test]
    fn board_flip_swaps() {
        let mut g = setup_game();
        apply_board_flip(&mut g, 0, 1, 0);
        assert_eq!(g.players[0].pieces[0].pos, 10);
        assert_eq!(g.players[1].pieces[0].pos, 5);
    }

    #[test]
    fn revive_puts_home_piece_on_board() {
        let mut g = setup_game();
        // pieces[1] is at HOME
        assert!(g.players[0].pieces[1].is_home());
        apply_immediate(&mut g, Power::Revive);
        assert_eq!(g.players[0].pieces[1].pos, 0);
    }

    #[test]
    fn grant_random_returns_valid_power() {
        let mut rng = SmallRng::seed_from_u64(99);
        for _ in 0..50 {
            let _p = grant_random_power(&mut rng);
        }
    }
}
