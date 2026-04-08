//! Difficulty tier curve. See spec §3.6.

pub const BASE_SPEED: f32 = 280.0;
pub const SPEED_CAP: f32 = 640.0;
/// Score at which speed reaches SPEED_CAP. Chosen so the player experiences
/// most of the speed curve before maxing out.
pub const SCORE_AT_CAP: u32 = 22000;
/// Score required to advance one tier. Tuned so a player staying alive spends
/// at least ~30 seconds in each tier at low speeds.
pub const SCORE_PER_TIER: u32 = 2500;
pub const MAX_TIER: u32 = 8;
pub const SPARK_BURST_UNLOCK_TIER: u32 = 3;

pub fn tier_for_score(score: u32) -> u32 {
    (score / SCORE_PER_TIER).min(MAX_TIER)
}

/// Smooth linear speed ramp from BASE_SPEED at score 0 to SPEED_CAP at
/// SCORE_AT_CAP.
pub fn speed_for_score(score: u32) -> f32 {
    let t = (score as f32 / SCORE_AT_CAP as f32).min(1.0);
    BASE_SPEED + (SPEED_CAP - BASE_SPEED) * t
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    DepartmentStore,
    PangyoStreet,
    Highway,
    Ansan,
    AeiRobotHQ,
}

pub fn stage_for_tier(tier: u32) -> Stage {
    match tier {
        0 => Stage::DepartmentStore,
        1 | 2 => Stage::PangyoStreet,
        3 | 4 => Stage::Highway,
        5 | 6 => Stage::Ansan,
        _ => Stage::AeiRobotHQ,
    }
}

pub fn stage_name(stage: Stage) -> &'static str {
    match stage {
        Stage::DepartmentStore => "PANGYO DEPARTMENT STORE",
        Stage::PangyoStreet => "PANGYO STREET",
        Stage::Highway => "HIGHWAY TO ANSAN",
        Stage::Ansan => "HANYANG UNIV (ERICA)",
        Stage::AeiRobotHQ => "AEIROBOT HQ",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_speed_at_zero() {
        assert_eq!(speed_for_score(0), BASE_SPEED);
    }

    #[test]
    fn monotonic() {
        let mut prev = speed_for_score(0);
        for s in (0..10000).step_by(100) {
            let cur = speed_for_score(s);
            assert!(cur >= prev, "speed went down at {s}: {prev} -> {cur}");
            prev = cur;
        }
    }

    #[test]
    fn capped() {
        for s in (0..50000).step_by(500) {
            assert!(speed_for_score(s) <= SPEED_CAP);
        }
    }

    #[test]
    fn tier_indexable_no_panic() {
        for s in (0..50000).step_by(123) {
            let _ = tier_for_score(s);
        }
    }

    #[test]
    fn tier_caps_at_max() {
        assert_eq!(tier_for_score(999999), MAX_TIER);
    }
}
