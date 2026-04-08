//! Aurora Stones: collectible pickups that fill the dash energy meter.
//! See spec §3.5.

use crate::game::player::{Aabb, GROUND_Y};
use rand::rngs::SmallRng;
use rand::Rng;

pub const MAX_AURORA: u32 = 3;
pub const PICKUP_W: f32 = 48.0;
pub const PICKUP_H: f32 = 48.0;
const SPAWN_X: f32 = 1400.0;
const SPAWN_INTERVAL_MIN: f32 = 8.0;
const SPAWN_INTERVAL_MAX: f32 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuroraColor {
    Purple,
    Green,
}

#[derive(Debug, Clone)]
pub struct AuroraStone {
    pub x: f32,
    pub y: f32,
    pub color: AuroraColor,
    pub collected: bool,
}

impl AuroraStone {
    pub fn hitbox(&self) -> Aabb {
        Aabb { x: self.x, y: self.y, w: PICKUP_W, h: PICKUP_H }
    }
}

pub struct PickupField {
    pub stones: Vec<AuroraStone>,
    pub time_to_next: f32,
}

impl Default for PickupField {
    fn default() -> Self {
        Self { stones: Vec::new(), time_to_next: SPAWN_INTERVAL_MIN }
    }
}

impl PickupField {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, dt: f32, speed: f32, rng: &mut SmallRng) {
        let dx = speed * dt;
        for s in &mut self.stones {
            s.x -= dx;
        }
        self.stones.retain(|s| !s.collected && s.x + PICKUP_W > -50.0);

        self.time_to_next -= dt;
        if self.time_to_next <= 0.0 {
            let tier = rng.gen_range(0..3u32);
            let y = match tier {
                0 => GROUND_Y - PICKUP_H - 8.0,
                1 => GROUND_Y - 110.0,
                _ => GROUND_Y - 160.0,
            };
            let color = if rng.gen_bool(0.5) {
                AuroraColor::Purple
            } else {
                AuroraColor::Green
            };
            self.stones.push(AuroraStone { x: SPAWN_X, y, color, collected: false });
            self.time_to_next = rng.gen_range(SPAWN_INTERVAL_MIN..SPAWN_INTERVAL_MAX);
        }
    }

    pub fn collisions_with(&self, player: &Aabb) -> Vec<usize> {
        self.stones
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.collected && s.hitbox().intersects(player))
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn spawns_within_interval_window() {
        let mut field = PickupField::new();
        let mut rng = SmallRng::seed_from_u64(1);
        let dt = crate::time::DT;
        let steps = (30.0 / dt) as u32;
        let mut spawn_count = 0u32;
        let mut last_len = 0;
        for _ in 0..steps {
            field.update(dt, 320.0, &mut rng);
            if field.stones.len() > last_len {
                spawn_count += 1;
            }
            last_len = field.stones.len();
        }
        assert!(spawn_count >= 2 && spawn_count <= 5, "got {spawn_count} spawns");
    }

    #[test]
    fn collected_stones_pruned() {
        let mut field = PickupField::new();
        field.stones.push(AuroraStone {
            x: 100.0,
            y: 100.0,
            color: AuroraColor::Purple,
            collected: true,
        });
        let mut rng = SmallRng::seed_from_u64(0);
        field.update(0.001, 100.0, &mut rng);
        assert!(field.stones.iter().all(|s| !s.collected));
    }

    #[test]
    fn collisions_with_overlapping_player() {
        let mut field = PickupField::new();
        field.stones.push(AuroraStone {
            x: 50.0,
            y: 50.0,
            color: AuroraColor::Green,
            collected: false,
        });
        let player = Aabb { x: 60.0, y: 60.0, w: 10.0, h: 10.0 };
        assert_eq!(field.collisions_with(&player), vec![0]);
    }
}
