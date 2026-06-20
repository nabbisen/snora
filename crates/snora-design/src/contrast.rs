//! Pure-Rust contrast utilities (no iced, no rendering).
//!
//! These follow the WCAG definition: linearize sRGB channels, compute relative
//! luminance with the `0.2126 / 0.7152 / 0.0722` coefficients, then compute the
//! contrast ratio as `(L_bright + 0.05) / (L_dark + 0.05)`.
//!
//! The built-in Snora Design palettes are validated against these functions in
//! the crate test suite. Applications can reuse them to check their own custom
//! tokens.
//!
//! ```
//! use snora_design::{Color, contrast};
//!
//! let black = Color::rgb(0.0, 0.0, 0.0);
//! let white = Color::rgb(1.0, 1.0, 1.0);
//! assert!((contrast::contrast_ratio(black, white) - 21.0).abs() < 0.01);
//! ```

use crate::Color;

/// Converts a single normalized sRGB channel to linear-light.
fn linearize_srgb_channel(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Computes WCAG relative luminance from a color's (opaque) sRGB channels.
///
/// Alpha is ignored; composite a translucent color over its background with
/// [`composite_over`] first if it is not opaque.
#[must_use]
pub fn relative_luminance(color: Color) -> f32 {
    let r = linearize_srgb_channel(color.r);
    let g = linearize_srgb_channel(color.g);
    let b = linearize_srgb_channel(color.b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Computes the WCAG contrast ratio between two colors (range `1.0..=21.0`).
///
/// Both colors are treated as opaque; see [`composite_over`] for alpha roles.
#[must_use]
pub fn contrast_ratio(a: Color, b: Color) -> f32 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let bright = la.max(lb);
    let dark = la.min(lb);
    (bright + 0.05) / (dark + 0.05)
}

/// Alpha-composites `fg` over an opaque `bg`, returning an opaque color.
///
/// Use this before computing contrast when a foreground role legitimately
/// uses alpha (e.g. a translucent border or focus ring).
#[must_use]
pub fn composite_over(fg: Color, bg: Color) -> Color {
    let a = fg.a;
    Color::rgb(
        fg.r * a + bg.r * (1.0 - a),
        fg.g * a + bg.g * (1.0 - a),
        fg.b * a + bg.b * (1.0 - a),
    )
}

#[cfg(test)]
mod tests;
