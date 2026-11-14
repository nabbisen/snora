//! RFC-128 R-2: **no text a design widget draws is under its preset's floor at rest**, on every
//! neutral surface it is drawn on, read from the pixels it draws; and R-1's other half, **the tone
//! still shows** in a notice's bar and border.
//!
//! The floor is 4.5, or 7 on `light_aaa` and `dark_aaa` (`aaa-presets.md`: every declared text pair
//! at 7:1). The label's ink is the pixel furthest in contrast from the control's own background
//! inside its box; the solid core of a 14 px glyph reaches the label's colour, so on a correct
//! widget it equals the palette's number (RFC-128 Unit 0: to two decimals). **A drawn colour that
//! no suite declared is the defect this finds**: `Notice::action` drew its label in the tone
//! colour, `palette.border` for `Neutral`, 3.12:1 on `light`.
//!
//! **At rest only.** Hovered and pressed are RFC-129's, which extends this sweep to every state.
//!
//! Renderer follows `ICED_TEST_BACKEND` (default tiny-skia); wgpu needs `--test-threads=1`.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use common::removal::Rig;
use iced::widget::container;
use iced::{Element, Length};
use snora::design::notice::Notice;
use snora::design::{Color as DColor, Tokens, Tone};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Press,
}

/// 8-bit rounding of the ink and the background moves a ratio by about 0.02; the sweep is
/// nowhere near that close to a floor on a correct widget (the tightest, `light_aaa` Accent, is
/// 7.05 against 7).
const TOLERANCE: f32 = 0.05;

fn floor(preset: &str) -> f32 {
    if preset.contains("aaa") { 7.0 } else { 4.5 }
}

fn leak(t: Tokens) -> &'static Tokens {
    Box::leak(Box::new(t))
}

fn ic(c: DColor) -> iced::Color {
    iced::Color {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

fn ratio(a: [u8; 3], b: [u8; 3]) -> f32 {
    let d = |p: [u8; 3]| DColor {
        r: p[0] as f32 / 255.0,
        g: p[1] as f32 / 255.0,
        b: p[2] as f32 / 255.0,
        a: 1.0,
    };
    snora::design::contrast::contrast_ratio(d(a), d(b))
}

fn px(frame: &[u8], x: usize, y: usize) -> [u8; 3] {
    let i = (y * 800 + x) * 4;
    [frame[i], frame[i + 1], frame[i + 2]]
}

/// The contrast the first focus target's label is drawn at: its background is the pixel just inside
/// the top edge, at the middle; its ink the pixel furthest in contrast from that, inside the box.
fn drawn_label(element: Element<'static, Msg>) -> f32 {
    let mut rig = Rig::new(element);
    let b = rig.targets()[0].bounds;
    let frame = rig.frame();
    let bg = px(&frame, (b.x + b.width / 2.0) as usize, b.y as usize + 3);
    let mut best = 1.0f32;
    for y in (b.y as usize + 3)..((b.y + b.height) as usize - 3) {
        for x in (b.x as usize + 3)..((b.x + b.width) as usize - 3) {
            best = best.max(ratio(px(&frame, x, y), bg));
        }
    }
    best
}

const TONES: [Tone; 6] = [
    Tone::Info,
    Tone::Success,
    Tone::Warning,
    Tone::Danger,
    Tone::Accent,
    Tone::Neutral,
];

fn on(surface: DColor, el: Element<'static, Msg>) -> Element<'static, Msg> {
    let fill = ic(surface);
    container(el)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(fill.into()),
            ..Default::default()
        })
        .into()
}

fn assert_none(what: &str, failures: Vec<String>, cells: usize) {
    assert!(
        failures.is_empty(),
        "{what}: {} of {cells} cells are under their floor:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The notice's action, every tone, every preset. A notice paints its own `surface`.
#[test]
fn a_notice_actions_label_clears_the_floor_in_every_tone_of_every_preset() {
    let mut failures = Vec::new();
    let mut cells = 0;
    for (preset, t) in Tokens::presets() {
        let t = leak(t);
        for tone in TONES {
            cells += 1;
            let el: Element<'static, Msg> = Notice::new(t, tone, "body")
                .action("Act", Msg::Press)
                .render();
            let r = drawn_label(el);
            if r < floor(preset) - TOLERANCE {
                failures.push(format!(
                    "{preset} {tone:?}: {r:.2}:1, floor {}",
                    floor(preset)
                ));
            }
        }
    }
    assert_none("notice action", failures, cells);
}

/// The buttons and chips, at rest, on the three neutral surfaces a control is drawn on: the page
/// `background`, `surface` and `surface_raised` (the set the focus ring is guaranteed against).
#[test]
fn every_button_and_chip_label_clears_the_floor_at_rest_on_every_neutral_surface() {
    use snora::design::{button, chip};
    let mut failures = Vec::new();
    let mut cells = 0;
    for (preset, t) in Tokens::presets() {
        let t = leak(t);
        let surfaces = [
            ("background", t.palette.background),
            ("surface", t.palette.surface),
            ("surface_raised", t.palette.surface_raised),
        ];
        type Make = fn(&'static Tokens) -> Element<'static, Msg>;
        let widgets: [(&str, Make); 7] = [
            ("button::primary", |t| {
                button::primary(t, "Save", Msg::Press)
            }),
            ("button::secondary", |t| {
                button::secondary(t, "Save", Msg::Press)
            }),
            ("button::ghost", |t| button::ghost(t, "Save", Msg::Press)),
            ("button::danger", |t| button::danger(t, "Save", Msg::Press)),
            ("button::link", |t| button::link(t, "Save", Msg::Press)),
            ("chip::filter selected", |t| {
                chip::filter(t, "Filter", true, Msg::Press)
            }),
            ("chip::filter unselected", |t| {
                chip::filter(t, "Filter", false, Msg::Press)
            }),
        ];
        for (surface, colour) in surfaces {
            for (name, make) in widgets {
                cells += 1;
                let r = drawn_label(on(colour, make(t)));
                if r < floor(preset) - TOLERANCE {
                    failures.push(format!(
                        "{preset} / {name} on {surface}: {r:.2}:1, floor {}",
                        floor(preset)
                    ));
                }
            }
        }
    }
    assert_none("buttons and chips at rest", failures, cells);
}

/// R-1 moves the action's label to the accent; **the tone must not lose its cue.** In every tone of
/// every preset, with and without an action, the notice's bar is the tone's colour and its border is
/// the tone's colour, and the bar differs from the surface (so the check could tell).
#[test]
fn the_tone_still_shows_in_a_notices_bar_and_border() {
    let rgb8 = |c: DColor| {
        let c = ic(c).into_rgba8();
        [c[0], c[1], c[2]]
    };
    let mut failures = Vec::new();
    for (preset, t) in Tokens::presets() {
        let t = leak(t);
        for tone in TONES {
            let want = rgb8(match tone {
                Tone::Accent => t.palette.accent,
                Tone::Success => t.palette.success,
                Tone::Warning => t.palette.warning,
                Tone::Danger => t.palette.danger,
                Tone::Info => t.palette.info,
                Tone::Neutral => t.palette.border,
            });
            for with_action in [false, true] {
                let mut n = Notice::new(t, tone, "body");
                if with_action {
                    n = n.action("Act", Msg::Press);
                }
                let el: Element<'static, Msg> = n.render();
                let mut rig = Rig::new(el);
                let frame = rig.frame();
                // The bar: 4 px wide inside the 1 px border, down the left edge; the notice is
                // laid out from the window's top-left corner.
                let bar = px(&frame, 3, 10);
                // The border: the top edge, well clear of the corner radius.
                let border = px(&frame, 200, 0);
                let surface = rgb8(t.palette.surface);
                if bar == surface {
                    failures.push(format!("{preset} {tone:?}: premise, the bar pixel is the surface's (not measuring the bar)"));
                }
                if bar != want {
                    failures.push(format!("{preset} {tone:?} action={with_action}: bar {bar:?}, the tone's colour is {want:?}"));
                }
                if border != want {
                    failures.push(format!("{preset} {tone:?} action={with_action}: border {border:?}, the tone's colour is {want:?}"));
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
