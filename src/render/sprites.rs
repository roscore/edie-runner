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
/// edie_happy_run is the full 17-frame smile loop from 1000027555.gif.
pub const EDIE_HAPPY_FRAMES: usize = 17;
pub const EDIE_HAPPY_FPS: f32 = 10.0;

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

    // Default playable face = happy (7555). Jump/Fall/Duck re-use the same
    // happy cycle so the mood never flips to the grumpy baseline. Only Hit
    // swaps to the dizzy x-eye animation.
    match player.state {
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
            // Duck: static sprite squashed shorter
            let duck_h = vis_h * 0.58;
            let duck_y = player.y + PLAYER_H - duck_h;
            draw_tex_at(
                &assets.edie_static_run,
                logical_x,
                duck_y,
                vis_w,
                duck_h,
                cam,
                WHITE,
            );
        }
        PlayerState::Jumping | PlayerState::Falling => {
            // Use the dedicated jump reference sprite
            draw_tex_at(
                &assets.edie_jump,
                logical_x,
                logical_y,
                vis_w,
                vis_h,
                cam,
                WHITE,
            );
        }
        PlayerState::Running => {
            // Static in-game running face (1000027542.png quantized)
            let bob = ((elapsed * 8.0).sin() * 1.0).round();
            logical_y += bob;
            draw_tex_at(
                &assets.edie_static_run,
                logical_x,
                logical_y,
                vis_w,
                vis_h,
                cam,
                WHITE,
            );
        }
    }
}

/// Returns true if this obstacle warrants a telegraph flash.
fn needs_telegraph(o: &Obstacle, speed: f32) -> bool {
    if !matches!(
        o.kind,
        ObstacleKind::Amy
            | ObstacleKind::BalloonDrone
            | ObstacleKind::Pigeon
            | ObstacleKind::Car
            | ObstacleKind::SportsCar
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
    infected: bool,
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
        ObstacleKind::CatOrange => {
            let f = frame_index(elapsed, 4.0, 2);
            draw_tex_frame(
                &assets.obstacle_cat_orange, f, 48.0, 40.0, 1.0, o.x, o.y, cam, WHITE,
            );
        }
        ObstacleKind::CatWhite => {
            let f = frame_index(elapsed, 4.0, 2);
            draw_tex_frame(
                &assets.obstacle_cat_white, f, 48.0, 40.0, 1.0, o.x, o.y, cam, WHITE,
            );
        }
        ObstacleKind::Car => {
            // Animated 17-frame GIF-sourced sprite (car0)
            draw_anim_sheet(
                &assets.obstacle_car_anim,
                17,
                12.0,
                elapsed,
                o.x,
                o.y,
                w,
                h,
                cam,
                WHITE,
            );
        }
        ObstacleKind::Truck => {
            draw_tex_at(&assets.obstacle_truck, o.x, o.y, w, h, cam, WHITE);
        }
        ObstacleKind::Bus => {
            draw_tex_at(&assets.obstacle_bus, o.x, o.y, w, h, cam, WHITE);
        }
        ObstacleKind::Taxi => {
            draw_tex_at(&assets.obstacle_taxi, o.x, o.y, w, h, cam, WHITE);
        }
        ObstacleKind::SportsCar => {
            // Animated 17-frame sky-blue sportscar (car1)
            draw_anim_sheet(
                &assets.obstacle_sportscar_anim,
                17,
                14.0,
                elapsed,
                o.x,
                o.y,
                w,
                h,
                cam,
                WHITE,
            );
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
        ObstacleKind::Pigeon => {
            let f = frame_index(elapsed, 8.0, 2);
            draw_tex_frame(
                &assets.obstacle_pigeon, f, 36.0, 32.0, 1.0, o.x, o.y, cam, WHITE,
            );
        }
        ObstacleKind::BoxBot => {
            let tex = if infected { &assets.obstacle_infected_boxbot } else { &assets.obstacle_boxbot };
            let f = frame_index(elapsed, 4.0, 2);
            draw_tex_frame(tex, f, 44.0, 40.0, 1.0, o.x, o.y, cam, WHITE);
        }
        ObstacleKind::Amy => {
            let tex = if infected { &assets.obstacle_infected_amy } else { &assets.obstacle_amy };
            draw_tex_at(tex, o.x, o.y, w, h, cam, WHITE);
        }
        ObstacleKind::AliceM1 => {
            let tex = if infected { &assets.obstacle_infected_alicem1 } else { &assets.obstacle_alicem1 };
            draw_tex_at(tex, o.x, o.y, w, h, cam, WHITE);
        }
        ObstacleKind::Alice3 => {
            let tex = if infected { &assets.obstacle_infected_alice3 } else { &assets.obstacle_alice3 };
            draw_tex_at(tex, o.x, o.y, w, h, cam, WHITE);
        }
        ObstacleKind::Alice4 => {
            let tex = if infected { &assets.obstacle_infected_alice4 } else { &assets.obstacle_alice4 };
            draw_tex_at(tex, o.x, o.y, w, h, cam, WHITE);
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
    // Visually weakens as the timer runs down.
    // ======================================================
    // health_frac: 1.0 at start, 0.0 when timer expires
    let health_frac =
        (boss.remaining / crate::game::boss::BOSS_DURATION).clamp(0.0, 1.0);
    // weakness: 0.0 at start, 1.0 at timer expiry
    let weakness = 1.0 - health_frac;

    // Damage shake (more violent as boss weakens)
    let shake_amp = weakness * 8.0;
    let shake_x = (boss.elapsed * 47.0).sin() * shake_amp;
    let shake_y = (boss.elapsed * 53.0).cos() * shake_amp * 0.5;

    let (bx_c, by_c) = boss.boss_center();
    let bx_c = bx_c + shake_x;
    let by_c = by_c + shake_y;
    let boss_x = bx_c - BOSS_SIZE * 0.5;
    let boss_y = by_c - BOSS_SIZE * 0.5;

    // Circular aura halo -- three concentric pulsing rings that fade with
    // boss health.
    let (sax, say) = cam.to_screen(bx_c, by_c);
    let pulse = (boss.boss_bob_t * 3.0).sin();
    for (r_mul, alpha) in [(1.18, 0.10), (1.30, 0.06), (1.44, 0.035)] {
        let r = BOSS_SIZE * 0.5 * r_mul * (1.0 + pulse * 0.04);
        draw_circle(
            sax,
            say,
            cam.scaled(r),
            Color::new(0.35, 1.0, 0.4, alpha * health_frac),
        );
    }

    // Main boss sprite -- color drains, alpha drops, hint of red as it dies
    let (sbx, sby) = cam.to_screen(boss_x, boss_y);
    let boss_tint = Color::new(
        1.0 - weakness * 0.2,                  // slight desaturation
        1.0 - weakness * 0.55,                 // green channel drops fast
        1.0 - weakness * 0.55,
        (0.55 + 0.45 * health_frac).max(0.45), // partial transparency near death
    );
    draw_texture_ex(
        &assets.boss_virus,
        sbx,
        sby,
        boss_tint,
        DrawTextureParams {
            dest_size: Some(vec2(cam.scaled(BOSS_SIZE), cam.scaled(BOSS_SIZE))),
            ..Default::default()
        },
    );

    // Crack overlays appear as boss weakens (more cracks the more damaged)
    let crack_count = (weakness * 6.0) as i32;
    for i in 0..crack_count {
        let seed = i as f32 * 11.7;
        let cx0 = bx_c + (seed.sin() * BOSS_SIZE * 0.35);
        let cy0 = by_c + (seed.cos() * BOSS_SIZE * 0.35);
        let cx1 = cx0 + (seed * 1.7).sin() * 32.0;
        let cy1 = cy0 + (seed * 1.7).cos() * 32.0;
        let (x1, y1) = cam.to_screen(cx0, cy0);
        let (x2, y2) = cam.to_screen(cx1, cy1);
        draw_line(
            x1,
            y1,
            x2,
            y2,
            2.0 * cam.scale,
            Color::new(0.05, 0.05, 0.08, 0.85),
        );
    }

    // Damage smoke/particles drifting up from boss (more = weaker)
    let smoke_count = (weakness * 8.0) as i32;
    for i in 0..smoke_count {
        let drift_seed = (boss.elapsed * 1.5 + i as f32 * 0.7) % 1.5;
        let alpha = (1.0 - drift_seed / 1.5) * 0.6;
        let dx = ((i as f32 * 3.1).sin() * BOSS_SIZE * 0.3) + (drift_seed * 6.0).sin() * 4.0;
        let dy = -drift_seed * 60.0;
        let (px, py) = cam.to_screen(bx_c + dx, by_c - 40.0 + dy);
        draw_rectangle(
            px,
            py,
            cam.scaled(6.0),
            cam.scaled(6.0),
            Color::new(0.3, 0.3, 0.35, alpha),
        );
    }

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
    // Sweep laser (during SweepLaser pattern)
    // ======================================================
    if boss.sweep_laser_active {
        let lx = boss.sweep_laser_x;
        let top = by_c + BOSS_SIZE * 0.4;
        let bottom = 400.0;
        let (sx, sy) = cam.to_screen(lx - 30.0, top);
        // Outer glow
        draw_rectangle(
            sx,
            sy,
            cam.scaled(60.0),
            cam.scaled(bottom - top),
            Color::new(1.0, 0.3, 0.4, 0.35),
        );
        // Core beam
        let (cx_l, cy_l) = cam.to_screen(lx - 14.0, top);
        draw_rectangle(
            cx_l,
            cy_l,
            cam.scaled(28.0),
            cam.scaled(bottom - top),
            Color::new(1.0, 0.85, 0.9, 0.9),
        );
        // Bright center line
        let (ccx, ccy) = cam.to_screen(lx - 2.0, top);
        draw_rectangle(
            ccx,
            ccy,
            cam.scaled(4.0),
            cam.scaled(bottom - top),
            Color::new(1.0, 1.0, 1.0, 1.0),
        );
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
    // Coordinates MUST match boss.rs hitbox math (BOSS_EDIE_*).
    // ======================================================
    use crate::game::boss::{BOSS_EDIE_BOTTOM_INSET, BOSS_EDIE_H, BOSS_EDIE_W};
    let vis_w = BOSS_EDIE_W;
    let vis_h = BOSS_EDIE_H;
    let player_y = 400.0 - vis_h - BOSS_EDIE_BOTTOM_INSET;
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

    // Current pattern label
    let pattern_label = match boss.pattern {
        crate::game::boss::BossPattern::Rain => "CORONA RAIN",
        crate::game::boss::BossPattern::DiagonalVolley => "CROSSFIRE VOLLEY",
        crate::game::boss::BossPattern::Spiral => "SPIRAL STORM",
        crate::game::boss::BossPattern::SweepLaser => "SWEEP LASER",
    };
    let psize = 16.0 * cam.scale;
    let pdim = measure_text(pattern_label, None, psize as u16, 1.0);
    let (px, py) = cam.to_screen(640.0, 60.0);
    draw_text(
        pattern_label,
        px - pdim.width * 0.5,
        py,
        psize,
        Color::new(1.0, 0.5, 0.5, 0.85),
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

/// Metal-Slug style stage transition wipe. Plays over the world for ~1.4s
/// when the player enters a new stage. Black bars slide in, the new stage
/// name flashes in the middle, then bars slide out.
pub fn draw_stage_wipe(effects: &crate::game::effects::Effects, cam: &Camera) {
    let wipe = match &effects.stage_wipe {
        Some(w) => w,
        None => return,
    };
    let t = 1.0 - (wipe.remaining / wipe.total); // 0..1
    // Three phases: 0.0-0.35 slide-in, 0.35-0.65 hold, 0.65-1.0 slide-out
    let (bar_in, bar_hold, bar_out) = (0.35, 0.65, 1.0);
    let bar_progress = if t < bar_in {
        (t / bar_in).clamp(0.0, 1.0)
    } else if t < bar_hold {
        1.0
    } else {
        (1.0 - (t - bar_hold) / (bar_out - bar_hold)).clamp(0.0, 1.0)
    };

    let bar_h = 1280.0; // slant length, diagonal feel
    let max_w = 720.0;
    let w = max_w * bar_progress;
    // Top bar slides in from left, bottom bar from right (for diagonal X)
    let (tlx, tly) = cam.to_screen(0.0, 0.0);
    draw_rectangle(
        tlx,
        tly,
        cam.scaled(w),
        cam.scaled(200.0),
        Color::new(0.02, 0.02, 0.04, 1.0),
    );
    let (blx, bly) = cam.to_screen(1280.0 - w, 200.0);
    draw_rectangle(
        blx,
        bly,
        cam.scaled(w),
        cam.scaled(200.0),
        Color::new(0.02, 0.02, 0.04, 1.0),
    );
    // Thin bright trim on the leading edges
    let trim = Color::new(1.0, 0.82, 0.2, 1.0);
    let (tx, ty) = cam.to_screen(w - 3.0, 0.0);
    draw_rectangle(tx, ty, cam.scaled(3.0), cam.scaled(200.0), trim);
    let (bx, by) = cam.to_screen(1280.0 - w, 200.0);
    draw_rectangle(bx, by, cam.scaled(3.0), cam.scaled(200.0), trim);

    // Text appears while bars are held
    if t > 0.25 && t < 0.85 {
        let fade = if t < 0.4 {
            ((t - 0.25) / 0.15).clamp(0.0, 1.0)
        } else if t > 0.7 {
            (1.0 - (t - 0.7) / 0.15).clamp(0.0, 1.0)
        } else {
            1.0
        };
        // "NOW ENTERING" label
        let label = "NOW ENTERING";
        let lsize = 22.0 * cam.scale;
        let ldim = measure_text(label, None, lsize as u16, 1.0);
        let (lx, ly) = cam.to_screen(640.0, 170.0);
        draw_text(
            label,
            lx - ldim.width * 0.5,
            ly,
            lsize,
            Color::new(1.0, 0.85, 0.2, fade),
        );
        // Stage name
        let size = 46.0 * cam.scale;
        let dim = measure_text(&wipe.new_stage_name, None, size as u16, 1.0);
        let (tx, ty) = cam.to_screen(640.0, 220.0);
        // Shadow
        draw_text(
            &wipe.new_stage_name,
            tx - dim.width * 0.5 + 3.0,
            ty + 3.0,
            size,
            Color::new(0.0, 0.0, 0.0, fade * 0.7),
        );
        draw_text(
            &wipe.new_stage_name,
            tx - dim.width * 0.5,
            ty,
            size,
            Color::new(1.0, 1.0, 1.0, fade),
        );
        let _ = bar_h;
    }
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

/// On-screen touch buttons for mobile. Positions are in logical coords and
/// drawn only if `show` is true. Returns the hit rects in LOGICAL coords so
/// the main loop can test touches against them.
#[derive(Debug, Clone, Copy)]
pub struct TouchButton {
    pub label: &'static str,
    pub logical_rect: (f32, f32, f32, f32), // x, y, w, h
}

pub fn boss_touch_buttons() -> [TouchButton; 2] {
    let by = 324.0;
    let bh = 66.0;
    [
        TouchButton { label: "<", logical_rect: (24.0, by, 120.0, bh) },
        TouchButton { label: ">", logical_rect: (150.0, by, 120.0, bh) },
    ]
}

pub fn play_touch_buttons() -> [TouchButton; 3] {
    let by = 324.0;
    let bh = 66.0;
    [
        TouchButton { label: "DUCK", logical_rect: (24.0, by, 130.0, bh) },
        TouchButton { label: "JUMP", logical_rect: (1280.0 - 24.0 - 130.0, by, 130.0, bh) },
        TouchButton { label: "DASH", logical_rect: (1280.0 - 24.0 - 130.0 - 150.0, by, 130.0, bh) },
    ]
}

pub fn draw_touch_buttons(buttons: &[TouchButton], pressed: &[bool], cam: &Camera) {
    for (i, b) in buttons.iter().enumerate() {
        let (lx, ly, lw, lh) = b.logical_rect;
        let (sx, sy) = cam.to_screen(lx, ly);
        let is_pressed = pressed.get(i).copied().unwrap_or(false);
        let bg = if is_pressed {
            Color::new(1.0, 0.85, 0.2, 0.75)
        } else {
            Color::new(0.1, 0.1, 0.15, 0.45)
        };
        draw_rectangle(sx, sy, cam.scaled(lw), cam.scaled(lh), bg);
        draw_rectangle_lines(
            sx,
            sy,
            cam.scaled(lw),
            cam.scaled(lh),
            3.0,
            Color::new(1.0, 0.9, 0.6, 0.9),
        );
        let size = 30.0 * cam.scale;
        let dim = measure_text(b.label, None, size as u16, 1.0);
        let tx = sx + cam.scaled(lw) * 0.5 - dim.width * 0.5;
        let ty = sy + cam.scaled(lh) * 0.5 + dim.height * 0.4;
        draw_text(b.label, tx, ty, size, Color::new(0.98, 0.95, 0.85, 1.0));
    }
}

/// Convert a logical-coordinate rect to screen space for touch hit-testing.
pub fn logical_rect_to_screen(
    rect: (f32, f32, f32, f32),
    cam: &Camera,
) -> (f32, f32, f32, f32) {
    let (lx, ly, lw, lh) = rect;
    let (sx, sy) = cam.to_screen(lx, ly);
    (sx, sy, cam.scaled(lw), cam.scaled(lh))
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
