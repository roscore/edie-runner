// sapp-jsutils plugin for EDIE Runner: localStorage bridge.
//
// Exposes two imports to the wasm module:
//   edie_storage_set(key, value)
//   edie_storage_get(key) -> string or nil
//
// Every call is wrapped in try/catch so that quota errors, private-mode
// restrictions, or a missing localStorage API never propagate back into
// the wasm -- the game always keeps running.

register_plugin = function (importObject) {
  importObject.env.edie_storage_set = function (key_js, val_js) {
    try {
      const key = consume_js_object(key_js);
      const value = consume_js_object(val_js);
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem(key, value);
      }
    } catch (e) {
      // swallow any localStorage failure
    }
  };

  importObject.env.edie_storage_get = function (key_js) {
    try {
      const key = consume_js_object(key_js);
      if (typeof localStorage === 'undefined') return js_object(null);
      const v = localStorage.getItem(key);
      if (v === null || v === undefined) return js_object(null);
      return js_object(v);
    } catch (e) {
      return js_object(null);
    }
  };
};

miniquad_add_plugin({
  register_plugin: register_plugin,
  on_init: function () {},
  name: "edie_plugin",
  version: "0.1.0",
});
