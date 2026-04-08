//! Parallax background scroll. Greybox uses three solid bands; Phase 2 swaps
//! these for tiled art.

#[derive(Debug, Default)]
pub struct Background {
    pub far_offset: f32,
    pub mid_offset: f32,
    pub floor_offset: f32,
}

impl Background {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, dt: f32, speed: f32) {
        self.far_offset = (self.far_offset + speed * 0.10 * dt) % 1280.0;
        self.mid_offset = (self.mid_offset + speed * 0.30 * dt) % 1280.0;
        self.floor_offset = (self.floor_offset + speed * 1.00 * dt) % 1280.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn far_lags_floor() {
        let mut bg = Background::new();
        bg.update(1.0, 320.0);
        assert!(bg.floor_offset > bg.mid_offset);
        assert!(bg.mid_offset > bg.far_offset);
    }
}
