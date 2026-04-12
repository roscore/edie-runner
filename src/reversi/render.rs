//! Board + HUD rendering for EDIE Battle Reverse (macroquad).
//! AeiROBOT orange->green gradient theme. All original art.

use crate::assets::AssetHandles;
use crate::render::camera::Camera;
use crate::reversi::board::{Board, Cell, Powerup, Side, BOARD_SIZE, INITIAL_HP};
use crate::reversi::game::{FlipAnim, Phase, ReversiGame};
use macroquad::prelude::*;

const BOARD_PX: f32 = 560.0;
const CELL_PX: f32 = BOARD_PX / BOARD_SIZE as f32;
const BOARD_X: f32 = (1280.0 - BOARD_PX) / 2.0;
const BOARD_Y: f32 = 80.0;
const ORANGE: Color = Color::new(0.91, 0.57, 0.23, 1.0);
const GREEN: Color = Color::new(0.36, 0.89, 0.66, 1.0);
const CELL_A: Color = Color::new(0.10, 0.21, 0.20, 1.0);
const CELL_B: Color = Color::new(0.16, 0.14, 0.13, 1.0);
const GRID_LINE: Color = Color::new(0.36, 0.89, 0.66, 0.22);

pub fn draw_reversi(game: &ReversiGame, assets: &AssetHandles, elapsed: f32) {
    let cam = Camera::new(screen_width(), screen_height());
    clear_background(Color::new(0.06, 0.06, 0.10, 1.0));
    match game.phase {
        Phase::Menu => draw_menu(&cam),
        Phase::Playing | Phase::Animating | Phase::UsingPowerup => {
            draw_board_frame(&cam);
            draw_cells(&cam);
            draw_aurora_cells(&game.board, elapsed, &cam);
            draw_viruses(&game.board, assets, elapsed, &cam);
            draw_pieces(&game.board, assets, elapsed, &cam);
            if game.phase == Phase::Playing {
                draw_valid_moves(&game.board, elapsed, &cam);
                if let Some((hr, hc)) = game.hover {
                    draw_hover(hr, hc, &cam);
                }
            }
            if game.phase == Phase::UsingPowerup {
                draw_powerup_targets(game, elapsed, &cam);
            }
            if let Some(anim) = &game.flip_anim {
                draw_flip_overlay(anim, &cam);
            }
            draw_hud(&game.board, &cam);
            draw_powerup_hud(&game.board, &cam);
            draw_turn_indicator(&game.board, elapsed, &cam);
            if let Some((ref msg, t)) = game.toast {
                draw_toast(msg, t, &cam);
            }
        }
        Phase::GameOver => {
            draw_board_frame(&cam);
            draw_cells(&cam);
            draw_pieces(&game.board, assets, elapsed, &cam);
            draw_hud(&game.board, &cam);
            draw_game_over(&game.board, &cam);
        }
    }
}

fn draw_board_frame(cam: &Camera) {
    let pad = 10.0;
    let (fx, fy) = cam.to_screen(BOARD_X - pad, BOARD_Y - pad);
    let fw = cam.scaled(BOARD_PX + pad * 2.0);
    let fh = cam.scaled(BOARD_PX + pad * 2.0);
    draw_rectangle(fx, fy, fw, fh, Color::new(0.08, 0.08, 0.12, 1.0));
    let strip_h = cam.scaled(4.0);
    for i in 0..40 {
        let t = i as f32 / 39.0;
        let c = Color::new(
            ORANGE.r + (GREEN.r - ORANGE.r) * t,
            ORANGE.g + (GREEN.g - ORANGE.g) * t,
            ORANGE.b + (GREEN.b - ORANGE.b) * t, 0.85);
        let sx = fx + (i as f32 / 40.0) * fw;
        let sw = fw / 40.0 + 1.0;
        draw_rectangle(sx, fy, sw, strip_h, c);
        draw_rectangle(sx, fy + fh - strip_h, sw, strip_h, c);
    }
    draw_rectangle_lines(fx, fy, fw, fh, 3.0, Color::new(0.5, 0.5, 0.55, 0.9));
}

fn draw_cells(cam: &Camera) {
    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            let lx = BOARD_X + c as f32 * CELL_PX;
            let ly = BOARD_Y + r as f32 * CELL_PX;
            let (sx, sy) = cam.to_screen(lx, ly);
            let s = cam.scaled(CELL_PX);
            draw_rectangle(sx, sy, s, s, if (r + c) % 2 == 0 { CELL_A } else { CELL_B });
            draw_rectangle_lines(sx, sy, s, s, 1.0, GRID_LINE);
        }
    }
    for &(cr, cc) in &[(2, 2), (2, 6), (6, 2), (6, 6)] {
        let (sx, sy) = cam.to_screen(BOARD_X + cc as f32 * CELL_PX, BOARD_Y + cr as f32 * CELL_PX);
        draw_circle(sx, sy, cam.scaled(4.0), Color::new(0.5, 0.9, 0.7, 0.45));
    }
}

fn draw_pieces(board: &Board, assets: &AssetHandles, elapsed: f32, cam: &Camera) {
    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            if let Cell::Piece(side) = board.cells[r][c] {
                let lx = BOARD_X + c as f32 * CELL_PX + 7.0;
                let ly = BOARD_Y + r as f32 * CELL_PX + 3.0;
                let bob = ((elapsed * 2.0 + (r * 3 + c) as f32 * 0.5).sin() * 2.0).round();
                let (sx, sy) = cam.to_screen(lx, ly + bob);
                let (pw, ph): (f32, f32) = match side {
                    Side::Edie => (56.0, 48.0),
                    Side::Alice => (50.0, 64.0),
                };
                let tex = match side {
                    Side::Edie => &assets.edie_static_run,
                    Side::Alice => &assets.obstacle_alice3,
                };
                draw_texture_ex(tex, sx, sy, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(cam.scaled(pw), cam.scaled(ph.min(CELL_PX - 6.0)))),
                    ..Default::default()
                });
            }
        }
    }
}

fn draw_viruses(board: &Board, assets: &AssetHandles, elapsed: f32, cam: &Camera) {
    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            if board.cells[r][c] != Cell::Virus { continue; }
            let lx = BOARD_X + c as f32 * CELL_PX + 11.0;
            let ly = BOARD_Y + r as f32 * CELL_PX + 11.0;
            let (sx, sy) = cam.to_screen(lx, ly);
            let size = CELL_PX - 22.0;
            let pulse = 0.5 + 0.5 * (elapsed * 2.0 + (r + c) as f32).sin();
            draw_circle(sx + cam.scaled(size * 0.5), sy + cam.scaled(size * 0.5),
                cam.scaled(size * 0.7), Color::new(0.3, 0.9, 0.5, 0.12 * pulse));
            let tex = if (r + c) % 2 == 0 { &assets.virus_green } else { &assets.virus_purple };
            let frame_w = (tex.width() - 3.0) / 4.0;
            let fi = ((elapsed * 2.0) as usize) % 4;
            draw_texture_ex(tex, sx, sy, WHITE, DrawTextureParams {
                dest_size: Some(vec2(cam.scaled(size), cam.scaled(size))),
                source: Some(Rect { x: fi as f32 * (frame_w + 1.0), y: 0.0, w: frame_w, h: tex.height() }),
                ..Default::default()
            });
        }
    }
}

fn draw_valid_moves(board: &Board, elapsed: f32, cam: &Camera) {
    let moves = board.valid_moves(board.turn);
    let pulse = 0.4 + 0.3 * (elapsed * 4.0).sin();
    for (r, c) in moves {
        let (sx, sy) = cam.to_screen(BOARD_X + c as f32 * CELL_PX, BOARD_Y + r as f32 * CELL_PX);
        let s = cam.scaled(CELL_PX);
        let col = match board.turn {
            Side::Edie => Color::new(ORANGE.r, ORANGE.g, ORANGE.b, pulse * 0.4),
            Side::Alice => Color::new(0.9, 0.3, 0.35, pulse * 0.4),
        };
        draw_rectangle(sx, sy, s, s, col);
        draw_circle(sx + s * 0.5, sy + s * 0.5, cam.scaled(6.0), Color::new(col.r, col.g, col.b, pulse * 0.8));
    }
}

fn draw_hover(row: usize, col: usize, cam: &Camera) {
    let (sx, sy) = cam.to_screen(BOARD_X + col as f32 * CELL_PX, BOARD_Y + row as f32 * CELL_PX);
    draw_rectangle_lines(sx, sy, cam.scaled(CELL_PX), cam.scaled(CELL_PX), 3.0, Color::new(1.0, 1.0, 1.0, 0.85));
}

fn draw_flip_overlay(anim: &FlipAnim, cam: &Camera) {
    let t = (anim.elapsed / anim.total).clamp(0.0, 1.0);
    let alpha = (1.0 - t) * 0.6;
    for &(r, c) in &anim.cells {
        let (sx, sy) = cam.to_screen(BOARD_X + c as f32 * CELL_PX, BOARD_Y + r as f32 * CELL_PX);
        draw_rectangle(sx, sy, cam.scaled(CELL_PX), cam.scaled(CELL_PX), Color::new(1.0, 1.0, 0.9, alpha));
    }
    let (pr, pc) = anim.placed;
    let (sx, sy) = cam.to_screen(BOARD_X + pc as f32 * CELL_PX, BOARD_Y + pr as f32 * CELL_PX);
    draw_rectangle_lines(sx, sy, cam.scaled(CELL_PX), cam.scaled(CELL_PX), 4.0, Color::new(1.0, 0.9, 0.3, alpha + 0.3));
}

fn draw_hud(board: &Board, cam: &Camera) {
    draw_hp_bar(30.0, 660.0, board.edie_hp, "EDIE", true, cam);
    draw_hp_bar(1050.0, 660.0, board.alice_hp, "ALICE", false, cam);
    let size = 20.0 * cam.scale;
    let ec = format!("{}", board.piece_count(Side::Edie));
    let ac = format!("{}", board.piece_count(Side::Alice));
    let (ex, ey) = cam.to_screen(130.0, 690.0);
    draw_text(&ec, ex, ey, size, WHITE);
    let (ax, ay) = cam.to_screen(1150.0, 690.0);
    draw_text(&ac, ax, ay, size, WHITE);
}

fn draw_hp_bar(lx: f32, ly: f32, hp: i32, label: &str, gradient: bool, cam: &Camera) {
    let bw = 200.0; let bh = 16.0;
    let (sx, sy) = cam.to_screen(lx, ly);
    draw_text(label, sx, sy - cam.scaled(4.0), 16.0 * cam.scale, Color::new(0.9, 0.9, 0.85, 1.0));
    draw_rectangle(sx, sy, cam.scaled(bw), cam.scaled(bh), Color::new(0.1, 0.1, 0.12, 1.0));
    let ratio = (hp as f32 / INITIAL_HP as f32).clamp(0.0, 1.0);
    let fw = bw * ratio;
    if gradient {
        for i in 0..20 {
            let t = i as f32 / 19.0;
            let c = Color::new(ORANGE.r + (GREEN.r - ORANGE.r) * t, ORANGE.g + (GREEN.g - ORANGE.g) * t, ORANGE.b + (GREEN.b - ORANGE.b) * t, 1.0);
            let step = fw / 20.0;
            let (bx, _) = cam.to_screen(lx + i as f32 * step, ly);
            draw_rectangle(bx, sy, cam.scaled(step + 0.5), cam.scaled(bh), c);
        }
    } else {
        draw_rectangle(sx, sy, cam.scaled(fw), cam.scaled(bh), Color::new(0.9, 0.3, 0.35, 1.0));
    }
    draw_rectangle_lines(sx, sy, cam.scaled(bw), cam.scaled(bh), 2.0, Color::new(0.6, 0.6, 0.55, 0.9));
    let txt = format!("{}/{}", hp.max(0), INITIAL_HP);
    let ts = 12.0 * cam.scale;
    let td = measure_text(&txt, None, ts as u16, 1.0);
    draw_text(&txt, sx + cam.scaled(bw * 0.5) - td.width * 0.5, sy + cam.scaled(bh * 0.5) + td.height * 0.3, ts, WHITE);
}

fn draw_turn_indicator(board: &Board, elapsed: f32, cam: &Camera) {
    let label = match board.turn { Side::Edie => "EDIE'S TURN", Side::Alice => "ALICE'S TURN" };
    let bounce = ((elapsed * 3.0).sin() * 4.0).abs();
    let size = 24.0 * cam.scale;
    let dim = measure_text(label, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(640.0, BOARD_Y + BOARD_PX + 30.0 + bounce);
    let col = match board.turn { Side::Edie => ORANGE, Side::Alice => Color::new(0.9, 0.3, 0.35, 1.0) };
    draw_text(label, tx - dim.width * 0.5, ty, size, col);
}

fn draw_menu(cam: &Camera) {
    let title = "EDIE BATTLE REVERSE";
    let size = 48.0 * cam.scale;
    let dim = measure_text(title, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(640.0, 200.0);
    draw_text(title, tx - dim.width * 0.5 + 3.0, ty + 3.0, size, Color::new(0.0, 0.0, 0.0, 0.5));
    draw_text(title, tx - dim.width * 0.5, ty, size, ORANGE);
    let opts = [("1. VS LOCAL (2P)", 320.0), ("2. VS AI - EASY", 370.0), ("3. VS AI - NORMAL", 420.0), ("4. VS AI - HARD", 470.0)];
    let os = 24.0 * cam.scale;
    for (l, y) in &opts {
        let d = measure_text(l, None, os as u16, 1.0);
        let (ox, oy) = cam.to_screen(640.0, *y);
        draw_text(l, ox - d.width * 0.5, oy, os, GREEN);
    }
    let hint = "CLICK OR PRESS 1-4";
    let hs = 16.0 * cam.scale;
    let hd = measure_text(hint, None, hs as u16, 1.0);
    let (hx, hy) = cam.to_screen(640.0, 550.0);
    draw_text(hint, hx - hd.width * 0.5, hy, hs, Color::new(0.6, 0.6, 0.6, 1.0));
}

fn draw_game_over(board: &Board, cam: &Camera) {
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(x0, y0, cam.scaled(1280.0), cam.scaled(720.0), Color::new(0.0, 0.0, 0.0, 0.55));
    let (title, color) = match board.winner() {
        Some(Side::Edie) => ("EDIE WINS!", ORANGE),
        Some(Side::Alice) => ("ALICE WINS!", Color::new(0.9, 0.3, 0.35, 1.0)),
        None => ("DRAW!", Color::new(0.8, 0.8, 0.8, 1.0)),
    };
    let size = 56.0 * cam.scale;
    let dim = measure_text(title, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(640.0, 320.0);
    draw_text(title, tx - dim.width * 0.5 + 4.0, ty + 4.0, size, Color::new(0.0, 0.0, 0.0, 0.7));
    draw_text(title, tx - dim.width * 0.5, ty, size, color);
    let sub = "PRESS SPACE TO PLAY AGAIN";
    let ss = 20.0 * cam.scale;
    let sd = measure_text(sub, None, ss as u16, 1.0);
    let (sx, sy) = cam.to_screen(640.0, 400.0);
    draw_text(sub, sx - sd.width * 0.5, sy, ss, Color::new(0.8, 0.8, 0.8, 1.0));
}

fn draw_aurora_cells(board: &Board, elapsed: f32, cam: &Camera) {
    for &(r, c) in &board.aurora_cells {
        let lx = BOARD_X + c as f32 * CELL_PX;
        let ly = BOARD_Y + r as f32 * CELL_PX;
        let (sx, sy) = cam.to_screen(lx, ly);
        let s = cam.scaled(CELL_PX);
        let pulse = 0.3 + 0.3 * (elapsed * 3.0 + (r + c) as f32).sin();
        // Orange-green gradient glow
        let t = (0.5 + 0.5 * (elapsed * 1.5).sin()) as f32;
        let glow = Color::new(
            ORANGE.r + (GREEN.r - ORANGE.r) * t,
            ORANGE.g + (GREEN.g - ORANGE.g) * t,
            ORANGE.b + (GREEN.b - ORANGE.b) * t,
            pulse,
        );
        draw_rectangle(sx, sy, s, s, glow);
        // Diamond shape in center
        let cx = sx + s * 0.5;
        let cy = sy + s * 0.5;
        let r2 = cam.scaled(12.0);
        draw_line(cx, cy - r2, cx + r2, cy, 2.0, Color::new(1.0, 1.0, 1.0, 0.9));
        draw_line(cx + r2, cy, cx, cy + r2, 2.0, Color::new(1.0, 1.0, 1.0, 0.9));
        draw_line(cx, cy + r2, cx - r2, cy, 2.0, Color::new(1.0, 1.0, 1.0, 0.9));
        draw_line(cx - r2, cy, cx, cy - r2, 2.0, Color::new(1.0, 1.0, 1.0, 0.9));
    }
}

fn draw_powerup_targets(game: &ReversiGame, elapsed: f32, cam: &Camera) {
    let pulse = 0.5 + 0.5 * (elapsed * 5.0).sin();
    match game.targeting_powerup {
        Some(Powerup::VirusCure) => {
            for r in 0..BOARD_SIZE {
                for c in 0..BOARD_SIZE {
                    if game.board.cells[r][c] == Cell::Virus {
                        let (sx, sy) = cam.to_screen(BOARD_X + c as f32 * CELL_PX, BOARD_Y + r as f32 * CELL_PX);
                        let s = cam.scaled(CELL_PX);
                        draw_rectangle(sx, sy, s, s, Color::new(0.2, 1.0, 0.5, pulse * 0.5));
                        draw_rectangle_lines(sx, sy, s, s, 3.0, Color::new(0.3, 1.0, 0.6, 0.9));
                    }
                }
            }
            let hint = "TAP a virus cell to cure  |  ESC to cancel";
            let hs = 18.0 * cam.scale;
            let hd = measure_text(hint, None, hs as u16, 1.0);
            let (hx, hy) = cam.to_screen(640.0, 50.0);
            draw_text(hint, hx - hd.width * 0.5, hy, hs, GREEN);
        }
        Some(Powerup::ForceFlip) => {
            let opp = game.board.turn.opponent();
            for r in 0..BOARD_SIZE {
                for c in 0..BOARD_SIZE {
                    if game.board.cells[r][c] == Cell::Piece(opp) {
                        let (sx, sy) = cam.to_screen(BOARD_X + c as f32 * CELL_PX, BOARD_Y + r as f32 * CELL_PX);
                        let s = cam.scaled(CELL_PX);
                        draw_rectangle(sx, sy, s, s, Color::new(1.0, 0.6, 0.2, pulse * 0.4));
                        draw_rectangle_lines(sx, sy, s, s, 3.0, Color::new(1.0, 0.7, 0.3, 0.9));
                    }
                }
            }
            let hint = "TAP an opponent piece to flip  |  ESC to cancel";
            let hs = 18.0 * cam.scale;
            let hd = measure_text(hint, None, hs as u16, 1.0);
            let (hx, hy) = cam.to_screen(640.0, 50.0);
            draw_text(hint, hx - hd.width * 0.5, hy, hs, ORANGE);
        }
        _ => {}
    }
}

fn draw_powerup_hud(board: &Board, cam: &Camera) {
    draw_powerup_icon(30.0, 700.0, board.edie_powerup, "EDIE", true, cam);
    draw_powerup_icon(1050.0, 700.0, board.alice_powerup, "ALICE", false, cam);
}

fn draw_powerup_icon(lx: f32, ly: f32, pw: Option<Powerup>, _label: &str, is_edie: bool, cam: &Camera) {
    if let Some(powerup) = pw {
        let (sx, sy) = cam.to_screen(lx, ly);
        let bw = cam.scaled(200.0);
        let bh = cam.scaled(22.0);
        let bg = if is_edie {
            Color::new(ORANGE.r, ORANGE.g, ORANGE.b, 0.3)
        } else {
            Color::new(0.9, 0.3, 0.35, 0.3)
        };
        draw_rectangle(sx, sy, bw, bh, bg);
        draw_rectangle_lines(sx, sy, bw, bh, 1.5, Color::new(1.0, 1.0, 1.0, 0.5));
        let name = match powerup {
            Powerup::DoubleStrike => "2x STRIKE",
            Powerup::VirusCure => "VIRUS CURE [Q]",
            Powerup::ForceFlip => "FORCE FLIP [Q]",
        };
        let ts = 13.0 * cam.scale;
        let td = measure_text(name, None, ts as u16, 1.0);
        draw_text(name, sx + bw * 0.5 - td.width * 0.5, sy + bh * 0.5 + td.height * 0.3, ts, WHITE);
    }
}

fn draw_toast(msg: &str, remaining: f32, cam: &Camera) {
    let alpha = remaining.min(1.0);
    let size = 28.0 * cam.scale;
    let dim = measure_text(msg, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(640.0, 40.0);
    let px = cam.scaled(16.0);
    let py = cam.scaled(8.0);
    draw_rectangle(
        tx - dim.width * 0.5 - px, ty - dim.height - py,
        dim.width + px * 2.0, dim.height + py * 2.0,
        Color::new(0.0, 0.0, 0.0, 0.7 * alpha),
    );
    draw_text(msg, tx - dim.width * 0.5, ty, size, Color::new(1.0, 0.95, 0.6, alpha));
}

pub fn screen_to_cell(screen_x: f32, screen_y: f32) -> Option<(usize, usize)> {
    let cam = Camera::new(screen_width(), screen_height());
    let lx = (screen_x - cam.offset_x) / cam.scale;
    let ly = (screen_y - cam.offset_y) / cam.scale;
    if lx < BOARD_X || lx >= BOARD_X + BOARD_PX || ly < BOARD_Y || ly >= BOARD_Y + BOARD_PX {
        return None;
    }
    let col = ((lx - BOARD_X) / CELL_PX) as usize;
    let row = ((ly - BOARD_Y) / CELL_PX) as usize;
    if row < BOARD_SIZE && col < BOARD_SIZE { Some((row, col)) } else { None }
}
