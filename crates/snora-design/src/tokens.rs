//! The top-level [`Tokens`] bundle and its preset constructors.

use crate::{Density, FocusTokens, Palette, Radius, Spacing, Typography};

/// A complete, resolved set of design tokens.
///
/// `Tokens` is `#[non_exhaustive]`: future token groups can be added without a
/// breaking change. Obtain one from a preset constructor below, then
/// optionally mutate fields the application owns:
///
/// ```
/// use snora_design::Tokens;
///
/// let mut tokens = Tokens::light();
/// tokens.palette.accent = snora_design::Color::rgb(0.0, 0.5, 0.4);
/// tokens.radius.md = 8.0;
/// ```
///
/// `Copy` is intentionally not derived (future fields may be non-`Copy`);
/// `Tokens` is cheap to `Clone`.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct Tokens {
    /// Semantic color roles.
    pub palette: Palette,
    /// Spacing scale.
    pub spacing: Spacing,
    /// Text-role / line-height scale.
    pub typography: Typography,
    /// Corner-radius scale.
    pub radius: Radius,
    /// Focus-ring tokens.
    pub focus: FocusTokens,
    /// UI density. All v0.20 presets are [`Density::Comfortable`].
    pub density: Density,
}

impl Tokens {
    /// Calm, readable light theme for productivity apps.
    #[must_use]
    pub fn light() -> Self {
        crate::presets::light::tokens()
    }

    /// Low-glare dark theme with readable text and visible borders.
    #[must_use]
    pub fn dark() -> Self {
        crate::presets::dark::tokens()
    }

    /// [`Tokens::light`]'s character, with every declared text pair at
    /// WCAG AAA (7:1, criterion 1.4.6) rather than AA (RFC-118).
    ///
    /// Light and Dark remain AA presets, which is their stated floor.
    /// This is for an application that wants Light's look at AAA without
    /// the starkness of [`Tokens::high_contrast_light`]. The background
    /// and surfaces are Light's own; the text roles and fills are
    /// darkened only as far as 7:1 requires, and the
    /// `text_primary`/`text_secondary`/`text_muted` hierarchy keeps its
    /// order, compressed.
    #[must_use]
    pub fn light_aaa() -> Self {
        crate::presets::light_aaa::tokens()
    }

    /// [`Tokens::dark`]'s character, with every declared text pair at
    /// WCAG AAA (RFC-118). See [`Tokens::light_aaa`].
    #[must_use]
    pub fn dark_aaa() -> Self {
        crate::presets::dark_aaa::tokens()
    }

    /// High-contrast light theme prioritizing legibility and border clarity.
    #[must_use]
    pub fn high_contrast_light() -> Self {
        crate::presets::high_contrast_light::tokens()
    }

    /// High-contrast dark theme prioritizing legibility and border clarity.
    #[must_use]
    pub fn high_contrast_dark() -> Self {
        crate::presets::high_contrast_dark::tokens()
    }
}
