//! Difficulty tier curve. See spec §3.6.

pub const BASE_SPEED: f32 = 320.0;
pub const SPEED_STEP: f32 = 20.0;
pub const SPEED_CAP: f32 = 720.0;
pub const SCORE_PER_TIER: u32 = 500;
pub const MAX_TIER: u32 = 8;
pub const SPARK_BURST_UNLOCK_TIER: u32 = 3;

pub fn tier_for_score(score: u32) -> u32 {
    (score / SCORE_PER_TIER).min(MAX_TIER)
}

pub fn speed_for_score(score: u32) -> f32 {
    let tier = tier_for_score(score);
    let raw = BASE_SPEED + SPEED_STEP * tier as f32;
    raw.min(SPEED_CAP)
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
