//! Difficulty tier curve. See spec §3.6.

pub const BASE_SPEED: f32 = 280.0;
pub const SPEED_CAP: f32 = 640.0;
/// Score at which speed reaches SPEED_CAP (end of Office stage).
pub const SCORE_AT_CAP: u32 = 17500;
pub const SCORE_PER_TIER: u32 = 2500;
pub const MAX_TIER: u32 = 9;
pub const SPARK_BURST_UNLOCK_TIER: u32 = 3;
/// Score at which the corona boss mode is triggered.
pub const BOSS_TRIGGER_SCORE: u32 = 50000;

pub fn tier_for_score(score: u32) -> u32 {
    (score / SCORE_PER_TIER).min(MAX_TIER)
}

/// Linear speed ramp from BASE_SPEED at score 0 to SPEED_CAP at SCORE_AT_CAP.
/// Beyond the cap, speed continues climbing slowly (CEO Room torture zone).
pub fn speed_for_score(score: u32) -> f32 {
    if score <= SCORE_AT_CAP {
        let t = score as f32 / SCORE_AT_CAP as f32;
        BASE_SPEED + (SPEED_CAP - BASE_SPEED) * t
    } else {
        // Post-cap ramp: +1 px/s per 100 score, soft ceiling via clamp.
        let extra = (score - SCORE_AT_CAP) as f32 * 0.01;
        (SPEED_CAP + extra).min(1100.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    DepartmentStore,
    PangyoStreet,
    PangyoTechPark,
    Highway,
    Ansan,
    AeiRobotOffice,
    AeiRobotCEORoom,
}

pub fn stage_for_tier(tier: u32) -> Stage {
    match tier {
        0 => Stage::DepartmentStore,
        1 => Stage::PangyoStreet,
        2 => Stage::PangyoTechPark,
        3 | 4 => Stage::Highway,
        5 | 6 => Stage::Ansan,
        7 => Stage::AeiRobotOffice,
        _ => Stage::AeiRobotCEORoom,
    }
}

pub fn stage_name(stage: Stage) -> &'static str {
    match stage {
        Stage::DepartmentStore => "PANGYO POP-UP STORE",
        Stage::PangyoStreet => "PANGYO STREET",
        Stage::PangyoTechPark => "PANGYO TECH PARK",
        Stage::Highway => "HIGHWAY TO ANSAN",
        Stage::Ansan => "HANYANG UNIV (ERICA)",
        Stage::AeiRobotOffice => "AEIROBOT OFFICE",
        Stage::AeiRobotCEORoom => "AEIROBOT CEO ROOM",
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
    fn capped_within_normal_range() {
        // Up to SCORE_AT_CAP, speed should stay at or below SPEED_CAP.
        for s in (0..=SCORE_AT_CAP).step_by(500) {
            assert!(speed_for_score(s) <= SPEED_CAP + 0.01);
        }
    }

    #[test]
    fn accelerates_past_cap_in_ceo_room() {
        // Post-cap (CEO Room torture zone) speed keeps climbing.
        assert!(speed_for_score(40000) > SPEED_CAP);
        // But has a soft ceiling.
        assert!(speed_for_score(999_999) <= 1100.1);
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
