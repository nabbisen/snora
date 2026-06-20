//! Dark preset.

use crate::{Color, Palette, Tokens};

pub(crate) fn palette() -> Palette {
    Palette {
        background: Color::rgb(0.0588235, 0.0705882, 0.0862745),
        surface: Color::rgb(0.0901961, 0.105882, 0.129412),
        surface_raised: Color::rgb(0.121569, 0.141176, 0.168627),
        text_primary: Color::rgb(0.945098, 0.952941, 0.964706),
        text_secondary: Color::rgb(0.682353, 0.713725, 0.752941),
        text_muted: Color::rgb(0.509804, 0.545098, 0.592157),
        border: Color::rgb(0.168627, 0.192157, 0.227451),
        accent: Color::rgb(0.356863, 0.607843, 1.0),
        accent_text: Color::rgb(0.0313725, 0.0666667, 0.121569),
        success: Color::rgb(0.290196, 0.870588, 0.501961),
        success_text: Color::rgb(0.0235294, 0.129412, 0.0588235),
        warning: Color::rgb(0.984314, 0.74902, 0.290196),
        warning_text: Color::rgb(0.141176, 0.0862745, 0.0),
        danger: Color::rgb(1.0, 0.419608, 0.419608),
        danger_text: Color::rgb(0.164706, 0.0235294, 0.0235294),
        info: Color::rgb(0.356863, 0.607843, 1.0),
        info_text: Color::rgb(0.0313725, 0.0666667, 0.121569),
        focus: Color::rgb(0.498039, 0.694118, 1.0),
    }
}

pub(crate) fn tokens() -> Tokens {
    super::assemble(palette(), 2.0)
}
