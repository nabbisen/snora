//! Dark AAA preset (RFC-118).
//!
//! [`crate::Tokens::dark`]'s character with every **declared text
//! pair** at WCAG AAA (7:1). Dark itself is unchanged.
//!
//! Derived the same way as [`super::light_aaa`] — read that module's
//! documentation for the method and for the `focus` consequence. Here
//! the roles **lighten** rather than darken.
//!
//! - **`success` and `warning` did not move.** Dark's are already well
//!   clear (9.78 and 10.63 against their own text), which is why the
//!   RFC's table lists them as short in Light only.
//! - **`text_primary` and `text_secondary` did not move** (14.04 and
//!   7.62). Only `text_muted` did, from 4.53 to 7.05, and the
//!   hierarchy keeps its order: 14.04 ≥ 7.62 ≥ 7.05.
//! - **`info` follows `accent`**, as in Light.

use crate::{Color, Palette, TextContrast, Tokens};

pub(crate) fn palette() -> Palette {
    let parent = super::dark::palette();
    let accent = Color::rgb(0.441152, 0.696961, 1.000000);

    Palette {
        // Dark's own, untouched.
        background: parent.background,
        surface: parent.surface,
        surface_raised: parent.surface_raised,
        border: parent.border,
        focus: parent.focus,
        text_primary: parent.text_primary,
        text_secondary: parent.text_secondary,
        // 4.53 at its worst in Dark; lightened to 7.05.
        text_muted: Color::rgb(0.650123, 0.686952, 0.736093),
        // 5.63 at its worst as text, on `surface_raised`.
        accent,
        accent_text: parent.accent_text,
        info: accent,
        info_text: parent.info_text,
        // Already clear in Dark.
        success: parent.success,
        success_text: parent.success_text,
        warning: parent.warning,
        warning_text: parent.warning_text,
        // 6.71 under its own text; lightened to 7.05.
        danger: Color::rgb(1.000000, 0.450782, 0.448162),
        danger_text: parent.danger_text,
        text_contrast: TextContrast::Aaa,
    }
}

pub(crate) fn tokens() -> Tokens {
    super::assemble(palette(), 2.0)
}
