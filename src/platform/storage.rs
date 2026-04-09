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

// =====================================================================
// BrowserStorage: wasm localStorage bridge, native HashMap fallback.
// =====================================================================
#[cfg(all(feature = "graphics", target_arch = "wasm32"))]
mod js_bridge {
    extern "C" {
        pub fn edie_ls_set(
            key_ptr: *const u8,
            key_len: usize,
            val_ptr: *const u8,
            val_len: usize,
        );
        pub fn edie_ls_get(
            key_ptr: *const u8,
            key_len: usize,
            out_ptr: *mut u8,
            out_cap: usize,
        ) -> i32;
    }
}

#[cfg(feature = "graphics")]
pub struct BrowserStorage {
    // In-session cache so the same value round-trips even if localStorage
    // is unavailable (private browsing, cookies disabled, non-wasm tests).
    cache: std::cell::RefCell<std::collections::HashMap<String, String>>,
}

#[cfg(feature = "graphics")]
impl BrowserStorage {
    pub fn new() -> Self {
        Self {
            cache: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn ls_get(key: &str) -> Option<String> {
        // 16 KiB is well above what our leaderboard + high score needs.
        let mut buf = vec![0u8; 16 * 1024];
        let key_bytes = key.as_bytes();
        let n = unsafe {
            js_bridge::edie_ls_get(
                key_bytes.as_ptr(),
                key_bytes.len(),
                buf.as_mut_ptr(),
                buf.len(),
            )
        };
        if n < 0 {
            return None;
        }
        let n = n as usize;
        buf.truncate(n);
        String::from_utf8(buf).ok()
    }

    #[cfg(target_arch = "wasm32")]
    fn ls_set(key: &str, value: &str) {
        let key_bytes = key.as_bytes();
        let val_bytes = value.as_bytes();
        unsafe {
            js_bridge::edie_ls_set(
                key_bytes.as_ptr(),
                key_bytes.len(),
                val_bytes.as_ptr(),
                val_bytes.len(),
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn ls_get(_key: &str) -> Option<String> {
        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn ls_set(_key: &str, _value: &str) {}
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
        // 1. In-session cache (fast path, also covers failed localStorage).
        if let Some(v) = self.cache.borrow().get(key).cloned() {
            return Some(v);
        }
        // 2. Real localStorage via the JS bridge (wasm only).
        if let Some(v) = Self::ls_get(key) {
            self.cache
                .borrow_mut()
                .insert(key.to_string(), v.clone());
            return Some(v);
        }
        None
    }

    fn set(&mut self, key: &str, value: &str) {
        self.cache
            .borrow_mut()
            .insert(key.to_string(), value.to_string());
        Self::ls_set(key, value);
    }
}
