use ash::vk;

/// Basic color operations/constants.
pub trait ColorFormat: Copy + Sized {
    /// The color's black.
    const BLACK: Self;

    /// The color's white.
    const WHITE: Self;

    /// The color's red.
    const RED: Self;

    /// The color's green.
    const GREEN: Self;

    /// The color's blue.
    const BLUE: Self;

    /// Returns the color as an RGBA value.
    #[must_use]
    fn as_rgba(self) -> RGBA;

    /// Returns the value as a clear value.
    #[must_use]
    fn as_clear_color_value(self) -> vk::ClearValue {
        let p = self.as_rgba();

        let color = vk::ClearColorValue {
            float32: [
                pixel_u_to_f(p.r),
                pixel_u_to_f(p.g),
                pixel_u_to_f(p.b),
                pixel_u_to_f(p.a),
            ],
        };
        vk::ClearValue { color }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RGBA {
    /// Returns an RGBA color with the given values.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl ColorFormat for RGBA {
    const BLACK: Self = Self::new(0, 0, 0, 255);

    const WHITE: Self = Self::new(255, 255, 255, 255);

    const RED: Self = Self::new(255, 0, 0, 255);

    const GREEN: Self = Self::new(0, 255, 0, 255);

    const BLUE: Self = Self::new(0, 0, 255, 255);

    fn as_rgba(self) -> RGBA {
        self
    }
}

/// Converts the `f32` pixel data to `u8`.
#[inline]
#[must_use]
pub const fn pixel_f_to_u(p: f32) -> u8 {
    (p.clamp(0.0, 1.0) * 255.0) as u8
}

/// Converts the `u8` pixel data to `f32`.
#[inline]
#[must_use]
pub const fn pixel_u_to_f(p: u8) -> f32 {
    p as f32 / 255.0
}
