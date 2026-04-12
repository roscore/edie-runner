//! EDIE Yut Nori — standalone macroquad binary.

use edie_runner::yut::game::{Phase, YutGame};
use edie_runner::yut::render::{draw_yut, screen_to_board_cell, screen_to_home_piece};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "EDIE Yut Nori".to_string(),
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
    let mut game = YutGame::new(seed);

    loop {
        let dt = get_frame_time().min(0.1);
        let elapsed = get_time() as f32;

        match game.phase {
            Phase::Menu => {
                if is_key_pressed(KeyCode::Key1) { game.start_game(2); }
                else if is_key_pressed(KeyCode::Key2) { game.start_game(3); }
                else if is_key_pressed(KeyCode::Key3) { game.start_game(4); }
                // Mouse/touch menu
                let menu_click = |my: f32| -> Option<usize> {
                    let cam = edie_runner::render::camera::Camera::with_logical(
                        1280.0, 720.0, screen_width(), screen_height());
                    let ly = (my - cam.offset_y) / cam.scale;
                    if ly > 330.0 && ly < 375.0 { Some(2) }
                    else if ly > 380.0 && ly < 425.0 { Some(3) }
                    else if ly > 430.0 && ly < 475.0 { Some(4) }
                    else { None }
                };
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (_, my) = mouse_position();
                    if let Some(n) = menu_click(my) { game.start_game(n); }
                }
                for t in touches() {
                    if let macroquad::input::TouchPhase::Started = t.phase {
                        if let Some(n) = menu_click(t.position.y) { game.start_game(n); }
                    }
                }
            }
            Phase::Throwing => {
                // Q/W to use power cards before throwing
                if is_key_pressed(KeyCode::Q) { game.use_power(0); }
                if is_key_pressed(KeyCode::W) { game.use_power(1); }
                // Tap or space to throw
                let mut do_throw = is_key_pressed(KeyCode::Space);
                if is_mouse_button_pressed(MouseButton::Left) { do_throw = true; }
                for t in touches() {
                    if let macroquad::input::TouchPhase::Started = t.phase { do_throw = true; }
                }
                if do_throw { game.do_throw(); }
            }
            Phase::SelectPiece => {
                // Click/tap a piece to select it
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (mx, my) = mouse_position();
                    if let Some(piece_idx) = find_clicked_piece(mx, my, &game) {
                        game.select_piece(piece_idx);
                    }
                }
                for t in touches() {
                    if let macroquad::input::TouchPhase::Started = t.phase {
                        if let Some(piece_idx) = find_clicked_piece(t.position.x, t.position.y, &game) {
                            game.select_piece(piece_idx);
                        }
                    }
                }
                // Keyboard: 1-4 to select piece index
                if is_key_pressed(KeyCode::Key1) { game.select_piece(0); }
                if is_key_pressed(KeyCode::Key2) { game.select_piece(1); }
                if is_key_pressed(KeyCode::Key3) { game.select_piece(2); }
                if is_key_pressed(KeyCode::Key4) { game.select_piece(3); }
            }
            Phase::SelectPath => {
                if is_key_pressed(KeyCode::Key1) { game.choose_path(true); }
                if is_key_pressed(KeyCode::Key2) { game.choose_path(false); }
                // Touch: top half = shortcut, bottom half = outer
                if is_mouse_button_pressed(MouseButton::Left) {
                    let (_, my) = mouse_position();
                    let cam = edie_runner::render::camera::Camera::with_logical(
                        1280.0, 720.0, screen_width(), screen_height());
                    let ly = (my - cam.offset_y) / cam.scale;
                    if ly < 360.0 { game.choose_path(true); }
                    else { game.choose_path(false); }
                }
                for t in touches() {
                    if let macroquad::input::TouchPhase::Started = t.phase {
                        let cam = edie_runner::render::camera::Camera::with_logical(
                            1280.0, 720.0, screen_width(), screen_height());
                        let ly = (t.position.y - cam.offset_y) / cam.scale;
                        if ly < 360.0 { game.choose_path(true); }
                        else { game.choose_path(false); }
                    }
                }
            }
            Phase::Moving => {}
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
        }

        game.update(dt);
        draw_yut(&game, &assets, elapsed);
        next_frame().await;
    }
}

fn find_clicked_piece(sx: f32, sy: f32, game: &YutGame) -> Option<usize> {
    // Check home pieces first
    if let Some(idx) = screen_to_home_piece(sx, sy, game) {
        return Some(idx);
    }
    // Check board pieces
    if let Some(cell) = screen_to_board_cell(sx, sy) {
        let pi = game.current_player;
        for (qi, piece) in game.players[pi].pieces.iter().enumerate() {
            if piece.pos == cell && piece.is_on_board() && piece.stack > 0 {
                return Some(qi);
            }
        }
    }
    None
}
