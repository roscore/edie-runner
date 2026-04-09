// EDIE Runner -- miniquad JS plugin.
//
// Adds a tiny synchronous localStorage bridge so the high score and
// leaderboard actually persist across page reloads. Rust imports three
// extern "C" functions; this plugin installs them into the wasm import
// object before the wasm module is instantiated.
//
// Protocol for get: Rust hands us a key and a pre-allocated destination
// buffer. If the stored value fits, we copy UTF-8 bytes into the buffer and
// return the byte length. If the value is missing, we return -1. If it is
// longer than cap, we return -2 (Rust will treat it as "missing").
(function () {
    "use strict";

    function read_str(ptr, len) {
        var view = new Uint8Array(wasm_memory.buffer, ptr, len);
        return new TextDecoder("utf-8").decode(view);
    }

    function write_str(ptr, cap, str) {
        var encoded = new TextEncoder().encode(str);
        if (encoded.length > cap) {
            return -2;
        }
        var dst = new Uint8Array(wasm_memory.buffer, ptr, encoded.length);
        dst.set(encoded);
        return encoded.length;
    }

    function edie_ls_set(key_ptr, key_len, val_ptr, val_len) {
        try {
            var key = read_str(key_ptr, key_len);
            var val = read_str(val_ptr, val_len);
            localStorage.setItem(key, val);
        } catch (e) {
            // localStorage may be blocked (private mode, disabled cookies).
            // Swallow the error so the game keeps running.
        }
    }

    function edie_ls_get(key_ptr, key_len, out_ptr, out_cap) {
        try {
            var key = read_str(key_ptr, key_len);
            var val = localStorage.getItem(key);
            if (val === null || val === undefined) {
                return -1;
            }
            return write_str(out_ptr, out_cap, val);
        } catch (e) {
            return -1;
        }
    }

    function register_plugin(importObject) {
        importObject.env.edie_ls_set = edie_ls_set;
        importObject.env.edie_ls_get = edie_ls_get;
    }

    miniquad_add_plugin({
        register_plugin: register_plugin,
        version: "0.1.0",
        name: "edie_storage"
    });
})();
