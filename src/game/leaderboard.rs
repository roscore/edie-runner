//! Local leaderboard (top-5) persisted to browser localStorage via the
//! edie_plugin.js shim. All I/O is best effort -- if persistence fails the
//! game still runs with an in-memory ledger.

use crate::platform::storage::Storage;
use serde::{Deserialize, Serialize};

pub const LEADERBOARD_KEY: &str = "edie_runner.leaderboard.v3";
pub const LEADERBOARD_MAX: usize = 5;
pub const NAME_LEN: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub name: String, // exactly NAME_LEN chars, uppercase
    pub score: u32,
    pub ts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardFile {
    pub version: u32,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Default)]
pub struct Leaderboard {
    pub entries: Vec<Entry>,
}

impl Leaderboard {
    const CURRENT_VERSION: u32 = 3;

    /// Load from storage. Always preserves existing scores regardless of
    /// the stored version — version bumps must never erase player data.
    pub fn load<S: Storage>(storage: &S) -> Self {
        let mut lb = Self::default();
        let raw = storage.get(LEADERBOARD_KEY);
        if let Some(json) = raw {
            if let Ok(file) = serde_json::from_str::<LeaderboardFile>(&json) {
                // Accept entries from any version — scores are always valid.
                lb.entries = file.entries;
            }
        }
        lb.ensure_seed();
        lb.sort_and_trim();
        lb
    }

    /// Guarantee the ledger always contains at least the SHP seed entry so
    /// the Title screen never shows an empty leaderboard.
    fn ensure_seed(&mut self) {
        let has_shp = self.entries.iter().any(|e| e.name == "SHP");
        if !has_shp {
            self.entries.push(Entry {
                name: "SHP".to_string(),
                score: 51000,
                ts: 0,
            });
        }
    }

    fn sort_and_trim(&mut self) {
        self.entries
            .sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.ts.cmp(&b.ts)));
        if self.entries.len() > LEADERBOARD_MAX {
            self.entries.truncate(LEADERBOARD_MAX);
        }
    }

    /// True if a run with `score` would place in the top LEADERBOARD_MAX.
    /// This is how we decide whether to prompt for a name at all.
    pub fn qualifies(&self, score: u32) -> bool {
        if score == 0 {
            return false;
        }
        if self.entries.len() < LEADERBOARD_MAX {
            return true;
        }
        // entries is sorted highest first; worst qualifying is at index
        // LEADERBOARD_MAX-1.
        let cutoff = self
            .entries
            .get(LEADERBOARD_MAX - 1)
            .map(|e| e.score)
            .unwrap_or(0);
        score > cutoff
    }

    pub fn insert<S: Storage>(&mut self, storage: &mut S, entry: Entry) {
        self.entries.push(entry);
        self.sort_and_trim();
        self.persist(storage);
        // Also push to the remote endpoint so scores are visible
        // cross-device. This is fire-and-forget — the JS plugin
        // handles the async PUT.
        self.push_remote(storage);
    }

    /// Push the current leaderboard to the remote endpoint if running
    /// in wasm. Falls back to no-op on native builds.
    pub fn push_remote<S: Storage>(&self, _storage: &S) {
        #[cfg(feature = "graphics")]
        if let Some(json) = self.to_json() {
            crate::platform::storage::BrowserStorage::push_remote_static(&json);
        }
    }

    pub fn persist<S: Storage>(&self, storage: &mut S) {
        let file = LeaderboardFile {
            version: Self::CURRENT_VERSION,
            entries: self.entries.clone(),
        };
        if let Ok(json) = serde_json::to_string(&file) {
            storage.set(LEADERBOARD_KEY, &json);
        }
    }

    /// Merge entries from a remote JSON blob (fetched by the JS plugin).
    /// New high-score entries that aren't already in the local board get
    /// added; duplicates (same name + score) are skipped.
    pub fn merge_remote(&mut self, json: &str) {
        if let Ok(file) = serde_json::from_str::<LeaderboardFile>(json) {
            // Accept entries from any version — never discard remote scores.
            for remote_e in &file.entries {
                let already = self.entries.iter().any(|e| {
                    e.name == remote_e.name && e.score == remote_e.score
                });
                if !already {
                    self.entries.push(remote_e.clone());
                }
            }
            self.sort_and_trim();
        }
    }

    /// Serialize the current leaderboard to JSON (for pushing to the
    /// remote endpoint via the JS plugin).
    pub fn to_json(&self) -> Option<String> {
        let file = LeaderboardFile {
            version: Self::CURRENT_VERSION,
            entries: self.entries.clone(),
        };
        serde_json::to_string(&file).ok()
    }

    /// Best score currently on the board (for HUD HI display).
    pub fn high_score(&self) -> u32 {
        self.entries.first().map(|e| e.score).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::storage::InMemoryStorage;

    #[test]
    fn fresh_load_has_shp_seed() {
        let storage = InMemoryStorage::new();
        let lb = Leaderboard::load(&storage);
        assert!(lb.entries.iter().any(|e| e.name == "SHP" && e.score == 51000));
    }

    #[test]
    fn qualifies_empty_slot() {
        let storage = InMemoryStorage::new();
        let lb = Leaderboard::load(&storage);
        // SHP is 51000; only one entry seeded.
        assert!(lb.qualifies(10));
        assert!(lb.qualifies(100_000));
    }

    #[test]
    fn qualifies_requires_beating_cutoff() {
        let mut storage = InMemoryStorage::new();
        let mut lb = Leaderboard::load(&storage);
        for name in ["AAA", "BBB", "CCC", "DDD"] {
            lb.insert(
                &mut storage,
                Entry { name: name.to_string(), score: 60000, ts: 0 },
            );
        }
        // Board is now full with [60000x4, 51000]. Cutoff = 51000.
        assert!(lb.qualifies(52000));
        assert!(!lb.qualifies(51000));
        assert!(!lb.qualifies(1000));
    }

    #[test]
    fn insert_trims_to_max() {
        let mut storage = InMemoryStorage::new();
        let mut lb = Leaderboard::load(&storage);
        for i in 0..10u32 {
            lb.insert(
                &mut storage,
                Entry {
                    name: format!("P{i:02}"),
                    score: 100_000 + i,
                    ts: i as u64,
                },
            );
        }
        assert_eq!(lb.entries.len(), LEADERBOARD_MAX);
    }

    #[test]
    fn round_trip_persists_across_loads() {
        let mut storage = InMemoryStorage::new();
        let mut lb = Leaderboard::load(&storage);
        lb.insert(
            &mut storage,
            Entry { name: "ABC".to_string(), score: 99999, ts: 1 },
        );
        let lb2 = Leaderboard::load(&storage);
        assert!(lb2.entries.iter().any(|e| e.name == "ABC"));
    }
}
