//! EDIE Runner - macroquad entry point. See spec §4.2 for the loop shape.

use edie_runner::assets::{load_all, AssetHandles};
use edie_runner::game::state::Game;
use edie_runner::platform::input::{InputSource, MacroquadInput};
use edie_runner::platform::storage::BrowserStorage;
use edie_runner::platform::visibility::VisibilityTracker;
use edie_runner::render::camera::Camera;
use edie_runner::render::sprites::{
    boss_touch_buttons, draw_aurora, draw_boss_intro, draw_boss_mode, draw_countdown,
    draw_effects, draw_heart_pickup, draw_hit_flash, draw_obstacle, draw_player,
    draw_stage_wipe, draw_tier_banner, draw_touch_buttons, draw_vignette,
    logical_rect_to_screen, name_entry_slot_rects, name_entry_touch_buttons,
    pause_touch_button, play_touch_buttons,
};
use edie_runner::game::state::GameState;
use edie_runner::platform::input::Action;
use edie_runner::render::ui::{
    draw_background, draw_ending, draw_help, draw_hud, draw_name_entry, draw_overlay,
    draw_story,
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
    let mut storage = BrowserStorage::new();
    let mut input = MacroquadInput::new();
    let mut visibility = VisibilityTracker::new();
    let mut step = FixedStep::new();
    let initial_seed = (get_time() * 1000.0) as u64;
    let mut game = Game::new(initial_seed, &storage);

    // Merge remote leaderboard (prefetched by JS on page load) so
    // scores from other devices / sessions are visible immediately.
    if let Some(json) = storage.remote_leaderboard_json() {
        game.leaderboard.merge_remote(&json);
    }
    let mut remote_synced = false;

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

    // Skip the first many render frames after loading: the huge frame_time
    // from asset init + first-draw GPU pipeline compilation can otherwise
    // trip the visibility tracker and pause the game the instant it starts.
    let mut warmup_frames = 30u32;
    // BGM: started once after the first user interaction (mobile autoplay
    // policy requires a gesture before audio plays).
    let mut bgm_started = false;
    // Track countdown integer step so we only emit one beep per second.
    let mut last_countdown_step: i32 = -1;
    let mut prev_stage_wipe_active = false;
    // Track whether the player tapped any interactive area -- used as a
    // "confirm" on Title / GameOver / Paused when no on-screen JUMP button
    // was used.
    let mut was_touching = false;
    // Pre-allocated buffer for touch positions — cleared each frame,
    // never reallocated (avoids per-frame heap allocation).
    let mut touch_points: Vec<(f32, f32)> = Vec::with_capacity(8);

    loop {
        let raw_frame_time = get_frame_time();
        let frame_time = if warmup_frames > 0 {
            warmup_frames -= 1;
            DT
        } else {
            raw_frame_time
        };

        if let Some(visible) = visibility.observe(frame_time) {
            game.on_visibility_change(visible);
        }

        // Touch sampling -- works on mobile (multi-touch) and desktop mouse.
        let cam_for_touch = Camera::new(screen_width(), screen_height());
        touch_points.clear();
        // touch_taps: only newly-started touches (for one-shot buttons).
        let mut touch_taps: Vec<(f32, f32)> = Vec::new();
        for t in touches() {
            touch_points.push((t.position.x, t.position.y));
            if matches!(t.phase, macroquad::input::TouchPhase::Started) {
                touch_taps.push((t.position.x, t.position.y));
            }
        }
        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            touch_points.push((mx, my));
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            touch_taps.push((mx, my));
        }

        // Touch -> button state mapping
        let mut touch_jump = false;
        let mut touch_duck = false;
        let mut touch_dash = false;
        let mut touch_left = false;
        let mut touch_right = false;

        let hit =
            |(x, y): (f32, f32), (rx, ry, rw, rh): (f32, f32, f32, f32)| -> bool {
                x >= rx && x <= rx + rw && y >= ry && y <= ry + rh
            };

        // One-shot touch actions (pause, name entry)
        let mut extra_actions: Vec<Action> = Vec::new();

        if matches!(game.state, GameState::BossFight) {
            let btns = boss_touch_buttons();
            for t in &touch_points {
                if hit(*t, logical_rect_to_screen(btns[0].logical_rect, &cam_for_touch)) {
                    touch_left = true;
                }
                if hit(*t, logical_rect_to_screen(btns[1].logical_rect, &cam_for_touch)) {
                    touch_right = true;
                }
            }
            // Pause button (one-shot tap)
            let pause_btn = pause_touch_button();
            for t in &touch_taps {
                if hit(*t, logical_rect_to_screen(pause_btn.logical_rect, &cam_for_touch)) {
                    extra_actions.push(Action::Pause);
                }
            }
        } else if matches!(game.state, GameState::Playing) {
            let btns = play_touch_buttons();
            for t in &touch_points {
                if hit(*t, logical_rect_to_screen(btns[0].logical_rect, &cam_for_touch)) {
                    touch_duck = true;
                }
                if hit(*t, logical_rect_to_screen(btns[1].logical_rect, &cam_for_touch)) {
                    touch_jump = true;
                }
                if hit(*t, logical_rect_to_screen(btns[2].logical_rect, &cam_for_touch)) {
                    touch_dash = true;
                }
            }
            // Pause button (one-shot tap)
            let pause_btn = pause_touch_button();
            for t in &touch_taps {
                if hit(*t, logical_rect_to_screen(pause_btn.logical_rect, &cam_for_touch)) {
                    extra_actions.push(Action::Pause);
                }
            }
        } else if matches!(game.state, GameState::NameEntry) {
            // NameEntry touch: buttons [<] [UP] [DN] [>] [OK]
            let ne_btns = name_entry_touch_buttons();
            let ne_actions = [
                Action::NamePrev, Action::NameUp, Action::NameDown,
                Action::NameNext, Action::NameCommit,
            ];
            for t in &touch_taps {
                for (i, btn) in ne_btns.iter().enumerate() {
                    if hit(*t, logical_rect_to_screen(btn.logical_rect, &cam_for_touch)) {
                        extra_actions.push(ne_actions[i]);
                    }
                }
                // Tap on character slot → select it + cycle letter up
                let slots = name_entry_slot_rects();
                for (i, slot) in slots.iter().enumerate() {
                    if hit(*t, logical_rect_to_screen(*slot, &cam_for_touch)) {
                        game.name_cursor = i;
                        extra_actions.push(Action::NameUp);
                    }
                }
            }
        } else {
            // Title / GameOver / Paused / Help / Story / Ending: any tap acts
            // as a confirm (fires a Jump press pulse exactly once per tap).
            let touching = !touch_points.is_empty();
            if touching && !was_touching {
                touch_jump = true;
            }
        }

        input.set_touch_buttons(touch_jump, touch_duck, touch_dash, touch_left, touch_right);
        was_touching = !touch_points.is_empty();

        let mut actions = input.poll();
        actions.extend(extra_actions);
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
        // During the boss intro / fight the run may have been force-started
        // via debug (score=0). Always pin the background to the Factory so
        // the player never sees a jarring DepartmentStore mall behind the
        // Mungchi boss.
        let bg_stage = if matches!(
            game.state,
            GameState::BossFight
        ) || game.boss_intro.is_some()
        {
            edie_runner::game::difficulty::Stage::AeiRobotFactory
        } else {
            game.world.current_stage()
        };
        draw_background(
            &game.world.background,
            &assets,
            bg_stage,
            day_phase,
            game.world.landmark.as_ref(),
            &cam,
        );

        // Auto-cue: countdown beeps + stage-wipe whoosh. We do this in
        // main.rs (not the simulation layer) because they are pure audio
        // feedback driven by transient frame state.
        if matches!(game.state, GameState::Playing) && game.countdown_remaining > 0.0 {
            // Emit a beep on every full integer crossing (3, 2, 1, GO).
            let step = game.countdown_remaining.ceil() as i32;
            if step != last_countdown_step {
                last_countdown_step = step;
                game.world.effects.sfx(edie_runner::game::effects::SfxCue::Beep);
            }
        } else {
            last_countdown_step = -1;
        }
        let stage_wipe_active = game.world.effects.stage_wipe.is_some();
        if stage_wipe_active && !prev_stage_wipe_active {
            game.world.effects.sfx(edie_runner::game::effects::SfxCue::Whoosh);
        }
        prev_stage_wipe_active = stage_wipe_active;

        // Drain SFX queue and play cued sounds. We use `play_sound_once`
        // for one-shots because in macroquad's wasm audio backend a
        // sequence of `play_sound` calls while a looping track is active
        // (the BGM) occasionally drops short one-shots like the jump
        // sfx. `play_sound_once` allocates a fresh voice every call and
        // plays reliably alongside the looping BGM.
        // Drain SFX directly — no intermediate Vec allocation.
        for cue in game.world.effects.sfx_queue.drain(..) {
            let sound = match cue {
                edie_runner::game::effects::SfxCue::Jump => &assets.sfx_jump,
                edie_runner::game::effects::SfxCue::Hit => &assets.sfx_hit,
                edie_runner::game::effects::SfxCue::Pickup => &assets.sfx_pickup,
                edie_runner::game::effects::SfxCue::Dash => &assets.sfx_dash,
                edie_runner::game::effects::SfxCue::Smash => &assets.sfx_smash,
                edie_runner::game::effects::SfxCue::Heart => &assets.sfx_heart,
                edie_runner::game::effects::SfxCue::Beep => &assets.sfx_beep,
                edie_runner::game::effects::SfxCue::Whoosh => &assets.sfx_whoosh,
            };
            macroquad::audio::play_sound_once(sound);
        }

        // Start BGM as soon as we leave the Title screen for the first
        // time. By then the player has definitely interacted (mobile
        // autoplay policy is satisfied). The track loops forever after.
        if !bgm_started && !matches!(game.state, GameState::Title) {
            macroquad::audio::play_sound(
                &assets.sfx_bgm,
                macroquad::audio::PlaySoundParams {
                    looped: true,
                    volume: 0.16,
                },
            );
            bgm_started = true;
        }
        let elapsed = game.world.elapsed;
        let speed_for_telegraph = game.world.current_speed();
        // Robots are virus-infected in stages leading up to the boss fight.
        let infected = matches!(
            game.world.current_stage(),
            edie_runner::game::difficulty::Stage::Ansan
                | edie_runner::game::difficulty::Stage::AeiRobotOffice
                | edie_runner::game::difficulty::Stage::AeiRobotFactory
        );
        for o in &game.world.obstacles.obstacles {
            if o.alive && o.x > -100.0 && o.x < 1400.0 {
                draw_obstacle(o, &assets, elapsed, speed_for_telegraph, infected, &cam);
            }
        }
        for s in &game.world.pickups.stones {
            if !s.collected && s.x > -60.0 && s.x < 1360.0 {
                draw_aurora(s, &assets, elapsed, &cam);
            }
        }
        for h in &game.world.pickups.hearts {
            if !h.collected && h.x > -60.0 && h.x < 1360.0 {
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
        // Stage wipe (Metal Slug style)
        draw_stage_wipe(&game.world.effects, &cam);
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
        if let Some(intro) = game.boss_intro.as_ref() {
            draw_boss_intro(intro, &assets, &cam);
        }

        // Boss fight overlay drawn on top of the normal world
        if matches!(game.state, GameState::BossFight) {
            if let Some(ref boss) = game.boss {
                draw_boss_mode(boss, &assets, &cam);
            }
            // Touch controls: left/right dodge buttons + pause
            let btns = boss_touch_buttons();
            draw_touch_buttons(
                &btns,
                &[touch_left, touch_right],
                &cam,
            );
            draw_touch_buttons(&[pause_touch_button()], &[false], &cam);
        }

        // In-game touch buttons (only during actual Playing state)
        if matches!(game.state, GameState::Playing) && game.countdown_remaining <= 0.0 {
            let btns = play_touch_buttons();
            // Show dash as pressed only when we actually had >=1 aurora
            let dash_armed = game.world.dash.aurora > 0;
            draw_touch_buttons(
                &btns,
                &[touch_duck, touch_jump, dash_armed && touch_dash],
                &cam,
            );
            draw_touch_buttons(&[pause_touch_button()], &[false], &cam);
        }

        // Help / Story / Ending / NameEntry screens drawn on top of everything
        match game.state {
            GameState::Help => draw_help(&assets, wall_time, &cam),
            GameState::Story => {
                let t = wall_time - game.story_start_time;
                draw_story(t, &assets, &cam);
            }
            GameState::Ending => draw_ending(&assets, wall_time, game.last_ending_true, &cam),
            GameState::NameEntry => {
                draw_name_entry(&game, wall_time, &cam);
                // Touch buttons for name entry
                let ne_btns = name_entry_touch_buttons();
                draw_touch_buttons(&ne_btns, &[false; 5], &cam);
            }
            _ => {}
        }

        // Lazy remote leaderboard sync: keep trying once per second
        // until the prefetched data arrives, then stop polling.
        if !remote_synced {
            if let Some(json) = storage.remote_leaderboard_json() {
                game.leaderboard.merge_remote(&json);
                remote_synced = true;
            }
        }

        next_frame().await;
    }
}
