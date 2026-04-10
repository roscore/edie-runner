// EDIE Runner -- miniquad JS plugin.
//
// 1. Synchronous localStorage bridge for in-session cache + offline.
// 2. Async remote leaderboard via jsonblob.com so scores persist across
//    devices and are visible to all players.
(function () {
    "use strict";

    // ============================================================
    // Remote leaderboard endpoint (jsonblob.com, no auth required).
    // ============================================================
    var BLOB_ID = "019d75c3-cbb7-779e-82f6-97066b77c410";
    var BLOB_URL = "https://jsonblob.com/api/jsonBlob/" + BLOB_ID;
    // In-memory cache of the last-fetched remote leaderboard JSON.
    var _remote_lb_cache = null;
    // Kick off a background fetch immediately so the data is likely
    // ready by the time the title screen renders.
    (function prefetch() {
        fetch(BLOB_URL, { headers: { "Accept": "application/json" } })
            .then(function (r) { return r.text(); })
            .then(function (t) { _remote_lb_cache = t; })
            .catch(function () {});
    })();

    // ============================================================
    // localStorage helpers (unchanged from before).
    // ============================================================
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
            console.warn("[edie_storage] localStorage.setItem failed:", e);
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

    // ============================================================
    // Remote leaderboard: called from Rust via extern "C".
    // ============================================================

    // edie_remote_lb_fetch_len(): copies the cached remote JSON into
    // wasm memory and returns its byte length, or -1 if not yet
    // available. Non-blocking — just reads the prefetched cache.
    function edie_remote_lb_get(out_ptr, out_cap) {
        if (!_remote_lb_cache) {
            return -1;
        }
        return write_str(out_ptr, out_cap, _remote_lb_cache);
    }

    // edie_remote_lb_put(json_ptr, json_len): fire-and-forget PUT of
    // the full leaderboard JSON to jsonblob. Also refreshes the cache.
    function edie_remote_lb_put(json_ptr, json_len) {
        var json = read_str(json_ptr, json_len);
        _remote_lb_cache = json;
        fetch(BLOB_URL, {
            method: "PUT",
            headers: {
                "Content-Type": "application/json",
                "Accept": "application/json",
            },
            body: json,
        }).catch(function (e) {
            console.warn("[edie_storage] remote PUT failed:", e);
        });
    }

    // ============================================================
    // Plugin registration.
    // ============================================================
    function register_plugin(importObject) {
        importObject.env.edie_ls_set = edie_ls_set;
        importObject.env.edie_ls_get = edie_ls_get;
        importObject.env.edie_remote_lb_get = edie_remote_lb_get;
        importObject.env.edie_remote_lb_put = edie_remote_lb_put;
    }

    miniquad_add_plugin({
        register_plugin: register_plugin,
        version: "0.2.0",
        name: "edie_storage"
    });
})();
