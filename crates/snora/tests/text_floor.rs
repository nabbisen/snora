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
//! **Every state** (RFC-129): the same sweep at hovered and pressed, with the pointer on the
//! control and the button down, and a frame test that each state differs from rest by a visible
//! step (ΔE76 of 3, Q-3). Disabled is exempt (WCAG 1.4.3 exempts inactive controls).
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

/// This sweep's own table of each preset's text level (the book's), never the palette's token: a
/// guard does not read what it guards (RFC-129 Q-4). **The high-contrast presets are `Aaa`** since
/// Q-4 (`Tokens::presets()`, `theme.md` and the accessibility guide promise it). A preset that is
/// not here **panics, naming it**: it is not measured at a guessed level by what its name contains.
const LEVELS: [(&str, f32); 6] = [
    ("light", 4.5),
    ("dark", 4.5),
    ("light_aaa", 7.0),
    ("dark_aaa", 7.0),
    ("high_contrast_light", 7.0),
    ("high_contrast_dark", 7.0),
];

fn floor(preset: &str) -> f32 {
    LEVELS
        .iter()
        .find(|(name, _)| *name == preset)
        .unwrap_or_else(|| panic!("`{preset}` is a new preset: say its text level in LEVELS"))
        .1
}

/// The table names exactly the presets (a seventh cannot go unlisted, a renamed one cannot linger).
#[test]
fn the_sweeps_level_table_names_every_preset_and_only_them() {
    let presets: Vec<&str> = Tokens::presets().into_iter().map(|(n, _)| n).collect();
    let table: Vec<&str> = LEVELS.iter().map(|(n, _)| *n).collect();
    assert_eq!(presets, table);
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

/// Where the pointer is while a control is read.
#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Rest,
    /// The pointer is over the control.
    Hovered,
    /// The pointer is over the control and the left button is down.
    Pressed,
}

const STATES: [State; 3] = [State::Rest, State::Hovered, State::Pressed];
const MOVING: [State; 2] = [State::Hovered, State::Pressed];

/// What the first focus target looks like in a state: its fill (the pixel just inside the top
/// edge, at the middle), its ink (the pixel furthest in contrast from that, inside the box), and
/// the contrast between the two.
#[derive(Clone, Copy)]
struct Look {
    fill: [u8; 3],
    ink: [u8; 3],
    ratio: f32,
}

fn look(element: Element<'static, Msg>, state: State) -> Look {
    let mut rig = Rig::new(element);
    let b = rig.targets()[0].bounds;
    match state {
        State::Rest => {}
        State::Hovered => {
            rig.point_at(Some(b.center()));
            rig.redraw();
        }
        State::Pressed => {
            rig.point_at(Some(b.center()));
            rig.left_button(true);
            rig.redraw();
        }
    }
    let frame = rig.frame();
    let fill = px(&frame, (b.x + b.width / 2.0) as usize, b.y as usize + 3);
    let (mut ink, mut best) = (fill, 1.0f32);
    for y in (b.y as usize + 3)..((b.y + b.height) as usize - 3) {
        for x in (b.x as usize + 3)..((b.x + b.width) as usize - 3) {
            let r = ratio(px(&frame, x, y), fill);
            if r > best {
                best = r;
                ink = px(&frame, x, y);
            }
        }
    }
    Look {
        fill,
        ink,
        ratio: best,
    }
}

/// CIE ΔE76 between two 8-bit colours: written out here, independent of the one the style code
/// uses (a mistake in one must not be hidden by the same mistake in the other).
fn delta_e(a: [u8; 3], b: [u8; 3]) -> f32 {
    fn lab(c: [u8; 3]) -> [f64; 3] {
        let lin = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        let (r, g, bl) = (lin(c[0]), lin(c[1]), lin(c[2]));
        let x = (0.4124564 * r + 0.3575761 * g + 0.1804375 * bl) / 0.95047;
        let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * bl;
        let z = (0.0193339 * r + 0.1191920 * g + 0.9503041 * bl) / 1.08883;
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

/// The notice's action, every tone, every preset, in every state. A notice paints its own
/// `surface`.
#[test]
fn a_notice_actions_label_clears_the_floor_in_every_tone_of_every_preset() {
    let mut failures = Vec::new();
    let mut cells = 0;
    for (preset, t) in Tokens::presets() {
        let t = leak(t);
        for tone in TONES {
            for state in STATES {
                cells += 1;
                let el: Element<'static, Msg> = Notice::new(t, tone, "body")
                    .action("Act", Msg::Press)
                    .render();
                let r = look(el, state).ratio;
                if r < floor(preset) - TOLERANCE {
                    failures.push(format!(
                        "{preset} {tone:?} {state:?}: {r:.2}:1, floor {}",
                        floor(preset)
                    ));
                }
            }
        }
    }
    assert_none("notice action", failures, cells);
}

type Make = fn(&'static Tokens) -> Element<'static, Msg>;

/// Every design control that draws a label: the five buttons and both chips.
fn controls() -> [(&'static str, Make); 7] {
    use snora::design::{button, chip};
    [
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
    ]
}

fn neutral_surfaces(t: &Tokens) -> [(&'static str, DColor); 3] {
    [
        ("background", t.palette.background),
        ("surface", t.palette.surface),
        ("surface_raised", t.palette.surface_raised),
    ]
}

/// The buttons and chips, **in every state**, on the three neutral surfaces a control is drawn on:
/// the page `background`, `surface` and `surface_raised` (the set the focus ring is guaranteed
/// against). Rest passed before RFC-129; hovered and pressed did not, in 17 groups (a hovered
/// `light_aaa` primary read 6.23:1 against its 7).
#[test]
fn every_button_and_chip_label_clears_the_floor_in_every_state_on_every_neutral_surface() {
    let mut failures = Vec::new();
    let mut cells = 0;
    for (preset, t) in Tokens::presets() {
        let t = leak(t);
        for (surface, colour) in neutral_surfaces(t) {
            for (name, make) in controls() {
                for state in STATES {
                    cells += 1;
                    let r = look(on(colour, make(t)), state).ratio;
                    if r < floor(preset) - TOLERANCE {
                        failures.push(format!(
                            "{preset} / {name} {state:?} on {surface}: {r:.2}:1, floor {}",
                            floor(preset)
                        ));
                    }
                }
            }
        }
    }
    assert_none("buttons and chips", failures, cells);
}

/// **Q-3: a state still reads as a state.** Hovered and pressed each differ from rest by a ΔE76
/// of at least 3 in the control's fill or its ink, on every neutral surface (the notice's action
/// too). The style code guarantees 3.0 exactly (`snora-style`'s own suite checks it in floats);
/// the frame is 8-bit, which moves a step by up to a third, so the bar here is 2.5.
///
/// Before RFC-129 the unselected chip on `high_contrast_dark` drew **exactly** like rest hovered
/// and pressed (ΔE 0.00): its `surface` is black, and darkening black is a no-op.
#[test]
fn hovered_and_pressed_each_differ_from_rest_by_a_visible_step() {
    // The exact 3.0 is held in floats by `snora-style`'s suite for every button and chip
    // (`state/tests.rs`). This is the rendered frame: 8-bit rounding moves a small step by up to a
    // third, so the bar only absorbs that, on a step the floats already guarantee (as the 0.05 on
    // contrast does).
    const BAR: f32 = 2.5;
    let mut failures = Vec::new();
    let mut cells = 0;
    let mut smallest = f32::MAX;
    let mut check = |what: String, rest: Look, now: Look| {
        cells += 1;
        let step = delta_e(rest.fill, now.fill).max(delta_e(rest.ink, now.ink));
        smallest = smallest.min(step);
        if step < BAR {
            failures.push(format!("{what}: ΔE {step:.2}"));
        }
    };
    for (preset, t) in Tokens::presets() {
        let t = leak(t);
        for (surface, colour) in neutral_surfaces(t) {
            for (name, make) in controls() {
                let rest = look(on(colour, make(t)), State::Rest);
                for state in MOVING {
                    let now = look(on(colour, make(t)), state);
                    check(
                        format!("{preset} / {name} {state:?} on {surface}"),
                        rest,
                        now,
                    );
                }
            }
        }
        for tone in TONES {
            let make = || -> Element<'static, Msg> {
                Notice::new(t, tone, "body")
                    .action("Act", Msg::Press)
                    .render()
            };
            for state in MOVING {
                check(
                    format!("{preset} / notice action {tone:?} {state:?}"),
                    look(make(), State::Rest),
                    look(make(), state),
                );
            }
        }
    }
    assert!(
        smallest.is_finite() && cells > 0,
        "premise: some cells were read"
    );
    println!("{cells} cells; the smallest step from rest is ΔE {smallest:.2}");
    assert_none("visible step", failures, cells);
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
