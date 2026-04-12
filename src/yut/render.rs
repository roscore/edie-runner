//! Board + HUD rendering for EDIE Yut Nori (macroquad).

use crate::assets::AssetHandles;
use crate::render::camera::Camera;
use crate::yut::board::*;
use crate::yut::game::*;
use macroquad::prelude::*;

const YUT_W: f32 = 1280.0;
const YUT_H: f32 = 720.0;

const ORANGE: Color = Color::new(0.91, 0.57, 0.23, 1.0);
const GREEN: Color = Color::new(0.36, 0.89, 0.66, 1.0);
const PURPLE: Color = Color::new(0.61, 0.44, 0.83, 1.0);
const CYAN: Color = Color::new(0.31, 0.76, 0.97, 1.0);

fn player_color(idx: usize) -> Color {
    match idx {
        0 => ORANGE,
        1 => GREEN,
        2 => PURPLE,
        3 => CYAN,
        _ => WHITE,
    }
}

/// Logical position of each board cell for rendering.
fn cell_pos(pos: usize) -> (f32, f32) {
    let cx = 640.0;
    let cy = 360.0;
    let sp = 52.0; // spacing between cells
    match pos {
        // Bottom edge (right to left): 0=start, 1..4
        0 => (cx + sp * 2.5, cy + sp * 2.5),
        1 => (cx + sp * 1.5, cy + sp * 2.5),
        2 => (cx + sp * 0.5, cy + sp * 2.5),
        3 => (cx - sp * 0.5, cy + sp * 2.5),
        4 => (cx - sp * 1.5, cy + sp * 2.5),
        // Right edge (bottom to top): 5=NE corner, 6..9
        5 => (cx - sp * 2.5, cy + sp * 2.5),
        6 => (cx - sp * 2.5, cy + sp * 1.5),
        7 => (cx - sp * 2.5, cy + sp * 0.5),
        8 => (cx - sp * 2.5, cy - sp * 0.5),
        9 => (cx - sp * 2.5, cy - sp * 1.5),
        // Top edge (left to right): 10=NW corner, 11..14
        10 => (cx - sp * 2.5, cy - sp * 2.5),
        11 => (cx - sp * 1.5, cy - sp * 2.5),
        12 => (cx - sp * 0.5, cy - sp * 2.5),
        13 => (cx + sp * 0.5, cy - sp * 2.5),
        14 => (cx + sp * 1.5, cy - sp * 2.5),
        // Left edge (top to bottom): 15=SW corner, 16..19
        15 => (cx + sp * 2.5, cy - sp * 2.5),
        16 => (cx + sp * 2.5, cy - sp * 1.5),
        17 => (cx + sp * 2.5, cy - sp * 0.5),
        18 => (cx + sp * 2.5, cy + sp * 0.5),
        19 => (cx + sp * 2.5, cy + sp * 1.5),
        // Diagonal A: 5→20→21→24→27→28
        20 => (cx - sp * 1.7, cy + sp * 1.7),
        21 => (cx - sp * 0.85, cy + sp * 0.85),
        // Diagonal B: 10→22→23→24→25→26
        22 => (cx - sp * 1.7, cy - sp * 1.7),
        23 => (cx - sp * 0.85, cy - sp * 0.85),
        // Center
        24 => (cx, cy),
        // From center toward exit
        25 => (cx + sp * 0.85, cy + sp * 0.85),
        26 => (cx + sp * 1.7, cy + sp * 1.7),
        27 => (cx + sp * 0.85, cy - sp * 0.85),
        28 => (cx + sp * 1.7, cy - sp * 1.7),
        _ => (0.0, 0.0),
    }
}

pub fn draw_yut(game: &YutGame, _assets: &AssetHandles, elapsed: f32) {
    let cam = Camera::with_logical(YUT_W, YUT_H, screen_width(), screen_height());
    clear_background(Color::new(0.06, 0.06, 0.10, 1.0));

    match game.phase {
        Phase::Menu => draw_menu(&cam),
        Phase::GameOver => {
            draw_board(&cam);
            draw_all_pieces(game, elapsed, &cam);
            draw_game_over(game, &cam);
        }
        _ => {
            draw_board(&cam);
            draw_all_pieces(game, elapsed, &cam);
            draw_hud(game, &cam);
            draw_throw_result(game, &cam);
            if game.phase == Phase::SelectPiece {
                draw_piece_selection_hint(game, elapsed, &cam);
            }
            if game.phase == Phase::SelectPath {
                draw_path_choice(game, &cam);
            }
            if let Some((ref msg, t)) = game.toast {
                draw_toast(msg, t, &cam);
            }
        }
    }
}

fn draw_board(cam: &Camera) {
    // Draw connections
    let edges: &[(usize, usize)] = &[
        (0,1),(1,2),(2,3),(3,4),(4,5),
        (5,6),(6,7),(7,8),(8,9),(9,10),
        (10,11),(11,12),(12,13),(13,14),(14,15),
        (15,16),(16,17),(17,18),(18,19),(19,0),
        // Diagonal A
        (5,20),(20,21),(21,24),(24,27),(27,28),
        // Diagonal B
        (10,22),(22,23),(23,24),(24,25),(25,26),
    ];
    for &(a, b) in edges {
        let (ax, ay) = cell_pos(a);
        let (bx, by) = cell_pos(b);
        let (sax, say) = cam.to_screen(ax, ay);
        let (sbx, sby) = cam.to_screen(bx, by);
        draw_line(sax, say, sbx, sby, 2.0, Color::new(0.3, 0.3, 0.35, 0.6));
    }
    // Draw cells
    for pos in 0..NUM_POSITIONS {
        let (lx, ly) = cell_pos(pos);
        let (sx, sy) = cam.to_screen(lx, ly);
        let r = cam.scaled(if is_shortcut_corner(pos) { 16.0 } else if pos == CENTER { 18.0 } else { 12.0 });
        let col = if is_shortcut_corner(pos) || pos == CENTER {
            Color::new(0.36, 0.89, 0.66, 0.5)
        } else if pos == 0 {
            Color::new(0.91, 0.57, 0.23, 0.5)
        } else {
            Color::new(0.25, 0.25, 0.30, 0.8)
        };
        draw_circle(sx, sy, r, col);
        draw_circle_lines(sx, sy, r, 2.0, Color::new(0.5, 0.5, 0.55, 0.7));
    }
    // Labels
    let labels = [("START", 0), ("NE", 5), ("NW", 10), ("SW", 15)];
    for (label, pos) in labels {
        let (lx, ly) = cell_pos(pos);
        let size = 11.0 * cam.scale;
        let dim = measure_text(label, None, size as u16, 1.0);
        let (sx, sy) = cam.to_screen(lx, ly - 22.0);
        draw_text(label, sx - dim.width * 0.5, sy, size, Color::new(0.7, 0.7, 0.7, 0.7));
    }
}

fn draw_all_pieces(game: &YutGame, elapsed: f32, cam: &Camera) {
    for (pi, player) in game.players.iter().enumerate() {
        let color = player_color(pi);
        for (qi, piece) in player.pieces.iter().enumerate() {
            if piece.is_exited() || piece.stack == 0 { continue; }
            if piece.is_home() {
                // Draw in home area
                let home_x = 80.0 + pi as f32 * 60.0;
                let home_y = 600.0 + qi as f32 * 24.0;
                let (sx, sy) = cam.to_screen(home_x, home_y);
                let r = cam.scaled(8.0);
                draw_circle(sx, sy, r, color);
            } else {
                // Draw on board
                let (lx, ly) = cell_pos(piece.pos);
                let offset = qi as f32 * 6.0 - 9.0;
                let bob = (elapsed * 2.5 + pi as f32 + qi as f32).sin() * 2.0;
                let (sx, sy) = cam.to_screen(lx + offset, ly + bob);
                let r = cam.scaled(if piece.stack > 1 { 12.0 } else { 9.0 });
                draw_circle(sx, sy, r, color);
                draw_circle_lines(sx, sy, r, 2.0, WHITE);
                if piece.stack > 1 {
                    let txt = format!("{}", piece.stack);
                    let ts = 12.0 * cam.scale;
                    let td = measure_text(&txt, None, ts as u16, 1.0);
                    draw_text(&txt, sx - td.width * 0.5, sy + td.height * 0.3, ts, WHITE);
                }
                if piece.shield > 0 {
                    draw_circle_lines(sx, sy, r + cam.scaled(4.0), 2.0,
                        Color::new(0.3, 0.9, 1.0, 0.8));
                }
            }
        }
    }
}

fn draw_hud(game: &YutGame, cam: &Camera) {
    let turn_txt = format!("Turn {} — {}'s turn", game.turn_count + 1, game.current_player_name());
    let size = 22.0 * cam.scale;
    let dim = measure_text(&turn_txt, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(640.0, 30.0);
    draw_text(&turn_txt, tx - dim.width * 0.5, ty, size, player_color(game.current_player));

    // Player piece status
    for (pi, player) in game.players.iter().enumerate() {
        let home = player.pieces.iter().filter(|p| p.is_home()).count();
        let board = player.pieces.iter().filter(|p| p.is_on_board()).count();
        let done = player.pieces.iter().filter(|p| p.is_exited()).count();
        let txt = format!("{}: H{} B{} D{}", match pi { 0=>"EDIE",1=>"ALICE",2=>"AMY",_=>"BOX" }, home, board, done);
        let s = 14.0 * cam.scale;
        let (sx, sy) = cam.to_screen(30.0, 60.0 + pi as f32 * 22.0);
        draw_text(&txt, sx, sy, s, player_color(pi));
    }

    // Phase hint
    let hint = match game.phase {
        Phase::Throwing => "TAP to throw yut!",
        Phase::SelectPiece => "Select a piece to move",
        Phase::SelectPath => "Choose: [1] Shortcut  [2] Outer path",
        _ => "",
    };
    if !hint.is_empty() {
        let hs = 18.0 * cam.scale;
        let hd = measure_text(hint, None, hs as u16, 1.0);
        let (hx, hy) = cam.to_screen(640.0, 700.0);
        draw_text(hint, hx - hd.width * 0.5, hy, hs, Color::new(0.8, 0.8, 0.8, 0.9));
    }
}

fn draw_throw_result(game: &YutGame, cam: &Camera) {
    if let Some(result) = game.last_throw {
        let txt = format!("{} {}", result.name_ko(), result.name_en());
        let size = 36.0 * cam.scale;
        let dim = measure_text(&txt, None, size as u16, 1.0);
        let (tx, ty) = cam.to_screen(640.0, 680.0);
        draw_text(&txt, tx - dim.width * 0.5, ty, size, ORANGE);
        if let Some(sticks) = game.last_sticks {
            // Draw yut sticks
            let stick_w = 20.0;
            let total = stick_w * 4.0 + 12.0 * 3.0;
            let start_x = 640.0 - total * 0.5;
            for (i, &flat) in sticks.iter().enumerate() {
                let lx = start_x + i as f32 * (stick_w + 12.0);
                let ly = 640.0;
                let (sx, sy) = cam.to_screen(lx, ly);
                let w = cam.scaled(stick_w);
                let h = cam.scaled(8.0);
                let col = if flat { ORANGE } else { Color::new(0.3, 0.3, 0.35, 1.0) };
                draw_rectangle(sx, sy, w, h, col);
                draw_rectangle_lines(sx, sy, w, h, 1.5, Color::new(0.8, 0.7, 0.5, 0.8));
            }
        }
    }
}

fn draw_piece_selection_hint(game: &YutGame, elapsed: f32, cam: &Camera) {
    let player = &game.players[game.current_player];
    let pulse = 0.5 + 0.5 * (elapsed * 5.0).sin();
    for (i, piece) in player.pieces.iter().enumerate() {
        if piece.is_exited() || piece.stack == 0 { continue; }
        if piece.is_home() {
            let home_x = 80.0 + game.current_player as f32 * 60.0;
            let home_y = 600.0 + i as f32 * 24.0;
            let (sx, sy) = cam.to_screen(home_x, home_y);
            draw_circle_lines(sx, sy, cam.scaled(14.0), 2.5,
                Color::new(1.0, 1.0, 0.3, pulse));
        } else {
            let (lx, ly) = cell_pos(piece.pos);
            let (sx, sy) = cam.to_screen(lx, ly);
            draw_circle_lines(sx, sy, cam.scaled(16.0), 2.5,
                Color::new(1.0, 1.0, 0.3, pulse));
        }
    }
}

fn draw_path_choice(game: &YutGame, cam: &Camera) {
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(x0, y0, cam.scaled(YUT_W), cam.scaled(YUT_H),
        Color::new(0.0, 0.0, 0.0, 0.3));
    let opts = ["[1] SHORTCUT (diagonal)", "[2] OUTER PATH (around)"];
    for (i, opt) in opts.iter().enumerate() {
        let size = 28.0 * cam.scale;
        let dim = measure_text(opt, None, size as u16, 1.0);
        let (tx, ty) = cam.to_screen(640.0, 320.0 + i as f32 * 60.0);
        draw_text(opt, tx - dim.width * 0.5, ty, size, GREEN);
    }
}

fn draw_menu(cam: &Camera) {
    let title = "EDIE YUT NORI";
    let sub = "초능력 윷놀이";
    let size = 52.0 * cam.scale;
    let dim = measure_text(title, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(640.0, 200.0);
    draw_text(title, tx - dim.width * 0.5 + 3.0, ty + 3.0, size, Color::new(0.0, 0.0, 0.0, 0.5));
    draw_text(title, tx - dim.width * 0.5, ty, size, ORANGE);
    let ss = 24.0 * cam.scale;
    let sd = measure_text(sub, None, ss as u16, 1.0);
    let (sxp, syp) = cam.to_screen(640.0, 250.0);
    draw_text(sub, sxp - sd.width * 0.5, syp, ss, GREEN);

    let opts = [("1. 2P GAME", 350.0), ("2. 3P GAME", 400.0), ("3. 4P GAME", 450.0)];
    let os = 24.0 * cam.scale;
    for (l, y) in &opts {
        let d = measure_text(l, None, os as u16, 1.0);
        let (ox, oy) = cam.to_screen(640.0, *y);
        draw_text(l, ox - d.width * 0.5, oy, os, GREEN);
    }
    let hint = "CLICK OR PRESS 1-3";
    let hs = 16.0 * cam.scale;
    let hd = measure_text(hint, None, hs as u16, 1.0);
    let (hx, hy) = cam.to_screen(640.0, 550.0);
    draw_text(hint, hx - hd.width * 0.5, hy, hs, Color::new(0.6, 0.6, 0.6, 1.0));
}

fn draw_game_over(game: &YutGame, cam: &Camera) {
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(x0, y0, cam.scaled(YUT_W), cam.scaled(YUT_H),
        Color::new(0.0, 0.0, 0.0, 0.6));
    let winner_name = game.winner.map(|w| game.players.get(w).map(|_|
        match w { 0=>"EDIE",1=>"ALICE",2=>"AMY",_=>"BOXBOT" }
    ).unwrap_or("???")).unwrap_or("???");
    let title = format!("{} WINS!", winner_name);
    let color = game.winner.map(|w| player_color(w)).unwrap_or(WHITE);
    let size = 56.0 * cam.scale;
    let dim = measure_text(&title, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(640.0, 330.0);
    draw_text(&title, tx - dim.width * 0.5 + 4.0, ty + 4.0, size, Color::new(0.0, 0.0, 0.0, 0.7));
    draw_text(&title, tx - dim.width * 0.5, ty, size, color);
    let turns = format!("in {} turns", game.turn_count);
    let ts = 22.0 * cam.scale;
    let td = measure_text(&turns, None, ts as u16, 1.0);
    let (ttx, tty) = cam.to_screen(640.0, 400.0);
    draw_text(&turns, ttx - td.width * 0.5, tty, ts, WHITE);
    let sub = "TAP or SPACE to play again";
    let ss = 18.0 * cam.scale;
    let sd = measure_text(sub, None, ss as u16, 1.0);
    let (sx, sy) = cam.to_screen(640.0, 460.0);
    draw_text(sub, sx - sd.width * 0.5, sy, ss, Color::new(0.7, 0.7, 0.7, 1.0));
}

fn draw_toast(msg: &str, remaining: f32, cam: &Camera) {
    let alpha = remaining.min(1.0);
    let size = 26.0 * cam.scale;
    let dim = measure_text(msg, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(640.0, 60.0);
    let px = cam.scaled(14.0);
    let py = cam.scaled(6.0);
    draw_rectangle(tx - dim.width * 0.5 - px, ty - dim.height - py,
        dim.width + px * 2.0, dim.height + py * 2.0,
        Color::new(0.0, 0.0, 0.0, 0.7 * alpha));
    draw_text(msg, tx - dim.width * 0.5, ty, size, Color::new(1.0, 0.95, 0.6, alpha));
}

/// Convert screen coords to board cell index (for touch/click).
pub fn screen_to_board_cell(screen_x: f32, screen_y: f32) -> Option<usize> {
    let cam = Camera::with_logical(YUT_W, YUT_H, screen_width(), screen_height());
    let lx = (screen_x - cam.offset_x) / cam.scale;
    let ly = (screen_y - cam.offset_y) / cam.scale;
    let threshold = 24.0;
    for pos in 0..NUM_POSITIONS {
        let (cx, cy) = cell_pos(pos);
        let dx = lx - cx;
        let dy = ly - cy;
        if dx * dx + dy * dy < threshold * threshold {
            return Some(pos);
        }
    }
    None
}

/// Check if screen coords hit a player's home piece.
pub fn screen_to_home_piece(screen_x: f32, screen_y: f32, game: &YutGame) -> Option<usize> {
    let cam = Camera::with_logical(YUT_W, YUT_H, screen_width(), screen_height());
    let lx = (screen_x - cam.offset_x) / cam.scale;
    let ly = (screen_y - cam.offset_y) / cam.scale;
    let pi = game.current_player;
    for (qi, piece) in game.players[pi].pieces.iter().enumerate() {
        if !piece.is_home() || piece.stack == 0 { continue; }
        let home_x = 80.0 + pi as f32 * 60.0;
        let home_y = 600.0 + qi as f32 * 24.0;
        let dx = lx - home_x;
        let dy = ly - home_y;
        if dx * dx + dy * dy < 20.0 * 20.0 {
            return Some(qi);
        }
    }
    None
}
