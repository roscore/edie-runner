//! Obstacles: types, spawning, collision shapes. See spec §3.4.

use crate::game::difficulty::{tier_for_score, SPARK_BURST_UNLOCK_TIER};
use crate::game::player::{Aabb, GROUND_Y};
use rand::rngs::SmallRng;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleKind {
    // Pangyo street obstacles (low tiers)
    CoffeeCup,
    ShoppingCart,
    TrafficCone,
    SignBoard,
    Cat,
    // AeiROBOT robots — appear as we approach AeiROBOT HQ
    VacuumBot, // generic intro robot
    Amy,       // small flying robot (replaces drone role)
    AliceM1,   // mobile ground robot
    Alice3,    // humanoid v3 (heavy)
    Alice4,    // humanoid v4 (heavy)
}

impl ObstacleKind {
    pub fn destroyable_by_dash(&self) -> bool {
        // EDIE's Aurora Dash smashes everything in its path.
        true
    }

    /// True if this obstacle is a robot. Used for the "approaching AeiROBOT"
    /// scaling — higher tiers spawn more robots.
    pub fn is_robot(&self) -> bool {
        matches!(
            self,
            ObstacleKind::VacuumBot
                | ObstacleKind::Amy
                | ObstacleKind::AliceM1
                | ObstacleKind::Alice3
                | ObstacleKind::Alice4
        )
    }

    pub fn size(&self) -> (f32, f32) {
        match self {
            ObstacleKind::CoffeeCup => (24.0, 32.0),
            ObstacleKind::ShoppingCart => (80.0, 44.0),
            ObstacleKind::TrafficCone => (24.0, 32.0),
            ObstacleKind::SignBoard => (24.0, 24.0),
            ObstacleKind::Cat => (40.0, 24.0),
            ObstacleKind::VacuumBot => (40.0, 20.0),
            ObstacleKind::Amy => (44.0, 32.0),
            ObstacleKind::AliceM1 => (36.0, 36.0),
            ObstacleKind::Alice3 => (32.0, 64.0),
            ObstacleKind::Alice4 => (36.0, 68.0),
        }
    }

    pub fn y_for_kind(&self) -> f32 {
        let (_, h) = self.size();
        match self {
            // Amy hovers so the player MUST duck.
            ObstacleKind::Amy => GROUND_Y - 56.0,
            ObstacleKind::SignBoard => GROUND_Y - 160.0,
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

    /// Spawn weight table by tier. Higher tiers shift toward AeiROBOT bots
    /// to convey "approaching AeiROBOT HQ".
    fn random_kind(&self, score: u32, rng: &mut SmallRng) -> ObstacleKind {
        let tier = tier_for_score(score);
        let mut pool: Vec<ObstacleKind> = Vec::new();

        // Tiers 0-1: Pangyo street level (no robots yet)
        pool.push(ObstacleKind::CoffeeCup);
        pool.push(ObstacleKind::CoffeeCup);
        pool.push(ObstacleKind::Cat);
        pool.push(ObstacleKind::Cat);
        pool.push(ObstacleKind::TrafficCone);
        pool.push(ObstacleKind::ShoppingCart);

        if tier >= 1 {
            pool.push(ObstacleKind::ShoppingCart);
            pool.push(ObstacleKind::TrafficCone);
        }

        // Tier 2: vacuum bots — entering the tech district
        if tier >= 2 {
            pool.push(ObstacleKind::VacuumBot);
            pool.push(ObstacleKind::VacuumBot);
        }

        // Tier 3: Amy (small flying AeiROBOT) + signboards
        if tier >= SPARK_BURST_UNLOCK_TIER {
            pool.push(ObstacleKind::SignBoard);
            pool.push(ObstacleKind::Amy);
        }

        // Tier 4: Alice-M1 (mobile AeiROBOT)
        if tier >= 4 {
            pool.push(ObstacleKind::AliceM1);
            pool.push(ObstacleKind::Amy);
            pool.push(ObstacleKind::VacuumBot);
        }

        // Tier 5: Alice3 (humanoid)
        if tier >= 5 {
            pool.push(ObstacleKind::Alice3);
            pool.push(ObstacleKind::AliceM1);
            pool.push(ObstacleKind::Amy);
        }

        // Tier 6: Alice4 (newer humanoid) — AeiROBOT zone in full effect
        if tier >= 6 {
            pool.push(ObstacleKind::Alice4);
            pool.push(ObstacleKind::Alice3);
            pool.push(ObstacleKind::AliceM1);
        }

        // Tier 7-8: dense AeiROBOT presence
        if tier >= 7 {
            pool.push(ObstacleKind::Alice4);
            pool.push(ObstacleKind::Alice4);
            pool.push(ObstacleKind::Alice3);
            pool.push(ObstacleKind::Amy);
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
        // Aurora Dash smashes ALL obstacles now.
        for kind in [
            ObstacleKind::CoffeeCup,
            ObstacleKind::ShoppingCart,
            ObstacleKind::TrafficCone,
            ObstacleKind::SignBoard,
            ObstacleKind::Cat,
            ObstacleKind::VacuumBot,
            ObstacleKind::Amy,
            ObstacleKind::AliceM1,
            ObstacleKind::Alice3,
            ObstacleKind::Alice4,
        ] {
            assert!(kind.destroyable_by_dash(), "{:?} should be destroyable", kind);
        }
    }

    #[test]
    fn higher_tiers_spawn_more_robots() {
        let field = ObstacleField::new();
        let mut rng = SmallRng::seed_from_u64(99);
        let count_robots = |score: u32, samples: usize, rng: &mut SmallRng| {
            (0..samples)
                .filter(|_| field.random_kind(score, rng).is_robot())
                .count()
        };
        let low = count_robots(0, 1000, &mut rng);
        let high = count_robots(7 * 500, 1000, &mut rng);
        assert!(
            high > low * 2,
            "tier 7 should produce far more robots than tier 0 (low={low}, high={high})"
        );
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
    fn signboard_only_at_tier_3_plus() {
        let mut rng = SmallRng::seed_from_u64(7);
        let field = ObstacleField::new();
        for _ in 0..200 {
            let k = field.random_kind(0, &mut rng);
            assert_ne!(k, ObstacleKind::SignBoard);
        }
        let mut saw = false;
        for _ in 0..2000 {
            if field.random_kind(SPARK_BURST_UNLOCK_TIER * 500, &mut rng)
                == ObstacleKind::SignBoard
            {
                saw = true;
                break;
            }
        }
        assert!(saw);
    }
}
