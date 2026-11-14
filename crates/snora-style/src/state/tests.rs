//! The rule for a hovered or pressed control's colours (RFC-129), at the function level, with
//! numbers computed independently of `shifted` (a plain re-statement of WCAG and CIE Lab here, so
//! a mistake in one is not hidden by the same mistake in the other).

use super::*;
use snora_design::{Color as DC, TextContrast, Tokens, contrast::contrast_ratio};

fn ic(c: DC) -> Color {
    Color::from_rgba(c.r, c.g, c.b, c.a)
}

fn opaque(c: Color) -> DC {
    DC::rgb(c.r, c.g, c.b)
}

/// The test's own ΔE76, written out again rather than reaching for the module's.
fn de76(a: Color, b: Color) -> f32 {
    fn lab(c: Color) -> [f64; 3] {
        let lin = |v: f32| {
            let v = f64::from(v);
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        let (r, g, b) = (lin(c.r), lin(c.g), lin(c.b));
        let (x, y, z) = (
            (0.4124564 * r + 0.3575761 * g + 0.1804375 * b) / 0.95047,
            0.2126729 * r + 0.7151522 * g + 0.0721750 * b,
            (0.0193339 * r + 0.1191920 * g + 0.9503041 * b) / 1.08883,
        );
        let f = |t: f64| {
            if t > 216.0 / 24389.0 {
                t.cbrt()
            } else {
                (24389.0 / 27.0 * t + 16.0) / 116.0
            }
        };
        let (fx, fy, fz) = (f(x), f(y), f(z));
        [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
    }
    let (p, q) = (lab(a), lab(b));
    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt() as f32
}

#[test]
fn shade_adds_to_every_channel_and_clamps_both_ways() {
    // What the chips' private `darken` test used to say: a shift never leaves 0..=1.
    let black = shade(Color::BLACK, -1.0);
    assert_eq!((black.r, black.g, black.b), (0.0, 0.0, 0.0));
    let white = shade(Color::WHITE, 1.0);
    assert_eq!((white.r, white.g, white.b), (1.0, 1.0, 1.0));
    let c = shade(Color::from_rgba(0.2, 0.5, 0.9, 0.3), 0.1);
    assert!((c.r - 0.3).abs() < 1e-6 && (c.g - 0.6).abs() < 1e-6 && (c.b - 1.0).abs() < 1e-6);
    assert_eq!(c.a, 0.3, "the alpha is not shifted");
}

#[test]
fn the_candidates_are_today_the_opposite_then_larger_steps_today_s_direction_first() {
    let c: Vec<f32> = candidates(0.06).collect();
    assert_eq!(&c[..6], &[0.06, -0.06, 0.08, -0.08, 0.10, -0.10]);
    assert_eq!(c.len(), 2 + 2 * 22, "up to 0.50, 0.08 and on");
    let last = c[c.len() - 2];
    assert!(
        (last - 0.50).abs() < 1e-5,
        "the largest is 0.50, got {last}"
    );
    let c: Vec<f32> = candidates(-0.06).collect();
    assert_eq!(&c[..6], &[-0.06, 0.06, -0.08, 0.08, -0.10, 0.10]);
    let c: Vec<f32> = candidates(0.0).collect();
    assert_eq!(
        &c[..5],
        &[0.0, 0.02, -0.02, 0.04, -0.04],
        "no state shift today: 0 first, once"
    );
    assert!(c.iter().all(|a| a.abs() <= 0.5 + 1e-5), "bounded");
}

fn primary_rest(t: &Tokens) -> Pair {
    Pair {
        label: ic(t.palette.accent_text),
        fill: ic(t.palette.accent),
    }
}

#[test]
fn a_shift_that_already_holds_the_rule_is_kept() {
    // Light, primary hovered: lighter, today, and it clears 4.5 with room.
    let t = Tokens::light();
    let rest = primary_rest(&t);
    let got = shifted(&t.palette, Knob::Fill, 0.06, rest, rest);
    assert_eq!(got.fill, shade(rest.fill, 0.06));
    assert_eq!(got.label, rest.label);
}

#[test]
fn a_shift_that_would_fall_under_the_level_takes_another() {
    // light_aaa, primary hovered: today's lightening takes white text to 6.23:1; AAA is 7.
    let t = Tokens::light_aaa();
    let rest = primary_rest(&t);
    let today = shade(rest.fill, 0.06);
    assert!(
        contrast_ratio(opaque(rest.label), opaque(today)) < 7.0,
        "premise: today's shift is under 7"
    );
    let got = shifted(&t.palette, Knob::Fill, 0.06, rest, rest);
    assert_ne!(got.fill, today);
    assert!(contrast_ratio(opaque(got.label), opaque(got.fill)) >= 7.0);
    assert!(
        de76(rest.fill, got.fill) >= 3.0,
        "and still reads as a change"
    );
}

#[test]
fn the_level_is_the_palettes_not_the_functions() {
    // The same colours, held to 7 and to 4.5, take different shifts: the token is read.
    let mut t = Tokens::light_aaa();
    let rest = primary_rest(&t);
    let aaa = shifted(&t.palette, Knob::Fill, 0.06, rest, rest);
    t.palette.text_contrast = TextContrast::Aa;
    let aa = shifted(&t.palette, Knob::Fill, 0.06, rest, rest);
    assert_ne!(aaa.fill, aa.fill);
    assert_eq!(
        aa.fill,
        shade(rest.fill, 0.06),
        "6.23 clears 4.5, so today's shift is kept"
    );
}

/// A palette no candidate can satisfy: a mid-grey label on a mid-grey fill. The best any fill
/// can do against grey 0.5 is black, 5.28:1, under AAA's 7.
fn impossible() -> (Tokens, Pair) {
    let mut t = Tokens::light();
    let grey = snora_design::Color::rgb(0.5, 0.5, 0.5);
    t.palette.accent = grey;
    t.palette.accent_text = grey;
    t.palette.text_contrast = TextContrast::Aaa;
    let rest = primary_rest(&t);
    (t, rest)
}

#[test]
fn when_no_candidate_holds_the_level_the_state_takes_the_best_visible_one_and_does_not_panic() {
    let (t, rest) = impossible();
    for today in [0.06, -0.06, 0.0] {
        let got = shifted(&t.palette, Knob::Fill, today, rest, rest);
        assert_ne!(
            got.fill, rest.fill,
            "never the resting colour (today {today})"
        );
        assert!(
            de76(rest.fill, got.fill) >= 3.0,
            "still visibly different (today {today})"
        );
        let best = contrast_ratio(opaque(got.label), opaque(got.fill));
        assert!(best < 7.0, "premise: no candidate reaches 7, got {best:.2}");
        // The highest contrast among the visible candidates: no other visible fill is better.
        for k in -25..=25 {
            let other = shade(rest.fill, k as f32 * 0.02);
            if de76(rest.fill, other) >= 3.0 {
                assert!(
                    contrast_ratio(opaque(rest.label), opaque(other)) <= best + 1e-4,
                    "a visible shift of {} reaches more than the one taken ({best:.3})",
                    k as f32 * 0.02
                );
            }
        }
    }
}

#[test]
fn the_search_is_deterministic() {
    let (t, rest) = impossible();
    let a = shifted(&t.palette, Knob::Fill, 0.06, rest, rest);
    let b = shifted(&t.palette, Knob::Fill, 0.06, rest, rest);
    assert_eq!(a, b);
}

#[test]
fn a_state_with_no_visible_candidate_still_moves() {
    // A fill that clamps: white, with every candidate that lightens it a no-op and every one that
    // darkens it a change. Even so the result differs from rest.
    let mut t = Tokens::light();
    let white = snora_design::Color::rgb(1.0, 1.0, 1.0);
    t.palette.accent = white;
    t.palette.accent_text = white;
    let rest = primary_rest(&t);
    let got = shifted(&t.palette, Knob::Fill, 0.06, rest, rest);
    assert_ne!(got.fill, rest.fill);
}

#[test]
fn the_label_knob_is_held_on_the_worst_of_the_three_surfaces() {
    // dark, ghost pressed: accent at 14% behind the label, darker label today. The tint sits over
    // `surface_raised` lightest, and the rule is the lowest of the three.
    let t = Tokens::dark();
    let accent = ic(t.palette.accent);
    let rest = Pair {
        label: accent,
        fill: Color::TRANSPARENT,
    };
    let state = Pair {
        label: accent,
        fill: Color { a: 0.14, ..accent },
    };
    let got = shifted(&t.palette, Knob::Label, -0.06, rest, state);
    for surface in [
        t.palette.background,
        t.palette.surface,
        t.palette.surface_raised,
    ] {
        let s = surface;
        let fill = Color {
            r: 0.14 * accent.r + 0.86 * s.r,
            g: 0.14 * accent.g + 0.86 * s.g,
            b: 0.14 * accent.b + 0.86 * s.b,
            a: 1.0,
        };
        let r = contrast_ratio(opaque(got.label), opaque(fill));
        assert!(r >= 4.5, "label on {fill:?}: {r:.2}");
    }
    assert_eq!(
        got.fill, state.fill,
        "the label is the knob, the fill is as given"
    );
}

// ---------------------------------------------------------------------
// The four button styles, every preset, every state, every neutral surface
// ---------------------------------------------------------------------

use iced::widget::button::{self, Status};

/// This suite's own level for each preset, from the book's table; the token is never read to
/// choose the floor here (RFC-129 Q-4, point 2). A new preset has to be added by hand, and the
/// sweep says so.
fn level_of(preset: &str) -> f32 {
    match preset {
        "light" | "dark" => 4.5,
        "light_aaa" | "dark_aaa" | "high_contrast_light" | "high_contrast_dark" => 7.0,
        other => panic!("`{other}` is a new preset: say its text level in `level_of`"),
    }
}

type StyleFn = fn(&Tokens, Status) -> button::Style;
const BUTTONS: [(&str, StyleFn); 4] = [
    ("primary", crate::button::primary),
    ("secondary", crate::button::secondary),
    ("ghost", crate::button::ghost),
    ("danger", crate::button::danger),
];

fn drawn(style: &button::Style, surface: Color) -> (Color, Color) {
    let fill = match style.background {
        Some(iced::Background::Color(c)) => over(c, surface),
        _ => surface,
    };
    (style.text_color, fill)
}

fn surfaces_of(t: &Tokens) -> [Color; 3] {
    [
        t.palette.background,
        t.palette.surface,
        t.palette.surface_raised,
    ]
    .map(ic)
}

/// **R-1 and R-2, at the function level and exactly (no rounding)**: a hovered or pressed label
/// is at its preset's level on every neutral surface. Fails on the fixed shifts the styles had
/// before RFC-129 (17 groups).
#[test]
fn a_hovered_or_pressed_button_label_holds_its_presets_level_on_every_surface() {
    let mut failures = Vec::new();
    for (preset, t) in Tokens::presets() {
        let floor = level_of(preset);
        for (name, style) in BUTTONS {
            for status in [Status::Hovered, Status::Pressed] {
                for surface in surfaces_of(&t) {
                    let (label, fill) = drawn(&style(&t, status), surface);
                    let r = contrast_ratio(opaque(label), opaque(fill));
                    if r < floor {
                        failures.push(format!(
                            "{preset} {name} {status:?} on {surface:?}: {r:.2}, level {floor}"
                        ));
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} cells:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// **Q-3: a state still reads as a state.** Each hovered and pressed look differs from rest by a
/// ΔE76 of at least 3, in its fill or its label, on every neutral surface. (Computed here, not by
/// `shifted`.)
#[test]
fn a_hovered_or_pressed_button_differs_from_rest_by_a_visible_step() {
    let mut failures = Vec::new();
    for (preset, t) in Tokens::presets() {
        for (name, style) in BUTTONS {
            for status in [Status::Hovered, Status::Pressed] {
                for surface in surfaces_of(&t) {
                    let (rl, rf) = drawn(&style(&t, Status::Active), surface);
                    let (sl, sf) = drawn(&style(&t, status), surface);
                    let step = de76(rf, sf).max(de76(rl, sl));
                    if step < 3.0 {
                        failures.push(format!("{preset} {name} {status:?}: ΔE {step:.2}"));
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} cells:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// `secondary` draws its border in the label's colour, so the rule moves the border too. A border
/// is non-text: 3:1 against each neutral surface, in every state.
#[test]
fn secondarys_border_stays_at_three_to_one_in_every_state() {
    let mut failures = Vec::new();
    for (preset, t) in Tokens::presets() {
        for status in [Status::Active, Status::Hovered, Status::Pressed] {
            let style = crate::button::secondary(&t, status);
            assert_eq!(
                style.border.color, style.text_color,
                "{preset} {status:?}: the border is drawn in the label's colour"
            );
            for surface in surfaces_of(&t) {
                let r = contrast_ratio(opaque(style.border.color), opaque(surface));
                if r < 3.0 {
                    failures.push(format!("{preset} {status:?} on {surface:?}: {r:.2}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// **Q-2: nothing that passes today is moved.** The shifts the styles made before RFC-129, kept as
/// a table here, are the answer wherever they held the rule. The cells that did not hold it are
/// the ones in the migration list (RFC-129 Unit 1): 17 groups.
#[test]
fn a_state_that_held_the_rule_before_rfc_129_has_its_old_colours() {
    // (variant, state, label shift, fill shift): what each style did before.
    let old = |name: &str, status: Status, t: &Tokens| -> (Color, Color) {
        let accent = ic(t.palette.accent);
        let tint = |a: f32| Color { a, ..accent };
        match (name, status) {
            ("primary", Status::Hovered) => (ic(t.palette.accent_text), shade(accent, 0.06)),
            ("primary", Status::Pressed) => (ic(t.palette.accent_text), shade(accent, -0.06)),
            ("danger", Status::Hovered) => {
                (ic(t.palette.danger_text), shade(ic(t.palette.danger), 0.06))
            }
            ("danger", Status::Pressed) => (
                ic(t.palette.danger_text),
                shade(ic(t.palette.danger), -0.06),
            ),
            ("secondary", Status::Hovered) => (shade(accent, 0.06), tint(0.08)),
            ("secondary", Status::Pressed) => (shade(accent, -0.06), tint(0.14)),
            ("ghost", Status::Hovered) => (accent, tint(0.08)),
            ("ghost", Status::Pressed) => (shade(accent, -0.06), tint(0.14)),
            _ => unreachable!(),
        }
    };
    let (mut kept, mut moved_that_held) = (0, Vec::new());
    for (preset, t) in Tokens::presets() {
        let floor = level_of(preset);
        for (name, style) in BUTTONS {
            for status in [Status::Hovered, Status::Pressed] {
                let (old_label, old_fill) = old(name, status, &t);
                let held = surfaces_of(&t).into_iter().all(|s| {
                    let f = over(old_fill, s);
                    contrast_ratio(opaque(old_label), opaque(f)) >= floor
                });
                let now = style(&t, status);
                let same = now.text_color == old_label
                    && match now.background {
                        Some(iced::Background::Color(c)) => c == old_fill,
                        _ => false,
                    };
                if held {
                    kept += 1;
                    if !same {
                        moved_that_held.push(format!("{preset} {name} {status:?}"));
                    }
                }
            }
        }
    }
    assert!(kept > 0, "premise: some groups held the rule before");
    assert!(
        moved_that_held.is_empty(),
        "groups that held the rule before RFC-129 were moved:\n{}",
        moved_that_held.join("\n")
    );
}

/// **The level travels with the palette** (RFC-129 Q-4, point 1), end to end: Light's tokens with
/// `light_aaa`'s palette swapped in are held to 7 by the styles, not to Light's 4.5. (This suite
/// states 7 itself; it does not read the token to choose it.)
#[test]
fn a_palette_swapped_from_light_aaa_is_held_to_seven_by_the_styles() {
    let mut t = Tokens::light();
    t.palette = Tokens::light_aaa().palette;
    let mut checked = 0;
    for (name, style) in BUTTONS {
        for status in [Status::Hovered, Status::Pressed] {
            for surface in surfaces_of(&t) {
                let (label, fill) = drawn(&style(&t, status), surface);
                let r = contrast_ratio(opaque(label), opaque(fill));
                checked += 1;
                assert!(r >= 7.0, "{name} {status:?} on {surface:?}: {r:.2} < 7");
            }
        }
    }
    assert_eq!(checked, 24);
}
