//! Approximate visibility tracking. See spec §4.2.
//!
//! We used to flip "invisible" on any single frame > 0.5s, which fires false
//! positives during startup texture warmups, phone rotation, and first-time
//! GPU pipeline compilation. The new tracker:
//!   - Uses a higher threshold (2.0s) so only real "tab backgrounded" pauses
//!     are detected.
//!   - Requires TWO consecutive slow frames before flipping to invisible.
//!   - Stays invisible until a normal frame is observed (instant recovery).

const SLOW_FRAME_THRESHOLD: f32 = 2.0;
const REQUIRED_SLOW_FRAMES: u32 = 2;

pub struct VisibilityTracker {
    pub visible: bool,
    slow_streak: u32,
}

impl Default for VisibilityTracker {
    fn default() -> Self {
        Self { visible: true, slow_streak: 0 }
    }
}

impl VisibilityTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns Some(new_visibility) if the visibility changed this frame.
    pub fn observe(&mut self, frame_time: f32) -> Option<bool> {
        if frame_time > SLOW_FRAME_THRESHOLD {
            self.slow_streak = self.slow_streak.saturating_add(1);
            if self.visible && self.slow_streak >= REQUIRED_SLOW_FRAMES {
                self.visible = false;
                return Some(false);
            }
            return None;
        }
        // Normal frame: reset streak, bounce back to visible if we were hidden.
        self.slow_streak = 0;
        if !self.visible {
            self.visible = true;
            return Some(true);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_frames_dont_change() {
        let mut v = VisibilityTracker::new();
        for _ in 0..100 {
            assert_eq!(v.observe(0.016), None);
        }
    }

    #[test]
    fn single_slow_frame_does_not_hide() {
        // Startup texture warmup — do not flip to invisible.
        let mut v = VisibilityTracker::new();
        assert_eq!(v.observe(3.5), None);
        assert_eq!(v.observe(0.016), None);
    }

    #[test]
    fn two_consecutive_slow_frames_hide() {
        let mut v = VisibilityTracker::new();
        assert_eq!(v.observe(3.0), None);
        assert_eq!(v.observe(3.0), Some(false));
        assert_eq!(v.observe(0.016), Some(true));
    }

    #[test]
    fn moderate_slow_frame_is_ignored() {
        // 0.8s is not enough to count.
        let mut v = VisibilityTracker::new();
        for _ in 0..5 {
            assert_eq!(v.observe(0.8), None);
        }
    }
}
