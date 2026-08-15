//! Explicit `snora_design::Color` → `iced::Color` conversion.
//!
//! The conversion is a named function rather than a `From` impl so that the
//! iced boundary is always visible at the call site. `snora-design` must stay
//! iced-free; this is the single controlled crossing point.

/// Converts a `snora_design::Color` into an `iced::Color`.
///
/// All four channels (`r`, `g`, `b`, `a`) are passed through unchanged.
/// Both types represent normalized sRGB, so this is a lossless field copy.
///
/// ```rust,ignore
/// use snora_design::Color;
/// use snora_widgets::design::style::color::to_iced_color;
///
/// let ic = to_iced_color(Color::rgb(0.1, 0.3, 0.85));
/// assert_eq!(ic.r, 0.1);
/// ```
#[must_use]
pub fn to_iced_color(color: snora_design::Color) -> iced::Color {
    iced::Color::from_rgba(color.r, color.g, color.b, color.a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channels_round_trip() {
        let sd = snora_design::Color::rgba(0.1, 0.2, 0.3, 0.75);
        let ic = to_iced_color(sd);
        assert!((ic.r - 0.1).abs() < 1e-6);
        assert!((ic.g - 0.2).abs() < 1e-6);
        assert!((ic.b - 0.3).abs() < 1e-6);
        assert!((ic.a - 0.75).abs() < 1e-6);
    }

    #[test]
    fn opaque_white_round_trips() {
        let ic = to_iced_color(snora_design::Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(ic, iced::Color::WHITE);
    }
}
