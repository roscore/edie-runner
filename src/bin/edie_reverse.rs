//! EDIE Battle Reverse — standalone macroquad binary.

use edie_runner::reversi::game::{GameMode, Phase, ReversiGame};
use edie_runner::reversi::render::{draw_reversi, screen_to_cell};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "EDIE Battle Reverse".to_string(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let assets = match edie_runner::assets::load_all().await {
        Ok(a) => a,
        Err(e) => {
            loop {
                clear_background(BLACK);
                draw_text(&format!("Asset load error: {}", e), 40.0, 360.0, 28.0, RED);
                next_frame().await;
            }
        }
    };

    let seed = (get_time() * 1000.0) as u64;
    let mut game = ReversiGame::new(seed);

    loop {
        let dt = get_frame_time().min(0.1);
        let elapsed = get_time() as f32;

        match game.phase {
            Phase::Menu => {
                if is_key_pressed(KeyCode::Key1) { game.start_game(GameMode::VsLocal); }
                else if is_key_pressed(KeyCode::Key2) { game.start_game(GameMode::VsAiEasy); }
                else if is_key_pressed(KeyCode::Key3) { game.start_game(GameMode::VsAiNormal); }
                else if is_key_pressed(KeyCode::Key4) { game.start_game(GameMode::VsAiHard); }
                else if is_key_pressed(KeyCode::Key5) { game.start_game(GameMode::VsAiInsane); }
                // Mouse/touch menu selection
                let menu_click = |my: f32| -> Option<GameMode> {
                    let cam = edie_runner::render::camera::Camera::with_logical(
                        1280.0, 720.0, screen_width(), screen_height());
                    let ly = (my - cam.offset_y) / cam.scale;
                    if ly > 270.0 && ly < 315.0 { Some(GameMode::VsLocal) }
                    else if ly > 320.0 && ly < 365.0 { Some(GameMode::VsAiEasy) }
                    else if ly > 370.0 && ly < 415.0 { Some(GameMode::VsAiNormal) }
                    else if ly > 420.0 && ly < 465.0 { Some(GameMode::VsAiHard) }
                    else if ly > 470.0 && ly < 515.0 { Some(GameMode::VsAiInsane) }
                    else { None }
                };
                // Easter egg: click title to cycle character theme
                let title_hit = |mx: f32, my: f32| -> bool {
                    let cam = edie_runner::render::camera::Camera::with_logical(
                        1280.0, 720.0, screen_width(), screen_height());
                    let lx = (mx - cam.offset_x) / cam.scale;
                    let ly = (my - cam.offset_y) / cam.scale;
                    let (rx, ry, rw, rh) = edie_runner::reversi::render::TITLE_RECT;
                    lx >= rx && lx <= rx + rw && ly >= ry && ly <= ry + rh
                };
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mx, my) = mouse_position();
                    if title_hit(mx, my) {
                        game.theme_index = (game.theme_index + 1) % edie_runner::reversi::render::THEME_COUNT;
                    } else if let Some(mode) = menu_click(my) {
                        game.start_game(mode);
                    }
                }
                for t in touches() {
                    if let macroquad::input::TouchPhase::Started = t.phase {
                        if title_hit(t.position.x, t.position.y) {
                            game.theme_index = (game.theme_index + 1) % edie_runner::reversi::render::THEME_COUNT;
                        } else if let Some(mode) = menu_click(t.position.y) {
                            game.start_game(mode);
                        }
                    }
                }
            }
            Phase::Playing => {
                let (mx, my) = mouse_position();
                game.hover = screen_to_cell(mx, my);
                if is_mouse_button_pressed(MouseButton::Left) {
                    if let Some((r, c)) = screen_to_cell(mx, my) {
                        game.on_cell_click(r, c);
                    }
                }
                for t in touches() {
                    if let macroquad::input::TouchPhase::Started = t.phase {
                        if let Some((r, c)) = screen_to_cell(t.position.x, t.position.y) {
                            game.on_cell_click(r, c);
                        }
                    }
                }
                // Activate powerup with Q key
                if is_key_pressed(KeyCode::Q) {
                    game.activate_powerup();
                }
            }
            Phase::UsingPowerup => {
                let (mx, my) = mouse_position();
                game.hover = screen_to_cell(mx, my);
                if is_mouse_button_pressed(MouseButton::Left) {
                    if let Some((r, c)) = screen_to_cell(mx, my) {
                        game.on_cell_click(r, c);
                    }
                }
                for t in touches() {
                    if let macroquad::input::TouchPhase::Started = t.phase {
                        if let Some((r, c)) = screen_to_cell(t.position.x, t.position.y) {
                            game.on_cell_click(r, c);
                        }
                    }
                }
                if is_key_pressed(KeyCode::Escape) {
                    game.cancel_powerup();
                }
            }
            Phase::GameOver => {
                if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left) {
                    game.phase = Phase::Menu;
                }
                for t in touches() {
                    if let macroquad::input::TouchPhase::Started = t.phase {
                        game.phase = Phase::Menu;
                    }
                }
            }
            Phase::Animating => {}
        }

        game.update(dt);

        if game.phase == Phase::Playing {
            let is_ai_turn = match game.mode {
                GameMode::VsAiEasy | GameMode::VsAiNormal | GameMode::VsAiHard | GameMode::VsAiInsane => {
                    game.board.turn == edie_runner::reversi::board::Side::Alice
                }
                _ => false,
            };
            if is_ai_turn {
                let seed = (elapsed * 10000.0) as u64;
                if let Some((r, c)) = edie_runner::reversi::ai::pick_move(&game.board, game.mode, seed) {
                    game.on_cell_click(r, c);
                }
            }
        }

        draw_reversi(&game, &assets, elapsed);
        next_frame().await;
    }
}
