//! UI: background bands, HUD, title/pause/game-over overlays.

use crate::assets::AssetHandles;
use crate::game::background::Background;
use crate::game::dash::{DashState, DASH_COOLDOWN, DASH_DURATION};
use crate::game::pickups::MAX_AURORA;
use crate::game::score::Score;
use crate::game::state::GameState;
use crate::render::camera::{Camera, LOGICAL_H, LOGICAL_W};
use crate::render::sprites::{
    draw_anim_sheet, EDIE_BLINK_FPS, EDIE_BLINK_FRAMES, EDIE_GAMEOVER_FPS, EDIE_GAMEOVER_FRAMES,
    EDIE_LOOK_FPS, EDIE_LOOK_FRAMES, EDIE_SAD_FPS, EDIE_SAD_FRAMES, EDIE_SLEEPY_FPS,
    EDIE_SLEEPY_FRAMES, EDIE_TITLE_FPS, EDIE_TITLE_FRAMES,
};
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

pub fn draw_hud(
    score: &Score,
    dash: &DashState,
    assets: &AssetHandles,
    elapsed: f32,
    cam: &Camera,
) {
    // Score (right)
    let score_text = format!("{:06}", score.current);
    let high_text = format!("HI {:06}", score.high);
    let font_size = 28.0 * cam.scale;
    let (sx, sy) = cam.to_screen(LOGICAL_W - 200.0, 30.0);
    draw_text(&score_text, sx, sy, font_size, BLACK);
    let (hx, hy) = cam.to_screen(LOGICAL_W - 200.0, 60.0);
    draw_text(&high_text, hx, hy, 20.0 * cam.scale, DARKGRAY);

    // Aurora gauge (top-left) — three pulsing slots using the real aurora
    // sprite for filled slots, a dim outline ring for empty slots, and a
    // thin dash-status bar below.
    let slot_size = 42.0;
    let slot_gap = 8.0;
    let gauge_x = 24.0;
    let gauge_y = 20.0;
    let label = "AURORA";
    let label_size = 16.0 * cam.scale;
    let (lx, ly) = cam.to_screen(gauge_x, gauge_y - 4.0);
    draw_text(label, lx, ly, label_size, Color::new(0.1, 0.1, 0.1, 0.9));

    let aurora_frame_w = 48.0;
    let aurora_frame_h = 48.0;
    let frame_idx = ((elapsed * 8.0) as usize) % 6;
    let src = Rect {
        x: frame_idx as f32 * (aurora_frame_w + 1.0),
        y: 0.0,
        w: aurora_frame_w,
        h: aurora_frame_h,
    };

    for i in 0..MAX_AURORA {
        let lx = gauge_x + i as f32 * (slot_size + slot_gap);
        let ly = gauge_y + 10.0;
        let (sx, sy) = cam.to_screen(lx, ly);
        let filled = i < dash.aurora;

        // Slot frame — rounded square background
        draw_rectangle(
            sx - cam.scaled(2.0),
            sy - cam.scaled(2.0),
            cam.scaled(slot_size + 4.0),
            cam.scaled(slot_size + 4.0),
            Color::new(0.1, 0.1, 0.1, 0.25),
        );
        draw_rectangle_lines(
            sx - cam.scaled(2.0),
            sy - cam.scaled(2.0),
            cam.scaled(slot_size + 4.0),
            cam.scaled(slot_size + 4.0),
            2.0,
            Color::new(0.1, 0.1, 0.1, 0.6),
        );

        if filled {
            draw_texture_ex(
                &assets.aurora_purple,
                sx,
                sy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(cam.scaled(slot_size), cam.scaled(slot_size))),
                    source: Some(src),
                    ..Default::default()
                },
            );
        } else {
            // Empty slot — faded core
            draw_rectangle(
                sx + cam.scaled(slot_size * 0.3),
                sy + cam.scaled(slot_size * 0.3),
                cam.scaled(slot_size * 0.4),
                cam.scaled(slot_size * 0.4),
                Color::new(0.62, 0.42, 1.00, 0.15),
            );
        }
    }

    // Dash status bar below the slots
    let bar_y = gauge_y + 10.0 + slot_size + 8.0;
    let bar_w = MAX_AURORA as f32 * slot_size + (MAX_AURORA - 1) as f32 * slot_gap;
    let bar_h = 6.0;
    let (bsx, bsy) = cam.to_screen(gauge_x, bar_y);
    // background
    draw_rectangle(
        bsx,
        bsy,
        cam.scaled(bar_w),
        cam.scaled(bar_h),
        Color::new(0.1, 0.1, 0.1, 0.35),
    );

    let (fill_ratio, bar_color) = if dash.is_active() {
        (
            dash.active_remaining / DASH_DURATION,
            Color::new(0.18, 0.77, 0.71, 1.0), // ok teal during dash
        )
    } else if dash.cooldown_remaining > 0.0 {
        (
            1.0 - dash.cooldown_remaining / DASH_COOLDOWN,
            Color::new(0.9, 0.5, 0.2, 0.9), // orange during cooldown
        )
    } else if dash.aurora > 0 {
        (1.0, Color::new(0.62, 0.42, 1.00, 1.0)) // ready purple
    } else {
        (0.0, Color::new(0.3, 0.3, 0.3, 0.6)) // empty grey
    };
    if fill_ratio > 0.0 {
        draw_rectangle(
            bsx,
            bsy,
            cam.scaled(bar_w * fill_ratio.clamp(0.0, 1.0)),
            cam.scaled(bar_h),
            bar_color,
        );
    }
    draw_rectangle_lines(
        bsx,
        bsy,
        cam.scaled(bar_w),
        cam.scaled(bar_h),
        1.0,
        Color::new(0.1, 0.1, 0.1, 0.8),
    );
}

pub fn draw_overlay(
    state: GameState,
    score: &Score,
    assets: &AssetHandles,
    elapsed: f32,
    cam: &Camera,
) {
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

    // Mascot animation for Title / Paused / GameOver — shown above the text.
    let mascot_size = 160.0;
    let mascot_x = LOGICAL_W * 0.5 - mascot_size * 0.5;
    let mascot_y = LOGICAL_H * 0.15;
    match state {
        GameState::Title => {
            // Rotate through three idle variants every ~4 seconds.
            let variant = ((elapsed / 4.0) as usize) % 3;
            let (tex, frames, fps) = match variant {
                0 => (
                    &assets.edie_title_idle,
                    EDIE_TITLE_FRAMES,
                    EDIE_TITLE_FPS,
                ),
                1 => (&assets.edie_look, EDIE_LOOK_FRAMES, EDIE_LOOK_FPS),
                _ => (
                    &assets.edie_blink_alt,
                    EDIE_BLINK_FRAMES,
                    EDIE_BLINK_FPS,
                ),
            };
            draw_anim_sheet(
                tex,
                frames,
                fps,
                elapsed,
                mascot_x,
                mascot_y,
                mascot_size,
                mascot_size,
                cam,
                WHITE,
            );
        }
        GameState::Paused => {
            draw_anim_sheet(
                &assets.edie_sleepy,
                EDIE_SLEEPY_FRAMES,
                EDIE_SLEEPY_FPS,
                elapsed,
                mascot_x,
                mascot_y,
                mascot_size,
                mascot_size,
                cam,
                WHITE,
            );
        }
        GameState::GameOver => {
            // Alternate between teardrop and sad closed-eye every 3 seconds.
            let alt = ((elapsed / 3.0) as usize) % 2;
            let (tex, frames, fps) = if alt == 0 {
                (
                    &assets.edie_gameover_anim,
                    EDIE_GAMEOVER_FRAMES,
                    EDIE_GAMEOVER_FPS,
                )
            } else {
                (&assets.edie_sad_alt, EDIE_SAD_FRAMES, EDIE_SAD_FPS)
            };
            draw_anim_sheet(
                tex,
                frames,
                fps,
                elapsed,
                mascot_x,
                mascot_y,
                mascot_size,
                mascot_size,
                cam,
                WHITE,
            );
        }
        _ => {}
    }

    let (cx, cy) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.70);
    let title = match state {
        GameState::Title => "EDIE RUNNER",
        GameState::Paused => "PAUSED",
        GameState::GameOver => "GAME OVER",
        _ => "",
    };
    let size = 56.0 * cam.scale;
    let dim_text = measure_text(title, None, size as u16, 1.0);
    draw_text(title, cx - dim_text.width * 0.5, cy, size, WHITE);

    let sub = match state {
        GameState::Title => "PRESS SPACE TO START".to_string(),
        GameState::Paused => "PRESS P OR SPACE TO RESUME".to_string(),
        GameState::GameOver => format!("SCORE {} | HI {} | SPACE TO RETRY", score.current, score.high),
        GameState::Playing => return,
    };
    let sub_size = 22.0 * cam.scale;
    let (sx, sy) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.88);
    let dim_sub = measure_text(&sub, None, sub_size as u16, 1.0);
    draw_text(&sub, sx - dim_sub.width * 0.5, sy, sub_size, WHITE);
}
