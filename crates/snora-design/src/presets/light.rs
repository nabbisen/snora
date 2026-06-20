//! Light preset.

use crate::{Color, Palette, Tokens};

pub(crate) fn palette() -> Palette {
    Palette {
        background: Color::rgb(1.0, 1.0, 1.0),
        surface: Color::rgb(0.956863, 0.964706, 0.972549),
        surface_raised: Color::rgb(1.0, 1.0, 1.0),
        text_primary: Color::rgb(0.0823529, 0.0941176, 0.109804),
        text_secondary: Color::rgb(0.270588, 0.294118, 0.329412),
        text_muted: Color::rgb(0.419608, 0.447059, 0.501961),
        border: Color::rgb(0.843137, 0.858824, 0.878431),
        accent: Color::rgb(0.113725, 0.305882, 0.847059),
        accent_text: Color::rgb(1.0, 1.0, 1.0),
        success: Color::rgb(0.0823529, 0.501961, 0.239216),
        success_text: Color::rgb(1.0, 1.0, 1.0),
        warning: Color::rgb(0.603922, 0.356863, 0.0),
        warning_text: Color::rgb(1.0, 1.0, 1.0),
        danger: Color::rgb(0.701961, 0.14902, 0.117647),
        danger_text: Color::rgb(1.0, 1.0, 1.0),
        info: Color::rgb(0.113725, 0.305882, 0.847059),
        info_text: Color::rgb(1.0, 1.0, 1.0),
        focus: Color::rgb(0.113725, 0.305882, 0.847059),
    }
}

pub(crate) fn tokens() -> Tokens {
    super::assemble(palette(), 2.0)
}
