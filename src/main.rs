//! EDIE Runner - macroquad entry point. See spec §4.2 for the loop shape.

use edie_runner::assets::{load_all, AssetHandles};
use edie_runner::game::state::Game;
use edie_runner::platform::input::{InputSource, MacroquadInput};
use edie_runner::platform::storage::InMemoryStorage;
use edie_runner::platform::visibility::VisibilityTracker;
use edie_runner::render::camera::Camera;
use edie_runner::render::sprites::{
    draw_aurora, draw_boss_intro, draw_boss_mode, draw_countdown, draw_effects, draw_heart_pickup,
    draw_hit_flash, draw_obstacle, draw_player, draw_tier_banner, draw_vignette,
};
use edie_runner::game::state::GameState;
use edie_runner::platform::input::Action;
use edie_runner::render::ui::{
    draw_background, draw_ending, draw_help, draw_hud, draw_overlay, draw_story,
};
use edie_runner::time::{FixedStep, DT};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "EDIE Runner".to_string(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

async fn show_loading_then_load() -> Result<AssetHandles, String> {
    // Single render of the loading screen, then synchronously await assets.
    clear_background(Color::new(0.96, 0.94, 0.89, 1.0));
    let msg = "Loading EDIE...";
    let size = 32.0;
    let dims = measure_text(msg, None, size as u16, 1.0);
    draw_text(
        msg,
        (screen_width() - dims.width) * 0.5,
        screen_height() * 0.5,
        size,
        BLACK,
    );
    next_frame().await;

    load_all().await.map_err(|e| e.to_string())
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut storage = InMemoryStorage::new();
    let mut input = MacroquadInput::new();
    let mut visibility = VisibilityTracker::new();
    let mut step = FixedStep::new();
    let initial_seed = (get_time() * 1000.0) as u64;
    let mut game = Game::new(initial_seed, &storage);

    let assets = match show_loading_then_load().await {
        Ok(a) => a,
        Err(msg) => {
            // Render the failure forever - never enter the game loop.
            loop {
                clear_background(Color::new(0.96, 0.94, 0.89, 1.0));
                let dims = measure_text(&msg, None, 28, 1.0);
                draw_text(
                    &msg,
                    (screen_width() - dims.width) * 0.5,
                    screen_height() * 0.5,
                    28.0,
                    RED,
                );
                next_frame().await;
            }
        }
    };

    loop {
        let frame_time = get_frame_time();

        if let Some(visible) = visibility.observe(frame_time) {
            game.on_visibility_change(visible);
        }

        let actions = input.poll();
        for a in actions {
            // Stamp story start time when entering Story state.
            let was_title_or_gameover = matches!(
                game.state,
                GameState::Title | GameState::GameOver
            );
            game.handle(a, &mut storage);
            if was_title_or_gameover && matches!(game.state, GameState::Story) {
                game.story_start_time = get_time() as f32;
            }
            // Story auto-skip handled below in render section
            if matches!(game.state, GameState::Story) && matches!(a, Action::Back) {
                game.state = GameState::Title;
            }
        }

        // Story auto-finish: return to Title after STORY_DURATION seconds.
        if matches!(game.state, GameState::Story) {
            let now = get_time() as f32;
            if now - game.story_start_time
                >= edie_runner::game::state::STORY_DURATION
            {
                game.state = GameState::Title;
            }
        }

        let n = step.advance(frame_time);
        for _ in 0..n {
            game.update(DT, &mut storage);
        }

        clear_background(Color::new(0.96, 0.94, 0.89, 1.0));
        let wall_time = get_time() as f32;
        // Apply screen shake offset to the camera.
        let (shake_ox, shake_oy) = game.world.effects.shake_offset(wall_time);
        let cam = Camera::new(screen_width(), screen_height())
            .with_shake(shake_ox * screen_width() / 1280.0, shake_oy * screen_width() / 1280.0);
        // Day/night cycles only on the Title screen (for visual flavor).
        // During gameplay and other states, the background stays in a fixed
        // daylight tint so readability is consistent.
        let day_phase = if matches!(game.state, GameState::Title) {
            // Wall-clock driven cycle: 90 seconds per full day/night loop.
            ((wall_time / 90.0) % 1.0 + 1.0) % 1.0
        } else {
            0.25 // fixed noon-ish daylight
        };
        draw_background(&game.world.background, &assets, game.world.current_stage(), day_phase, &cam);

        // Drain SFX queue and play cued sounds.
        if !game.world.effects.sfx_queue.is_empty() {
            let cues: Vec<_> = game.world.effects.sfx_queue.drain(..).collect();
            for cue in cues {
                let sound = match cue {
                    edie_runner::game::effects::SfxCue::Jump => &assets.sfx_jump,
                    edie_runner::game::effects::SfxCue::Hit => &assets.sfx_hit,
                    edie_runner::game::effects::SfxCue::Pickup => &assets.sfx_pickup,
                    edie_runner::game::effects::SfxCue::Dash => &assets.sfx_dash,
                    edie_runner::game::effects::SfxCue::Smash => &assets.sfx_smash,
                    edie_runner::game::effects::SfxCue::Heart => &assets.sfx_heart,
                };
                macroquad::audio::play_sound_once(sound);
            }
        }
        let elapsed = game.world.elapsed;
        let speed_for_telegraph = game.world.current_speed();
        for o in &game.world.obstacles.obstacles {
            if o.alive {
                draw_obstacle(o, &assets, elapsed, speed_for_telegraph, &cam);
            }
        }
        for s in &game.world.pickups.stones {
            if !s.collected {
                draw_aurora(s, &assets, elapsed, &cam);
            }
        }
        for h in &game.world.pickups.hearts {
            if !h.collected {
                draw_heart_pickup(h, &assets, elapsed, &cam);
            }
        }
        // Player flickers during HP invuln
        let invuln_visible =
            game.world.hp_invuln <= 0.0 || ((wall_time * 16.0).sin() > 0.0);
        if invuln_visible {
            draw_player(&game.world.player, &game.world.dash, &assets, elapsed, &cam);
        }
        // Particles + score popups on top of world, under HUD
        draw_effects(&game.world.effects, &cam);
        // Tier banner
        draw_tier_banner(&game.world.effects, &cam);
        // Speed-tier vignette
        draw_vignette(game.world.current_speed(), &cam);
        // Hit flash
        draw_hit_flash(&game.world.effects, &cam);
        // Countdown overlay (only during Playing with pending countdown)
        if matches!(game.state, GameState::Playing) && game.countdown_remaining > 0.0 {
            draw_countdown(game.countdown_remaining, &cam);
        }
        draw_hud(
            &game.world.score,
            &game.world.dash,
            game.world.hp,
            &assets,
            elapsed,
            &cam,
        );
        draw_overlay(&game, &assets, wall_time, &cam);

        // Boss intro cinematic
        if game.boss_intro_remaining > 0.0 {
            draw_boss_intro(game.boss_intro_remaining, &assets, &cam);
        }

        // Boss fight overlay drawn on top of the normal world
        if matches!(game.state, GameState::BossFight) {
            if let Some(ref boss) = game.boss {
                draw_boss_mode(boss, &assets, &cam);
            }
        }

        // Help / Story / Ending screens drawn on top of everything else
        match game.state {
            GameState::Help => draw_help(&assets, wall_time, &cam),
            GameState::Story => {
                let t = wall_time - game.story_start_time;
                draw_story(t, &assets, &cam);
            }
            GameState::Ending => draw_ending(&assets, wall_time, &cam),
            _ => {}
        }

        next_frame().await;
    }
}
