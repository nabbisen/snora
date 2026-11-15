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

fn lum(c: snora_design::Color) -> f32 {
    let f = |v: f32| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b)
}

/// WCAG's ratio, written out here.
fn ratio(a: snora_design::Color, b: snora_design::Color) -> f32 {
    let (la, lb) = (lum(a), lum(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

fn surfaces(t: &Tokens) -> [snora_design::Color; 3] {
    [
        t.palette.background,
        t.palette.surface,
        t.palette.surface_raised,
    ]
}

/// **The track is outlined at 3:1** (RFC-135): 1 px, the palette's `border` role, which holds 3:1 against every neutral
/// surface the track can sit on. Before, the track was `surface` with no outline: 1.00 to 1.11:1, and exactly 1.00 on
/// both high-contrast presets.
#[test]
fn the_track_has_a_one_pixel_outline_that_holds_3_to_1_on_every_neutral_surface() {
    for (name, t) in Tokens::presets() {
        let s = toned(&t, Tone::Accent);
        assert_eq!(s.border.width, 1.0, "{name}: a 1 px outline");
        let o = s.border.color;
        let outline = snora_design::Color {
            r: o.r,
            g: o.g,
            b: o.b,
            a: 1.0,
        };
        assert_eq!(
            outline,
            snora_design::Color {
                a: 1.0,
                ..t.palette.border
            },
            "{name}: the border role"
        );
        for surface in surfaces(&t) {
            let r = ratio(outline, surface);
            assert!(
                r >= 3.0,
                "{name}: outline {r:.2}:1 against a neutral surface"
            );
        }
    }
}

/// **The decision, recorded** (RFC-135 Q-1): a darker track was the other way to give the track a boundary. It takes the
/// colour between `surface` and the text that holds 3:1 on all three neutral surfaces, and the bar then sits on it.
/// On the two ordinary light and dark presets that takes **every tone's bar under 3:1 against its track** (it was 3.12
/// and up), so the outline, which leaves the bar's contrast as it was, is the one that changes the bar less.
#[test]
fn a_darker_track_would_have_taken_the_bar_under_3_to_1() {
    for (name, t) in Tokens::presets()
        .into_iter()
        .filter(|(n, _)| ["light", "dark"].contains(n))
    {
        let mix = |k: f32| snora_design::Color {
            r: t.palette.surface.r + (t.palette.text_primary.r - t.palette.surface.r) * k,
            g: t.palette.surface.g + (t.palette.text_primary.g - t.palette.surface.g) * k,
            b: t.palette.surface.b + (t.palette.text_primary.b - t.palette.surface.b) * k,
            a: 1.0,
        };
        let track = (0..=100)
            .map(|k| mix(k as f32 / 100.0))
            .find(|c| surfaces(&t).iter().all(|s| ratio(*c, *s) >= 3.0))
            .expect("some mix toward the text reaches 3:1");
        let fallen = [
            Tone::Accent,
            Tone::Success,
            Tone::Warning,
            Tone::Danger,
            Tone::Info,
            Tone::Neutral,
        ]
        .iter()
        .filter(|tone| {
            let bar = match tone {
                Tone::Accent => t.palette.accent,
                Tone::Success => t.palette.success,
                Tone::Warning => t.palette.warning,
                Tone::Danger => t.palette.danger,
                Tone::Info => t.palette.info,
                Tone::Neutral => t.palette.border,
            };
            ratio(bar, track) < 3.0
        })
        .count();
        assert!(
            fallen >= 5,
            "{name}: {fallen} of 6 tones fall under 3:1 on a darker track"
        );
    }
}
