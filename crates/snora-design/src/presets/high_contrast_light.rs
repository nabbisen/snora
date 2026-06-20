//! High-contrast light preset.

use crate::{Color, Palette, Tokens};

pub(crate) fn palette() -> Palette {
    Palette {
        background: Color::rgb(1.0, 1.0, 1.0),
        surface: Color::rgb(1.0, 1.0, 1.0),
        surface_raised: Color::rgb(1.0, 1.0, 1.0),
        text_primary: Color::rgb(0.0, 0.0, 0.0),
        text_secondary: Color::rgb(0.101961, 0.101961, 0.101961),
        text_muted: Color::rgb(0.227451, 0.227451, 0.227451),
        border: Color::rgb(0.0, 0.0, 0.0),
        accent: Color::rgb(0.0, 0.188235, 0.690196),
        accent_text: Color::rgb(1.0, 1.0, 1.0),
        success: Color::rgb(0.0392157, 0.352941, 0.156863),
        success_text: Color::rgb(1.0, 1.0, 1.0),
        warning: Color::rgb(0.478431, 0.239216, 0.0),
        warning_text: Color::rgb(1.0, 1.0, 1.0),
        danger: Color::rgb(0.603922, 0.0, 0.0),
        danger_text: Color::rgb(1.0, 1.0, 1.0),
        info: Color::rgb(0.0, 0.188235, 0.690196),
        info_text: Color::rgb(1.0, 1.0, 1.0),
        focus: Color::rgb(0.0, 0.188235, 0.690196),
    }
}

pub(crate) fn tokens() -> Tokens {
    // Thicker ring for high-contrast legibility.
    super::assemble(palette(), 3.0)
}
