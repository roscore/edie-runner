//! Texture / audio asset registry.
//!
//! All assets are XOR-scrambled and embedded into the wasm binary at compile
//! time by `build.rs`. At runtime, bytes are decrypted in memory and fed to
//! macroquad via `Texture2D::from_file_with_format` / `load_sound_from_bytes`.
//! There are no separate PNG/WAV files on the webserver — a network inspector
//! will only see `edie_runner.wasm`.

use macroquad::prelude::*;

include!(concat!(env!("OUT_DIR"), "/asset_index.rs"));
const ASSET_BLOB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/asset_blob.bin"));

const KEY: &[u8] = b"EDIE_RUNNER_v1_AeiROBOT_virus_scramble_2026_PANGYO";

fn unscramble(name: &str) -> Option<Vec<u8>> {
    let (_, offset, len) = *ASSET_INDEX.iter().find(|(n, _, _)| *n == name)?;
    let raw = &ASSET_BLOB[offset..offset + len];
    let out: Vec<u8> = raw
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let idx = offset + i;
            let k = KEY[idx % KEY.len()];
            b ^ k ^ ((idx as u8).wrapping_mul(31))
        })
        .collect();
    Some(out)
}

pub struct AssetHandles {
    pub edie_run: Texture2D,
    pub edie_jump: Texture2D,
    pub edie_duck: Texture2D,
    pub edie_dash: Texture2D,
    pub edie_hit: Texture2D,
    pub edie_shadow: Texture2D,

    pub edie_run_anim: Texture2D,
    pub edie_happy_run: Texture2D,
    pub edie_static_run: Texture2D,
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
    pub obstacle_sign: Texture2D,
    pub obstacle_cat_orange: Texture2D,
    pub obstacle_cat_white: Texture2D,
    pub obstacle_car: Texture2D,
    pub obstacle_car_anim: Texture2D,
    pub obstacle_truck: Texture2D,
    pub obstacle_bus: Texture2D,
    pub obstacle_taxi: Texture2D,
    pub obstacle_sportscar: Texture2D,
    pub obstacle_sportscar_anim: Texture2D,
    pub obstacle_deer: Texture2D,
    pub obstacle_balloon: Texture2D,
    pub obstacle_pigeon: Texture2D,
    pub obstacle_mallballoon: Texture2D,
    pub obstacle_soccerball: Texture2D,
    pub obstacle_boxbot: Texture2D,
    pub obstacle_infected_amy: Texture2D,
    pub obstacle_infected_alicem1: Texture2D,
    pub obstacle_infected_alice3: Texture2D,
    pub obstacle_infected_alice4: Texture2D,
    pub obstacle_infected_boxbot: Texture2D,
    pub obstacle_amy: Texture2D,
    pub obstacle_alicem1: Texture2D,
    pub obstacle_alice3: Texture2D,
    pub obstacle_alice4: Texture2D,
    pub obstacle_shadow: Texture2D,

    pub aurora_purple: Texture2D,
    pub aurora_green: Texture2D,
    pub heart: Texture2D,
    pub virus_green: Texture2D,
    pub virus_purple: Texture2D,
    pub boss_virus: Texture2D,

    pub bg_sky: Texture2D,
    pub bg_stars: Texture2D,
    pub bg_far: Texture2D,
    pub bg_mid: Texture2D,
    pub bg_floor: Texture2D,

    pub stage_store: StageBg,
    pub stage_street: StageBg,
    pub stage_techpark: StageBg,
    pub stage_highway: StageBg,
    pub stage_ansan: StageBg,
    pub stage_office: StageBg,
    pub stage_factory: StageBg,

    pub sfx_jump: macroquad::audio::Sound,
    pub sfx_hit: macroquad::audio::Sound,
    pub sfx_pickup: macroquad::audio::Sound,
    pub sfx_dash: macroquad::audio::Sound,
    pub sfx_smash: macroquad::audio::Sound,
    pub sfx_heart: macroquad::audio::Sound,
    pub sfx_bgm: macroquad::audio::Sound,
    pub sfx_beep: macroquad::audio::Sound,
    pub sfx_whoosh: macroquad::audio::Sound,
}

pub struct StageBg {
    pub far: Texture2D,
    pub mid: Texture2D,
    pub floor: Texture2D,
    /// Optional extra far-layer patterns. Render cycles through
    /// `[base, ...far_variants]` per tile so stages with multiple
    /// shopfronts (store) or landmarks (ansan) don't visibly repeat.
    pub far_variants: Vec<Texture2D>,
    /// Optional extra mid-layer variants used in sync with `far_variants`.
    pub mid_variants: Vec<Texture2D>,
}

#[derive(Debug)]
pub struct LoadError {
    pub which: String,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing bundled asset: {}", self.which)
    }
}

fn tex(name: &str) -> Result<Texture2D, LoadError> {
    let bytes = unscramble(name).ok_or_else(|| LoadError { which: name.to_string() })?;
    let t = Texture2D::from_file_with_format(&bytes, None);
    t.set_filter(FilterMode::Nearest);
    Ok(t)
}

/// Load the three-layer parallax background for a stage. `far_variants`
/// and `mid_variants` are loaded by filename convention: `bg_<key>_<layer>_v<N>.png`
/// starting at v2. Missing files silently stop the lookup so stages only
/// need to ship as many variants as they have hand-drawn.
fn load_stage(key: &str) -> Result<StageBg, LoadError> {
    let mut far_variants: Vec<Texture2D> = Vec::new();
    for n in 2..=8 {
        let name = format!("bg_{key}_far_v{n}.png");
        match tex(&name) {
            Ok(t) => far_variants.push(t),
            Err(_) => break,
        }
    }
    let mut mid_variants: Vec<Texture2D> = Vec::new();
    for n in 2..=8 {
        let name = format!("bg_{key}_mid_v{n}.png");
        match tex(&name) {
            Ok(t) => mid_variants.push(t),
            Err(_) => break,
        }
    }
    Ok(StageBg {
        far: tex(&format!("bg_{key}_far.png"))?,
        mid: tex(&format!("bg_{key}_mid.png"))?,
        floor: tex(&format!("bg_{key}_floor.png"))?,
        far_variants,
        mid_variants,
    })
}

async fn snd(name: &str) -> Result<macroquad::audio::Sound, LoadError> {
    let bytes = unscramble(name).ok_or_else(|| LoadError { which: name.to_string() })?;
    macroquad::audio::load_sound_from_bytes(&bytes)
        .await
        .map_err(|_| LoadError { which: name.to_string() })
}

pub async fn load_all() -> Result<AssetHandles, LoadError> {
    Ok(AssetHandles {
        edie_run: tex("edie_run.png")?,
        edie_jump: tex("edie_jump.png")?,
        edie_duck: tex("edie_duck.png")?,
        edie_dash: tex("edie_dash.png")?,
        edie_hit: tex("edie_hit.png")?,
        edie_shadow: tex("edie_shadow.png")?,

        edie_run_anim: tex("edie_run_anim.png")?,
        edie_happy_run: tex("edie_happy_run.png")?,
        edie_static_run: tex("edie_static_run.png")?,
        edie_title_idle: tex("edie_title_idle.png")?,
        edie_sad_alt: tex("edie_sad_alt.png")?,
        edie_sleepy: tex("edie_sleepy.png")?,
        edie_hit_anim: tex("edie_hit_anim.png")?,
        edie_look: tex("edie_look.png")?,
        edie_gameover_anim: tex("edie_gameover_anim.png")?,
        edie_blink_alt: tex("edie_blink_alt.png")?,
        edie_cheer_anim: tex("edie_cheer_anim.png")?,

        obstacle_coffee: tex("obstacle_coffee.png")?,
        obstacle_cart: tex("obstacle_cart.png")?,
        obstacle_cone: tex("obstacle_cone.png")?,
        obstacle_sign: tex("obstacle_sign.png")?,
        obstacle_cat_orange: tex("obstacle_cat_orange.png")?,
        obstacle_cat_white: tex("obstacle_cat_white.png")?,
        obstacle_car: tex("obstacle_car.png")?,
        obstacle_car_anim: tex("obstacle_car_anim.png")?,
        obstacle_truck: tex("obstacle_truck.png")?,
        obstacle_bus: tex("obstacle_bus.png")?,
        obstacle_taxi: tex("obstacle_taxi.png")?,
        obstacle_sportscar: tex("obstacle_sportscar.png")?,
        obstacle_sportscar_anim: tex("obstacle_sportscar_anim.png")?,
        obstacle_deer: tex("obstacle_deer.png")?,
        obstacle_balloon: tex("obstacle_balloon.png")?,
        obstacle_pigeon: tex("obstacle_pigeon.png")?,
        obstacle_mallballoon: tex("obstacle_mallballoon.png")?,
        obstacle_soccerball: tex("obstacle_soccerball.png")?,
        obstacle_boxbot: tex("obstacle_boxbot.png")?,
        obstacle_infected_amy: tex("obstacle_infected_amy.png")?,
        obstacle_infected_alicem1: tex("obstacle_infected_alicem1.png")?,
        obstacle_infected_alice3: tex("obstacle_infected_alice3.png")?,
        obstacle_infected_alice4: tex("obstacle_infected_alice4.png")?,
        obstacle_infected_boxbot: tex("obstacle_infected_boxbot.png")?,
        obstacle_amy: tex("obstacle_amy.png")?,
        obstacle_alicem1: tex("obstacle_alicem1.png")?,
        obstacle_alice3: tex("obstacle_alice3.png")?,
        obstacle_alice4: tex("obstacle_alice4.png")?,
        obstacle_shadow: tex("edie_shadow.png")?,

        aurora_purple: tex("aurora_purple.png")?,
        aurora_green: tex("aurora_green.png")?,
        heart: tex("heart.png")?,
        virus_green: tex("virus_green.png")?,
        virus_purple: tex("virus_purple.png")?,
        boss_virus: tex("boss_virus.png")?,

        bg_sky: tex("bg_sky.png")?,
        bg_stars: tex("bg_stars.png")?,
        bg_far: tex("bg_far.png")?,
        bg_mid: tex("bg_mid.png")?,
        bg_floor: tex("bg_floor.png")?,

        stage_store: load_stage("store")?,
        stage_street: load_stage("street")?,
        stage_techpark: load_stage("techpark")?,
        stage_highway: load_stage("highway")?,
        stage_ansan: load_stage("ansan")?,
        stage_office: load_stage("office")?,
        stage_factory: load_stage("factory")?,

        sfx_jump: snd("sfx_jump.wav").await?,
        sfx_hit: snd("sfx_hit.wav").await?,
        sfx_pickup: snd("sfx_pickup.wav").await?,
        sfx_dash: snd("sfx_dash.wav").await?,
        sfx_smash: snd("sfx_smash.wav").await?,
        sfx_heart: snd("sfx_heart.wav").await?,
        sfx_bgm: snd("sfx_bgm.wav").await?,
        sfx_beep: snd("sfx_beep.wav").await?,
        sfx_whoosh: snd("sfx_whoosh.wav").await?,
    })
}
