//! Greybox sprite drawing — solid colored rectangles. Phase 2 swaps these
//! bodies for textured draws while keeping the same function signatures.

use crate::game::obstacles::{Obstacle, ObstacleKind};
use crate::game::pickups::{AuroraColor, AuroraStone};
use crate::game::player::{Player, PlayerState, PLAYER_H, PLAYER_W, PLAYER_X};
use crate::render::camera::Camera;
use macroquad::prelude::*;

pub fn draw_player(player: &Player, cam: &Camera) {
    let color = match player.state {
        PlayerState::Hit => RED,
        PlayerState::Ducking => Color::new(0.95, 0.55, 0.20, 1.0),
        _ => Color::new(0.24, 0.24, 0.24, 1.0),
    };
    let h = if matches!(player.state, PlayerState::Ducking) {
        PLAYER_H * 0.55
    } else {
        PLAYER_H
    };
    let y = if matches!(player.state, PlayerState::Ducking) {
        player.y + PLAYER_H * 0.45
    } else {
        player.y
    };
    let (sx, sy) = cam.to_screen(PLAYER_X, y);
    draw_rectangle(sx, sy, cam.scaled(PLAYER_W), cam.scaled(h), color);
}

pub fn draw_obstacle(o: &Obstacle, cam: &Camera) {
    let color = match o.kind {
        ObstacleKind::CoiledCable => Color::new(0.30, 0.30, 0.30, 1.0),
        ObstacleKind::ChargingDock => Color::new(0.55, 0.20, 0.20, 1.0),
        ObstacleKind::ToolCart => Color::new(0.40, 0.30, 0.20, 1.0),
        ObstacleKind::SensorCone => Color::new(0.91, 0.57, 0.24, 1.0),
        ObstacleKind::QuadDrone => Color::new(0.20, 0.30, 0.50, 1.0),
        ObstacleKind::SparkBurst => Color::new(0.95, 0.85, 0.30, 1.0),
    };
    let (w, h) = o.kind.size();
    let (sx, sy) = cam.to_screen(o.x, o.y);
    draw_rectangle(sx, sy, cam.scaled(w), cam.scaled(h), color);
}

pub fn draw_aurora(s: &AuroraStone, cam: &Camera) {
    let color = match s.color {
        AuroraColor::Purple => Color::new(0.62, 0.42, 1.00, 1.0),
        AuroraColor::Green => Color::new(0.36, 0.89, 0.66, 1.0),
    };
    let (sx, sy) = cam.to_screen(s.x, s.y);
    draw_rectangle(
        sx,
        sy,
        cam.scaled(crate::game::pickups::PICKUP_W),
        cam.scaled(crate::game::pickups::PICKUP_H),
        color,
    );
    let halo_inset = 4.0;
    draw_rectangle(
        sx - cam.scaled(halo_inset),
        sy - cam.scaled(halo_inset),
        cam.scaled(crate::game::pickups::PICKUP_W + 2.0 * halo_inset),
        cam.scaled(crate::game::pickups::PICKUP_H + 2.0 * halo_inset),
        Color::new(color.r, color.g, color.b, 0.25),
    );
}
