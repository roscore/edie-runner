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

/// Browser localStorage backend. Uses sapp-jsutils on wasm32 targets,
/// falls back to an in-memory map on native builds (tests, bots) so
/// there are no undefined externs outside wasm. Every call is best-effort:
/// failures never panic or block the game loop.
#[cfg(feature = "graphics")]
pub struct BrowserStorage {
    fallback: std::cell::RefCell<std::collections::HashMap<String, String>>,
}

#[cfg(all(feature = "graphics", target_arch = "wasm32"))]
mod js {
    use sapp_jsutils::JsObject;
    extern "C" {
        pub fn edie_storage_set(key: JsObject, value: JsObject);
        pub fn edie_storage_get(key: JsObject) -> JsObject;
    }
}

#[cfg(feature = "graphics")]
impl BrowserStorage {
    pub fn new() -> Self {
        Self {
            fallback: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }
}

#[cfg(feature = "graphics")]
impl Default for BrowserStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(feature = "graphics", target_arch = "wasm32"))]
impl Storage for BrowserStorage {
    fn get(&self, key: &str) -> Option<String> {
        let js_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            let key_js = sapp_jsutils::JsObject::string(key);
            let result = js::edie_storage_get(key_js);
            if result.is_nil() {
                None
            } else {
                let mut s = String::new();
                result.to_string(&mut s);
                Some(s)
            }
        }));
        match js_result {
            Ok(Some(s)) => Some(s),
            _ => self.fallback.borrow().get(key).cloned(),
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            let key_js = sapp_jsutils::JsObject::string(key);
            let val_js = sapp_jsutils::JsObject::string(value);
            js::edie_storage_set(key_js, val_js);
        }));
        self.fallback
            .borrow_mut()
            .insert(key.to_string(), value.to_string());
    }
}

// Native (non-wasm) build: storage is just the in-memory fallback. This
// lets `cargo test` + the headless bot run without pulling in JS externs.
#[cfg(all(feature = "graphics", not(target_arch = "wasm32")))]
impl Storage for BrowserStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.fallback.borrow().get(key).cloned()
    }
    fn set(&mut self, key: &str, value: &str) {
        self.fallback
            .borrow_mut()
            .insert(key.to_string(), value.to_string());
    }
}
