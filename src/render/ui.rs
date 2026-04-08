//! UI: background bands, HUD, title/pause/game-over overlays.

use crate::assets::AssetHandles;
use crate::game::background::Background;
use crate::game::dash::DashState;
use crate::game::pickups::MAX_AURORA;
use crate::game::score::Score;
use crate::game::state::GameState;
use crate::render::camera::{Camera, LOGICAL_H, LOGICAL_W};
use macroquad::prelude::*;

pub fn draw_background(bg: &Background, assets: &AssetHandles, cam: &Camera) {
    // Sky
    let (sx, sy) = cam.to_screen(0.0, 0.0);
    draw_texture_ex(
        &assets.bg_sky,
        sx,
        sy,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(cam.scaled(LOGICAL_W), cam.scaled(200.0))),
            ..Default::default()
        },
    );

    // Far servers (parallax)
    let far_tile_w = 256.0;
    let far_y = 200.0;
    let far_h = 100.0;
    let mut x = -(bg.far_offset % far_tile_w);
    while x < LOGICAL_W {
        let (px, py) = cam.to_screen(x, far_y);
        draw_texture_ex(
            &assets.bg_far,
            px,
            py,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(cam.scaled(far_tile_w), cam.scaled(far_h))),
                ..Default::default()
            },
        );
        x += far_tile_w;
    }

    // Mid workbenches
    let mid_tile_w = 256.0;
    let mid_y = 270.0;
    let mid_h = 60.0;
    let mut x = -(bg.mid_offset % mid_tile_w);
    while x < LOGICAL_W {
        let (px, py) = cam.to_screen(x, mid_y);
        draw_texture_ex(
            &assets.bg_mid,
            px,
            py,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(cam.scaled(mid_tile_w), cam.scaled(mid_h))),
                ..Default::default()
            },
        );
        x += mid_tile_w;
    }

    // Floor
    let floor_tile_w = 256.0;
    let floor_y = 320.0;
    let floor_h = 80.0;
    let mut x = -(bg.floor_offset % floor_tile_w);
    while x < LOGICAL_W {
        let (px, py) = cam.to_screen(x, floor_y);
        draw_texture_ex(
            &assets.bg_floor,
            px,
            py,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(cam.scaled(floor_tile_w), cam.scaled(floor_h))),
                ..Default::default()
            },
        );
        x += floor_tile_w;
    }
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
