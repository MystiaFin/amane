#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ScaleFactor {
    factor: f32,
}

impl ScaleFactor {
    pub fn new(factor: f32) -> Self {
        assert!(
            factor.is_finite() && factor > 0.0,
            "scale factor must be positive and finite"
        );

        Self { factor }
    }

    pub fn get(self) -> f32 {
        self.factor
    }

    // surface dimensions must not round a positive fixed size to Wayland's 0 sentinel
    pub fn pixels(self, size: f32) -> u32 {
        let pixels = (size * self.factor).round() as u32;

        if size > 0.0 { pixels.max(1) } else { pixels }
    }

    pub fn coordinate(self, value: i32) -> i32 {
        (f64::from(value) * f64::from(self.factor)).round() as i32
    }

    pub fn logical(self, pixels: f32) -> f32 {
        pixels / self.factor
    }
}

impl Default for ScaleFactor {
    fn default() -> Self {
        Self { factor: 1.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scale_preserves_surface_coordinates() {
        let factor = ScaleFactor::default();

        assert_eq!(factor.get(), 1.0);
        assert_eq!(factor.pixels(30.0), 30);
        assert_eq!(factor.logical(1920.0), 1920.0);
        assert_eq!(factor.coordinate(i32::MAX), i32::MAX);
        assert_eq!(factor.coordinate(i32::MIN), i32::MIN);
    }

    #[test]
    fn scales_fixed_sizes_and_signed_margins_without_creating_a_full_size() {
        let factor = ScaleFactor::new(1.5);

        assert_eq!(factor.pixels(20.0), 30);
        assert_eq!(factor.pixels(0.1), 1);
        assert_eq!(factor.pixels(0.0), 0);
        assert_eq!(factor.coordinate(3), 5);
        assert_eq!(factor.coordinate(-3), -5);
        assert_eq!(factor.logical(1920.0), 1280.0);
    }
}
