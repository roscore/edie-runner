//! UI: background bands, HUD, title/pause/game-over overlays.

use crate::game::background::Background;
use crate::game::dash::DashState;
use crate::game::pickups::MAX_AURORA;
use crate::game::score::Score;
use crate::game::state::GameState;
use crate::render::camera::{Camera, LOGICAL_H, LOGICAL_W};
use macroquad::prelude::*;

pub fn draw_background(bg: &Background, cam: &Camera) {
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(
        x0,
        y0,
        cam.scaled(LOGICAL_W),
        cam.scaled(LOGICAL_H),
        Color::new(0.96, 0.94, 0.89, 1.0),
    );

    let band_h = 100.0;
    let (fx, fy) = cam.to_screen(-bg.far_offset, 200.0);
    draw_rectangle(
        fx,
        fy,
        cam.scaled(LOGICAL_W * 2.0),
        cam.scaled(band_h),
        Color::new(0.79, 0.76, 0.70, 1.0),
    );

    let (mx, my) = cam.to_screen(-bg.mid_offset, 280.0);
    draw_rectangle(
        mx,
        my,
        cam.scaled(LOGICAL_W * 2.0),
        cam.scaled(40.0),
        Color::new(0.56, 0.53, 0.46, 1.0),
    );

    let (flx, fly) = cam.to_screen(0.0, 320.0);
    draw_rectangle(
        flx,
        fly,
        cam.scaled(LOGICAL_W),
        cam.scaled(80.0),
        Color::new(0.29, 0.27, 0.22, 1.0),
    );
    let (lx, ly) = cam.to_screen(0.0, 320.0);
    draw_line(
        lx,
        ly,
        lx + cam.scaled(LOGICAL_W),
        ly,
        2.0 * cam.scale,
        Color::new(0.18, 0.16, 0.13, 1.0),
    );
}

pub fn draw_hud(score: &Score, dash: &DashState, cam: &Camera) {
    let score_text = format!("{:06}", score.current);
    let high_text = format!("HI {:06}", score.high);
    let font_size = 28.0 * cam.scale;
    let (sx, sy) = cam.to_screen(LOGICAL_W - 200.0, 30.0);
    draw_text(&score_text, sx, sy, font_size, BLACK);
    let (hx, hy) = cam.to_screen(LOGICAL_W - 200.0, 60.0);
    draw_text(&high_text, hx, hy, 20.0 * cam.scale, DARKGRAY);

    let icon_w = 28.0;
    let gap = 8.0;
    for i in 0..MAX_AURORA {
        let x = 20.0 + i as f32 * (icon_w + gap);
        let (sx, sy) = cam.to_screen(x, 20.0);
        let filled = i < dash.aurora;
        let color = if filled {
            Color::new(0.62, 0.42, 1.00, 1.0)
        } else {
            Color::new(0.62, 0.42, 1.00, 0.25)
        };
        draw_rectangle(sx, sy, cam.scaled(icon_w), cam.scaled(icon_w), color);
    }
}

pub fn draw_overlay(state: GameState, score: &Score, cam: &Camera) {
    let dim = match state {
        GameState::Title | GameState::Paused | GameState::GameOver => 0.45,
        _ => return,
    };
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(
        x0,
        y0,
        cam.scaled(LOGICAL_W),
        cam.scaled(LOGICAL_H),
        Color::new(0.0, 0.0, 0.0, dim),
    );

    let (cx, cy) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.4);
    let title = match state {
        GameState::Title => "EDIE RUNNER",
        GameState::Paused => "PAUSED",
        GameState::GameOver => "GAME OVER",
        _ => "",
    };
    let size = 64.0 * cam.scale;
    let dim_text = measure_text(title, None, size as u16, 1.0);
    draw_text(title, cx - dim_text.width * 0.5, cy, size, WHITE);

    let sub = match state {
        GameState::Title => "PRESS SPACE TO START".to_string(),
        GameState::Paused => "PRESS P OR SPACE TO RESUME".to_string(),
        GameState::GameOver => format!("SCORE {} | HI {} | SPACE TO RETRY", score.current, score.high),
        GameState::Playing => return,
    };
    let sub_size = 24.0 * cam.scale;
    let (sx, sy) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.6);
    let dim_sub = measure_text(&sub, None, sub_size as u16, 1.0);
    draw_text(&sub, sx - dim_sub.width * 0.5, sy, sub_size, WHITE);
}
