//! EDIE Runner library entry point.
//!
//! The `graphics` feature (default) compiles the full game with macroquad
//! rendering, texture bundling, and audio. The `headless` feature compiles
//! ONLY the pure game logic needed by external bots / AI testing harnesses.

pub mod game;
pub mod platform;
pub mod time;

#[cfg(feature = "graphics")]
pub mod assets;

#[cfg(feature = "graphics")]
pub mod render;
