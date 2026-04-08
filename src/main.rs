//! EDIE Runner — macroquad entry point. See spec §4.2 for the loop shape.

use edie_runner::assets::{load_all, AssetHandles};
use edie_runner::game::state::Game;
use edie_runner::platform::input::{InputSource, MacroquadInput};
use edie_runner::platform::storage::InMemoryStorage;
use edie_runner::platform::visibility::VisibilityTracker;
use edie_runner::render::camera::Camera;
use edie_runner::render::sprites::{draw_aurora, draw_heart_pickup, draw_obstacle, draw_player};
use edie_runner::render::ui::{draw_background, draw_hud, draw_overlay};
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
    let msg = "Loading EDIE…";
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
            // Render the failure forever — never enter the game loop.
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
            game.handle(a, &mut storage);
        }

        let n = step.advance(frame_time);
        for _ in 0..n {
            game.update(DT, &mut storage);
        }

        clear_background(Color::new(0.96, 0.94, 0.89, 1.0));
        let cam = Camera::new(screen_width(), screen_height());
        let wall_time = get_time() as f32;
        draw_background(&game.world.background, &assets, wall_time, &cam);
        let elapsed = game.world.elapsed;
        for o in &game.world.obstacles.obstacles {
            if o.alive {
                draw_obstacle(o, &assets, elapsed, &cam);
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
        draw_hud(
            &game.world.score,
            &game.world.dash,
            game.world.hp,
            &assets,
            elapsed,
            &cam,
        );
        draw_overlay(&game, &assets, wall_time, &cam);

        next_frame().await;
    }
}
