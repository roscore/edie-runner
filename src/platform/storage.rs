//! Persistent key-value storage behind a trait. See spec §5.

use std::collections::HashMap;

pub trait Storage {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str);
}

#[derive(Default)]
pub struct InMemoryStorage {
    map: HashMap<String, String>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for InMemoryStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.map.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: &str) {
        self.map.insert(key.to_string(), value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut s = InMemoryStorage::new();
        assert_eq!(s.get("k"), None);
        s.set("k", "v");
        assert_eq!(s.get("k"), Some("v".to_string()));
    }

    #[test]
    fn overwrite() {
        let mut s = InMemoryStorage::new();
        s.set("k", "a");
        s.set("k", "b");
        assert_eq!(s.get("k"), Some("b".to_string()));
    }
}

/// In-memory "browser" storage. The localStorage bridge will be re-added
/// in a follow-up commit once the JS plugin is verified in isolation; for
/// now we keep the game bulletproof by only using an in-memory map.
#[cfg(feature = "graphics")]
pub struct BrowserStorage {
    map: std::cell::RefCell<std::collections::HashMap<String, String>>,
}

#[cfg(feature = "graphics")]
impl BrowserStorage {
    pub fn new() -> Self {
        Self {
            map: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }
}

#[cfg(feature = "graphics")]
impl Default for BrowserStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "graphics")]
impl Storage for BrowserStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.map.borrow().get(key).cloned()
    }
    fn set(&mut self, key: &str, value: &str) {
        self.map
            .borrow_mut()
            .insert(key.to_string(), value.to_string());
    }
}
