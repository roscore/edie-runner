//! Logical-to-screen coordinate mapping. Logical resolution is 1280×400
//! per spec §3.3. The screen is letterboxed to preserve aspect ratio.

pub const LOGICAL_W: f32 = 1280.0;
pub const LOGICAL_H: f32 = 400.0;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub screen_w: f32,
    pub screen_h: f32,
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Camera {
    pub fn new(screen_w: f32, screen_h: f32) -> Self {
        Self::with_logical(LOGICAL_W, LOGICAL_H, screen_w, screen_h)
    }

    pub fn with_logical(lw: f32, lh: f32, screen_w: f32, screen_h: f32) -> Self {
        let scale_x = screen_w / lw;
        let scale_y = screen_h / lh;
        let scale = scale_x.min(scale_y);
        let used_w = lw * scale;
        let used_h = lh * scale;
        let offset_x = (screen_w - used_w) * 0.5;
        let offset_y = (screen_h - used_h) * 0.5;
        Self { screen_w, screen_h, scale, offset_x, offset_y }
    }

    pub fn with_shake(mut self, ox: f32, oy: f32) -> Self {
        self.offset_x += ox;
        self.offset_y += oy;
        self
    }

    pub fn to_screen(&self, lx: f32, ly: f32) -> (f32, f32) {
        (self.offset_x + lx * self.scale, self.offset_y + ly * self.scale)
    }

    pub fn scaled(&self, v: f32) -> f32 {
        v * self.scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_inside_wide_screen() {
        let c = Camera::new(1920.0, 600.0);
        assert_eq!(c.scale, 1.5);
    }

    #[test]
    fn letterboxes_too_wide() {
        let c = Camera::new(2000.0, 400.0);
        assert_eq!(c.scale, 1.0);
        assert!(c.offset_x > 0.0);
    }

    #[test]
    fn origin_maps_to_offset() {
        let c = Camera::new(1280.0, 400.0);
        let (x, y) = c.to_screen(0.0, 0.0);
        assert!((x - c.offset_x).abs() < 0.001);
        assert!((y - c.offset_y).abs() < 0.001);
    }
}
