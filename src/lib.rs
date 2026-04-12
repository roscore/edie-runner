//! EDIE Runner / Minigames library entry point.

pub mod game;
pub mod platform;
pub mod time;
pub mod reversi;

#[cfg(feature = "graphics")]
pub mod assets;

#[cfg(feature = "graphics")]
pub mod render;
