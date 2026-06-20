//! High-contrast dark preset.

use crate::{Color, Palette, Tokens};

pub(crate) fn palette() -> Palette {
    Palette {
        background: Color::rgb(0.0, 0.0, 0.0),
        surface: Color::rgb(0.0, 0.0, 0.0),
        surface_raised: Color::rgb(0.0392157, 0.0392157, 0.0392157),
        text_primary: Color::rgb(1.0, 1.0, 1.0),
        text_secondary: Color::rgb(0.901961, 0.901961, 0.901961),
        text_muted: Color::rgb(0.752941, 0.752941, 0.752941),
        border: Color::rgb(1.0, 1.0, 1.0),
        accent: Color::rgb(0.611765, 0.768627, 1.0),
        accent_text: Color::rgb(0.0, 0.0, 0.0),
        success: Color::rgb(0.482353, 0.894118, 0.603922),
        success_text: Color::rgb(0.0, 0.0, 0.0),
        warning: Color::rgb(1.0, 0.827451, 0.478431),
        warning_text: Color::rgb(0.0, 0.0, 0.0),
        danger: Color::rgb(1.0, 0.603922, 0.603922),
        danger_text: Color::rgb(0.0, 0.0, 0.0),
        info: Color::rgb(0.611765, 0.768627, 1.0),
        info_text: Color::rgb(0.0, 0.0, 0.0),
        focus: Color::rgb(1.0, 1.0, 1.0),
    }
}

pub(crate) fn tokens() -> Tokens {
    super::assemble(palette(), 3.0)
}
