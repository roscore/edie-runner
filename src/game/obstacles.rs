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
    CatOrange,
    CatWhite,
    // Highway hazards
    Car,    // charging car
    Deer,   // sudden leap
    // Air obstacles forcing duck
    BalloonDrone, // balloon-style drone (duck required)
    // AeiROBOT robots - appear as we approach AeiROBOT HQ
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
    /// scaling - higher tiers spawn more robots.
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
            ObstacleKind::ShoppingCart => (56.0, 36.0),
            ObstacleKind::TrafficCone => (24.0, 32.0),
            ObstacleKind::SignBoard => (24.0, 24.0),
            ObstacleKind::CatOrange => (48.0, 40.0),
            ObstacleKind::CatWhite => (48.0, 40.0),
            ObstacleKind::Car => (96.0, 40.0),
            ObstacleKind::Deer => (48.0, 52.0),
            ObstacleKind::BalloonDrone => (40.0, 48.0),
            ObstacleKind::VacuumBot => (40.0, 20.0),
            ObstacleKind::Amy => (24.0, 60.0),
            ObstacleKind::AliceM1 => (28.0, 64.0),
            ObstacleKind::Alice3 => (25.0, 64.0),
            ObstacleKind::Alice4 => (27.0, 68.0),
        }
    }

    pub fn y_for_kind(&self) -> f32 {
        let (_, h) = self.size();
        match self {
            // Balloon drone: hover so that its BOTTOM sits clearly above the
            // ducked player hitbox. Duck hitbox top is ~296 (GROUND_Y - 24);
            // balloon bottom at GROUND_Y - 88 = 232 -> clean 64 px of duck
            // clearance, plenty forgiving.
            ObstacleKind::BalloonDrone => GROUND_Y - 88.0 - h,
            ObstacleKind::SignBoard => GROUND_Y - 160.0,
            _ => GROUND_Y - h,
        }
    }

    /// True if this obstacle rests on the ground (draw a shadow under it).
    pub fn has_ground_shadow(&self) -> bool {
        !matches!(
            self,
            ObstacleKind::BalloonDrone | ObstacleKind::SignBoard
        )
    }
}

#[derive(Debug, Clone)]
pub struct Obstacle {
    pub kind: ObstacleKind,
    pub x: f32,
    pub y: f32,
    pub alive: bool,
    /// Extra horizontal velocity on top of world scroll (negative = charging
    /// at the player faster than background).
    pub extra_vx: f32,
    /// Vertical velocity. Used by Deer leap and Car dart.
    pub vy: f32,
    /// Seconds the obstacle has existed in the world — drives timed patterns.
    pub age: f32,
    /// Pattern-specific counter (e.g. deer-leap trigger).
    pub pattern_t: f32,
}

impl Obstacle {
    pub fn new(kind: ObstacleKind, x: f32) -> Self {
        let y = kind.y_for_kind();
        Self {
            kind,
            x,
            y,
            alive: true,
            extra_vx: 0.0,
            vy: 0.0,
            age: 0.0,
            pattern_t: 0.0,
        }
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

    /// Spawn pool per stage. Each stage has a distinct vibe that matches
    /// EDIE's journey from the pop-up store to AeiROBOT HQ.
    fn random_kind(&self, score: u32, rng: &mut SmallRng) -> ObstacleKind {
        use crate::game::difficulty::{stage_for_tier, Stage};
        let tier = tier_for_score(score);
        let stage = stage_for_tier(tier);
        let mut pool: Vec<ObstacleKind> = Vec::new();

        match stage {
            Stage::DepartmentStore => {
                pool.push(ObstacleKind::CoffeeCup);
                pool.push(ObstacleKind::CoffeeCup);
                pool.push(ObstacleKind::CatOrange);
                pool.push(ObstacleKind::CatWhite);
                pool.push(ObstacleKind::TrafficCone);
                pool.push(ObstacleKind::ShoppingCart);
                pool.push(ObstacleKind::BalloonDrone);
            }
            Stage::PangyoStreet => {
                pool.push(ObstacleKind::CoffeeCup);
                pool.push(ObstacleKind::CatOrange);
                pool.push(ObstacleKind::CatWhite);
                pool.push(ObstacleKind::TrafficCone);
                pool.push(ObstacleKind::ShoppingCart);
                pool.push(ObstacleKind::BalloonDrone);
                if tier >= 2 {
                    pool.push(ObstacleKind::VacuumBot);
                    pool.push(ObstacleKind::VacuumBot);
                    pool.push(ObstacleKind::BalloonDrone);
                }
            }
            Stage::Highway => {
                // Highway: charging cars, sudden deer leaps, signs, drones.
                // NO cats or shopping carts here.
                pool.push(ObstacleKind::Car);
                pool.push(ObstacleKind::Car);
                pool.push(ObstacleKind::Car);
                pool.push(ObstacleKind::Deer);
                pool.push(ObstacleKind::Deer);
                pool.push(ObstacleKind::TrafficCone);
                pool.push(ObstacleKind::SignBoard);
                pool.push(ObstacleKind::BalloonDrone);
                pool.push(ObstacleKind::Amy);
                if tier >= 4 {
                    pool.push(ObstacleKind::AliceM1);
                    pool.push(ObstacleKind::Car);
                }
            }
            Stage::Ansan => {
                // Hanyang ERICA campus: a mix, with AeiROBOT presence.
                pool.push(ObstacleKind::CatOrange);
                pool.push(ObstacleKind::CatWhite);
                pool.push(ObstacleKind::TrafficCone);
                pool.push(ObstacleKind::Amy);
                pool.push(ObstacleKind::BalloonDrone);
                pool.push(ObstacleKind::BalloonDrone);
                pool.push(ObstacleKind::AliceM1);
                pool.push(ObstacleKind::VacuumBot);
                if tier >= 5 {
                    pool.push(ObstacleKind::Alice3);
                }
                if tier >= 6 {
                    pool.push(ObstacleKind::Alice4);
                    pool.push(ObstacleKind::Alice3);
                }
            }
            Stage::AeiRobotOffice => {
                // Tier 7 office: mix of AeiROBOT bots, manageable density.
                pool.push(ObstacleKind::VacuumBot);
                pool.push(ObstacleKind::Amy);
                pool.push(ObstacleKind::Amy);
                pool.push(ObstacleKind::BalloonDrone);
                pool.push(ObstacleKind::AliceM1);
                pool.push(ObstacleKind::Alice3);
                pool.push(ObstacleKind::SignBoard);
            }
            Stage::AeiRobotCEORoom => {
                // CEO room: wall-to-wall robots, extreme difficulty.
                pool.push(ObstacleKind::Alice3);
                pool.push(ObstacleKind::Alice3);
                pool.push(ObstacleKind::Alice4);
                pool.push(ObstacleKind::Alice4);
                pool.push(ObstacleKind::Alice4);
                pool.push(ObstacleKind::AliceM1);
                pool.push(ObstacleKind::Amy);
                pool.push(ObstacleKind::Amy);
                pool.push(ObstacleKind::BalloonDrone);
            }
        }

        let idx = rng.gen_range(0..pool.len());
        pool[idx]
    }

    pub fn update(&mut self, dt: f32, speed: f32, score: u32, rng: &mut SmallRng) {
        use crate::game::difficulty::{stage_for_tier, Stage};
        let dx = speed * dt;
        for o in &mut self.obstacles {
            o.age += dt;
            // Baseline scroll
            o.x -= dx;
            // Extra charge velocity
            o.x += o.extra_vx * dt;
            // Vertical pattern
            o.y += o.vy * dt;
            let ground_y = o.kind.y_for_kind();
            if o.y > ground_y && o.vy > 0.0 {
                o.y = ground_y;
                o.vy = 0.0;
            }

            match o.kind {
                ObstacleKind::Car => {
                    // Idle cruise, then random surge toward the player.
                    if o.pattern_t <= 0.0 && o.age > 0.25 && o.age < 0.4 {
                        // One-time commit to a surge speed
                        o.extra_vx = -(rng.gen_range(120.0..220.0));
                        o.pattern_t = 1.0;
                    }
                }
                ObstacleKind::Deer => {
                    // Deer leaps diagonally: wait, then launch upward + charge.
                    if o.pattern_t <= 0.0 && o.x < 1100.0 && o.age > 0.35 {
                        o.vy = -220.0;
                        o.extra_vx = -(rng.gen_range(90.0..170.0));
                        o.pattern_t = 1.0;
                    }
                    // Gravity on the leap
                    if o.pattern_t > 0.0 {
                        o.vy += 520.0 * dt;
                    }
                }
                _ => {}
            }
        }
        self.obstacles.retain(|o| o.alive && o.x + o.kind.size().0 > -50.0);

        self.scrolled_since_spawn += dx;
        if self.scrolled_since_spawn >= self.next_spawn_gap {
            let kind = self.random_kind(score, rng);
            self.obstacles.push(Obstacle::new(kind, SPAWN_X));
            self.scrolled_since_spawn = 0.0;
            // CEO Room is extreme: tighter spacing + less random extra.
            let stage = stage_for_tier(tier_for_score(score));
            let (density, jitter) = match stage {
                Stage::AeiRobotCEORoom => (0.55, 120.0),
                Stage::AeiRobotOffice => (0.80, 160.0),
                _ => (1.0, 200.0),
            };
            let extra = rng.gen_range(0.0..jitter);
            self.next_spawn_gap = Self::min_gap(speed) * density + extra;
        }
    }

    pub fn first_collision(&self, player: &Aabb) -> Option<usize> {
        self.obstacles
            .iter()
            .position(|o| o.alive && o.hitbox().intersects(player))
    }

    /// True if an AABB would overlap any existing obstacle hitbox (padded).
    pub fn collides_with_any(&self, aabb: &Aabb, padding: f32) -> bool {
        self.obstacles.iter().any(|o| {
            if !o.alive {
                return false;
            }
            let hb = o.hitbox();
            let padded = Aabb {
                x: hb.x - padding,
                y: hb.y - padding,
                w: hb.w + 2.0 * padding,
                h: hb.h + 2.0 * padding,
            };
            padded.intersects(aabb)
        })
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
            ObstacleKind::CatOrange,
            ObstacleKind::CatWhite,
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
    fn min_gap_grows_with_speed() {
        assert!(ObstacleField::min_gap(720.0) > ObstacleField::min_gap(320.0));
    }

    #[test]
    fn spawn_respects_min_spacing_in_normal_stages() {
        use crate::game::difficulty::{stage_for_tier, Stage, SCORE_PER_TIER};
        // Only check static-spacing stages. Highway obstacles (Car, Deer)
        // use dynamic charge/leap patterns so their spacing is not stable by
        // design. CEO Room uses intentionally tighter density.
        for tier in 0..=6u32 {
            let score = tier * SCORE_PER_TIER;
            if !matches!(
                stage_for_tier(tier),
                Stage::DepartmentStore | Stage::PangyoStreet | Stage::Ansan
            ) {
                continue;
            }
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
        use crate::game::difficulty::SCORE_PER_TIER;
        let mut rng = SmallRng::seed_from_u64(7);
        let field = ObstacleField::new();
        for _ in 0..200 {
            let k = field.random_kind(0, &mut rng);
            assert_ne!(k, ObstacleKind::SignBoard);
        }
        let mut saw = false;
        for _ in 0..2000 {
            if field.random_kind(SPARK_BURST_UNLOCK_TIER * SCORE_PER_TIER, &mut rng)
                == ObstacleKind::SignBoard
            {
                saw = true;
                break;
            }
        }
        assert!(saw);
    }

    #[test]
    fn higher_tiers_spawn_more_robots_v2() {
        use crate::game::difficulty::SCORE_PER_TIER;
        let field = ObstacleField::new();
        let mut rng = SmallRng::seed_from_u64(99);
        let count_robots = |score: u32, samples: usize, rng: &mut SmallRng| {
            (0..samples)
                .filter(|_| field.random_kind(score, rng).is_robot())
                .count()
        };
        let low = count_robots(0, 1000, &mut rng);
        let high = count_robots(7 * SCORE_PER_TIER, 1000, &mut rng);
        assert!(high > low * 2, "low={low}, high={high}");
    }
}
