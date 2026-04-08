//! Phase 2 sprite drawing - textured.

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

/// Returns true if this obstacle warrants a telegraph flash.
fn needs_telegraph(o: &Obstacle, speed: f32) -> bool {
    if !matches!(
        o.kind,
        ObstacleKind::Amy
            | ObstacleKind::BalloonDrone
            | ObstacleKind::Car
            | ObstacleKind::Deer
    ) {
        return false;
    }
    let dist = o.x - PLAYER_X;
    if dist <= 0.0 || speed <= 0.0 {
        return false;
    }
    let t = dist / speed;
    t > 0.0 && t < 0.28
}

pub fn draw_obstacle(
    o: &Obstacle,
    assets: &AssetHandles,
    elapsed: f32,
    current_speed: f32,
    cam: &Camera,
) {
    let (w, h) = o.kind.size();

    // Drop shadow under ground obstacles (visibility aid)
    if o.kind.has_ground_shadow() {
        let shadow_w = w * 0.9;
        let shadow_x = o.x + (w - shadow_w) * 0.5;
        let (sx, sy) = cam.to_screen(shadow_x, GROUND_Y - 3.0);
        draw_texture_ex(
            &assets.obstacle_shadow,
            sx,
            sy,
            Color::new(0.0, 0.0, 0.0, 0.6),
            DrawTextureParams {
                dest_size: Some(vec2(cam.scaled(shadow_w), cam.scaled(6.0))),
                ..Default::default()
            },
        );
    }

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
        ObstacleKind::Car => {
            draw_tex_at(&assets.obstacle_car, o.x, o.y, w, h, cam, WHITE);
        }
        ObstacleKind::Deer => {
            let f = frame_index(elapsed, 6.0, 2);
            draw_tex_frame(
                &assets.obstacle_deer, f, 48.0, 52.0, 1.0, o.x, o.y, cam, WHITE,
            );
        }
        ObstacleKind::BalloonDrone => {
            let f = frame_index(elapsed, 6.0, 4);
            draw_tex_frame(
                &assets.obstacle_balloon, f, 40.0, 48.0, 1.0, o.x, o.y, cam, WHITE,
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

/// Boss entrance cinematic — 3.5s of building-breaks + flash + boss descent.
/// `remaining` counts down from BOSS_INTRO_DURATION to 0.
pub fn draw_boss_intro(remaining: f32, assets: &AssetHandles, cam: &Camera) {
    use crate::game::boss::{BOSS_INTRO_DURATION, BOSS_SIZE, BOSS_X, BOSS_Y_BASE};
    let total = BOSS_INTRO_DURATION;
    let t = total - remaining; // 0 -> total
    let p = (t / total).clamp(0.0, 1.0); // 0..1

    // Phase 1 (0.0-1.2s): cracks spreading across the screen
    // Phase 2 (1.2-1.8s): white flash peak
    // Phase 3 (1.8-3.5s): boss descends from top, settles into position

    // Dim overlay building up
    let dim = (p * 0.7).min(0.7);
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(
        x0,
        y0,
        cam.scaled(1280.0),
        cam.scaled(400.0),
        Color::new(0.02, 0.04, 0.02, dim),
    );

    // Cracks spreading from center (accumulate over first ~1.2s)
    let crack_progress = (t / 1.2).clamp(0.0, 1.0);
    let crack_defs: &[(f32, f32, f32, f32)] = &[
        (640.0, 200.0, -300.0, -180.0),
        (640.0, 200.0, 320.0, -160.0),
        (640.0, 200.0, -260.0, 180.0),
        (640.0, 200.0, 290.0, 190.0),
        (640.0, 200.0, -420.0, -20.0),
        (640.0, 200.0, 420.0, 40.0),
        (640.0, 200.0, -60.0, -220.0),
        (640.0, 200.0, 80.0, 220.0),
    ];
    for (sx, sy, ex, ey) in crack_defs.iter().copied() {
        let cx = sx + ex * crack_progress;
        let cy = sy + ey * crack_progress;
        let (x1, y1) = cam.to_screen(sx, sy);
        let (x2, y2) = cam.to_screen(cx, cy);
        draw_line(
            x1,
            y1,
            x2,
            y2,
            3.0 * cam.scale,
            Color::new(0.95, 0.95, 0.8, 0.85),
        );
        draw_line(
            x1,
            y1,
            x2,
            y2,
            1.5 * cam.scale,
            Color::new(1.0, 0.6, 0.2, 1.0),
        );
    }

    // Falling debris particles (simple deterministic)
    if t > 0.6 {
        for i in 0..12i32 {
            let seed = i as f32 * 7.13;
            let fx = (seed * 53.0).sin() * 620.0 + 640.0;
            let fall = ((t - 0.6).max(0.0) * 180.0) + seed * 40.0;
            let fy = (fall % 380.0) + 20.0;
            let (px, py) = cam.to_screen(fx, fy);
            draw_rectangle(
                px,
                py,
                cam.scaled(4.0),
                cam.scaled(4.0),
                Color::new(0.7, 0.7, 0.65, 0.9),
            );
        }
    }

    // Phase 2: white flash peak at t ~1.4
    let flash_center = 1.4;
    let flash_width = 0.5;
    let flash_dist = (t - flash_center).abs();
    if flash_dist < flash_width {
        let flash_a = 1.0 - (flash_dist / flash_width);
        draw_rectangle(
            0.0,
            0.0,
            cam.screen_w,
            cam.screen_h,
            Color::new(1.0, 1.0, 0.95, flash_a.powi(2) * 0.9),
        );
    }

    // Phase 3: boss descends
    if t > 1.6 {
        let descent_t = ((t - 1.6) / (total - 1.6)).clamp(0.0, 1.0);
        // Ease out: starts fast, settles gently
        let ease = 1.0 - (1.0 - descent_t).powi(3);
        let start_y = -BOSS_SIZE;
        let end_y = BOSS_Y_BASE - BOSS_SIZE * 0.5;
        let boss_y = start_y + (end_y - start_y) * ease;
        let boss_x = BOSS_X - BOSS_SIZE * 0.5;
        let (sbx, sby) = cam.to_screen(boss_x, boss_y);
        // Trail glow behind descending boss
        for i in 1..4 {
            let offset_y = i as f32 * -18.0;
            let alpha = 0.25 - (i as f32 * 0.07);
            draw_rectangle(
                sbx,
                sby + cam.scaled(offset_y),
                cam.scaled(BOSS_SIZE),
                cam.scaled(BOSS_SIZE * 0.3),
                Color::new(0.4, 1.0, 0.5, alpha),
            );
        }
        draw_texture_ex(
            &assets.boss_virus,
            sbx,
            sby,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(cam.scaled(BOSS_SIZE), cam.scaled(BOSS_SIZE))),
                ..Default::default()
            },
        );
    }

    // "VIRUS INTRUSION" text appears around flash time
    if t > 1.2 {
        let alpha = ((t - 1.2) / 0.4).clamp(0.0, 1.0);
        let txt = "VIRUS INTRUSION";
        let size = 48.0 * cam.scale;
        let dim_t = measure_text(txt, None, size as u16, 1.0);
        let (cx, cy) = cam.to_screen(640.0, 260.0);
        // Shadow
        draw_text(
            txt,
            cx - dim_t.width * 0.5 + 3.0,
            cy + 3.0,
            size,
            Color::new(0.0, 0.0, 0.0, alpha * 0.7),
        );
        draw_text(
            txt,
            cx - dim_t.width * 0.5,
            cy,
            size,
            Color::new(0.95, 0.2, 0.2, alpha),
        );
    }
}

pub fn draw_boss_mode(
    boss: &crate::game::boss::BossWorld,
    assets: &AssetHandles,
    cam: &Camera,
) {
    use crate::game::boss::{Facing, LaserPhase, VirusColor, BOSS_SIZE, LASER_WIDTH};

    // Dark sickly overlay background
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(
        x0,
        y0,
        cam.scaled(1280.0),
        cam.scaled(400.0),
        Color::new(0.05, 0.10, 0.06, 0.92),
    );

    // Scan-line tint bands
    for i in 0..8 {
        let y = i as f32 * 52.0;
        let (bx, by) = cam.to_screen(0.0, y);
        draw_rectangle(
            bx,
            by,
            cam.scaled(1280.0),
            cam.scaled(2.0),
            Color::new(0.2, 0.85, 0.35, 0.14),
        );
    }

    // ======================================================
    // Boss: giant corona virus, central, slight bob
    // ======================================================
    let (bx_c, by_c) = boss.boss_center();
    let boss_x = bx_c - BOSS_SIZE * 0.5;
    let boss_y = by_c - BOSS_SIZE * 0.5;
    // Pulsing outer aura
    let aura_scale = 1.0 + ((boss.boss_bob_t * 3.0).sin() * 0.05);
    let aura_w = BOSS_SIZE * aura_scale * 1.15;
    let aura_x = bx_c - aura_w * 0.5;
    let aura_y = by_c - aura_w * 0.5;
    let (sax, say) = cam.to_screen(aura_x, aura_y);
    draw_rectangle(
        sax,
        say,
        cam.scaled(aura_w),
        cam.scaled(aura_w),
        Color::new(0.2, 0.9, 0.3, 0.08),
    );
    // Main boss sprite
    let (sbx, sby) = cam.to_screen(boss_x, boss_y);
    draw_texture_ex(
        &assets.boss_virus,
        sbx,
        sby,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(cam.scaled(BOSS_SIZE), cam.scaled(BOSS_SIZE))),
            ..Default::default()
        },
    );

    // ======================================================
    // Laser (warn / firing)
    // ======================================================
    if let Some(laser) = &boss.laser {
        let lx_min = laser.target_x - LASER_WIDTH * 0.5;
        let laser_top = boss_y + BOSS_SIZE * 0.9;
        let laser_bottom = 400.0;
        let laser_h = laser_bottom - laser_top;
        let (sx, sy) = cam.to_screen(lx_min, laser_top);

        match laser.phase {
            LaserPhase::Warn => {
                // Pulsing red warning beam (thin)
                let pulse = ((boss.elapsed * 20.0).sin() * 0.5 + 0.5) * 0.45 + 0.25;
                // Crosshair line
                draw_rectangle(
                    sx,
                    sy,
                    cam.scaled(LASER_WIDTH),
                    cam.scaled(laser_h),
                    Color::new(1.0, 0.15, 0.2, pulse * 0.5),
                );
                // Center line
                let (mcx, _mcy) = cam.to_screen(laser.target_x - 1.0, laser_top);
                draw_rectangle(
                    mcx,
                    sy,
                    cam.scaled(2.0),
                    cam.scaled(laser_h),
                    Color::new(1.0, 0.2, 0.25, pulse),
                );
                // WARNING text under boss
                let wtxt = "! WARNING !";
                let wsize = 22.0 * cam.scale;
                let wdim = measure_text(wtxt, None, wsize as u16, 1.0);
                let (wx, wy) = cam.to_screen(laser.target_x, laser_top - 20.0);
                draw_text(
                    wtxt,
                    wx - wdim.width * 0.5,
                    wy,
                    wsize,
                    Color::new(1.0, 0.3, 0.3, pulse + 0.3),
                );
            }
            LaserPhase::Firing => {
                // Outer glow
                let glow_w = LASER_WIDTH + 24.0;
                let (gx, gy) = cam.to_screen(laser.target_x - glow_w * 0.5, laser_top);
                draw_rectangle(
                    gx,
                    gy,
                    cam.scaled(glow_w),
                    cam.scaled(laser_h),
                    Color::new(1.0, 0.8, 0.3, 0.25),
                );
                // Main beam (yellow-orange)
                draw_rectangle(
                    sx,
                    sy,
                    cam.scaled(LASER_WIDTH),
                    cam.scaled(laser_h),
                    Color::new(1.0, 0.82, 0.2, 0.85),
                );
                // Inner core (white)
                let core_w = LASER_WIDTH * 0.35;
                let (cx, cy) = cam.to_screen(laser.target_x - core_w * 0.5, laser_top);
                draw_rectangle(
                    cx,
                    cy,
                    cam.scaled(core_w),
                    cam.scaled(laser_h),
                    Color::new(1.0, 1.0, 1.0, 0.9),
                );
            }
        }
    }

    // ======================================================
    // Viruses (green + purple)
    // ======================================================
    let green_fw = (assets.virus_green.width() - 3.0) / 4.0;
    let green_fh = assets.virus_green.height();
    let purple_fw = (assets.virus_purple.width() - 3.0) / 4.0;
    let purple_fh = assets.virus_purple.height();
    let vf = ((boss.elapsed * 10.0) as usize) % 4;

    for v in &boss.viruses {
        if !v.alive {
            continue;
        }
        let (vx, vy) = cam.to_screen(v.x, v.y);
        let (tex, fw, fh) = match v.color {
            VirusColor::Green => (&assets.virus_green, green_fw, green_fh),
            VirusColor::Purple => (&assets.virus_purple, purple_fw, purple_fh),
        };
        draw_texture_ex(
            tex,
            vx,
            vy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    cam.scaled(crate::game::boss::VIRUS_W),
                    cam.scaled(crate::game::boss::VIRUS_H),
                )),
                source: Some(Rect {
                    x: vf as f32 * (fw + 1.0),
                    y: 0.0,
                    w: fw,
                    h: fh,
                }),
                ..Default::default()
            },
        );
    }

    // ======================================================
    // Player EDIE at bottom, with facing flip
    // ======================================================
    let vis_w = 56.0;
    let vis_h = 48.0;
    let player_y = 400.0 - vis_h - 16.0;
    let frame_w = (assets.edie_run_anim.width() - 6.0) / 7.0;
    let frame_h = assets.edie_run_anim.height();
    let frame_idx = ((boss.elapsed * 10.0) as usize) % 7;
    let (pxs, pys) = cam.to_screen(boss.player_x, player_y);
    draw_texture_ex(
        &assets.edie_run_anim,
        pxs,
        pys,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(cam.scaled(vis_w), cam.scaled(vis_h))),
            source: Some(Rect {
                x: frame_idx as f32 * (frame_w + 1.0),
                y: 0.0,
                w: frame_w,
                h: frame_h,
            }),
            flip_x: matches!(boss.player_facing, Facing::Left),
            ..Default::default()
        },
    );

    // ======================================================
    // HUD: timer bar, countdown text
    // ======================================================
    let progress = (boss.remaining / crate::game::boss::BOSS_DURATION).clamp(0.0, 1.0);
    let bar_w = 800.0;
    let bar_x = 640.0 - bar_w * 0.5;
    let bar_y = 32.0;
    let (bsx, bsy) = cam.to_screen(bar_x, bar_y);
    draw_rectangle(
        bsx,
        bsy,
        cam.scaled(bar_w),
        cam.scaled(16.0),
        Color::new(0.1, 0.1, 0.1, 0.9),
    );
    draw_rectangle(
        bsx,
        bsy,
        cam.scaled(bar_w * progress),
        cam.scaled(16.0),
        Color::new(0.2, 0.85, 0.35, 1.0),
    );
    draw_rectangle_lines(
        bsx,
        bsy,
        cam.scaled(bar_w),
        cam.scaled(16.0),
        2.0,
        Color::new(0.9, 0.95, 0.85, 1.0),
    );

    let txt = format!("SURVIVE  {:>4.1}s", boss.remaining.max(0.0));
    let size = 22.0 * cam.scale;
    let dim = measure_text(&txt, None, size as u16, 1.0);
    let (tx, ty) = cam.to_screen(640.0, 22.0);
    draw_text(
        &txt,
        tx - dim.width * 0.5,
        ty,
        size,
        Color::new(0.95, 0.95, 0.8, 1.0),
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

/// Tier-change banner - top-of-screen pulse when crossing a difficulty tier.
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

/// Speed-tier vignette - darkens edges as speed climbs.
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
