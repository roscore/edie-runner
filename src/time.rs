//! Fixed-timestep accumulator. See spec §4.2.

pub const DT: f32 = 1.0 / 120.0;
const MAX_FRAME: f32 = 0.1;

#[derive(Debug, Default)]
pub struct FixedStep {
    accumulator: f32,
}

impl FixedStep {
    pub fn new() -> Self {
        Self { accumulator: 0.0 }
    }

    /// Reset the accumulator to zero. Call at state transitions that shouldn't
    /// inherit time debt from the previous state (e.g. start of a new run).
    pub fn reset(&mut self) {
        self.accumulator = 0.0;
    }

    /// Feed a real-time delta and return how many fixed steps to run this frame.
    pub fn advance(&mut self, frame_time: f32) -> u32 {
        let clamped = frame_time.clamp(0.0, MAX_FRAME);
        self.accumulator += clamped;
        let mut steps = 0;
        while self.accumulator >= DT {
            self.accumulator -= DT;
            steps += 1;
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_frame_time_yields_zero_steps() {
        let mut s = FixedStep::new();
        assert_eq!(s.advance(0.0), 0);
    }

    #[test]
    fn one_dt_yields_one_step() {
        let mut s = FixedStep::new();
        assert_eq!(s.advance(DT), 1);
    }

    #[test]
    fn small_frames_accumulate_then_fire() {
        let mut s = FixedStep::new();
        assert_eq!(s.advance(DT / 2.0), 0);
        assert_eq!(s.advance(DT / 2.0), 1);
    }

    #[test]
    fn large_frame_clamped_no_death_spiral() {
        let mut s = FixedStep::new();
        let steps = s.advance(5.0);
        assert!(steps <= (MAX_FRAME / DT).ceil() as u32 + 1);
    }
}
