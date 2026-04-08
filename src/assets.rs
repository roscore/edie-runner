//! Texture asset registry. All paths are relative to the wasm host directory
//! (i.e. the `web/` folder in production).

use macroquad::prelude::*;

pub struct AssetHandles {
    pub edie_run: Texture2D,
    pub edie_jump: Texture2D,
    pub edie_duck: Texture2D,
    pub edie_dash: Texture2D,
    pub edie_hit: Texture2D,
    pub edie_shadow: Texture2D,

    // GIF-extracted animated sheets (from user-provided gifs).
    pub edie_run_anim: Texture2D,
    pub edie_title_idle: Texture2D,
    pub edie_sad_alt: Texture2D,
    pub edie_sleepy: Texture2D,
    pub edie_hit_anim: Texture2D,
    pub edie_look: Texture2D,
    pub edie_gameover_anim: Texture2D,
    pub edie_blink_alt: Texture2D,
    pub edie_cheer_anim: Texture2D,

    pub obstacle_coffee: Texture2D,
    pub obstacle_cart: Texture2D,
    pub obstacle_cone: Texture2D,
    pub obstacle_sign: Texture2D,    // 4 frames
    pub obstacle_cat: Texture2D,     // 2 frames
    pub obstacle_vacuum: Texture2D,  // 4 frames
    pub obstacle_amy: Texture2D,     // 4 frames
    pub obstacle_alicem1: Texture2D, // 2 frames
    pub obstacle_alice3: Texture2D,  // 2 frames
    pub obstacle_alice4: Texture2D,  // 2 frames

    pub aurora_purple: Texture2D, // 6 frames horizontal
    pub aurora_green: Texture2D,  // 6 frames horizontal
    pub heart: Texture2D,         // 4 frames pulse

    pub bg_sky: Texture2D,
    pub bg_stars: Texture2D,
    pub bg_far: Texture2D,
    pub bg_mid: Texture2D,
    pub bg_floor: Texture2D,

    // Stage-specific background tiles (5 stages x 3 layers each)
    pub stage_store: StageBg,
    pub stage_street: StageBg,
    pub stage_highway: StageBg,
    pub stage_ansan: StageBg,
    pub stage_hq: StageBg,

    // SFX
    pub sfx_jump: macroquad::audio::Sound,
    pub sfx_hit: macroquad::audio::Sound,
    pub sfx_pickup: macroquad::audio::Sound,
    pub sfx_dash: macroquad::audio::Sound,
    pub sfx_smash: macroquad::audio::Sound,
    pub sfx_heart: macroquad::audio::Sound,
}

pub struct StageBg {
    pub far: Texture2D,
    pub mid: Texture2D,
    pub floor: Texture2D,
}

#[derive(Debug)]
pub struct LoadError {
    pub which: String,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to load: {}", self.which)
    }
}

async fn load_pixel(name: &str) -> Result<Texture2D, LoadError> {
    match load_texture(name).await {
        Ok(t) => {
            t.set_filter(FilterMode::Nearest);
            Ok(t)
        }
        Err(_) => Err(LoadError { which: name.to_string() }),
    }
}

pub async fn load_all() -> Result<AssetHandles, LoadError> {
    Ok(AssetHandles {
        edie_run: load_pixel("edie_run.png").await?,
        edie_jump: load_pixel("edie_jump.png").await?,
        edie_duck: load_pixel("edie_duck.png").await?,
        edie_dash: load_pixel("edie_dash.png").await?,
        edie_hit: load_pixel("edie_hit.png").await?,
        edie_shadow: load_pixel("edie_shadow.png").await?,

        edie_run_anim: load_pixel("edie_run_anim.png").await?,
        edie_title_idle: load_pixel("edie_title_idle.png").await?,
        edie_sad_alt: load_pixel("edie_sad_alt.png").await?,
        edie_sleepy: load_pixel("edie_sleepy.png").await?,
        edie_hit_anim: load_pixel("edie_hit_anim.png").await?,
        edie_look: load_pixel("edie_look.png").await?,
        edie_gameover_anim: load_pixel("edie_gameover_anim.png").await?,
        edie_blink_alt: load_pixel("edie_blink_alt.png").await?,
        edie_cheer_anim: load_pixel("edie_cheer_anim.png").await?,

        obstacle_coffee: load_pixel("obstacle_coffee.png").await?,
        obstacle_cart: load_pixel("obstacle_cart.png").await?,
        obstacle_cone: load_pixel("obstacle_cone.png").await?,
        obstacle_sign: load_pixel("obstacle_sign.png").await?,
        obstacle_cat: load_pixel("obstacle_cat.png").await?,
        obstacle_vacuum: load_pixel("obstacle_vacuum.png").await?,
        obstacle_amy: load_pixel("obstacle_amy.png").await?,
        obstacle_alicem1: load_pixel("obstacle_alicem1.png").await?,
        obstacle_alice3: load_pixel("obstacle_alice3.png").await?,
        obstacle_alice4: load_pixel("obstacle_alice4.png").await?,

        aurora_purple: load_pixel("aurora_purple.png").await?,
        aurora_green: load_pixel("aurora_green.png").await?,
        heart: load_pixel("heart.png").await?,

        bg_sky: load_pixel("bg_sky.png").await?,
        bg_stars: load_pixel("bg_stars.png").await?,
        bg_far: load_pixel("bg_far.png").await?,
        bg_mid: load_pixel("bg_mid.png").await?,
        bg_floor: load_pixel("bg_floor.png").await?,

        stage_store: StageBg {
            far: load_pixel("bg_store_far.png").await?,
            mid: load_pixel("bg_store_mid.png").await?,
            floor: load_pixel("bg_store_floor.png").await?,
        },
        stage_street: StageBg {
            far: load_pixel("bg_street_far.png").await?,
            mid: load_pixel("bg_street_mid.png").await?,
            floor: load_pixel("bg_street_floor.png").await?,
        },
        stage_highway: StageBg {
            far: load_pixel("bg_highway_far.png").await?,
            mid: load_pixel("bg_highway_mid.png").await?,
            floor: load_pixel("bg_highway_floor.png").await?,
        },
        stage_ansan: StageBg {
            far: load_pixel("bg_ansan_far.png").await?,
            mid: load_pixel("bg_ansan_mid.png").await?,
            floor: load_pixel("bg_ansan_floor.png").await?,
        },
        stage_hq: StageBg {
            far: load_pixel("bg_hq_far.png").await?,
            mid: load_pixel("bg_hq_mid.png").await?,
            floor: load_pixel("bg_hq_floor.png").await?,
        },

        sfx_jump: load_sound("sfx_jump.wav").await?,
        sfx_hit: load_sound("sfx_hit.wav").await?,
        sfx_pickup: load_sound("sfx_pickup.wav").await?,
        sfx_dash: load_sound("sfx_dash.wav").await?,
        sfx_smash: load_sound("sfx_smash.wav").await?,
        sfx_heart: load_sound("sfx_heart.wav").await?,
    })
}

async fn load_sound(name: &str) -> Result<macroquad::audio::Sound, LoadError> {
    match macroquad::audio::load_sound(name).await {
        Ok(s) => Ok(s),
        Err(_) => Err(LoadError { which: name.to_string() }),
    }
}
