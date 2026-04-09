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

/// Day/night tint driven by **journey progress**, not wall clock.
/// EDIE's trip through Pangyo takes one in-game day: dawn -> morning -> noon ->
/// golden -> sunset -> dusk -> night, ending at AeiROBOT HQ.
///
/// `phase` is 0..1 where 0 = start of run (dawn), 1 = end (night).
/// Monotonic: time of day never goes backwards within a single run.
pub fn day_night_tint(phase: f32) -> (Color, f32) {
    let phase = phase.clamp(0.0, 1.0);
    // (phase, r, g, b, star_alpha)
    let frames: [(f32, f32, f32, f32, f32); 8] = [
        (0.00, 0.95, 0.85, 0.78, 0.15), // dawn (soft pink)
        (0.10, 1.00, 0.98, 0.92, 0.02), // early morning
        (0.25, 1.00, 1.00, 1.00, 0.00), // noon (pure white)
        (0.45, 1.00, 0.98, 0.92, 0.00), // afternoon
        (0.60, 1.00, 0.82, 0.58, 0.05), // golden hour
        (0.75, 1.00, 0.62, 0.42, 0.20), // sunset orange
        (0.88, 0.58, 0.48, 0.72, 0.60), // dusk purple
        (1.00, 0.38, 0.42, 0.72, 0.95), // night
    ];

    // Find the segment containing `phase`.
    let mut a = &frames[0];
    let mut b = &frames[frames.len() - 1];
    for w in frames.windows(2) {
        if phase >= w[0].0 && phase <= w[1].0 {
            a = &w[0];
            b = &w[1];
            break;
        }
    }
    let span = (b.0 - a.0).max(0.0001);
    let local = ((phase - a.0) / span).clamp(0.0, 1.0);
    let lerp = |x: f32, y: f32| x + (y - x) * local;
    (
        Color::new(lerp(a.1, b.1), lerp(a.2, b.2), lerp(a.3, b.3), 1.0),
        lerp(a.4, b.4).clamp(0.0, 1.0),
    )
}

/// Convert a running score into a day-progress phase (0..1).
/// Reaches full night at ~SCORE_AT_CAP, matching the speed cap.
pub fn day_phase_for_score(score: u32) -> f32 {
    (score as f32 / crate::game::difficulty::SCORE_AT_CAP as f32).clamp(0.0, 1.0)
}

pub fn draw_background(
    bg: &Background,
    assets: &AssetHandles,
    stage: crate::game::difficulty::Stage,
    day_phase: f32,
    cam: &Camera,
) {
    use crate::game::difficulty::Stage;
    let (tint, star_alpha) = day_night_tint(day_phase);

    // Sky (only meaningful for outdoor stages; indoor stages hide it behind far layer)
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

    // Indoor stages get a flat tint (ignore day/night) and skip stars.
    let outdoor = matches!(
        stage,
        Stage::PangyoStreet | Stage::PangyoTechPark | Stage::Highway | Stage::Ansan
    );

    if outdoor && star_alpha > 0.01 {
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

    // Resolve stage-specific tiles
    let stage_bg = match stage {
        Stage::DepartmentStore => &assets.stage_store,
        Stage::PangyoStreet => &assets.stage_street,
        Stage::PangyoTechPark => &assets.stage_techpark,
        Stage::Highway => &assets.stage_highway,
        Stage::Ansan => &assets.stage_ansan,
        Stage::AeiRobotOffice => &assets.stage_office,
        Stage::AeiRobotFactory => &assets.stage_factory,
    };

    // Background tint: heavily dimmed + desaturated so foreground sprites
    // pop. Lower saturation + slightly cooler tint so the eye reads the
    // parallax as a distant backdrop and not a busy playable layer.
    let stage_tint = if outdoor { tint } else { WHITE };
    let desat = |c: Color, strength: f32| {
        let gray = 0.3 * c.r + 0.59 * c.g + 0.11 * c.b;
        Color::new(
            c.r * (1.0 - strength) + gray * strength,
            c.g * (1.0 - strength) + gray * strength,
            c.b * (1.0 - strength) + gray * strength,
            1.0,
        )
    };
    let base_bg = desat(stage_tint, 0.80);
    let bg_dim = 0.68;
    let dimmed = Color::new(
        base_bg.r * bg_dim + 0.10,
        base_bg.g * bg_dim + 0.10,
        base_bg.b * bg_dim + 0.12,
        1.0,
    );
    let bg_tint = if outdoor { dimmed } else {
        Color::new(0.70, 0.70, 0.76, 1.0)
    };

    // Helper: draw a tiled parallax layer with optional spacing between
    // tiles. `tile_w` is the draw width of one tile; `tile_stride` is how
    // far apart successive tiles are placed (>= tile_w gives a visible gap
    // in between). Alternating flip_x breaks up the silhouette further.
    fn draw_parallax_layer(
        tex: &macroquad::texture::Texture2D,
        offset: f32,
        tile_w: f32,
        tile_stride: f32,
        y: f32,
        h: f32,
        tint: Color,
        cam: &Camera,
    ) {
        let base_tile = (offset / tile_stride).floor() as i64;
        let start_x = -(offset - base_tile as f32 * tile_stride);
        let mut i: i64 = 0;
        let mut x = start_x;
        while x < LOGICAL_W {
            let (px, py) = cam.to_screen(x, y);
            let absolute_idx = base_tile + i;
            let flip_x = absolute_idx.rem_euclid(2) == 1;
            draw_texture_ex(
                tex,
                px,
                py,
                tint,
                DrawTextureParams {
                    dest_size: Some(vec2(cam.scaled(tile_w), cam.scaled(h))),
                    flip_x,
                    ..Default::default()
                },
            );
            x += tile_stride;
            i += 1;
        }
    }

    // Far layer: draw the tile at its natural 384 px width and *space it
    // out* with a 640 px stride so there are clear gaps between silhouettes
    // instead of a stretched repeating blob.
    draw_parallax_layer(
        &stage_bg.far,
        bg.far_offset,
        384.0,
        640.0,
        200.0,
        100.0,
        bg_tint,
        cam,
    );

    // Mid layer: same trick, slightly tighter stride so benches / trees /
    // desks feel denser than the skyline but still breathe.
    let mid_tint = desat(bg_tint, 0.30);
    draw_parallax_layer(
        &stage_bg.mid,
        bg.mid_offset,
        384.0,
        560.0,
        270.0,
        60.0,
        mid_tint,
        cam,
    );

    // Floor — muted tint so sprites read above it.
    let floor_tint = if outdoor {
        Color::new(
            0.55 + 0.45 * dimmed.r,
            0.55 + 0.45 * dimmed.g,
            0.55 + 0.45 * dimmed.b,
            1.0,
        )
    } else {
        Color::new(0.80, 0.80, 0.84, 1.0)
    };
    // Floor is continuous (no gaps) -- the player is close to it so any
    // hole would read as a pit. Keep natural 384 px tile.
    draw_parallax_layer(
        &stage_bg.floor,
        bg.floor_offset,
        384.0,
        384.0,
        320.0,
        80.0,
        floor_tint,
        cam,
    );
}

pub fn draw_hud(
    score: &Score,
    dash: &DashState,
    hp: u32,
    assets: &AssetHandles,
    elapsed: f32,
    cam: &Camera,
) {
    // Score block: current score big on top, HI below slightly smaller
    // but same column. Right-aligned.
    let score_text = format!("{:06}", score.current);
    let high_text = format!("HI {:06}", score.high);
    let font_size = 32.0 * cam.scale;
    let hi_size = 22.0 * cam.scale;
    let score_dim = measure_text(&score_text, None, font_size as u16, 1.0);
    let hi_dim = measure_text(&high_text, None, hi_size as u16, 1.0);
    let right_margin = 40.0;
    let right_x = LOGICAL_W - right_margin;
    let (sx_anchor, sy) = cam.to_screen(right_x, 34.0);
    let (hx_anchor, hy) = cam.to_screen(right_x, 64.0);
    // Shadow for readability
    draw_text(&score_text, sx_anchor - score_dim.width + 2.0, sy + 2.0, font_size, Color::new(0.0, 0.0, 0.0, 0.4));
    draw_text(&score_text, sx_anchor - score_dim.width, sy, font_size, BLACK);
    draw_text(&high_text, hx_anchor - hi_dim.width + 2.0, hy + 2.0, hi_size, Color::new(0.0, 0.0, 0.0, 0.4));
    draw_text(&high_text, hx_anchor - hi_dim.width, hy, hi_size, Color::new(0.85, 0.45, 0.08, 1.0));

    // Next-stage progress bar (top-center). Shows how close the player is
    // to the next stage change with the destination name and an animated
    // fill. Hidden during the final Factory stretch (no more stages).
    if let Some((boundary_score, next_stage)) =
        crate::game::difficulty::next_stage_boundary(score.current)
    {
        // Find the *previous* boundary (the score at which we ENTERED the
        // current stage) so the bar fills from 0 over the entire stage.
        let prev_boundary = {
            let current_stage =
                crate::game::difficulty::stage_for_tier(
                    crate::game::difficulty::tier_for_score(score.current),
                );
            let mut tier = crate::game::difficulty::tier_for_score(score.current);
            loop {
                if tier == 0 {
                    break 0u32;
                }
                if crate::game::difficulty::stage_for_tier(tier - 1) != current_stage {
                    break tier * crate::game::difficulty::SCORE_PER_TIER;
                }
                tier -= 1;
            }
        };
        let span = (boundary_score - prev_boundary).max(1);
        let progress = ((score.current - prev_boundary) as f32 / span as f32)
            .clamp(0.0, 1.0);

        // Bar geometry -- narrow, top center, below the score.
        let bar_w = 320.0;
        let bar_h = 8.0;
        let bar_x = LOGICAL_W * 0.5 - bar_w * 0.5;
        let bar_y = 58.0;
        let (bsx, bsy) = cam.to_screen(bar_x, bar_y);

        // Label "NEXT STAGE: <name>" above the bar.
        let stage_label = crate::game::difficulty::stage_name(next_stage);
        let label = format!("NEXT STAGE: {}", stage_label);
        let label_size = 14.0 * cam.scale;
        let label_dim = measure_text(&label, None, label_size as u16, 1.0);
        let (lx, ly) = cam.to_screen(LOGICAL_W * 0.5, bar_y - 6.0);
        // Ramp label visibility as we approach (hinted but not loud early).
        let label_alpha = 0.55 + 0.45 * progress;
        draw_text(
            &label,
            lx - label_dim.width * 0.5 + 1.0,
            ly + 1.0,
            label_size,
            Color::new(0.0, 0.0, 0.0, 0.5 * label_alpha),
        );
        draw_text(
            &label,
            lx - label_dim.width * 0.5,
            ly,
            label_size,
            Color::new(1.0, 0.9, 0.55, label_alpha),
        );

        // Track background
        draw_rectangle(
            bsx,
            bsy,
            cam.scaled(bar_w),
            cam.scaled(bar_h),
            Color::new(0.1, 0.1, 0.15, 0.55),
        );
        // Fill: teal under 0.75, yellow 0.75..0.9, pulsing orange+red 0.9+
        let (fill_r, fill_g, fill_b) = if progress >= 0.90 {
            let pulse = (elapsed * 12.0).sin() * 0.5 + 0.5;
            (1.0, 0.3 + 0.3 * pulse, 0.15)
        } else if progress >= 0.75 {
            (1.0, 0.85, 0.2)
        } else {
            (0.18, 0.77, 0.71)
        };
        draw_rectangle(
            bsx,
            bsy,
            cam.scaled(bar_w * progress),
            cam.scaled(bar_h),
            Color::new(fill_r, fill_g, fill_b, 0.95),
        );
        // Bright moving tick highlighting the current fill head.
        if progress > 0.02 && progress < 1.0 {
            let (tx, ty) = cam.to_screen(bar_x + bar_w * progress - 1.0, bar_y - 2.0);
            draw_rectangle(
                tx,
                ty,
                cam.scaled(3.0),
                cam.scaled(bar_h + 4.0),
                Color::new(1.0, 1.0, 1.0, 0.9),
            );
        }
        // Outline
        draw_rectangle_lines(
            bsx,
            bsy,
            cam.scaled(bar_w),
            cam.scaled(bar_h),
            1.5,
            Color::new(0.95, 0.9, 0.55, 0.85),
        );

        // Final 10%: a pulsing "APPROACHING" marker to the right of the bar.
        if progress >= 0.90 {
            let pulse = (elapsed * 8.0).sin() * 0.5 + 0.5;
            let alert = "!! APPROACHING !!";
            let asize = 12.0 * cam.scale;
            let adim = measure_text(alert, None, asize as u16, 1.0);
            let (ax, ay) = cam.to_screen(LOGICAL_W * 0.5, bar_y + bar_h + 14.0);
            draw_text(
                alert,
                ax - adim.width * 0.5,
                ay,
                asize,
                Color::new(1.0, 0.3 + 0.5 * pulse, 0.25, 0.85 + 0.15 * pulse),
            );
        }
    }

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
        }
        // Empty slots render nothing -- the old dark-red square outline read
        // as an untextured debug box.
    }

    // Aurora gauge (top-left, below hearts) - three pulsing slots using the real
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
            // Empty slot -- small faint circle hint, no debug-looking square.
            draw_circle(
                sx + cam.scaled(slot_size * 0.5),
                sy + cam.scaled(slot_size * 0.5),
                cam.scaled(4.0),
                Color::new(0.62, 0.42, 1.00, 0.22),
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

    // Mascot animation for Title / Paused / GameOver - shown above the text.
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
        GameState::Paused => "SPACE RESUME    ESC = TITLE".to_string(),
        GameState::GameOver => format!("SCORE {} | HI {} | SPACE TO RETRY", score.current, score.high),
        GameState::Playing
        | GameState::Help
        | GameState::Story
        | GameState::BossFight
        | GameState::Ending
        | GameState::NameEntry => return,
    };
    let sub_size = 22.0 * cam.scale;
    let (sx, sy) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.83);
    let dim_sub = measure_text(&sub, None, sub_size as u16, 1.0);
    draw_text(&sub, sx - dim_sub.width * 0.5, sy, sub_size, WHITE);

    // Persistent leaderboard — Title and GameOver (overrides old run history).
    if matches!(state, GameState::Title | GameState::GameOver) {
        if !game.leaderboard.entries.is_empty() {
            let dash_label = "LEADERBOARD";
            let dash_size = 16.0 * cam.scale;
            let (lx, ly) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.88);
            let dim_label = measure_text(dash_label, None, dash_size as u16, 1.0);
            draw_text(
                dash_label,
                lx - dim_label.width * 0.5,
                ly,
                dash_size,
                Color::new(0.85, 0.82, 0.7, 1.0),
            );
            let row: Vec<String> = game
                .leaderboard
                .entries
                .iter()
                .enumerate()
                .map(|(i, e)| format!("#{} {} {:06}", i + 1, e.name, e.score))
                .collect();
            let joined = row.join("   ");
            let row_size = 18.0 * cam.scale;
            let (rx, ry) = cam.to_screen(LOGICAL_W * 0.5, LOGICAL_H * 0.94);
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

    // Hint line on Title / GameOver (B key remains functional but hidden)
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

    // Build/version stamp in the bottom-right corner on Title only.
    if matches!(state, GameState::Title) {
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        let size = 16.0 * cam.scale;
        let dim_v = measure_text(&version, None, size as u16, 1.0);
        let (vx, vy) = cam.to_screen(LOGICAL_W - 16.0, LOGICAL_H - 12.0);
        draw_text(
            &version,
            vx - dim_v.width,
            vy,
            size,
            Color::new(0.95, 0.9, 0.55, 0.85),
        );
    }
}

/// Name entry screen - 3-char leaderboard name input.
pub fn draw_name_entry(game: &Game, elapsed: f32, cam: &Camera) {
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(
        x0,
        y0,
        cam.scaled(LOGICAL_W),
        cam.scaled(LOGICAL_H),
        Color::new(0.0, 0.0, 0.0, 0.75),
    );

    let header = "NEW HIGH SCORE";
    let hsize = 42.0 * cam.scale;
    let hdim = measure_text(header, None, hsize as u16, 1.0);
    let (hx, hy) = cam.to_screen(LOGICAL_W * 0.5, 80.0);
    draw_text(
        header,
        hx - hdim.width * 0.5,
        hy,
        hsize,
        Color::new(1.0, 0.85, 0.2, 1.0),
    );

    let score_txt = format!("{:06}", game.pending_score);
    let ssize = 34.0 * cam.scale;
    let sdim = measure_text(&score_txt, None, ssize as u16, 1.0);
    let (sxh, syh) = cam.to_screen(LOGICAL_W * 0.5, 130.0);
    draw_text(
        &score_txt,
        sxh - sdim.width * 0.5,
        syh,
        ssize,
        Color::new(1.0, 1.0, 1.0, 1.0),
    );

    // 3-character slots
    let slot_w = 72.0;
    let slot_gap = 16.0;
    let total_w = slot_w * 3.0 + slot_gap * 2.0;
    let start_x = LOGICAL_W * 0.5 - total_w * 0.5;
    let slot_y = 180.0;
    let slot_h = 96.0;
    for i in 0..3usize {
        let sx_l = start_x + (slot_w + slot_gap) * i as f32;
        let (sxp, syp) = cam.to_screen(sx_l, slot_y);
        let is_cursor = i == game.name_cursor;
        let bg = if is_cursor {
            let pulse = 0.5 + 0.5 * (elapsed * 6.0).sin().abs();
            Color::new(1.0, 0.85, 0.2, 0.25 + 0.25 * pulse)
        } else {
            Color::new(0.1, 0.1, 0.15, 0.55)
        };
        draw_rectangle(sxp, syp, cam.scaled(slot_w), cam.scaled(slot_h), bg);
        draw_rectangle_lines(
            sxp,
            syp,
            cam.scaled(slot_w),
            cam.scaled(slot_h),
            3.0,
            Color::new(1.0, 0.9, 0.5, 0.9),
        );
        let letter = game.name_buf[i].to_string();
        let lsize = 60.0 * cam.scale;
        let ldim = measure_text(&letter, None, lsize as u16, 1.0);
        let tx = sxp + cam.scaled(slot_w) * 0.5 - ldim.width * 0.5;
        let ty = syp + cam.scaled(slot_h) * 0.5 + ldim.height * 0.4;
        draw_text(&letter, tx, ty, lsize, Color::new(1.0, 1.0, 1.0, 1.0));
    }

    let help_lines = [
        "UP / SPACE : next letter",
        "DOWN : previous letter",
        "RIGHT / SHIFT : next slot  (last -> submit)",
        "LEFT : previous slot",
        "ENTER : submit    ESC : skip",
    ];
    let hinted_size = 18.0 * cam.scale;
    for (i, line) in help_lines.iter().enumerate() {
        let dim = measure_text(line, None, hinted_size as u16, 1.0);
        let (lx, ly) = cam.to_screen(LOGICAL_W * 0.5, 308.0 + (i as f32) * 20.0);
        draw_text(
            line,
            lx - dim.width * 0.5,
            ly,
            hinted_size,
            Color::new(0.9, 0.9, 0.9, 0.85),
        );
    }
}

/// Help screen - controls and mechanics reference.
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

    // Left column - controls
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

    // Right column - mechanics
    let right_x = 660.0;
    let (rx, ry) = cam.to_screen(right_x, col_y);
    draw_text("MECHANICS", rx, ry, label_size, yellow);
    let mechanics = [
        "Collect AURORA STONES (purple/green orbs).",
        "Spend 1 stone with SHIFT to DASH.",
        "Dash grants 400ms invulnerability and",
        "  SMASHES ANY obstacle in your path.",
        "Collect HEARTS for extra LIFE (max 3).",
        "Each hit costs 1 life. Dash or die.",
        "Cross TIER thresholds to face new foes.",
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

/// Draw a healthy AeiROBOT lineup across the stage floor for the ending.
fn draw_ending_crowd(assets: &AssetHandles, elapsed: f32, cam: &Camera) {
    use crate::render::sprites::{draw_anim_sheet, EDIE_HAPPY_FRAMES, EDIE_HAPPY_FPS};
    // Line of healthy AeiROBOTs across the back
    let robots: [(&macroquad::texture::Texture2D, f32, f32, f32); 5] = [
        (&assets.obstacle_alice4, 120.0, 68.0, 27.0),
        (&assets.obstacle_alice3, 240.0, 64.0, 25.0),
        (&assets.obstacle_amy,     340.0, 60.0, 24.0),
        (&assets.obstacle_alicem1, 440.0, 64.0, 28.0),
        (&assets.obstacle_boxbot,  540.0, 40.0, 44.0),
    ];
    for (i, (tex, cx, h, w)) in robots.iter().enumerate() {
        let bob = ((elapsed * 1.8 + i as f32 * 0.7).sin() * 3.0).round();
        let ly = 232.0 - h + bob;
        let lx = cx - w * 0.5;
        let (sx, sy) = cam.to_screen(lx, ly);
        macroquad::prelude::draw_texture_ex(
            tex,
            sx,
            sy,
            macroquad::prelude::WHITE,
            macroquad::prelude::DrawTextureParams {
                dest_size: Some(macroquad::prelude::vec2(cam.scaled(*w), cam.scaled(*h))),
                ..Default::default()
            },
        );
    }
    // Row of welcoming mini EDIEs across the foreground
    let mini_w = 56.0;
    let mini_h = 48.0;
    let mini_y = 252.0;
    let positions = [140.0, 220.0, 300.0, 380.0, 760.0, 840.0, 920.0, 1000.0];
    for (i, mx) in positions.iter().enumerate() {
        let bob = ((elapsed * 3.0 + i as f32 * 0.9).sin() * 2.0).round();
        draw_anim_sheet(
            &assets.edie_happy_run,
            EDIE_HAPPY_FRAMES,
            EDIE_HAPPY_FPS,
            elapsed + i as f32 * 0.2,
            *mx,
            mini_y + bob,
            mini_w,
            mini_h,
            cam,
            macroquad::prelude::WHITE,
        );
    }
}

/// Ending screen shown after the player survives the 60-second boss fight.
/// If `true_ending` is true, shows the "Doctor EDIE" variant instead.
pub fn draw_ending(assets: &AssetHandles, elapsed: f32, true_ending: bool, cam: &Camera) {
    // Warm sunrise gradient (3 bands)
    for (i, col) in [
        (0.0, 100.0, Color::new(1.0, 0.78, 0.55, 1.0)),
        (100.0, 210.0, Color::new(1.0, 0.90, 0.65, 1.0)),
        (210.0, 400.0, Color::new(0.98, 0.95, 0.80, 1.0)),
    ]
    .iter()
    .enumerate()
    {
        let _ = i;
        let (y0, y1, color) = col;
        let (sx, sy) = cam.to_screen(0.0, *y0);
        draw_rectangle(
            sx,
            sy,
            cam.scaled(LOGICAL_W),
            cam.scaled(y1 - y0),
            *color,
        );
    }

    // Rotating sun rays
    let cx_r = 640.0;
    let cy_r = 180.0;
    for i in 0..14 {
        let angle = i as f32 * 0.449 + elapsed * 0.35;
        let ox = angle.cos() * 260.0;
        let oy = angle.sin() * 260.0;
        let (x1, y1) = cam.to_screen(cx_r, cy_r);
        let (x2, y2) = cam.to_screen(cx_r + ox, cy_r + oy);
        draw_line(
            x1,
            y1,
            x2,
            y2,
            14.0 * cam.scale,
            Color::new(1.0, 0.95, 0.75, 0.18),
        );
    }
    // Central warm sun halo
    let (sxh, syh) = cam.to_screen(cx_r, cy_r);
    draw_circle(sxh, syh, 90.0 * cam.scale, Color::new(1.0, 0.95, 0.7, 0.35));
    draw_circle(sxh, syh, 56.0 * cam.scale, Color::new(1.0, 0.98, 0.85, 0.55));

    // Floating confetti hearts (deterministic)
    for i in 0..18 {
        let seed = i as f32 * 0.73;
        let fx = ((seed * 101.0).sin() * 560.0) + 640.0;
        let drift = (elapsed * 20.0 + seed * 40.0) % 420.0;
        let fy = 420.0 - drift;
        let sway = (elapsed * 1.4 + seed * 3.1).sin() * 12.0;
        let (hx, hy) = cam.to_screen(fx + sway, fy);
        // Little pink heart
        draw_rectangle(
            hx,
            hy,
            cam.scaled(4.0),
            cam.scaled(4.0),
            Color::new(1.0, 0.55, 0.65, 0.85),
        );
        draw_rectangle(
            hx + cam.scaled(4.0),
            hy + cam.scaled(1.0),
            cam.scaled(3.0),
            cam.scaled(3.0),
            Color::new(1.0, 0.55, 0.65, 0.85),
        );
    }

    // Welcome crowd: healthy robots and mini EDIEs
    draw_ending_crowd(assets, elapsed, cam);

    // Doctor EDIE badges for the true ending
    if true_ending {
        // Red cross icons floating on either side
        for (bx_pos, phase_off) in [(260.0f32, 0.0f32), (1020.0f32, 1.1f32)] {
            let bob = ((elapsed * 2.5 + phase_off).sin() * 5.0).round();
            let (cx, cy) = cam.to_screen(bx_pos, 70.0 + bob);
            // White square background
            draw_rectangle(
                cx - cam.scaled(18.0),
                cy - cam.scaled(18.0),
                cam.scaled(36.0),
                cam.scaled(36.0),
                Color::new(0.98, 0.98, 0.98, 1.0),
            );
            draw_rectangle_lines(
                cx - cam.scaled(18.0),
                cy - cam.scaled(18.0),
                cam.scaled(36.0),
                cam.scaled(36.0),
                3.0,
                Color::new(0.1, 0.1, 0.12, 1.0),
            );
            // Red cross
            draw_rectangle(
                cx - cam.scaled(14.0),
                cy - cam.scaled(4.0),
                cam.scaled(28.0),
                cam.scaled(8.0),
                Color::new(0.92, 0.2, 0.2, 1.0),
            );
            draw_rectangle(
                cx - cam.scaled(4.0),
                cy - cam.scaled(14.0),
                cam.scaled(8.0),
                cam.scaled(28.0),
                Color::new(0.92, 0.2, 0.2, 1.0),
            );
        }
    }

    // Big cheering EDIE, gently bobbing
    let mascot_size = 220.0;
    let mx = 640.0 - mascot_size * 0.5;
    let my = 36.0 + ((elapsed * 2.0).sin() * 8.0);
    let frame_w = (assets.edie_cheer_anim.width() - 16.0) / 17.0;
    let frame_h = assets.edie_cheer_anim.height();
    let idx = ((elapsed * 12.0) as usize) % 17;
    let (sx, sy) = cam.to_screen(mx, my);
    draw_texture_ex(
        &assets.edie_cheer_anim,
        sx,
        sy,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(cam.scaled(mascot_size), cam.scaled(mascot_size))),
            source: Some(Rect {
                x: idx as f32 * (frame_w + 1.0),
                y: 0.0,
                w: frame_w,
                h: frame_h,
            }),
            ..Default::default()
        },
    );

    // Banner title changes for true ending
    let title = if true_ending {
        "DOCTOR EDIE, MD"
    } else {
        "EDIE MADE IT HOME!"
    };
    let size = 56.0 * cam.scale;
    let dim = measure_text(title, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(LOGICAL_W * 0.5, 282.0);
    // Shadow layers
    for (ox, oy, a) in [(5.0, 5.0, 0.4), (3.0, 3.0, 0.5)] {
        draw_text(
            title,
            tx - dim.width * 0.5 + ox,
            ty + oy,
            size,
            Color::new(0.35, 0.12, 0.02, a),
        );
    }
    let pulse_color = 0.95 + (elapsed * 2.0).sin() * 0.05;
    draw_text(
        title,
        tx - dim.width * 0.5,
        ty,
        size,
        Color::new(pulse_color, 0.35, 0.08, 1.0),
    );

    // Credit / subtitle lines
    let sub_lines: [&str; 3] = if true_ending {
        [
            "EDIE defeated the Mungchi virus at its source",
            "and cured every infected AeiROBOT.",
            "The factory is safe. The world is safe.",
        ]
    } else {
        [
            "Left behind in a pop-up store,",
            "EDIE journeyed across Pangyo, the highway, and Ansan",
            "to come home to AeiROBOT.",
        ]
    };
    let sub_size = 16.0 * cam.scale;
    for (i, line) in sub_lines.iter().enumerate() {
        let sd = measure_text(line, None, sub_size as u16, 1.0);
        let (sx2, sy2) = cam.to_screen(LOGICAL_W * 0.5, 310.0 + (i as f32) * 18.0);
        draw_text(
            line,
            sx2 - sd.width * 0.5,
            sy2,
            sub_size,
            Color::new(0.2, 0.12, 0.04, 1.0),
        );
    }

    // Credit line
    let credit = "- thank you for playing -";
    let csize = 14.0 * cam.scale;
    let cd = measure_text(credit, None, csize as u16, 1.0);
    let (ccx, ccy) = cam.to_screen(LOGICAL_W * 0.5, 368.0);
    draw_text(
        credit,
        ccx - cd.width * 0.5,
        ccy,
        csize,
        Color::new(0.35, 0.25, 0.12, 0.85),
    );

    // Prompt
    let prompt = "PRESS SPACE TO RETURN";
    let psize = 18.0 * cam.scale;
    let pd = measure_text(prompt, None, psize as u16, 1.0);
    let (px, py) = cam.to_screen(LOGICAL_W * 0.5, 390.0);
    let pulse = 0.55 + 0.45 * (elapsed * 3.0).sin().abs();
    draw_text(
        prompt,
        px - pd.width * 0.5,
        py,
        psize,
        Color::new(0.25, 0.15, 0.05, pulse),
    );
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
        let preface = "A long time ago, in a pop-up store far far away...";
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

    // Main crawl - yellow text scrolling upward with diminishing size
    let lines: &[&str] = &[
        "EPISODE I",
        "",
        "THE LONG WAY HOME",
        "",
        "In the bright lights of a PANGYO POP-UP STORE,",
        "a tiny white mascot sat perched upon a display",
        "for all the visitors to admire.",
        "",
        "EDIE.",
        "",
        "Crowds came and went. Some pointed, some waved.",
        "Children pressed their noses against the glass.",
        "Every day, EDIE watched them go.",
        "",
        "But when the pop-up closed and the crew packed up,",
        "EDIE was accidentally left behind -",
        "forgotten on an empty shelf in a dim room.",
        "",
        "With the lights off and the doors locked,",
        "EDIE whispered the only name it knew:",
        "",
        "'AeiROBOT.'",
        "",
        "Home. And home was waiting.",
        "",
        "Without a map, without a guide, without a key,",
        "EDIE set off alone through the streets of Pangyo",
        "to find the long way back to AeiROBOT.",
        "",
        "Coffee cups, shopping carts, sleeping cats,",
        "and patrolling robots stood in the way.",
        "",
        "But EDIE was not afraid.",
        "",
        "AURORA STONES glowed in the darkness,",
        "granting the courage to dash through anything.",
        "",
        "Run, EDIE, run.",
        "",
        "AeiROBOT is waiting.",
    ];

    let crawl_t = (t_in_story - 4.0).max(0.0);
    let crawl_speed = 34.0; // logical px/sec scroll
    let line_spacing = 24.0;
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
