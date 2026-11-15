use super::*;
use snora_design::{Tokens, Tone};

#[test]
fn all_tones_produce_valid_style_for_all_presets() {
    let tones = [
        Tone::Neutral,
        Tone::Accent,
        Tone::Success,
        Tone::Warning,
        Tone::Danger,
        Tone::Info,
    ];
    // RFC-118: every preset, from the one public source.
    for (_name, tokens) in Tokens::presets() {
        for tone in tones {
            let s = toned(&tokens, tone);
            // bar and background must be Color backgrounds (not gradients for
            // the simple token path); just confirming the call doesn't panic.
            let _ = s.bar;
            let _ = s.background;
        }
    }
}

/// **What the style says (0.63 audit): the bar is the tone's role, the track is `surface`, the corners are `radius.sm`.**
/// The test above only called the function. Every tone and preset; the mapping is stated here, not read from the
/// function (`Neutral` is the `border` role, as the style's rustdoc says).
#[test]
fn the_bar_is_the_tones_role_and_the_track_is_surface() {
    use iced::Background;
    use snora_design::Color as DC;
    let role = |t: &Tokens, tone: Tone| -> DC {
        match tone {
            Tone::Accent => t.palette.accent,
            Tone::Success => t.palette.success,
            Tone::Warning => t.palette.warning,
            Tone::Danger => t.palette.danger,
            Tone::Info => t.palette.info,
            Tone::Neutral => t.palette.border,
        }
    };
    let ic = |c: DC| iced::Color::from_rgba(c.r, c.g, c.b, c.a);
    for (name, t) in Tokens::presets() {
        for tone in [
            Tone::Neutral,
            Tone::Accent,
            Tone::Success,
            Tone::Warning,
            Tone::Danger,
            Tone::Info,
        ] {
            let s = toned(&t, tone);
            assert_eq!(
                s.bar,
                Background::Color(ic(role(&t, tone))),
                "{name} {tone:?}: the bar is the tone's role"
            );
            assert_eq!(
                s.background,
                Background::Color(ic(t.palette.surface)),
                "{name} {tone:?}: the track is `surface`"
            );
            assert_eq!(
                s.border.radius,
                iced::border::Radius::from(t.radius.sm),
                "{name}: corners"
            );
        }
    }
}
