//! Obstacles: types, spawning, collision shapes. See spec §3.4.

use crate::game::difficulty::{tier_for_score, SPARK_BURST_UNLOCK_TIER};
use crate::game::player::{Aabb, GROUND_Y};
use rand::rngs::SmallRng;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleKind {
    CoiledCable,
    ChargingDock,
    ToolCart,
    SensorCone,
    QuadDrone,
    SparkBurst,
}

impl ObstacleKind {
    pub fn destroyable_by_dash(&self) -> bool {
        !matches!(self, ObstacleKind::ChargingDock)
    }

    pub fn size(&self) -> (f32, f32) {
        match self {
            ObstacleKind::CoiledCable => (32.0, 32.0),
            ObstacleKind::ChargingDock => (40.0, 96.0),
            ObstacleKind::ToolCart => (128.0, 48.0),
            ObstacleKind::SensorCone => (24.0, 32.0),
            ObstacleKind::QuadDrone => (56.0, 32.0),
            ObstacleKind::SparkBurst => (24.0, 24.0),
        }
    }

    pub fn y_for_kind(&self) -> f32 {
        let (_, h) = self.size();
        match self {
            ObstacleKind::QuadDrone => GROUND_Y - 96.0,
            ObstacleKind::SparkBurst => GROUND_Y - 160.0,
            _ => GROUND_Y - h,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Obstacle {
    pub kind: ObstacleKind,
    pub x: f32,
    pub y: f32,
    pub alive: bool,
}

impl Obstacle {
    pub fn new(kind: ObstacleKind, x: f32) -> Self {
        let y = kind.y_for_kind();
        Self { kind, x, y, alive: true }
    }

    pub fn hitbox(&self) -> Aabb {
        let (w, h) = self.kind.size();
        Aabb { x: self.x, y: self.y, w, h }
    }
}

const SPAWN_X: f32 = 1400.0;

pub struct ObstacleField {
    pub obstacles: Vec<Obstacle>,
    pub scrolled_since_spawn: f32,
    pub next_spawn_gap: f32,
}

impl Default for ObstacleField {
    fn default() -> Self {
        Self {
            obstacles: Vec::new(),
            scrolled_since_spawn: 0.0,
            next_spawn_gap: 0.0,
        }
    }
}

impl ObstacleField {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min_gap(speed: f32) -> f32 {
        (speed * 1.0).max(180.0)
    }

    fn random_kind(&self, score: u32, rng: &mut SmallRng) -> ObstacleKind {
        let tier = tier_for_score(score);
        let mut pool: Vec<ObstacleKind> = vec![
            ObstacleKind::CoiledCable,
            ObstacleKind::CoiledCable,
            ObstacleKind::SensorCone,
            ObstacleKind::ToolCart,
        ];
        if tier >= 1 {
            pool.push(ObstacleKind::ChargingDock);
            pool.push(ObstacleKind::QuadDrone);
        }
        if tier >= 2 {
            pool.push(ObstacleKind::QuadDrone);
            pool.push(ObstacleKind::ChargingDock);
        }
        if tier >= SPARK_BURST_UNLOCK_TIER {
            pool.push(ObstacleKind::SparkBurst);
        }
        let idx = rng.gen_range(0..pool.len());
        pool[idx]
    }

    pub fn update(&mut self, dt: f32, speed: f32, score: u32, rng: &mut SmallRng) {
        let dx = speed * dt;
        for o in &mut self.obstacles {
            o.x -= dx;
        }
        self.obstacles.retain(|o| o.alive && o.x + o.kind.size().0 > -50.0);

        self.scrolled_since_spawn += dx;
        if self.scrolled_since_spawn >= self.next_spawn_gap {
            let kind = self.random_kind(score, rng);
            self.obstacles.push(Obstacle::new(kind, SPAWN_X));
            self.scrolled_since_spawn = 0.0;
            let extra = rng.gen_range(0.0..200.0);
            self.next_spawn_gap = Self::min_gap(speed) + extra;
        }
    }

    pub fn first_collision(&self, player: &Aabb) -> Option<usize> {
        self.obstacles
            .iter()
            .position(|o| o.alive && o.hitbox().intersects(player))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::difficulty::speed_for_score;
    use rand::SeedableRng;

    #[test]
    fn destroyable_flags() {
        assert!(!ObstacleKind::ChargingDock.destroyable_by_dash());
        assert!(ObstacleKind::CoiledCable.destroyable_by_dash());
        assert!(ObstacleKind::ToolCart.destroyable_by_dash());
        assert!(ObstacleKind::QuadDrone.destroyable_by_dash());
    }

    #[test]
    fn min_gap_grows_with_speed() {
        assert!(ObstacleField::min_gap(720.0) > ObstacleField::min_gap(320.0));
    }

    #[test]
    fn spawn_respects_min_spacing_at_every_tier() {
        for tier in 0..=8u32 {
            let score = tier * 500;
            let speed = speed_for_score(score);
            let mut field = ObstacleField::new();
            let mut rng = SmallRng::seed_from_u64(42 + tier as u64);
            let steps = (60.0 / crate::time::DT) as u32;
            for _ in 0..steps {
                field.update(crate::time::DT, speed, score, &mut rng);
            }
            let mut xs: Vec<f32> = field.obstacles.iter().map(|o| o.x).collect();
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            for w in xs.windows(2) {
                let gap = w[1] - w[0];
                assert!(
                    gap >= ObstacleField::min_gap(speed) - 1.0,
                    "tier {tier} speed {speed}: gap {gap} < min {}",
                    ObstacleField::min_gap(speed)
                );
            }
        }
    }

    #[test]
    fn spark_burst_only_at_tier_3_plus() {
        let mut rng = SmallRng::seed_from_u64(7);
        let field = ObstacleField::new();
        for _ in 0..200 {
            let k = field.random_kind(0, &mut rng);
            assert_ne!(k, ObstacleKind::SparkBurst);
        }
        let mut saw_spark = false;
        for _ in 0..1000 {
            if field.random_kind(SPARK_BURST_UNLOCK_TIER * 500, &mut rng)
                == ObstacleKind::SparkBurst
            {
                saw_spark = true;
                break;
            }
        }
        assert!(saw_spark);
    }
}
