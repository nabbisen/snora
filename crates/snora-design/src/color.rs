//! The renderer-independent [`Color`] type used throughout Snora Design.

/// A color expressed as normalized sRGB channels in `0.0..=1.0`, plus alpha.
///
/// This is deliberately **not** `iced::Color`: `snora-design` is iced-free.
/// The `snora-widgets` style bridge converts this into the pinned iced
/// version's color type at the boundary.
///
/// ```
/// use snora_design::Color;
///
/// let blue = Color::rgb(0.11, 0.31, 0.85);
/// assert!(blue.is_opaque());
/// assert_eq!(Color::rgba(0.0, 0.0, 0.0, 0.5).a, 0.5);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    /// Red channel, `0.0..=1.0`.
    pub r: f32,
    /// Green channel, `0.0..=1.0`.
    pub g: f32,
    /// Blue channel, `0.0..=1.0`.
    pub b: f32,
    /// Alpha channel, `0.0..=1.0` (`1.0` is fully opaque).
    pub a: f32,
}

impl Color {
    /// Constructs an opaque color from red, green, and blue channels.
    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Constructs a color with an explicit alpha channel.
    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Returns `true` if the color is fully opaque (`a == 1.0`).
    #[must_use]
    pub fn is_opaque(self) -> bool {
        (self.a - 1.0).abs() <= f32::EPSILON
    }

    /// Returns `true` if every channel is finite and within `0.0..=1.0`.
    #[must_use]
    pub fn is_valid(self) -> bool {
        [self.r, self.g, self.b, self.a]
            .iter()
            .all(|c| c.is_finite() && *c >= 0.0 && *c <= 1.0)
    }
}

#[cfg(test)]
mod tests;
