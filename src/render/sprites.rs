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

// `needs_telegraph` removed in v0.4.1 -- the red outline read as a
// debug hitbox marker in playtest.

/// Sickly green tint applied to the base robot sprite while it is infected.
/// Pulses very gently so the mob feels alive even when standing still.
fn infected_tint(elapsed: f32) -> Color {
    let pulse = 0.5 + 0.5 * (elapsed * 2.4).sin();
    // Mostly green, a dash of cyan/purple in the low spots.
    Color::new(
        0.60 + 0.10 * pulse,
        1.00,
        0.70 + 0.15 * pulse,
        1.0,
    )
}

/// Draw three tiny purple viruses orbiting an infected robot so the
/// "infected" state is legible at a glance instead of just a color tint.
/// The viruses are drawn as small pulsing circles with a bright core --
/// intentionally much smaller than the boss-fight viruses.
fn draw_virus_orbit(
    ox: f32,
    oy: f32,
    w: f32,
    h: f32,
    elapsed: f32,
    cam: &Camera,
) {
    // Lightweight: 2 orbiting dots instead of 3×3 draw_circle calls.
    let cx = ox + w * 0.5;
    let cy = oy + h * 0.5;
    let orbit_rx = (w * 0.5 + 4.0).max(16.0);
    let orbit_ry = (h * 0.3 + 3.0).max(8.0);
    for i in 0..2u32 {
        let phase = (i as f32) * std::f32::consts::PI;
        let t = elapsed * 2.0 + phase;
        let vx = cx + t.cos() * orbit_rx;
        let vy = cy + (t * 1.3).sin() * orbit_ry - 3.0;
        let (sx, sy) = cam.to_screen(vx, vy);
        draw_rectangle(
            sx - cam.scaled(2.5),
            sy - cam.scaled(2.5),
            cam.scaled(5.0),
            cam.scaled(5.0),
            Color::new(0.75, 0.40, 1.0, 0.85),
        );
    }
}

/// Speech bubble drawn above a jumping Alice3 telegraphing her hop.
/// Keeps the word "Squid!!!" short enough to fit at 320x80 logical space.
fn draw_squid_bubble(center_x: f32, top_y: f32, cam: &Camera) {
    let text = "Squid!!!";
    let size = 20.0 * cam.scale;
    let dim = measure_text(text, None, size as u16, 1.0);
    let pad_x = 12.0 * cam.scale;
    let pad_y = 6.0 * cam.scale;
    let (cx_screen, cy_screen) = cam.to_screen(center_x, top_y - 26.0);
    let bw = dim.width + pad_x * 2.0;
    let bh = dim.height + pad_y * 2.0;
    let bx = cx_screen - bw * 0.5;
    let by = cy_screen - bh * 0.5;
    // Bubble fill
    draw_rectangle(bx, by, bw, bh, Color::new(1.0, 1.0, 1.0, 0.95));
    draw_rectangle_lines(bx, by, bw, bh, 3.0, Color::new(0.10, 0.10, 0.12, 1.0));
    // Pointer tail
    draw_triangle(
        macroquad::prelude::Vec2::new(cx_screen - 8.0 * cam.scale, by + bh),
        macroquad::prelude::Vec2::new(cx_screen + 8.0 * cam.scale, by + bh),
        macroquad::prelude::Vec2::new(cx_screen, by + bh + 12.0 * cam.scale),
        Color::new(1.0, 1.0, 1.0, 0.95),
    );
    draw_line(
        cx_screen - 8.0 * cam.scale,
        by + bh,
        cx_screen,
        by + bh + 12.0 * cam.scale,
        2.5,
        Color::new(0.10, 0.10, 0.12, 1.0),
    );
    draw_line(
        cx_screen + 8.0 * cam.scale,
        by + bh,
        cx_screen,
        by + bh + 12.0 * cam.scale,
        2.5,
        Color::new(0.10, 0.10, 0.12, 1.0),
    );
    // Text on top
    draw_text(
        text,
        cx_screen - dim.width * 0.5,
        by + bh * 0.5 + dim.height * 0.35,
        size,
        Color::new(0.85, 0.15, 0.30, 1.0),
    );
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

    // Telegraph flash intentionally removed per v0.4.1 playtest --
    // the pulsing red rectangle outline read as a debug hitbox marker.
    let _ = current_speed;
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
            // Flip horizontally so the bird visibly faces the player
            // (toward the left of the screen) as it charges in.
            let f = frame_index(elapsed, 8.0, 2);
            let (sx, sy) = cam.to_screen(o.x, o.y);
            draw_texture_ex(
                &assets.obstacle_pigeon,
                sx,
                sy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(cam.scaled(36.0), cam.scaled(32.0))),
                    source: Some(Rect {
                        x: f as f32 * (36.0 + 1.0),
                        y: 0.0,
                        w: 36.0,
                        h: 32.0,
                    }),
                    flip_x: true,
                    ..Default::default()
                },
            );
        }
        ObstacleKind::MallBalloon => {
            let f = frame_index(elapsed, 3.0, 2);
            draw_tex_frame(
                &assets.obstacle_mallballoon, f, 44.0, 56.0, 1.0, o.x, o.y, cam, WHITE,
            );
        }
        ObstacleKind::BoxBot => {
            let tex = if infected { &assets.obstacle_infected_boxbot } else { &assets.obstacle_boxbot };
            let f = frame_index(elapsed, 4.0, 2);
            let tint = if infected { infected_tint(elapsed) } else { WHITE };
            draw_tex_frame(tex, f, 44.0, 40.0, 1.0, o.x, o.y, cam, tint);
            if infected {
                draw_virus_orbit(o.x, o.y, w, h, elapsed, cam);
            }
        }
        ObstacleKind::Amy => {
            let tex = if infected { &assets.obstacle_infected_amy } else { &assets.obstacle_amy };
            let tint = if infected { infected_tint(elapsed) } else { WHITE };
            draw_tex_at(tex, o.x, o.y, w, h, cam, tint);
            if infected {
                draw_virus_orbit(o.x, o.y, w, h, elapsed, cam);
            }
        }
        ObstacleKind::AliceM1 => {
            let tex = if infected { &assets.obstacle_infected_alicem1 } else { &assets.obstacle_alicem1 };
            let tint = if infected { infected_tint(elapsed) } else { WHITE };
            draw_tex_at(tex, o.x, o.y, w, h, cam, tint);
            if infected {
                draw_virus_orbit(o.x, o.y, w, h, elapsed, cam);
            }
        }
        ObstacleKind::Alice3 => {
            let tex = if infected { &assets.obstacle_infected_alice3 } else { &assets.obstacle_alice3 };
            let tint = if infected { infected_tint(elapsed) } else { WHITE };
            draw_tex_at(tex, o.x, o.y, w, h, cam, tint);
            if infected {
                draw_virus_orbit(o.x, o.y, w, h, elapsed, cam);
            }
            // "Squid!!!" speech bubble while Alice3 is in her airborne
            // hop (pattern_t > 1.0 marks the active hover window).
            if o.pattern_t > 1.0 {
                draw_squid_bubble(o.x + w * 0.5, o.y - 8.0, cam);
            }
        }
        ObstacleKind::Alice4 => {
            let tex = if infected { &assets.obstacle_infected_alice4 } else { &assets.obstacle_alice4 };
            let tint = if infected { infected_tint(elapsed) } else { WHITE };
            draw_tex_at(tex, o.x, o.y, w, h, cam, tint);
            if infected {
                draw_virus_orbit(o.x, o.y, w, h, elapsed, cam);
            }
        }
        ObstacleKind::InfectedEdie => {
            // Reuse the normal EDIE run sprite, painted with the same
            // sickly-green pulse used on infected robots, plus a ring of
            // purple viruses orbiting her so the falling threat reads
            // clearly against any background.
            let tint = infected_tint(elapsed);
            draw_tex_at(&assets.edie_static_run, o.x, o.y, w, h, cam, tint);
            draw_virus_orbit(o.x, o.y, w, h, elapsed, cam);
        }
        ObstacleKind::SoccerBall => {
            // High-visibility treatment: pulsing neon halo + streaking
            // motion trail under the rolling ball so it reads against the
            // busy Factory background and hordes of Alice bots.
            let f = frame_index(elapsed, 12.0, 2);
            let cx = o.x + w * 0.5;
            let cy = o.y + h * 0.5;
            let pulse = 0.55 + 0.35 * ((elapsed * 18.0).sin() * 0.5 + 0.5);
            let (gx, gy) = cam.to_screen(cx, cy);
            // Outer yellow glow
            draw_circle(
                gx,
                gy,
                cam.scaled(w * 0.85),
                Color::new(1.0, 0.92, 0.25, 0.22 * pulse),
            );
            // Inner red warning halo
            draw_circle(
                gx,
                gy,
                cam.scaled(w * 0.65),
                Color::new(1.0, 0.25, 0.15, 0.35 * pulse),
            );
            // Motion streak behind the ball (toward +x since ball rolls left)
            for i in 0..4i32 {
                let tx = cx + (i as f32 + 1.0) * 10.0;
                let (stx, sty) = cam.to_screen(tx, cy);
                draw_circle(
                    stx,
                    sty,
                    cam.scaled(w * 0.35 - i as f32 * 2.0),
                    Color::new(1.0, 0.88, 0.2, 0.18 - i as f32 * 0.04),
                );
            }
            // Scale the 24x24 source frame up to the obstacle's display
            // size so changes to size() propagate.
            let (bsx, bsy) = cam.to_screen(o.x, o.y);
            let src_w = 24.0f32;
            let src_h = 24.0f32;
            draw_texture_ex(
                &assets.obstacle_soccerball,
                bsx,
                bsy,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(cam.scaled(w), cam.scaled(h))),
                    source: Some(Rect {
                        x: f as f32 * (src_w + 1.0),
                        y: 0.0,
                        w: src_w,
                        h: src_h,
                    }),
                    ..Default::default()
                },
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
/// New v0.4.0 boss break-in cinematic. Driven by `BossIntroState` --
/// auto-advance phases roll through on a timer, dialog phases pause and
/// wait for player input. Effects per phase:
///
///   ALERT   -- pulsing red border, "!! WARNING !!" stinger, ground shake
///   GLITCH  -- CRT scanlines + RGB-split colour bars + chromatic name
///   SLAM    -- boss falls in, lightning bolts, expanding shockwave rings,
///              white flash, four virus minions converge
///   DIALOG1 -- Cave-Story text box, MUNGCHI portrait, typewriter, ▼ caret
///   DIALOG2 -- second taunt
///   DIALOG3 -- EDIE response (different portrait + name colour)
///   CHARGE  -- EDIE crouches, energy rings converge into her, flash glow
///   DASH    -- EDIE rockets across screen as a rainbow streak with
///              afterimage trail (slow-mo, time scaled inside)
///   IMPACT  -- screen-wide white flash, stacked POW/BAM/SMASH text,
///              radial debris particles, violent shake, boss recoils
///   FIGHT   -- boss settles back, "FIGHT!" banner zooms in
pub fn draw_boss_intro(
    intro: &crate::game::boss::BossIntroState,
    assets: &AssetHandles,
    cam: &Camera,
) {
    use crate::game::boss::{BossIntroPhase, BOSS_SIZE, BOSS_X, BOSS_Y_BASE};
    let phase = intro.phase;

    // Phase-to-cumulative-time mapping. The existing cinematic body
    // below is written against an old-style `t` that ticks from 0 at
    // the start of Alert up to the end of Fight. We derive `t` from
    // the new phase state machine so the body keeps working while
    // dialog phases still pause on the phase machine itself.
    const T_ALERT_END: f32 = 0.8;
    const T_GLITCH_END: f32 = 1.6;
    const T_SLAM_END: f32 = 2.6;
    const T_DIALOG1_END: f32 = 4.8;
    const T_DIALOG2_END: f32 = 6.8;
    const T_DIALOG3_END: f32 = 8.4;
    const T_CHARGE_END: f32 = 9.8;
    const T_IMPACT_END: f32 = 10.8;
    let total: f32 = 12.0;
    let phase_start = match phase {
        BossIntroPhase::Alert => 0.0,
        BossIntroPhase::Glitch => T_ALERT_END,
        BossIntroPhase::Slam => T_GLITCH_END,
        BossIntroPhase::Dialog1 => T_SLAM_END,
        BossIntroPhase::Dialog2 => T_DIALOG1_END,
        BossIntroPhase::Dialog3 => T_DIALOG2_END,
        BossIntroPhase::Charge => T_DIALOG3_END,
        BossIntroPhase::Dash => T_CHARGE_END - 0.55,
        BossIntroPhase::Impact => T_CHARGE_END,
        BossIntroPhase::Fight => T_IMPACT_END,
    };
    let phase_cap = match phase {
        BossIntroPhase::Alert => T_ALERT_END - 0.001,
        BossIntroPhase::Glitch => T_GLITCH_END - 0.001,
        BossIntroPhase::Slam => T_SLAM_END - 0.001,
        BossIntroPhase::Dialog1 => T_DIALOG1_END - 0.001,
        BossIntroPhase::Dialog2 => T_DIALOG2_END - 0.001,
        BossIntroPhase::Dialog3 => T_DIALOG3_END - 0.001,
        BossIntroPhase::Charge => T_CHARGE_END - 0.56,
        BossIntroPhase::Dash => T_CHARGE_END - 0.001,
        BossIntroPhase::Impact => T_IMPACT_END - 0.001,
        BossIntroPhase::Fight => total,
    };
    let t = (phase_start + intro.elapsed).min(phase_cap);

    // Base darkening that ramps in over the glitch phase and persists.
    let dim_alpha = if t < T_ALERT_END {
        (t / T_ALERT_END) * 0.35
    } else if t < T_GLITCH_END {
        0.35 + ((t - T_ALERT_END) / (T_GLITCH_END - T_ALERT_END)) * 0.4
    } else {
        0.78
    };
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(
        x0,
        y0,
        cam.scaled(1280.0),
        cam.scaled(400.0),
        Color::new(0.02, 0.04, 0.06, dim_alpha),
    );

    // ---------- PHASE 1: RED ALERT ----------
    if t < T_ALERT_END {
        let pulse = ((t * 20.0).sin() * 0.5 + 0.5) * 0.6 + 0.3;
        // Thick red border flashing around the whole play area.
        let border_w = 12.0;
        for (rx, ry, rw, rh) in [
            (0.0, 0.0, 1280.0, border_w),
            (0.0, 400.0 - border_w, 1280.0, border_w),
            (0.0, 0.0, border_w, 400.0),
            (1280.0 - border_w, 0.0, border_w, 400.0),
        ] {
            let (sxp, syp) = cam.to_screen(rx, ry);
            draw_rectangle(
                sxp,
                syp,
                cam.scaled(rw),
                cam.scaled(rh),
                Color::new(1.0, 0.2, 0.25, pulse),
            );
        }
        // Pulsing "!! WARNING !!" on the top band.
        let txt = "!! WARNING !!";
        let size = 42.0 * cam.scale;
        let dim = measure_text(txt, None, size as u16, 1.0);
        let (tx, ty) = cam.to_screen(640.0, 120.0);
        draw_text(
            txt,
            tx - dim.width * 0.5 + 3.0,
            ty + 3.0,
            size,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );
        draw_text(
            txt,
            tx - dim.width * 0.5,
            ty,
            size,
            Color::new(1.0, 0.3, 0.35, 0.85 + pulse * 0.15),
        );
    }

    // ---------- PHASE 2: GLITCH WIPE ----------
    if t >= T_ALERT_END - 0.1 && t < T_GLITCH_END + 0.2 {
        let g_t = ((t - T_ALERT_END + 0.1) / (T_GLITCH_END - T_ALERT_END + 0.3))
            .clamp(0.0, 1.0);
        // Rapid scanlines scrolling across the screen.
        for i in 0..40 {
            let ly = (i as f32 * 10.0 + t * 380.0) % 400.0;
            let (lsx, lsy) = cam.to_screen(0.0, ly);
            draw_rectangle(
                lsx,
                lsy,
                cam.scaled(1280.0),
                cam.scaled(2.0),
                Color::new(0.15, 0.85, 0.35, 0.22 * g_t),
            );
        }
        // Horizontal RGB-split colour bars at random y positions.
        for i in 0..6i32 {
            let seed = i as f32 * 7.11 + (t * 18.0).sin() * 40.0;
            let by = ((seed * 71.0) as i32).rem_euclid(360) as f32 + 20.0;
            let bh = 8.0 + (seed * 3.0).sin().abs() * 6.0;
            let (bsx, bsy) = cam.to_screen(0.0, by);
            let a = 0.28 * g_t;
            draw_rectangle(
                bsx,
                bsy,
                cam.scaled(1280.0),
                cam.scaled(bh),
                Color::new(1.0, 0.2, 0.25, a),
            );
            draw_rectangle(
                bsx + cam.scaled(6.0),
                bsy,
                cam.scaled(1280.0),
                cam.scaled(bh),
                Color::new(0.2, 1.0, 0.4, a * 0.8),
            );
        }
    }

    // ---------- PHASE 3: BOSS SLAM ----------
    // Clean, restrained slam: dark backdrop + cinematic letterbox +
    // one expanding shockwave ring + one clean white flash on impact.
    // No lightning bolts, no orbiting minions -- the v0.4.0 version had
    // too many overlapping effects and read as noise.
    if t >= T_GLITCH_END {
        let slam_t = ((t - T_GLITCH_END) / (T_SLAM_END - T_GLITCH_END)).clamp(0.0, 1.0);
        let ease = 1.0 - (1.0 - slam_t).powi(3);
        let start_y = -BOSS_SIZE - 40.0;
        let end_y = BOSS_Y_BASE - BOSS_SIZE * 0.5;
        let boss_y = start_y + (end_y - start_y) * ease;
        let boss_x = BOSS_X - BOSS_SIZE * 0.5;

        let (sbx, sby) = cam.to_screen(boss_x, boss_y);
        // Single soft green glow trail behind the descending boss.
        for i in 1..3 {
            let offset_y = i as f32 * -26.0;
            let alpha = 0.28 - (i as f32 * 0.12);
            draw_rectangle(
                sbx,
                sby + cam.scaled(offset_y),
                cam.scaled(BOSS_SIZE),
                cam.scaled(BOSS_SIZE * 0.28),
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
        // Landing effects kick in on the last 30% of the slam.
        if slam_t > 0.70 {
            let ring_t = (slam_t - 0.70) / 0.30;
            let (cxr, cyr) = cam.to_screen(BOSS_X, BOSS_Y_BASE + BOSS_SIZE * 0.45);
            // Single thick expanding ring.
            let r = ring_t * 340.0;
            draw_circle_lines(
                cxr,
                cyr,
                cam.scaled(r),
                6.0 * cam.scale,
                Color::new(1.0, 0.95, 0.7, (1.0 - ring_t) * 0.95),
            );
            // Inner lighter ring for depth.
            if ring_t > 0.2 {
                draw_circle_lines(
                    cxr,
                    cyr,
                    cam.scaled(r * 0.72),
                    3.0 * cam.scale,
                    Color::new(1.0, 0.85, 0.5, (1.0 - ring_t) * 0.75),
                );
            }
            // One strong white flash on land.
            let flash_a = (1.0 - ring_t).powi(2) * 0.9;
            draw_rectangle(
                0.0,
                0.0,
                cam.screen_w,
                cam.screen_h,
                Color::new(1.0, 1.0, 0.95, flash_a),
            );
        }
    }

    // Cinematic letterbox bars: slide in during Slam, hold through
    // Dialog, slide out during Dash. Gives the cinematic a "movie"
    // feel without obscuring the characters.
    {
        let bar_progress = match phase {
            BossIntroPhase::Alert | BossIntroPhase::Glitch => 0.0,
            BossIntroPhase::Slam => {
                (intro.elapsed / 0.8).clamp(0.0, 1.0)
            }
            BossIntroPhase::Dialog1
            | BossIntroPhase::Dialog2
            | BossIntroPhase::Dialog3
            | BossIntroPhase::Charge => 1.0,
            BossIntroPhase::Dash => {
                (1.0 - intro.elapsed / 0.55).clamp(0.0, 1.0)
            }
            BossIntroPhase::Impact | BossIntroPhase::Fight => 0.0,
        };
        if bar_progress > 0.01 {
            let bar_h = 44.0 * bar_progress;
            let (tx, ty) = cam.to_screen(0.0, 0.0);
            draw_rectangle(
                tx,
                ty,
                cam.scaled(1280.0),
                cam.scaled(bar_h),
                Color::new(0.0, 0.0, 0.0, 0.92),
            );
            let (bx, by) = cam.to_screen(0.0, 400.0 - bar_h);
            draw_rectangle(
                bx,
                by,
                cam.scaled(1280.0),
                cam.scaled(bar_h),
                Color::new(0.0, 0.0, 0.0, 0.92),
            );
        }
    }

    // After the slam, the boss stays put at its central position so we
    // keep drawing it underneath the dialog box.
    if t >= T_SLAM_END {
        let boss_x = BOSS_X - BOSS_SIZE * 0.5;
        let boss_y = BOSS_Y_BASE - BOSS_SIZE * 0.5;
        // Small idle bob
        let bob = (t * 2.5).sin() * 3.0;
        let (sbx, sby) = cam.to_screen(boss_x, boss_y + bob);
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

    // ---------- PHASES 4/5/6: DIALOG ----------
    // Cave-Story-style bottom text box. Uses the phase state machine
    // directly so the typewriter elapsed matches exactly what
    // `BossIntroState::typed_chars` is tracking (which is what the
    // input handler checks against for "advance on tap").
    if matches!(
        phase,
        BossIntroPhase::Dialog1 | BossIntroPhase::Dialog2 | BossIntroPhase::Dialog3
    ) {
        if let Some((speaker, line)) = phase.dialog_line() {
            draw_boss_dialog(
                speaker,
                line,
                intro.elapsed,
                intro.dialog_done_typing(),
                assets,
                cam,
            );
        }
    }

    // ---------- PHASE 6: EDIE CHARGE + DASH ----------
    // The Charge phase has EDIE crouch at PLAYER_X while 6 energy rings
    // converge inward; the Dash phase (last 0.55s before impact) is
    // her streaking across the screen as a rainbow rocket.
    if matches!(phase, BossIntroPhase::Charge) {
        let charge_t = (intro.elapsed / 1.4).clamp(0.0, 1.0);
        let ex_start = 200.0;
        let ey = 252.0;
        // 6 converging rings -- each at a different phase so the eye
        // always sees one ring approaching her body.
        for k in 0..6i32 {
            let ring_seed = (charge_t + k as f32 * 0.18).fract();
            let r = 200.0 * (1.0 - ring_seed);
            if r > 10.0 {
                let (rx, ry) = cam.to_screen(ex_start + 28.0, ey + 24.0);
                draw_circle_lines(
                    rx,
                    ry,
                    cam.scaled(r),
                    4.0 * cam.scale,
                    Color::new(0.95, 0.8, 0.25, 0.7 * ring_seed),
                );
            }
        }
        // Rising aura glow pulsing brighter as charge fills.
        let glow_r = 40.0 + charge_t * 40.0;
        let (rx, ry) = cam.to_screen(ex_start + 28.0, ey + 24.0);
        draw_circle(
            rx,
            ry,
            cam.scaled(glow_r),
            Color::new(1.0, 0.95, 0.4, 0.15 + charge_t * 0.25),
        );
        // Crouching wind-up: EDIE scrunches down slightly (smaller box).
        let crouch = charge_t * 0.2;
        let draw_h = 48.0 * (1.0 - crouch);
        let draw_w = 56.0 * (1.0 + crouch * 0.15);
        let (sx, sy) = cam.to_screen(ex_start, ey + 48.0 - draw_h);
        draw_texture_ex(
            &assets.edie_static_run,
            sx,
            sy,
            Color::new(
                1.0 + charge_t * 0.0,
                1.0,
                1.0 - charge_t * 0.3,
                1.0,
            ),
            DrawTextureParams {
                dest_size: Some(vec2(cam.scaled(draw_w), cam.scaled(draw_h))),
                ..Default::default()
            },
        );
        // "CHARGE!" text above her, pulsing bigger as charge fills.
        let txt = "CHARGE!";
        let size = (22.0 + charge_t * 18.0) * cam.scale;
        let dim = measure_text(txt, None, size as u16, 1.0);
        let (tx2, ty2) = cam.to_screen(ex_start + 28.0, ey - 28.0);
        draw_text(
            txt,
            tx2 - dim.width * 0.5 + 2.0,
            ty2 + 2.0,
            size,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );
        draw_text(
            txt,
            tx2 - dim.width * 0.5,
            ty2,
            size,
            Color::new(1.0, 0.9, 0.3, 1.0),
        );
    }

    if matches!(phase, BossIntroPhase::Dash) {
        // Cleaner dash: 3 white afterimages + horizontal speed lines.
        // No rainbow -- it read as garish in playtest.
        let dash_t = (intro.elapsed / 0.55).clamp(0.0, 1.0);
        let start_x = 200.0;
        let end_x = 560.0;
        let ex = start_x + (end_x - start_x) * (1.0 - (1.0 - dash_t).powi(2));
        let ey = 252.0;
        for k in 0..3i32 {
            let trail_x = ex - (k as f32 + 1.0) * 28.0;
            let (tx, ty) = cam.to_screen(trail_x, ey);
            let a = 0.55 - k as f32 * 0.15;
            draw_rectangle(
                tx,
                ty,
                cam.scaled(48.0),
                cam.scaled(40.0),
                Color::new(1.0, 1.0, 1.0, a.max(0.0)),
            );
        }
        // Horizontal motion lines scrolling past.
        for k in 0..6i32 {
            let ly = 110.0 + (k as f32) * 32.0;
            let (lsx, lsy) = cam.to_screen(0.0, ly);
            draw_rectangle(
                lsx,
                lsy,
                cam.scaled(1280.0),
                cam.scaled(1.5),
                Color::new(1.0, 1.0, 1.0, 0.20),
            );
        }
        // EDIE sprite on top
        let (sx, sy) = cam.to_screen(ex, ey);
        draw_texture_ex(
            &assets.edie_static_run,
            sx,
            sy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(cam.scaled(56.0), cam.scaled(48.0))),
                ..Default::default()
            },
        );
    }

    // ---------- PHASE 7: IMPACT ----------
    // Triggered on the Impact phase of the state machine. Shows a big
    // white flash, a stack of POW/BAM/SMASH text rotated at different
    // angles, a ring of debris particles, and a recoiling boss.
    if matches!(phase, BossIntroPhase::Impact) {
        let impact_t = (intro.elapsed / 1.0).clamp(0.0, 1.0);
        let flash_a = (1.0 - impact_t).powi(2) * 0.95;
        draw_rectangle(
            0.0,
            0.0,
            cam.screen_w,
            cam.screen_h,
            Color::new(1.0, 1.0, 0.95, flash_a),
        );
        // Radial debris ring expanding from the impact point.
        for k in 0..24i32 {
            let ang = (k as f32) * std::f32::consts::TAU / 24.0;
            let dist = impact_t * 320.0;
            let dx = BOSS_X - 40.0 + ang.cos() * dist;
            let dy = BOSS_Y_BASE + ang.sin() * dist * 0.8;
            let (dsx, dsy) = cam.to_screen(dx, dy);
            let size = (6.0 - impact_t * 2.0).max(2.0);
            draw_rectangle(
                dsx,
                dsy,
                cam.scaled(size),
                cam.scaled(size),
                Color::new(1.0, 0.95 - impact_t * 0.3, 0.4, (1.0 - impact_t) * 0.95),
            );
        }
        // Single big "SMASH!" impact text that pops in then settles.
        // The stacked 3-word version read as noisy in playtest.
        let pop = (1.0 - (impact_t * 2.2).min(1.0)).max(0.0);
        let size = (82.0 + pop * 48.0) * cam.scale;
        let txt = "SMASH!";
        let dim = measure_text(txt, None, size as u16, 1.0);
        let (tx, ty) = cam.to_screen(640.0, 190.0);
        for (ox, oy) in [(-4.0, 0.0), (4.0, 0.0), (0.0, -4.0), (0.0, 4.0)] {
            draw_text(
                txt,
                tx - dim.width * 0.5 + ox,
                ty + oy,
                size,
                Color::new(0.0, 0.0, 0.0, 0.9),
            );
        }
        draw_text(
            txt,
            tx - dim.width * 0.5,
            ty,
            size,
            Color::new(1.0, 0.85, 0.25, 1.0),
        );
    }

    // ---------- PHASE 8: FIGHT BANNER ----------
    // "FIGHT!" text zooms in from huge to normal size, accompanied by
    // a diagonal shine sweep so the banner feels punchy.
    if matches!(phase, BossIntroPhase::Fight) {
        let fight_t = (intro.elapsed / 1.2).clamp(0.0, 1.0);
        // Dim red vignette fading out.
        draw_rectangle(
            0.0,
            0.0,
            cam.screen_w,
            cam.screen_h,
            Color::new(0.04, 0.0, 0.0, 0.22 * (1.0 - fight_t)),
        );
        // Diagonal white shine sweep crossing the banner.
        let sweep_x = -200.0 + fight_t * 1700.0;
        let (ssx, ssy) = cam.to_screen(sweep_x, 160.0);
        draw_rectangle(
            ssx,
            ssy,
            cam.scaled(80.0),
            cam.scaled(110.0),
            Color::new(1.0, 1.0, 0.95, 0.35 * (1.0 - fight_t)),
        );
        // Huge "FIGHT!" zoom-in. Scale from 160% down to 100% over 0.4s
        // then hold.
        let z = if fight_t < 0.4 {
            1.6 - fight_t * 1.5
        } else {
            1.0
        };
        let txt = "FIGHT!";
        let size = (68.0 * z) * cam.scale;
        let dim = measure_text(txt, None, size as u16, 1.0);
        let (tx, ty) = cam.to_screen(640.0, 220.0);
        for (ox, oy) in [(-4.0, 0.0), (4.0, 0.0), (0.0, -4.0), (0.0, 4.0)] {
            draw_text(
                txt,
                tx - dim.width * 0.5 + ox,
                ty + oy,
                size,
                Color::new(0.0, 0.0, 0.0, 0.85),
            );
        }
        draw_text(
            txt,
            tx - dim.width * 0.5,
            ty,
            size,
            Color::new(1.0, 0.9, 0.25, 1.0),
        );
    }
}

/// Cave-Story-style dialog box used by the boss break-in cinematic.
/// Receives the speaker name so EDIE and MUNGCHI lines can use
/// different portraits + name colours. When `done_typing` is true an
/// "▼ TAP" advance caret pulses in the bottom-right to prompt input.
fn draw_boss_dialog(
    speaker: &str,
    line: &str,
    line_elapsed: f32,
    done_typing: bool,
    assets: &AssetHandles,
    cam: &Camera,
) {
    let is_edie = speaker == "EDIE";
    // Box geometry (logical coords).
    let box_x = 60.0;
    let box_y = 260.0;
    let box_w = 1160.0;
    let box_h = 120.0;
    let (bx, by) = cam.to_screen(box_x, box_y);
    // Fill + double border (thick outer + inner accent).
    draw_rectangle(
        bx,
        by,
        cam.scaled(box_w),
        cam.scaled(box_h),
        Color::new(0.04, 0.04, 0.08, 0.94),
    );
    draw_rectangle_lines(
        bx,
        by,
        cam.scaled(box_w),
        cam.scaled(box_h),
        4.0,
        Color::new(1.0, 0.95, 0.7, 0.95),
    );
    let accent_color = if is_edie {
        Color::new(1.0, 0.85, 0.25, 0.8)
    } else {
        Color::new(0.35, 0.95, 0.40, 0.75)
    };
    draw_rectangle_lines(
        bx + cam.scaled(4.0),
        by + cam.scaled(4.0),
        cam.scaled(box_w - 8.0),
        cam.scaled(box_h - 8.0),
        2.0,
        accent_color,
    );

    // Portrait frame (left side).
    let port_x = box_x + 20.0;
    let port_y = box_y + 18.0;
    let port_size = 84.0;
    let (px, py) = cam.to_screen(port_x, port_y);
    let port_bg = if is_edie {
        Color::new(0.22, 0.16, 0.05, 1.0)
    } else {
        Color::new(0.10, 0.20, 0.12, 1.0)
    };
    draw_rectangle(
        px,
        py,
        cam.scaled(port_size),
        cam.scaled(port_size),
        port_bg,
    );
    draw_rectangle_lines(
        px,
        py,
        cam.scaled(port_size),
        cam.scaled(port_size),
        3.0,
        accent_color,
    );
    let bob = (line_elapsed * 4.0).sin() * 2.0;
    if is_edie {
        // EDIE portrait -- her running-sprite head.
        draw_texture_ex(
            &assets.edie_static_run,
            px + cam.scaled(4.0),
            py + cam.scaled(4.0 + bob),
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    cam.scaled(port_size - 8.0),
                    cam.scaled(port_size - 8.0),
                )),
                ..Default::default()
            },
        );
    } else {
        // MUNGCHI portrait -- the boss head.
        draw_texture_ex(
            &assets.boss_virus,
            px + cam.scaled(4.0),
            py + cam.scaled(4.0 + bob),
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    cam.scaled(port_size - 8.0),
                    cam.scaled(port_size - 8.0),
                )),
                ..Default::default()
            },
        );
    }

    // Speaker name label above the portrait.
    let name_size = 18.0 * cam.scale;
    let (nx, ny) = cam.to_screen(port_x + port_size * 0.5, port_y - 6.0);
    let ndim = measure_text(speaker, None, name_size as u16, 1.0);
    let name_color = if is_edie {
        Color::new(1.0, 0.85, 0.25, 1.0)
    } else {
        Color::new(0.3, 1.0, 0.45, 1.0)
    };
    draw_text(
        speaker,
        nx - ndim.width * 0.5 + 2.0,
        ny + 2.0,
        name_size,
        Color::new(0.0, 0.0, 0.0, 0.85),
    );
    draw_text(
        speaker,
        nx - ndim.width * 0.5,
        ny,
        name_size,
        name_color,
    );

    // Typewriter effect: reveal 24 characters per second.
    let total_chars = line.len();
    let shown = ((line_elapsed * 24.0) as usize).min(total_chars);
    let visible = &line[..shown];

    let text_x = box_x + 130.0;
    let text_y = box_y + 60.0;
    let size = 30.0 * cam.scale;
    let (tx, ty) = cam.to_screen(text_x, text_y);
    draw_text(
        visible,
        tx + 3.0,
        ty + 3.0,
        size,
        Color::new(0.0, 0.0, 0.0, 0.6),
    );
    draw_text(
        visible,
        tx,
        ty,
        size,
        Color::new(0.95, 0.95, 0.8, 1.0),
    );

    // While still typing, blink a caret right after the last char.
    if shown < total_chars {
        let dim = measure_text(visible, None, size as u16, 1.0);
        if ((line_elapsed * 4.0) as usize) % 2 == 0 {
            draw_rectangle(
                tx + dim.width + 4.0,
                ty - dim.height * 0.8,
                cam.scaled(10.0),
                cam.scaled(5.0),
                Color::new(0.95, 0.95, 0.8, 0.9),
            );
        }
    }

    // When the line is fully typed, show a pulsing "▼ TAP" advance
    // indicator bottom-right of the box so players know the cinematic
    // is waiting on them.
    if done_typing {
        let pulse = 0.6 + 0.4 * (line_elapsed * 6.0).sin().abs();
        let hint = "TAP TO CONTINUE  >";
        let hint_size = 18.0 * cam.scale;
        let hdim = measure_text(hint, None, hint_size as u16, 1.0);
        let (hx, hy) = cam.to_screen(box_x + box_w - 24.0, box_y + box_h - 16.0);
        draw_text(
            hint,
            hx - hdim.width + 2.0,
            hy + 2.0,
            hint_size,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );
        draw_text(
            hint,
            hx - hdim.width,
            hy,
            hint_size,
            Color::new(1.0, 0.95, 0.6, pulse),
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
    // Boss: giant Mungchi virus, central, slight bob
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
    // Safe-lane burst telegraph (during SafeLaneBurst pattern)
    //
    // Paint the danger zones in pulsing red and the safe corridor in green
    // so the player can read exactly where to stand during the warn window.
    // ======================================================
    if let Some(lane) = &boss.safe_lane {
        let is_warn = lane.warn_remaining > 0.0;
        let top = 200.0;
        let bottom = 400.0;
        let height = bottom - top;
        let pulse = if is_warn {
            ((boss.elapsed * 18.0).sin() * 0.5 + 0.5) * 0.35 + 0.35
        } else {
            0.25
        };
        // Left danger band
        if lane.min_x > 0.0 {
            let (lx, ly) = cam.to_screen(0.0, top);
            draw_rectangle(
                lx,
                ly,
                cam.scaled(lane.min_x),
                cam.scaled(height),
                Color::new(1.0, 0.15, 0.2, pulse),
            );
        }
        // Right danger band
        if lane.max_x < 1280.0 {
            let (rx, ry) = cam.to_screen(lane.max_x, top);
            draw_rectangle(
                rx,
                ry,
                cam.scaled(1280.0 - lane.max_x),
                cam.scaled(height),
                Color::new(1.0, 0.15, 0.2, pulse),
            );
        }
        // Safe lane highlight (green)
        let (sx, sy) = cam.to_screen(lane.min_x, top);
        draw_rectangle(
            sx,
            sy,
            cam.scaled(lane.max_x - lane.min_x),
            cam.scaled(height),
            Color::new(0.2, 0.95, 0.35, 0.18 + pulse * 0.1),
        );
        // Safe-lane borders
        draw_rectangle(
            sx,
            sy,
            cam.scaled(4.0),
            cam.scaled(height),
            Color::new(0.35, 1.0, 0.45, 0.9),
        );
        let (erx, ery) = cam.to_screen(lane.max_x - 4.0, top);
        draw_rectangle(
            erx,
            ery,
            cam.scaled(4.0),
            cam.scaled(height),
            Color::new(0.35, 1.0, 0.45, 0.9),
        );
        if is_warn {
            // "STEP INTO SAFE LANE" text floating above the lane
            let txt = "SAFE ZONE";
            let size = 26.0 * cam.scale;
            let dim = measure_text(txt, None, size as u16, 1.0);
            let (tx, ty) = cam.to_screen((lane.min_x + lane.max_x) * 0.5, 240.0);
            draw_text(
                txt,
                tx - dim.width * 0.5,
                ty,
                size,
                Color::new(0.95, 1.0, 0.85, 1.0),
            );
        }
    }

    // ======================================================
    // Phase 2 pattern-specific telegraphs
    // ======================================================
    // Pincer Grid gap column highlight
    if let Some(wave) = &boss.pincer_wave {
        if wave.warn_remaining > 0.0 {
            let step = 1280.0 / (wave.cols as f32);
            // Red danger columns
            let pulse = ((boss.elapsed * 18.0).sin() * 0.5 + 0.5) * 0.4 + 0.3;
            for i in 0..wave.cols {
                if i == wave.gap_col {
                    continue;
                }
                let cx = step * (i as f32 + 0.5);
                let (sxc, syc) = cam.to_screen(cx - step * 0.4, 150.0);
                draw_rectangle(
                    sxc,
                    syc,
                    cam.scaled(step * 0.8),
                    cam.scaled(220.0),
                    Color::new(1.0, 0.25, 0.3, pulse * 0.35),
                );
            }
            // Green safe column
            let gx = step * (wave.gap_col as f32 + 0.5);
            let (sgx, sgy) = cam.to_screen(gx - step * 0.4, 150.0);
            draw_rectangle(
                sgx,
                sgy,
                cam.scaled(step * 0.8),
                cam.scaled(220.0),
                Color::new(0.3, 1.0, 0.4, 0.32),
            );
        }
    }

    // Hunter Bolts crosshairs
    for shot in &boss.hunter_shots {
        if !shot.fired && shot.warn_remaining > 0.0 {
            let pulse = ((boss.elapsed * 24.0).sin() * 0.5 + 0.5) * 0.6 + 0.4;
            let (hx, hy) = cam.to_screen(shot.target_x, 150.0);
            // Vertical warning line
            draw_rectangle(
                hx - cam.scaled(2.0),
                hy,
                cam.scaled(4.0),
                cam.scaled(250.0),
                Color::new(1.0, 0.25, 0.3, pulse),
            );
            // Crosshair on player line
            let (cxh, cyh) = cam.to_screen(shot.target_x, 340.0);
            draw_rectangle(
                cxh - cam.scaled(14.0),
                cyh - cam.scaled(2.0),
                cam.scaled(28.0),
                cam.scaled(4.0),
                Color::new(1.0, 0.25, 0.3, pulse),
            );
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
        crate::game::boss::BossPattern::Rain => "MUNGCHI RAIN",
        crate::game::boss::BossPattern::DiagonalVolley => "DIAGONAL VOLLEY",
        crate::game::boss::BossPattern::Spiral => "SPIRAL STORM",
        crate::game::boss::BossPattern::SafeLaneBurst => "SAFE LANE BURST",
        crate::game::boss::BossPattern::Crossfire => "HORIZONTAL CROSSFIRE",
        crate::game::boss::BossPattern::PincerGrid => "PINCER GRID",
        crate::game::boss::BossPattern::HunterBolts => "HUNTER BOLTS",
        crate::game::boss::BossPattern::RingPulse => "RING PULSE",
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

/// Stage transition: subtle full-screen dim, no obstructive bars, with the
/// new stage name floating in center. Keeps gameplay visible the whole
/// time so obstacles never "pop in" out of nowhere.
pub fn draw_stage_wipe(effects: &crate::game::effects::Effects, cam: &Camera) {
    let wipe = match &effects.stage_wipe {
        Some(w) => w,
        None => return,
    };
    let t = 1.0 - (wipe.remaining / wipe.total); // 0..1

    // Ease in-and-out curve for the dim overlay (peaks around t=0.5).
    let dim_curve = (t * std::f32::consts::PI).sin().clamp(0.0, 1.0);
    let dim = 0.35 * dim_curve;
    let (x0, y0) = cam.to_screen(0.0, 0.0);
    draw_rectangle(
        x0,
        y0,
        cam.scaled(1280.0),
        cam.scaled(400.0),
        Color::new(0.02, 0.02, 0.04, dim),
    );

    // Accent sweep: a thin bright horizontal line crossing the screen once.
    let sweep_t = (t * 1.2).clamp(0.0, 1.0);
    let sweep_x = -120.0 + sweep_t * (1280.0 + 240.0);
    let (ssx, ssy) = cam.to_screen(sweep_x, 0.0);
    draw_rectangle(
        ssx,
        ssy,
        cam.scaled(4.0),
        cam.scaled(400.0),
        Color::new(1.0, 0.9, 0.3, 0.45 * dim_curve),
    );

    // Text fades in then out, matching the dim curve.
    let fade = dim_curve;
    if fade > 0.02 {
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
        let size = 44.0 * cam.scale;
        let dim_t = measure_text(&wipe.new_stage_name, None, size as u16, 1.0);
        let (tx, ty) = cam.to_screen(640.0, 210.0);
        draw_text(
            &wipe.new_stage_name,
            tx - dim_t.width * 0.5 + 3.0,
            ty + 3.0,
            size,
            Color::new(0.0, 0.0, 0.0, fade * 0.6),
        );
        draw_text(
            &wipe.new_stage_name,
            tx - dim_t.width * 0.5,
            ty,
            size,
            Color::new(1.0, 1.0, 1.0, fade),
        );
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
    pub key_hint: &'static str,
    pub logical_rect: (f32, f32, f32, f32), // x, y, w, h
}

pub fn boss_touch_buttons() -> [TouchButton; 2] {
    let by = 324.0;
    let bh = 66.0;
    [
        TouchButton { label: "<", key_hint: "A / LEFT", logical_rect: (24.0, by, 120.0, bh) },
        TouchButton { label: ">", key_hint: "D / RIGHT", logical_rect: (150.0, by, 120.0, bh) },
    ]
}

pub fn play_touch_buttons() -> [TouchButton; 3] {
    let by = 324.0;
    let bh = 66.0;
    [
        TouchButton { label: "DUCK", key_hint: "DOWN", logical_rect: (24.0, by, 130.0, bh) },
        TouchButton { label: "JUMP", key_hint: "SPACE", logical_rect: (1280.0 - 24.0 - 130.0, by, 130.0, bh) },
        TouchButton { label: "DASH", key_hint: "SHIFT", logical_rect: (1280.0 - 24.0 - 130.0 - 150.0, by, 130.0, bh) },
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
        // Main label
        let size = 28.0 * cam.scale;
        let dim = measure_text(b.label, None, size as u16, 1.0);
        let tx = sx + cam.scaled(lw) * 0.5 - dim.width * 0.5;
        let ty = sy + cam.scaled(lh) * 0.5 + dim.height * 0.1;
        draw_text(b.label, tx, ty, size, Color::new(0.98, 0.95, 0.85, 1.0));
        // Tiny key hint under the label
        let hint_size = 13.0 * cam.scale;
        let hint_dim = measure_text(b.key_hint, None, hint_size as u16, 1.0);
        let htx = sx + cam.scaled(lw) * 0.5 - hint_dim.width * 0.5;
        let hty = sy + cam.scaled(lh) - cam.scaled(8.0);
        draw_text(
            b.key_hint,
            htx,
            hty,
            hint_size,
            Color::new(0.95, 0.85, 0.4, 0.85),
        );
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
