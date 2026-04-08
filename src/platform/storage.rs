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
