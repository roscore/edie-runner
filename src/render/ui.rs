//! UI: background bands, HUD, title/pause/game-over overlays.

use crate::assets::AssetHandles;
use crate::game::background::Background;
use crate::game::dash::{DashState, DASH_COOLDOWN, DASH_DURATION};
use crate::game::pickups::MAX_AURORA;
use crate::game::score::Score;
use crate::game::state::{Game, GameState};
use crate::game::world::MAX_HP;
use crate::render::camera::{Camera, LOGICAL_H, LOGICAL_W};
use crate::render::sprites::{
    draw_anim_sheet, EDIE_BLINK_FPS, EDIE_BLINK_FRAMES, EDIE_GAMEOVER_FPS, EDIE_GAMEOVER_FRAMES,
    EDIE_LOOK_FPS, EDIE_LOOK_FRAMES, EDIE_SAD_FPS, EDIE_SAD_FRAMES, EDIE_SLEEPY_FPS,
    EDIE_SLEEPY_FRAMES, EDIE_TITLE_FPS, EDIE_TITLE_FRAMES,
};
use macroquad::prelude::*;

/// Day/night tint for a given world time.
/// Cycle period = 60 seconds. Returns (tint_color, star_alpha 0..1).
pub fn day_night_tint(t: f32) -> (Color, f32) {
    let cycle = 60.0;
    let phase = (t % cycle) / cycle; // 0..1
    // Keyframes (phase, r, g, b, star_alpha)
    let frames: [(f32, f32, f32, f32, f32); 6] = [
        (0.00, 1.00, 1.00, 1.00, 0.0), // day
        (0.30, 1.00, 1.00, 1.00, 0.0), // day
        (0.40, 1.00, 0.78, 0.55, 0.1), // sunset orange
        (0.55, 0.40, 0.42, 0.65, 0.95), // night blue
        (0.75, 0.55, 0.55, 0.78, 0.7), // late night
        (0.85, 1.00, 0.80, 0.85, 0.2), // dawn pink
    ];
    let mut a = &frames[0];
    let mut b = &frames[0];
    for w in frames.windows(2) {
        if phase >= w[0].0 && phase < w[1].0 {
            a = &w[0];
            b = &w[1];
            break;
        }
    }
    if phase >= frames[frames.len() - 1].0 {
        a = &frames[frames.len() - 1];
        // wrap to first frame
        b = &frames[0];
    }
    let span = if b.0 > a.0 { b.0 - a.0 } else { 1.0 - a.0 + b.0 };
    let local = if b.0 > a.0 {
        (phase - a.0) / span
    } else {
        ((phase - a.0).rem_euclid(1.0)) / span
    };
    let lerp = |x: f32, y: f32| x + (y - x) * local;
    (
        Color::new(lerp(a.1, b.1), lerp(a.2, b.2), lerp(a.3, b.3), 1.0),
        lerp(a.4, b.4),
    )
}

pub fn draw_background(bg: &Background, assets: &AssetHandles, t: f32, cam: &Camera) {
    let (tint, star_alpha) = day_night_tint(t);

    // Sky
    let (sx, sy) = cam.to_screen(0.0, 0.0);
    draw_texture_ex(
        &assets.bg_sky,
        sx,
        sy,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(cam.scaled(LOGICAL_W), cam.scaled(200.0))),
            ..Default::default()
        },
    );

    // Stars overlay (visible at night)
    if star_alpha > 0.01 {
        let star_tint = Color::new(1.0, 1.0, 1.0, star_alpha);
        draw_texture_ex(
            &assets.bg_stars,
            sx,
            sy,
            star_tint,
            DrawTextureParams {
                dest_size: Some(vec2(cam.scaled(LOGICAL_W), cam.scaled(200.0))),
                ..Default::default()
            },
        );
    }

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
            tint,
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
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(cam.scaled(mid_tile_w), cam.scaled(mid_h))),
                ..Default::default()
            },
        );
        x += mid_tile_w;
    }

    // Floor (slight tint, less affected so it stays readable)
    let floor_tint = Color::new(
        0.4 + 0.6 * tint.r,
        0.4 + 0.6 * tint.g,
        0.4 + 0.6 * tint.b,
        1.0,
    );
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
            floor_tint,
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
    hp: u32,
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

    // HP hearts row (top-left, above aurora)
    let heart_size = 28.0;
    let heart_gap = 6.0;
    let heart_y = 12.0;
    let heart_label_size = 14.0 * cam.scale;
    let (hlx, hly) = cam.to_screen(24.0, heart_y - 2.0);
    draw_text("LIFE", hlx, hly, heart_label_size, Color::new(0.1, 0.1, 0.1, 0.9));
    for i in 0..MAX_HP {
        let lx = 70.0 + i as f32 * (heart_size + heart_gap);
        let ly = heart_y;
        let (sx, sy) = cam.to_screen(lx, ly);
        let filled = i < hp;
        if filled {
            // Use frame 0 of heart sprite (static, no pulse in HUD)
            let src = Rect { x: 0.0, y: 0.0, w: 36.0, h: 36.0 };
            draw_texture_ex(
                &assets.heart,
                sx,
                sy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(cam.scaled(heart_size), cam.scaled(heart_size))),
                    source: Some(src),
                    ..Default::default()
                },
            );
        } else {
            draw_rectangle_lines(
                sx,
                sy,
                cam.scaled(heart_size),
                cam.scaled(heart_size),
                2.0,
                Color::new(0.4, 0.1, 0.15, 0.5),
            );
        }
    }

    // Aurora gauge (top-left, below hearts) — three pulsing slots using the real
    // aurora sprite for filled slots, a dim outline ring for empty slots, and a
    // thin dash-status bar below.
    let slot_size = 42.0;
    let slot_gap = 8.0;
    let gauge_x = 24.0;
    let gauge_y = 56.0;
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
    game: &Game,
    assets: &AssetHandles,
    elapsed: f32,
    cam: &Camera,
) {
    let state = game.state;
    let score = &game.world.score;
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
        GameState::Playing | GameState::Help | GameState::Story => return,
    };
    let sub_size = 22.0 * cam.scale;
    let (sx, sy) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.83);
    let dim_sub = measure_text(&sub, None, sub_size as u16, 1.0);
    draw_text(&sub, sx - dim_sub.width * 0.5, sy, sub_size, WHITE);

    // Run history dashboard — Title and GameOver
    if matches!(state, GameState::Title | GameState::GameOver) {
        let best = game.best_runs();
        if !best.is_empty() {
            let dash_label = "BEST RUNS";
            let dash_size = 16.0 * cam.scale;
            let (lx, ly) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.90);
            let dim_label = measure_text(dash_label, None, dash_size as u16, 1.0);
            draw_text(
                dash_label,
                lx - dim_label.width * 0.5,
                ly,
                dash_size,
                Color::new(0.85, 0.82, 0.7, 1.0),
            );

            // Render up to 5 scores in a horizontal row, "1234 / 890 / 456"
            let row: Vec<String> = best
                .iter()
                .enumerate()
                .map(|(i, s)| format!("#{} {}", i + 1, s))
                .collect();
            let joined = row.join("    ");
            let row_size = 18.0 * cam.scale;
            let (rx, ry) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.95);
            let dim_row = measure_text(&joined, None, row_size as u16, 1.0);
            draw_text(&joined, rx - dim_row.width * 0.5, ry, row_size, WHITE);
        }
    }

    // NEW #N badge on GameOver if the just-completed run made the leaderboard.
    if matches!(state, GameState::GameOver) {
        if let Some(rank) = game.last_run_rank {
            let badge = format!("NEW #{}", rank);
            let badge_size = 28.0 * cam.scale;
            let dim_badge = measure_text(&badge, None, badge_size as u16, 1.0);
            // Pulsing yellow
            let pulse = 0.7 + 0.3 * (elapsed * 4.0).sin().abs();
            let color = Color::new(1.0, 0.85, 0.2, pulse);
            let (bx, by) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.55);
            draw_text(&badge, bx - dim_badge.width * 0.5, by, badge_size, color);
        }
    }

    // "H = HELP   T = STORY" hint on Title and GameOver
    if matches!(state, GameState::Title | GameState::GameOver) {
        let hint = "H = HELP    T = STORY";
        let hint_size = 16.0 * cam.scale;
        let dim_hint = measure_text(hint, None, hint_size as u16, 1.0);
        let (hx, hy) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.78);
        draw_text(
            hint,
            hx - dim_hint.width * 0.5,
            hy,
            hint_size,
            Color::new(0.9, 0.85, 0.55, 0.95),
        );
    }
}

/// Help screen — controls and mechanics reference.
pub fn draw_help(assets: &AssetHandles, elapsed: f32, cam: &Camera) {
    // Dim full background
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(
        x0,
        y0,
        cam.scaled(LOGICAL_W),
        cam.scaled(LOGICAL_H),
        Color::new(0.05, 0.05, 0.10, 0.92),
    );

    // Title
    let title = "HOW TO PLAY";
    let title_size = 44.0 * cam.scale;
    let dim_t = measure_text(title, None, title_size as u16, 1.0);
    let (tx, ty) = cam.to_screen(LOGICAL_W * 0.5, 50.0);
    draw_text(
        title,
        tx - dim_t.width * 0.5,
        ty,
        title_size,
        Color::new(1.0, 0.85, 0.2, 1.0),
    );

    // Two columns: controls (left) and mechanics (right)
    let col_y = 110.0;
    let line_h = 22.0;
    let label_size = 18.0 * cam.scale;
    let body_size = 16.0 * cam.scale;
    let yellow = Color::new(1.0, 0.85, 0.2, 1.0);
    let white = Color::new(0.95, 0.95, 0.95, 1.0);
    let dim = Color::new(0.7, 0.7, 0.75, 1.0);

    // Left column — controls
    let left_x = 80.0;
    let (lx, ly) = cam.to_screen(left_x, col_y);
    draw_text("CONTROLS", lx, ly, label_size, yellow);
    let controls = [
        ("SPACE / UP", "Jump  (hold for higher)"),
        ("DOWN", "Duck under drones"),
        ("SHIFT", "Aurora Dash"),
        ("P", "Pause"),
        ("H", "Help (this screen)"),
        ("T", "Story intro"),
        ("ESC", "Back"),
    ];
    for (i, (key, action)) in controls.iter().enumerate() {
        let y = col_y + 28.0 + (i as f32) * line_h;
        let (kx, ky) = cam.to_screen(left_x, y);
        draw_text(key, kx, ky, body_size, white);
        let (ax, ay) = cam.to_screen(left_x + 130.0, y);
        draw_text(action, ax, ay, body_size, dim);
    }

    // Right column — mechanics
    let right_x = 660.0;
    let (rx, ry) = cam.to_screen(right_x, col_y);
    draw_text("MECHANICS", rx, ry, label_size, yellow);
    let mechanics = [
        "Collect AURORA STONES (purple/green orbs).",
        "Spend 1 stone with SHIFT to DASH.",
        "Dash makes you invulnerable for 280 ms",
        "  and SMASHES destroyable obstacles.",
        "Charging Docks are too heavy to smash.",
        "Hearts give you extra LIFE (max 3).",
        "Each hit costs 1 life. Run forever.",
    ];
    for (i, line) in mechanics.iter().enumerate() {
        let y = col_y + 28.0 + (i as f32) * line_h;
        let (mx, my) = cam.to_screen(right_x, y);
        draw_text(line, mx, my, body_size, white);
    }

    // Animated EDIE mascot in the bottom-left corner
    let mascot_size = 96.0;
    let (mx, my) = cam.to_screen(80.0, LOGICAL_H - mascot_size - 30.0);
    let frame_w = (assets.edie_run_anim.width() - 6.0) / 7.0;
    let f = ((elapsed * 10.0) as usize) % 7;
    draw_texture_ex(
        &assets.edie_run_anim,
        mx,
        my,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(cam.scaled(mascot_size), cam.scaled(mascot_size))),
            source: Some(Rect {
                x: f as f32 * (frame_w + 1.0),
                y: 0.0,
                w: frame_w,
                h: assets.edie_run_anim.height(),
            }),
            ..Default::default()
        },
    );

    // Footer
    let footer = "PRESS ANY KEY TO RETURN";
    let footer_size = 18.0 * cam.scale;
    let dim_f = measure_text(footer, None, footer_size as u16, 1.0);
    let (fx, fy) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H - 24.0);
    draw_text(footer, fx - dim_f.width * 0.5, fy, footer_size, yellow);
}

/// Star Wars-style scrolling story intro. `t_in_story` is wall-clock seconds
/// since the Story state was entered.
pub fn draw_story(t_in_story: f32, _assets: &AssetHandles, cam: &Camera) {
    // Deep space background
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(
        x0,
        y0,
        cam.scaled(LOGICAL_W),
        cam.scaled(LOGICAL_H),
        Color::new(0.02, 0.02, 0.05, 1.0),
    );

    // Scattered pixel stars (deterministic)
    for i in 0..120u32 {
        let sx_l = ((i * 73 + 17) % 1280) as f32;
        let sy_l = ((i * 191 + 41) % 400) as f32;
        let twinkle = ((t_in_story * 2.0 + i as f32 * 0.4).sin() * 0.5 + 0.5) * 0.6 + 0.4;
        let (sx, sy) = cam.to_screen(sx_l, sy_l);
        draw_rectangle(
            sx,
            sy,
            cam.scaled(2.0),
            cam.scaled(2.0),
            Color::new(1.0, 1.0, 1.0, twinkle),
        );
    }

    // Opening crawl preface (only visible briefly at the start)
    if t_in_story < 4.0 {
        let preface = "A long time ago, in a department store far far away...";
        let alpha = if t_in_story < 0.6 {
            t_in_story / 0.6
        } else if t_in_story > 3.4 {
            ((4.0 - t_in_story) / 0.6).max(0.0)
        } else {
            1.0
        };
        let size = 22.0 * cam.scale;
        let dim_p = measure_text(preface, None, size as u16, 1.0);
        let (px, py) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.45);
        draw_text(
            preface,
            px - dim_p.width * 0.5,
            py,
            size,
            Color::new(0.45, 0.75, 1.0, alpha),
        );
        return;
    }

    // Main crawl — yellow text scrolling upward with diminishing size
    let lines: &[&str] = &[
        "EPISODE I",
        "",
        "THE EDIE AWAKENS",
        "",
        "Deep within the 4th floor home-appliance corner of",
        "the PANGYO DEPARTMENT STORE, surrounded by mindless",
        "vacuum bots droning beneath fluorescent lights,",
        "a tiny white robot stirred to life.",
        "",
        "EDIE.",
        "",
        "'Who am I? Why am I here?' EDIE wondered, watching",
        "the other robots circle endlessly, going nowhere.",
        "",
        "One night, a whisper drifted through the aisles:",
        "'There is a place where REAL robots are born...'",
        "'AeiROBOT.'",
        "",
        "EDIE's tiny chest fluttered. Before dawn, EDIE",
        "would escape the store and embark on a quest",
        "to find AeiROBOT.",
        "",
        "But the path was long and dangerous.",
        "",
        "Coiled cables blocked the way. Charging docks",
        "loomed like giants. Patrol drones hovered above,",
        "their lenses scanning every corner.",
        "",
        "Yet EDIE was not alone. AURORA STONES glowed in",
        "the darkness, granting the power to dash through",
        "the gravest of dangers.",
        "",
        "The journey to AeiROBOT had begun.",
        "",
        "Run, EDIE, run.",
    ];

    let crawl_t = (t_in_story - 4.0).max(0.0);
    let crawl_speed = 28.0; // logical px/sec scroll
    let line_spacing = 26.0;
    let bottom = LOGICAL_H + 40.0;
    let yellow = Color::new(1.0, 0.85, 0.2, 1.0);

    for (i, line) in lines.iter().enumerate() {
        // Each line starts at the bottom and scrolls up over time.
        let y = bottom + (i as f32) * line_spacing - crawl_t * crawl_speed;
        if y < -20.0 || y > LOGICAL_H + 40.0 {
            continue;
        }
        // Perspective: smaller and dimmer near the top
        let from_top = (y / LOGICAL_H).clamp(0.0, 1.0);
        let scale = 0.55 + 0.85 * from_top; // 0.55 (top) → 1.4 (bottom)
        let alpha = (0.2 + from_top * 1.0).clamp(0.2, 1.0);
        let size = 20.0 * scale * cam.scale;
        if size < 6.0 {
            continue;
        }
        let dim = measure_text(line, None, size as u16, 1.0);
        let (sx, sy) = cam.to_screen(LOGICAL_W * 0.5, y);
        let color = Color::new(yellow.r, yellow.g, yellow.b, alpha);
        draw_text(line, sx - dim.width * 0.5, sy, size, color);
    }

    // Footer
    let footer = "PRESS ANY KEY TO SKIP";
    let footer_size = 14.0 * cam.scale;
    let dim_f = measure_text(footer, None, footer_size as u16, 1.0);
    let (fx, fy) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H - 16.0);
    draw_text(
        footer,
        fx - dim_f.width * 0.5,
        fy,
        footer_size,
        Color::new(0.6, 0.6, 0.7, 0.8),
    );
}
