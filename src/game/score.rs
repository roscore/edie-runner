//! Score and persistent high score. See spec §5.

use crate::platform::storage::Storage;
use serde::{Deserialize, Serialize};

pub const STORAGE_KEY: &str = "edie_runner.high_score";
const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct StoredScore {
    version: u32,
    high_score: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Score {
    pub current: u32,
    pub high: u32,
}

impl Score {
    pub fn load<S: Storage>(storage: &S) -> Self {
        let high = storage
            .get(STORAGE_KEY)
            .and_then(|s| serde_json::from_str::<StoredScore>(&s).ok())
            .filter(|s| s.version == SCHEMA_VERSION)
            .map(|s| s.high_score)
            .unwrap_or(0);
        Self { current: 0, high }
    }

    pub fn save_if_new_high<S: Storage>(&self, storage: &mut S) -> bool {
        // Compare against the persisted value, not in-memory `self.high`,
        // because `add()` already updates `self.high` eagerly for HUD display.
        let stored_high = Self::load(storage).high;
        if self.current > stored_high {
            let stored = StoredScore {
                version: SCHEMA_VERSION,
                high_score: self.current,
            };
            let json = serde_json::to_string(&stored).expect("serializable");
            storage.set(STORAGE_KEY, &json);
            true
        } else {
            false
        }
    }

    pub fn add(&mut self, points: u32) {
        self.current = self.current.saturating_add(points);
        if self.current > self.high {
            self.high = self.current;
        }
    }

    pub fn reset(&mut self) {
        self.current = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::storage::InMemoryStorage;

    #[test]
    fn fresh_load_is_zero() {
        let s = InMemoryStorage::new();
        let score = Score::load(&s);
        assert_eq!(score.high, 0);
        assert_eq!(score.current, 0);
    }

    #[test]
    fn round_trip_high_score() {
        let mut storage = InMemoryStorage::new();
        let mut score = Score::load(&storage);
        score.add(1234);
        assert!(score.save_if_new_high(&mut storage));

        let reloaded = Score::load(&storage);
        assert_eq!(reloaded.high, 1234);
    }

    #[test]
    fn save_only_if_new_high() {
        let mut storage = InMemoryStorage::new();
        let mut score = Score::load(&storage);
        score.add(100);
        assert!(score.save_if_new_high(&mut storage));

        let mut later = Score::load(&storage);
        later.add(50);
        assert!(!later.save_if_new_high(&mut storage));
        assert_eq!(Score::load(&storage).high, 100);
    }

    #[test]
    fn malformed_json_returns_zero() {
        let mut storage = InMemoryStorage::new();
        storage.set(STORAGE_KEY, "this is not json");
        assert_eq!(Score::load(&storage).high, 0);
    }

    #[test]
    fn wrong_version_returns_zero() {
        let mut storage = InMemoryStorage::new();
        storage.set(STORAGE_KEY, r#"{"version":99,"high_score":500}"#);
        assert_eq!(Score::load(&storage).high, 0);
    }

    #[test]
    fn add_updates_high_in_memory() {
        let s = InMemoryStorage::new();
        let mut score = Score::load(&s);
        score.add(50);
        assert_eq!(score.high, 50);
        score.add(20);
        assert_eq!(score.high, 70);
        score.reset();
        assert_eq!(score.current, 0);
        assert_eq!(score.high, 70);
    }
}
