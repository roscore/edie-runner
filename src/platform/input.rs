//! Input abstraction. See spec §4.5.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Jump,
    JumpRelease,
    Duck,
    DuckRelease,
    Dash,
    Confirm,
    Pause,
}

pub trait InputSource {
    /// Drain pending actions for this frame.
    fn poll(&mut self) -> Vec<Action>;
}

/// Test-only input source.
#[derive(Default)]
pub struct ScriptedInput {
    script: Vec<(u32, Action)>,
    frame: u32,
}

impl ScriptedInput {
    pub fn new(script: Vec<(u32, Action)>) -> Self {
        Self { script, frame: 0 }
    }
}

impl InputSource for ScriptedInput {
    fn poll(&mut self) -> Vec<Action> {
        let now = self.frame;
        self.frame += 1;
        self.script
            .iter()
            .filter(|(f, _)| *f == now)
            .map(|(_, a)| *a)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_script_yields_nothing() {
        let mut s = ScriptedInput::new(vec![]);
        assert!(s.poll().is_empty());
        assert!(s.poll().is_empty());
    }

    #[test]
    fn fires_at_correct_frame() {
        let mut s = ScriptedInput::new(vec![(0, Action::Jump), (2, Action::Dash)]);
        assert_eq!(s.poll(), vec![Action::Jump]);
        assert!(s.poll().is_empty());
        assert_eq!(s.poll(), vec![Action::Dash]);
    }

    #[test]
    fn multiple_actions_same_frame() {
        let mut s = ScriptedInput::new(vec![(0, Action::Jump), (0, Action::Dash)]);
        let actions = s.poll();
        assert!(actions.contains(&Action::Jump));
        assert!(actions.contains(&Action::Dash));
    }
}

use macroquad::prelude::*;

/// Production input source: reads macroquad keyboard each frame.
pub struct MacroquadInput {
    jump_was_down: bool,
    duck_was_down: bool,
}

impl MacroquadInput {
    pub fn new() -> Self {
        Self { jump_was_down: false, duck_was_down: false }
    }
}

impl Default for MacroquadInput {
    fn default() -> Self {
        Self::new()
    }
}

impl InputSource for MacroquadInput {
    fn poll(&mut self) -> Vec<Action> {
        let mut out = Vec::new();
        let jump_now = is_key_down(KeyCode::Space) || is_key_down(KeyCode::Up);
        let duck_now = is_key_down(KeyCode::Down);

        if jump_now && !self.jump_was_down {
            out.push(Action::Jump);
            out.push(Action::Confirm);
        }
        if !jump_now && self.jump_was_down {
            out.push(Action::JumpRelease);
        }
        if duck_now && !self.duck_was_down {
            out.push(Action::Duck);
        }
        if !duck_now && self.duck_was_down {
            out.push(Action::DuckRelease);
        }
        if is_key_pressed(KeyCode::LeftShift) || is_key_pressed(KeyCode::RightShift) {
            out.push(Action::Dash);
        }
        if is_key_pressed(KeyCode::P) {
            out.push(Action::Pause);
        }

        self.jump_was_down = jump_now;
        self.duck_was_down = duck_now;
        out
    }
}
