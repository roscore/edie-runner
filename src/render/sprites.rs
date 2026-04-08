//! Phase 2 sprite drawing — textured.

use crate::assets::AssetHandles;
use crate::game::obstacles::{Obstacle, ObstacleKind};
use crate::game::pickups::{AuroraColor, AuroraStone};
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

pub fn draw_player(player: &Player, assets: &AssetHandles, elapsed: f32, cam: &Camera) {
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

    let tex = match player.state {
        PlayerState::Hit => &assets.edie_hit,
        PlayerState::Ducking => &assets.edie_duck,
        PlayerState::Jumping | PlayerState::Falling => &assets.edie_jump,
        _ => &assets.edie_run,
    };

    let tex_w = tex.width();
    let tex_h = tex.height();

    // Bottom-center the texture inside the physics box.
    let mut logical_x = PLAYER_X + (PLAYER_W - tex_w) * 0.5;
    let mut logical_y = player.y + PLAYER_H - tex_h;

    // Tiny vertical bob in Running state.
    if matches!(player.state, PlayerState::Running) {
        let bob = ((elapsed * 8.0).sin() * 1.0).round();
        logical_y += bob;
    }

    let _ = &mut logical_x;
    draw_tex_at(tex, logical_x, logical_y, tex_w, tex_h, cam, WHITE);
}

pub fn draw_obstacle(o: &Obstacle, assets: &AssetHandles, elapsed: f32, cam: &Camera) {
    let (w, h) = o.kind.size();
    match o.kind {
        ObstacleKind::CoiledCable => {
            draw_tex_at(&assets.obstacle_cable, o.x, o.y, w, h, cam, WHITE);
        }
        ObstacleKind::ChargingDock => {
            let f = frame_index(elapsed, DOCK_FPS, DOCK_FRAMES);
            draw_tex_frame(
                &assets.obstacle_dock, f, DOCK_FRAME_W, DOCK_FRAME_H, 1.0, o.x, o.y, cam, WHITE,
            );
        }
        ObstacleKind::ToolCart => {
            draw_tex_at(&assets.obstacle_cart, o.x, o.y, w, h, cam, WHITE);
        }
        ObstacleKind::SensorCone => {
            draw_tex_at(&assets.obstacle_cone, o.x, o.y, w, h, cam, WHITE);
        }
        ObstacleKind::QuadDrone => {
            let f = frame_index(elapsed, DRONE_FPS, DRONE_FRAMES);
            draw_tex_frame(
                &assets.obstacle_drone, f, DRONE_FRAME_W, DRONE_FRAME_H, 1.0, o.x, o.y, cam, WHITE,
            );
        }
        ObstacleKind::SparkBurst => {
            let f = frame_index(elapsed, SPARK_FPS, SPARK_FRAMES);
            draw_tex_frame(
                &assets.obstacle_spark, f, SPARK_FRAME_W, SPARK_FRAME_H, 1.0, o.x, o.y, cam, WHITE,
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
