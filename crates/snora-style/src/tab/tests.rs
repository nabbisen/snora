//! The tab style functions (RFC-131): what they draw is **today's, to the float, wherever today held
//! the palette's level**, and every label holds the level on every neutral surface.
//!
//! The level in this file is the book's own table (4.5 for `light` and `dark`, 7 for the other four),
//! never the palette's token: a guard does not read what it guards (RFC-129 Q-4). The ratio is
//! WCAG's, written out again rather than reaching for the crate's.

use super::*;
use iced::widget::button::Status;
use iced::{Background, Color};

const LEVELS: [(&str, f32); 6] = [
    ("light", 4.5),
    ("dark", 4.5),
    ("light_aaa", 7.0),
    ("dark_aaa", 7.0),
    ("high_contrast_light", 7.0),
    ("high_contrast_dark", 7.0),
];

fn level(preset: &str) -> f32 {
    LEVELS
        .iter()
        .find(|(n, _)| *n == preset)
        .unwrap_or_else(|| panic!("`{preset}` is a new preset: say its text level in LEVELS"))
        .1
}

#[test]
fn the_level_table_names_exactly_the_presets() {
    let presets: Vec<&str> = Tokens::presets().into_iter().map(|(n, _)| n).collect();
    let table: Vec<&str> = LEVELS.iter().map(|(n, _)| *n).collect();
    assert_eq!(presets, table);
}

fn lum(c: Color) -> f64 {
    let f = |v: f32| {
        let v = f64::from(v);
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b)
}

fn ratio(a: Color, b: Color) -> f32 {
    let (la, lb) = (lum(a), lum(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    ((hi + 0.05) / (lo + 0.05)) as f32
}

fn rgb(c: snora_design::Color) -> Color {
    Color::from_rgb(c.r, c.g, c.b)
}

fn fill(style: &button::Style) -> Option<Color> {
    match style.background {
        Some(Background::Color(c)) => Some(c),
        _ => None,
    }
}

const STATES: [(&str, Status); 3] = [
    ("rest", Status::Active),
    ("hovered", Status::Hovered),
    ("pressed", Status::Pressed),
];

/// **Every label holds its preset's level, in every state but disabled, on every neutral surface.**
/// A hovered inactive tab draws its own fill, so its label is read on that; everything else is read
/// on the three surfaces a tab bar can sit on.
#[test]
fn every_tab_label_holds_its_presets_level_on_every_neutral_surface() {
    let mut failures = Vec::new();
    for (preset, t) in Tokens::presets() {
        let surfaces = [
            ("background", rgb(t.palette.background)),
            ("surface", rgb(t.palette.surface)),
            ("surface_raised", rgb(t.palette.surface_raised)),
        ];
        for (which, make) in [
            ("active", active as fn(&Tokens, Status) -> button::Style),
            ("inactive", inactive),
        ] {
            for (state, status) in STATES {
                let s = make(&t, status);
                let on: Vec<(&str, Color)> = match fill(&s) {
                    Some(f) => vec![("its own fill", f)],
                    None => surfaces.to_vec(),
                };
                for (name, behind) in on {
                    let r = ratio(s.text_color, behind);
                    if r + 1e-3 < level(preset) {
                        failures.push(format!(
                            "{preset} {which} {state} on {name}: {r:.3}:1, level {}",
                            level(preset)
                        ));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 2e-6
}

fn same(a: Color, want: [f32; 4]) -> bool {
    near(a.r, want[0]) && near(a.g, want[1]) && near(a.b, want[2]) && near(a.a, want[3])
}

/// **Nothing that passes moves** (RFC-131 R-2): today's colours, recorded from the code before this
/// change (`tab_button_style` over `design::theme`, every preset, every state), asserted to the float.
/// The one row that moved is `light_aaa`'s inactive label at rest and pressed, which read 6.58:1
/// against the page and 6.10:1 on `surface`, under its 7; it is the same 30% mix on `light`, which
/// holds its 4.5, so `light` keeps it.
#[test]
fn the_table_of_todays_colours_holds_except_the_one_cell_that_was_under() {
    // preset, active text, inactive muted text (rest and pressed), inactive hovered fill
    type Row = (&'static str, [f32; 4], [f32; 4], Option<[f32; 4]>);
    let table: [Row; 6] = [
        (
            "light",
            [0.0823529, 0.0941176, 0.109804, 1.0],
            [0.35764703, 0.36588234, 0.37686282, 1.0],
            Some([0.8698165, 0.8698165, 0.8698163, 1.0]),
        ),
        (
            "dark",
            [0.945098, 0.952941, 0.964706, 1.0],
            [0.6792156, 0.68823516, 0.7011766, 1.0],
            Some([0.13555926, 0.163257, 0.20011327, 1.0]),
        ),
        (
            // 26% of the page, the first mix toward the text that holds 7:1 on `surface`.
            "light_aaa",
            [0.0823529, 0.0941176, 0.109804, 1.0],
            [0.320941, 0.329647, 0.341255, 1.0],
            Some([0.8698165, 0.8698165, 0.8698163, 1.0]),
        ),
        (
            "dark_aaa",
            [0.945098, 0.952941, 0.964706, 1.0],
            [0.6792156, 0.68823516, 0.7011766, 1.0],
            Some([0.13555926, 0.163257, 0.20011327, 1.0]),
        ),
        (
            "high_contrast_light",
            [0.0, 0.0, 0.0, 1.0],
            [0.3, 0.3, 0.3, 1.0],
            Some([0.8698165, 0.8698165, 0.8698163, 1.0]),
        ),
        (
            "high_contrast_dark",
            [1.0, 1.0, 1.0, 1.0],
            [0.7, 0.7, 0.7, 1.0],
            Some([0.012920003, 0.012920001, 0.012920001, 1.0]),
        ),
    ];
    let presets = Tokens::presets();
    assert_eq!(
        presets.len(),
        table.len(),
        "a preset is missing from the table"
    );
    for ((name, t), (tname, act, muted_text, hover_fill)) in presets.iter().zip(table) {
        assert_eq!(*name, tname, "the table is in the presets' order");
        for (state, status) in STATES {
            assert!(
                same(active(t, status).text_color, act),
                "{name} active {state}: {:?}",
                active(t, status).text_color
            );
            assert!(
                fill(&active(t, status)).is_none(),
                "{name} active {state} has a fill"
            );
        }
        for state in [Status::Active, Status::Pressed] {
            let s = inactive(t, state);
            assert!(fill(&s).is_none(), "{name} inactive {state:?} has a fill");
            assert!(
                same(s.text_color, muted_text),
                "{name} inactive {state:?}: {:?} against today's {muted_text:?}",
                s.text_color
            );
        }
        let h = inactive(t, Status::Hovered);
        assert!(
            same(h.text_color, act),
            "{name} inactive hovered label is the active one"
        );
        let f = fill(&h).expect("an inactive tab has a fill when hovered");
        let want = hover_fill.unwrap();
        assert!(
            same(f, want),
            "{name} inactive hovered fill: {f:?} against today's {want:?}"
        );
    }
}

/// The edge cases `state::muted` cannot reach through a preset: a palette whose text cannot hold
/// the level at all returns the text itself, the highest any mix can have (the search is total).
#[test]
fn a_palette_that_cannot_hold_the_level_gets_the_text_itself() {
    let mut t = Tokens::light();
    t.palette.text_primary = t.palette.background; // text the colour of the page: nothing holds
    let s = inactive(&t, Status::Active);
    assert!(same(
        s.text_color,
        [
            t.palette.text_primary.r,
            t.palette.text_primary.g,
            t.palette.text_primary.b,
            1.0
        ]
    ));
}

/// No tab style draws a shadow or a border, in any state: the underline is its own element
/// (RFC-102), and these are the properties that went wrong when it was a shadow.
#[test]
fn no_tab_style_draws_a_border_or_a_shadow() {
    for (_, t) in Tokens::presets() {
        for make in [active as fn(&Tokens, Status) -> button::Style, inactive] {
            for status in [
                Status::Active,
                Status::Hovered,
                Status::Pressed,
                Status::Disabled,
            ] {
                let s = make(&t, status);
                assert_eq!(s.border.width, 0.0);
                assert_eq!(s.shadow.blur_radius, 0.0);
                assert_eq!(s.shadow.offset, iced::Vector::ZERO);
            }
        }
    }
}
