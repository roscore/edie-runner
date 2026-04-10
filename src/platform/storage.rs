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
        /// Read the cached remote leaderboard JSON (fetched by JS on page
        /// load). Returns byte count written into out_ptr, or -1 if not
        /// yet available.
        pub fn edie_remote_lb_get(out_ptr: *mut u8, out_cap: usize) -> i32;
        /// Fire-and-forget PUT of the full leaderboard JSON to the remote
        /// jsonblob endpoint. JS handles the async fetch.
        pub fn edie_remote_lb_put(json_ptr: *const u8, json_len: usize);
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
impl BrowserStorage {
    /// Try to read the remote leaderboard JSON (prefetched by the JS
    /// plugin on page load). Returns `None` if the data hasn't arrived
    /// yet or if we're not running in wasm.
    pub fn remote_leaderboard_json(&self) -> Option<String> {
        Self::remote_lb_get()
    }

    /// Push the full leaderboard JSON to the remote endpoint (fire and
    /// forget). Safe to call every time the leaderboard changes.
    pub fn push_remote_leaderboard(&self, json: &str) {
        Self::remote_lb_put(json);
    }

    #[cfg(target_arch = "wasm32")]
    fn remote_lb_get() -> Option<String> {
        let mut buf = vec![0u8; 32 * 1024];
        let n = unsafe {
            js_bridge::edie_remote_lb_get(buf.as_mut_ptr(), buf.len())
        };
        if n < 0 {
            return None;
        }
        buf.truncate(n as usize);
        String::from_utf8(buf).ok()
    }

    #[cfg(target_arch = "wasm32")]
    fn remote_lb_put(json: &str) {
        let bytes = json.as_bytes();
        unsafe {
            js_bridge::edie_remote_lb_put(bytes.as_ptr(), bytes.len());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn remote_lb_get() -> Option<String> {
        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn remote_lb_put(_json: &str) {}

    /// Static variant of push so leaderboard.rs can call it without
    /// holding a &self reference (the JS bridge is stateless).
    pub fn push_remote_static(json: &str) {
        Self::remote_lb_put(json);
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
