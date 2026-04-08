//! Approximate visibility tracking. See spec §4.2.

const SUSPICIOUSLY_LARGE_FRAME: f32 = 0.5;

#[derive(Default)]
pub struct VisibilityTracker {
    pub visible: bool,
}

impl VisibilityTracker {
    pub fn new() -> Self {
        Self { visible: true }
    }

    /// Returns Some(new_visibility) if the visibility changed this frame.
    pub fn observe(&mut self, frame_time: f32) -> Option<bool> {
        if frame_time > SUSPICIOUSLY_LARGE_FRAME && self.visible {
            self.visible = false;
            return Some(false);
        }
        if frame_time <= SUSPICIOUSLY_LARGE_FRAME && !self.visible {
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
    fn large_frame_marks_invisible() {
        let mut v = VisibilityTracker::new();
        assert_eq!(v.observe(2.0), Some(false));
        assert_eq!(v.observe(0.016), Some(true));
    }
}
