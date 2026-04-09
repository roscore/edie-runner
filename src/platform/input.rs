//! Input abstraction. See spec §4.5.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Jump,
    JumpRelease,
    Duck,
    DuckRelease,
    Dash,
    DashRelease,
    Confirm,
    Pause,
    OpenHelp,
    OpenStory,
    Back,
    MoveLeft,
    MoveRight,
    MoveLeftRelease,
    MoveRightRelease,
    DebugBoss,
    NameUp,
    NameDown,
    NameNext,
    NamePrev,
    NameCommit,
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

#[cfg(feature = "graphics")]
use macroquad::prelude::*;

/// Production input source: reads macroquad keyboard + touch each frame.
#[cfg(feature = "graphics")]
pub struct MacroquadInput {
    jump_was_down: bool,
    duck_was_down: bool,
    left_was_down: bool,
    right_was_down: bool,
    dash_was_down: bool,
    touch_jump: bool,
    touch_duck: bool,
    touch_dash: bool,
    touch_left: bool,
    touch_right: bool,
}

#[cfg(feature = "graphics")]
impl MacroquadInput {
    pub fn new() -> Self {
        Self {
            jump_was_down: false,
            duck_was_down: false,
            left_was_down: false,
            right_was_down: false,
            dash_was_down: false,
            touch_jump: false,
            touch_duck: false,
            touch_dash: false,
            touch_left: false,
            touch_right: false,
        }
    }

    /// Called by main loop each frame with the current touch-button state.
    pub fn set_touch_buttons(
        &mut self,
        jump: bool,
        duck: bool,
        dash: bool,
        left: bool,
        right: bool,
    ) {
        self.touch_jump = jump;
        self.touch_duck = duck;
        self.touch_dash = dash;
        self.touch_left = left;
        self.touch_right = right;
    }
}

#[cfg(feature = "graphics")]
impl Default for MacroquadInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "graphics")]
impl InputSource for MacroquadInput {
    fn poll(&mut self) -> Vec<Action> {
        let mut out = Vec::new();
        let jump_now = is_key_down(KeyCode::Space)
            || is_key_down(KeyCode::Up)
            || self.touch_jump;
        let duck_now = is_key_down(KeyCode::Down) || self.touch_duck;

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
        // Dash is now hold-to-extend. Fire Dash on any down-edge (key or
        // touch) and DashRelease on the up-edge.
        let dash_now = is_key_down(KeyCode::LeftShift)
            || is_key_down(KeyCode::RightShift)
            || self.touch_dash;
        if dash_now && !self.dash_was_down {
            out.push(Action::Dash);
        }
        if !dash_now && self.dash_was_down {
            out.push(Action::DashRelease);
        }
        self.dash_was_down = dash_now;
        if is_key_pressed(KeyCode::P) {
            out.push(Action::Pause);
        }

        // Horizontal movement for boss mode (keys or touch)
        let left_now = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) || self.touch_left;
        let right_now = is_key_down(KeyCode::Right)
            || is_key_down(KeyCode::D)
            || self.touch_right;
        if left_now && !self.left_was_down {
            out.push(Action::MoveLeft);
        }
        if !left_now && self.left_was_down {
            out.push(Action::MoveLeftRelease);
        }
        if right_now && !self.right_was_down {
            out.push(Action::MoveRight);
        }
        if !right_now && self.right_was_down {
            out.push(Action::MoveRightRelease);
        }
        self.left_was_down = left_now;
        self.right_was_down = right_now;
        if is_key_pressed(KeyCode::H) {
            out.push(Action::OpenHelp);
        }
        if is_key_pressed(KeyCode::T) {
            out.push(Action::OpenStory);
        }
        if is_key_pressed(KeyCode::Escape) {
            out.push(Action::Back);
        }
        if is_key_pressed(KeyCode::B) {
            out.push(Action::DebugBoss);
        }

        // Dash touch is one-shot — consumed after this poll.
        self.touch_dash = false;

        self.jump_was_down = jump_now;
        self.duck_was_down = duck_now;
        out
    }
}
