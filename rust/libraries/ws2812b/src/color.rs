//! RGB Color representation and brightness manipulation for WS2812B.

/// Represents an 8-bit RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new color with red, green, and blue components.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const RED: Color = Color::new(255, 0, 0);
    pub const GREEN: Color = Color::new(0, 255, 0);
    pub const BLUE: Color = Color::new(0, 0, 255);
    pub const WHITE: Color = Color::new(255, 255, 255);
    pub const YELLOW: Color = Color::new(255, 255, 0);
    pub const CYAN: Color = Color::new(0, 255, 255);
    pub const MAGENTA: Color = Color::new(255, 0, 255);

    /// Return a copy of this color scaled by a brightness factor (0..255).
    #[inline]
    pub fn with_brightness(self, brightness: u8) -> Self {
        Self {
            r: scale_channel(self.r, brightness),
            g: scale_channel(self.g, brightness),
            b: scale_channel(self.b, brightness),
        }
    }

    /// Convert color to WS2812B wire format array `[G, R, B]` with scaling applied.
    #[inline]
    pub fn to_grb_scaled(self, brightness: u8) -> [u8; 3] {
        [
            scale_channel(self.g, brightness),
            scale_channel(self.r, brightness),
            scale_channel(self.b, brightness),
        ]
    }
}

/// Scale an individual 8-bit color channel by a brightness value (0..255).
#[inline(always)]
pub fn scale_channel(value: u8, brightness: u8) -> u8 {
    if brightness == 255 {
        value
    } else {
        ((value as u16 * brightness as u16 + 128) >> 8) as u8
    }
}
