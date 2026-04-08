//! Phase 2 sprite drawing — textured.

use crate::assets::AssetHandles;
use crate::game::dash::DashState;
use crate::game::obstacles::{Obstacle, ObstacleKind};
use crate::game::pickups::{AuroraColor, AuroraStone, HeartPod, HEART_H, HEART_W};
use crate::game::player::{Player, PlayerState, GROUND_Y, PLAYER_H, PLAYER_W, PLAYER_X};
use crate::render::camera::Camera;
use macroquad::prelude::*;

const AURORA_FRAMES: usize = 6;
const AURORA_FRAME_W: f32 = 48.0;
const AURORA_FRAME_H: f32 = 48.0;
const AURORA_FPS: f32 = 8.0;

const DRONE_FRAMES: usize = 4;
const DRONE_FRAME_W: f32 = 56.0;
const DRONE_FRAME_H: f32 = 32.0;
const DRONE_FPS: f32 = 16.0;

const SPARK_FRAMES: usize = 4;
const SPARK_FRAME_W: f32 = 24.0;
const SPARK_FRAME_H: f32 = 24.0;
const SPARK_FPS: f32 = 12.0;

const DOCK_FRAMES: usize = 2;
const DOCK_FRAME_W: f32 = 32.0;
const DOCK_FRAME_H: f32 = 64.0;
const DOCK_FPS: f32 = 2.0;

// GIF-based EDIE animation frame counts (match generate_art.py)
pub const EDIE_RUN_FRAMES: usize = 7;
pub const EDIE_RUN_FPS: f32 = 10.0;
pub const EDIE_TITLE_FRAMES: usize = 7;
pub const EDIE_TITLE_FPS: f32 = 6.0;
pub const EDIE_SAD_FRAMES: usize = 7;
pub const EDIE_SAD_FPS: f32 = 6.0;
pub const EDIE_SLEEPY_FRAMES: usize = 7;
pub const EDIE_SLEEPY_FPS: f32 = 5.0;
pub const EDIE_HIT_FRAMES: usize = 17;
pub const EDIE_HIT_FPS: f32 = 14.0;
pub const EDIE_LOOK_FRAMES: usize = 11;
pub const EDIE_LOOK_FPS: f32 = 7.0;
pub const EDIE_GAMEOVER_FRAMES: usize = 11;
pub const EDIE_GAMEOVER_FPS: f32 = 8.0;
pub const EDIE_BLINK_FRAMES: usize = 7;
pub const EDIE_BLINK_FPS: f32 = 7.0;
pub const EDIE_CHEER_FRAMES: usize = 17;
pub const EDIE_CHEER_FPS: f32 = 14.0;

/// Draw one frame from a horizontally-laid-out sprite sheet that uses the
/// generator's standard 1-px padding between frames.
pub fn draw_anim_sheet(
    tex: &Texture2D,
    frame_count: usize,
    fps: f32,
    elapsed: f32,
    logical_x: f32,
    logical_y: f32,
    dest_w: f32,
    dest_h: f32,
    cam: &Camera,
    tint: Color,
) {
    let sheet_w = tex.width();
    let frame_h = tex.height();
    let frame_w = (sheet_w - (frame_count - 1) as f32) / frame_count as f32;
    let idx = ((elapsed * fps) as usize) % frame_count.max(1);
    let (sx, sy) = cam.to_screen(logical_x, logical_y);
    let src = Rect {
        x: idx as f32 * (frame_w + 1.0),
        y: 0.0,
        w: frame_w,
        h: frame_h,
    };
    draw_texture_ex(
        tex,
        sx,
        sy,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(cam.scaled(dest_w), cam.scaled(dest_h))),
            source: Some(src),
            ..Default::default()
        },
    );
}

fn frame_index(elapsed: f32, fps: f32, count: usize) -> usize {
    ((elapsed * fps) as usize) % count
}

fn draw_tex_at(tex: &Texture2D, lx: f32, ly: f32, w: f32, h: f32, cam: &Camera, tint: Color) {
    let (sx, sy) = cam.to_screen(lx, ly);
    draw_texture_ex(
        tex,
        sx,
        sy,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(cam.scaled(w), cam.scaled(h))),
            ..Default::default()
        },
    );
}

fn draw_tex_frame(
    tex: &Texture2D,
    frame_idx: usize,
    frame_w: f32,
    frame_h: f32,
    pad: f32,
    lx: f32,
    ly: f32,
    cam: &Camera,
    tint: Color,
) {
    let (sx, sy) = cam.to_screen(lx, ly);
    let src = Rect {
        x: frame_idx as f32 * (frame_w + pad),
        y: 0.0,
        w: frame_w,
        h: frame_h,
    };
    draw_texture_ex(
        tex,
        sx,
        sy,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(cam.scaled(frame_w), cam.scaled(frame_h))),
            source: Some(src),
            ..Default::default()
        },
    );
}

/// EDIE visual box (matches new gif-extracted sprite scale).
const EDIE_VIS_W: f32 = 56.0;
const EDIE_VIS_H: f32 = 48.0;

pub fn draw_player(
    player: &Player,
    dash: &DashState,
    assets: &AssetHandles,
    elapsed: f32,
    cam: &Camera,
) {
    // Shadow first (under EDIE)
    let shadow_w = PLAYER_W * 0.85;
    let shadow_h = 6.0;
    let airborne = (GROUND_Y - (player.y + PLAYER_H)).max(0.0);
    let shrink = (1.0 - (airborne / 160.0).min(0.7)).max(0.3);
    let sw = shadow_w * shrink;
    let sx_logical = PLAYER_X + (PLAYER_W - sw) * 0.5;
    draw_tex_at(
        &assets.edie_shadow,
        sx_logical,
        GROUND_Y - 4.0,
        sw,
        shadow_h,
        cam,
        Color::new(1.0, 1.0, 1.0, 0.8),
    );

    let vis_w = EDIE_VIS_W;
    let vis_h = EDIE_VIS_H;
    let logical_x = PLAYER_X + (PLAYER_W - vis_w) * 0.5;
    let mut logical_y = player.y + PLAYER_H - vis_h;

    // Dash takes precedence: cheer animation any time EDIE is invulnerable,
    // as long as we're not dead or ducking.
    if dash.is_active() && !matches!(player.state, PlayerState::Hit | PlayerState::Ducking) {
        draw_anim_sheet(
            &assets.edie_cheer_anim,
            EDIE_CHEER_FRAMES,
            EDIE_CHEER_FPS,
            elapsed,
            logical_x,
            logical_y,
            vis_w,
            vis_h,
            cam,
            WHITE,
        );
        return;
    }

    match player.state {
        PlayerState::Running => {
            // Tiny bob for liveliness
            let bob = ((elapsed * 8.0).sin() * 1.0).round();
            logical_y += bob;
            draw_anim_sheet(
                &assets.edie_run_anim,
                EDIE_RUN_FRAMES,
                EDIE_RUN_FPS,
                elapsed,
                logical_x,
                logical_y,
                vis_w,
                vis_h,
                cam,
                WHITE,
            );
        }
        PlayerState::Hit => {
            draw_anim_sheet(
                &assets.edie_hit_anim,
                EDIE_HIT_FRAMES,
                EDIE_HIT_FPS,
                elapsed,
                logical_x,
                logical_y,
                vis_w,
                vis_h,
                cam,
                WHITE,
            );
        }
        PlayerState::Ducking => {
            // Duck: render shorter sprite, bottom-aligned
            let duck_h = vis_h * 0.55;
            let duck_y = player.y + PLAYER_H - duck_h;
            let tex = &assets.edie_duck;
            draw_tex_at(tex, logical_x, duck_y, vis_w, duck_h, cam, WHITE);
        }
        PlayerState::Jumping | PlayerState::Falling => {
            let tex = &assets.edie_jump;
            // Centered at the visual box
            draw_tex_at(tex, logical_x, logical_y, vis_w, vis_h, cam, WHITE);
        }
    }
}

/// Returns true if this obstacle warrants a telegraph flash (about to be
/// impossible to react to) given the current scroll speed.
fn needs_telegraph(o: &Obstacle, speed: f32) -> bool {
    if !matches!(o.kind, ObstacleKind::Amy) {
        return false;
    }
    let dist = o.x - PLAYER_X;
    if dist <= 0.0 || speed <= 0.0 {
        return false;
    }
    let t = dist / speed;
    t > 0.0 && t < 0.25
}

pub fn draw_obstacle(
    o: &Obstacle,
    assets: &AssetHandles,
    elapsed: f32,
    current_speed: f32,
    cam: &Camera,
) {
    let (w, h) = o.kind.size();

    // Telegraph flash: red outline pulse when Amy is about to reach the player
    if needs_telegraph(o, current_speed) {
        let pulse = ((elapsed * 24.0).sin() * 0.5 + 0.5) * 0.6 + 0.2;
        let (sx, sy) = cam.to_screen(o.x - 2.0, o.y - 2.0);
        draw_rectangle_lines(
            sx,
            sy,
            cam.scaled(w + 4.0),
            cam.scaled(h + 4.0),
            3.0,
            Color::new(0.95, 0.15, 0.2, pulse),
        );
    }
    match o.kind {
        ObstacleKind::CoffeeCup => {
            draw_tex_at(&assets.obstacle_coffee, o.x, o.y, w, h, cam, WHITE);
        }
        ObstacleKind::ShoppingCart => {
            draw_tex_at(&assets.obstacle_cart, o.x, o.y, w, h, cam, WHITE);
        }
        ObstacleKind::TrafficCone => {
            draw_tex_at(&assets.obstacle_cone, o.x, o.y, w, h, cam, WHITE);
        }
        ObstacleKind::SignBoard => {
            let f = frame_index(elapsed, SPARK_FPS, SPARK_FRAMES);
            draw_tex_frame(
                &assets.obstacle_sign, f, SPARK_FRAME_W, SPARK_FRAME_H, 1.0, o.x, o.y, cam, WHITE,
            );
        }
        ObstacleKind::Cat => {
            let f = frame_index(elapsed, 4.0, 2);
            draw_tex_frame(
                &assets.obstacle_cat, f, 40.0, 24.0, 1.0, o.x, o.y, cam, WHITE,
            );
        }
        ObstacleKind::VacuumBot => {
            let f = frame_index(elapsed, 6.0, 4);
            draw_tex_frame(
                &assets.obstacle_vacuum, f, 40.0, 20.0, 1.0, o.x, o.y, cam, WHITE,
            );
        }
        ObstacleKind::Amy => {
            let f = frame_index(elapsed, 12.0, 4);
            draw_tex_frame(
                &assets.obstacle_amy, f, 44.0, 32.0, 1.0, o.x, o.y, cam, WHITE,
            );
        }
        ObstacleKind::AliceM1 => {
            let f = frame_index(elapsed, 6.0, 2);
            draw_tex_frame(
                &assets.obstacle_alicem1, f, 36.0, 36.0, 1.0, o.x, o.y, cam, WHITE,
            );
        }
        ObstacleKind::Alice3 => {
            let f = frame_index(elapsed, 4.0, 2);
            draw_tex_frame(
                &assets.obstacle_alice3, f, 32.0, 64.0, 1.0, o.x, o.y, cam, WHITE,
            );
        }
        ObstacleKind::Alice4 => {
            let f = frame_index(elapsed, 4.0, 2);
            draw_tex_frame(
                &assets.obstacle_alice4, f, 36.0, 68.0, 1.0, o.x, o.y, cam, WHITE,
            );
        }
    }
}

pub fn draw_aurora(s: &AuroraStone, assets: &AssetHandles, elapsed: f32, cam: &Camera) {
    let tex = match s.color {
        AuroraColor::Purple => &assets.aurora_purple,
        AuroraColor::Green => &assets.aurora_green,
    };
    let f = frame_index(elapsed, AURORA_FPS, AURORA_FRAMES);
    draw_tex_frame(
        tex, f, AURORA_FRAME_W, AURORA_FRAME_H, 1.0, s.x, s.y, cam, WHITE,
    );
}

pub fn draw_heart_pickup(h: &HeartPod, assets: &AssetHandles, elapsed: f32, cam: &Camera) {
    let f = frame_index(elapsed, 6.0, 4);
    draw_tex_frame(
        &assets.heart,
        f,
        HEART_W,
        HEART_H,
        1.0,
        h.x,
        h.y,
        cam,
        WHITE,
    );
}

pub fn draw_effects(effects: &crate::game::effects::Effects, cam: &Camera) {
    for p in &effects.particles {
        let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
        let (sx, sy) = cam.to_screen(p.x, p.y);
        draw_rectangle(
            sx,
            sy,
            cam.scaled(p.size),
            cam.scaled(p.size),
            Color::new(p.r, p.g, p.b, alpha),
        );
    }
    for pop in &effects.popups {
        let alpha = (pop.life / pop.max_life).clamp(0.0, 1.0);
        let size = 26.0 * cam.scale;
        let dim = measure_text(&pop.text, None, size as u16, 1.0);
        let (sx, sy) = cam.to_screen(pop.x, pop.y);
        draw_text(
            &pop.text,
            sx - dim.width * 0.5,
            sy,
            size,
            Color::new(pop.r, pop.g, pop.b, alpha),
        );
    }
}

/// Fullscreen hit flash overlay.
pub fn draw_hit_flash(effects: &crate::game::effects::Effects, cam: &Camera) {
    if effects.hit_flash <= 0.0 {
        return;
    }
    let alpha = (effects.hit_flash / 0.5) * effects.flash_max;
    draw_rectangle(
        0.0,
        0.0,
        cam.screen_w,
        cam.screen_h,
        Color::new(0.95, 0.15, 0.2, alpha.clamp(0.0, 0.6)),
    );
}

/// Tier-change banner — top-of-screen pulse when crossing a difficulty tier.
pub fn draw_tier_banner(effects: &crate::game::effects::Effects, cam: &Camera) {
    let banner = match &effects.tier_banner {
        Some(b) => b,
        None => return,
    };
    let alpha = ((banner.remaining / banner.total) * 2.0).min(1.0);
    let pulse = 0.7 + 0.3 * (banner.remaining * 6.0).sin().abs();

    let (sx, sy) = cam.to_screen(640.0, 100.0);
    let size = 34.0 * cam.scale;
    let dim = measure_text(&banner.text, None, size as u16, 1.0);

    // Background pill
    let pad_x = 24.0 * cam.scale;
    let pad_y = 10.0 * cam.scale;
    draw_rectangle(
        sx - dim.width * 0.5 - pad_x,
        sy - dim.height - pad_y,
        dim.width + pad_x * 2.0,
        dim.height + pad_y * 2.0,
        Color::new(0.05, 0.05, 0.08, alpha * 0.85),
    );
    draw_rectangle_lines(
        sx - dim.width * 0.5 - pad_x,
        sy - dim.height - pad_y,
        dim.width + pad_x * 2.0,
        dim.height + pad_y * 2.0,
        3.0,
        Color::new(1.0, 0.82, 0.2, alpha * pulse),
    );
    draw_text(
        &banner.text,
        sx - dim.width * 0.5,
        sy,
        size,
        Color::new(1.0, 0.85, 0.2, alpha),
    );
}

/// 3-2-1-GO countdown overlay at the start of a run.
pub fn draw_countdown(remaining: f32, cam: &Camera) {
    if remaining <= 0.0 {
        return;
    }
    let label = if remaining > 2.5 {
        "3"
    } else if remaining > 1.5 {
        "2"
    } else if remaining > 0.5 {
        "1"
    } else {
        "GO!"
    };

    // Fade background slightly during countdown
    draw_rectangle(
        0.0,
        0.0,
        cam.screen_w,
        cam.screen_h,
        Color::new(0.0, 0.0, 0.0, 0.3),
    );

    // Giant pulsing number
    let frac_in_second = remaining.fract();
    let scale_boost = (1.0 - frac_in_second).max(0.0); // starts big, shrinks
    let base_size = if label == "GO!" { 140.0 } else { 180.0 };
    let size = (base_size + scale_boost * 40.0) * cam.scale;
    let color = if label == "GO!" {
        Color::new(0.18, 0.77, 0.71, 1.0)
    } else {
        Color::new(1.0, 0.85, 0.2, 1.0)
    };
    let dim = measure_text(label, None, size as u16, 1.0);
    let (sx, sy) = cam.to_screen(640.0, 200.0);
    // Drop shadow
    draw_text(
        label,
        sx - dim.width * 0.5 + 4.0,
        sy + dim.height * 0.5 + 4.0,
        size,
        Color::new(0.0, 0.0, 0.0, 0.7),
    );
    draw_text(
        label,
        sx - dim.width * 0.5,
        sy + dim.height * 0.5,
        size,
        color,
    );
}

/// Speed-tier vignette — darkens edges as speed climbs.
pub fn draw_vignette(speed: f32, cam: &Camera) {
    const BASE: f32 = 320.0;
    const CAP: f32 = 640.0;
    let t = ((speed - BASE) / (CAP - BASE)).clamp(0.0, 1.0);
    if t < 0.05 {
        return;
    }
    let intensity = t * 0.45;
    // Four edge bands, each growing stronger as speed climbs
    let band = cam.screen_h.min(cam.screen_w) * 0.18;
    // Top
    draw_rectangle(
        0.0,
        0.0,
        cam.screen_w,
        band,
        Color::new(0.0, 0.0, 0.0, intensity * 0.6),
    );
    // Bottom
    draw_rectangle(
        0.0,
        cam.screen_h - band,
        cam.screen_w,
        band,
        Color::new(0.0, 0.0, 0.0, intensity * 0.6),
    );
    // Left
    draw_rectangle(
        0.0,
        0.0,
        band,
        cam.screen_h,
        Color::new(0.0, 0.0, 0.0, intensity * 0.5),
    );
    // Right
    draw_rectangle(
        cam.screen_w - band,
        0.0,
        band,
        cam.screen_h,
        Color::new(0.0, 0.0, 0.0, intensity * 0.5),
    );
}
