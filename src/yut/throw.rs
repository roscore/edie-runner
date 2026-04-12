//! Yut stick throw mechanics.

use rand::rngs::SmallRng;
use rand::Rng;

/// Result of a yut throw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YutResult {
    Do,   // 1 flat  → move 1
    Gae,  // 2 flat  → move 2
    Geol, // 3 flat  → move 3
    Yut,  // 4 flat  → move 4, bonus turn
    Mo,   // 0 flat  → move 5, bonus turn
}

impl YutResult {
    pub fn steps(self) -> usize {
        match self {
            YutResult::Do => 1,
            YutResult::Gae => 2,
            YutResult::Geol => 3,
            YutResult::Yut => 4,
            YutResult::Mo => 5,
        }
    }

    pub fn grants_bonus(self) -> bool {
        matches!(self, YutResult::Yut | YutResult::Mo)
    }

    pub fn name_ko(self) -> &'static str {
        match self {
            YutResult::Do => "도",
            YutResult::Gae => "개",
            YutResult::Geol => "걸",
            YutResult::Yut => "윷",
            YutResult::Mo => "모",
        }
    }

    pub fn name_en(self) -> &'static str {
        match self {
            YutResult::Do => "DO",
            YutResult::Gae => "GAE",
            YutResult::Geol => "GEOL",
            YutResult::Yut => "YUT",
            YutResult::Mo => "MO",
        }
    }
}

/// Throw 4 yut sticks. Each stick is flat (true) or round (false)
/// with 50% probability.
pub fn throw_yut(rng: &mut SmallRng) -> (YutResult, [bool; 4]) {
    let sticks: [bool; 4] = [
        rng.gen_bool(0.5),
        rng.gen_bool(0.5),
        rng.gen_bool(0.5),
        rng.gen_bool(0.5),
    ];
    let flats = sticks.iter().filter(|&&s| s).count();
    let result = match flats {
        0 => YutResult::Mo,
        1 => YutResult::Do,
        2 => YutResult::Gae,
        3 => YutResult::Geol,
        4 => YutResult::Yut,
        _ => unreachable!(),
    };
    (result, sticks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn throw_produces_valid_result() {
        let mut rng = SmallRng::seed_from_u64(42);
        for _ in 0..100 {
            let (result, sticks) = throw_yut(&mut rng);
            let flats = sticks.iter().filter(|&&s| s).count();
            match flats {
                0 => assert_eq!(result, YutResult::Mo),
                1 => assert_eq!(result, YutResult::Do),
                2 => assert_eq!(result, YutResult::Gae),
                3 => assert_eq!(result, YutResult::Geol),
                4 => assert_eq!(result, YutResult::Yut),
                _ => panic!("invalid flat count"),
            }
        }
    }

    #[test]
    fn steps_match_result() {
        assert_eq!(YutResult::Do.steps(), 1);
        assert_eq!(YutResult::Gae.steps(), 2);
        assert_eq!(YutResult::Geol.steps(), 3);
        assert_eq!(YutResult::Yut.steps(), 4);
        assert_eq!(YutResult::Mo.steps(), 5);
    }

    #[test]
    fn bonus_turn_on_yut_and_mo() {
        assert!(!YutResult::Do.grants_bonus());
        assert!(!YutResult::Gae.grants_bonus());
        assert!(!YutResult::Geol.grants_bonus());
        assert!(YutResult::Yut.grants_bonus());
        assert!(YutResult::Mo.grants_bonus());
    }

    #[test]
    fn distribution_covers_all_results() {
        let mut rng = SmallRng::seed_from_u64(0);
        let mut seen = [false; 5];
        for _ in 0..1000 {
            let (r, _) = throw_yut(&mut rng);
            match r {
                YutResult::Do => seen[0] = true,
                YutResult::Gae => seen[1] = true,
                YutResult::Geol => seen[2] = true,
                YutResult::Yut => seen[3] = true,
                YutResult::Mo => seen[4] = true,
            }
        }
        assert!(seen.iter().all(|&s| s), "not all results seen in 1000 throws");
    }
}
