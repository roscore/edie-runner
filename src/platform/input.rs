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
