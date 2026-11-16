//! RFC-133: **the five widgets that took no direction now mirror**, and the progress bar fills from
//! the start edge.
//!
//! `removable` and `removable_with_tooltip`, `progress::row` and `progress::card`, both
//! `app_footer`s, `render_menu` and `Notice::new` each take a required `LayoutDirection` as their
//! last argument. Under RTL a picture is the left-to-right picture reflected, which this file reads
//! from what the tree reports (the box of every container and every focus target), never from how a
//! widget is built; and the fill is read from the pixels.
//!
//! The mirror law, as in `mirrored_layout.rs`: a box at `x`, `w` wide in a window of width `W` under
//! LTR is at `W - x - w` under RTL, within a pixel. Every box is paired with its reflection (not
//! compared by tree position), and a widget that does not move at all is a failure of its own: a
//! test that passes on a widget which ignores the direction would be a test that cannot fail.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia); layout bounds do not depend on it,
//! and the pixel reads do.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use std::sync::{Arc, Mutex};

use common::removal::Rig;
use common::sweep::{SCALE, design_theme, leak, menu};
use iced::widget::{column, container, progress_bar, text};
use iced::{Element, Length, Rectangle};
use iced_runtime::core::widget::{Id, Operation};
use snora::LayoutDirection::{self, Ltr, Rtl};
use snora::design::{Tokens, Tone, card, chip, notice::Notice, progress, widget as dw};
use snora::widget::{app_footer, render_menu};

const W: f32 = 800.0;

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Go,
    Menu(snora::MenuAction<u8, u8>),
}

static ON_MENU: fn(snora::MenuAction<u8, u8>) -> Msg = Msg::Menu;

// ---- readers ----------------------------------------------------------------------------------------------------------

/// Every container, every text and every focus target in the tree: the boxes a widget lays out.
fn boxes(rig: &mut Rig<Msg>) -> Vec<Rectangle> {
    struct Collect(Arc<Mutex<Vec<Rectangle>>>);
    impl Operation for Collect {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn container(&mut self, _id: Option<&Id>, bounds: Rectangle) {
            self.0.lock().unwrap().push(bounds);
        }
        fn text(&mut self, _id: Option<&Id>, bounds: Rectangle, _text: &str) {
            self.0.lock().unwrap().push(bounds);
        }
    }
    let out = Arc::new(Mutex::new(Vec::new()));
    rig.run(Box::new(Collect(out.clone())));
    let mut v = out.lock().unwrap().clone();
    v.extend(rig.targets().into_iter().map(|t| t.bounds));
    // Not the page: the container that fills the window is the test's, not the widget's.
    v.retain(|b| b.width < W - 1.0);
    v
}

fn page(el: Element<'static, Msg>) -> Element<'static, Msg> {
    container(column![el])
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// The smallest rectangle holding every box: the widget's own span.
fn span(boxes: &[Rectangle]) -> Rectangle {
    let x0 = boxes.iter().map(|b| b.x).fold(f32::MAX, f32::min);
    let y0 = boxes.iter().map(|b| b.y).fold(f32::MAX, f32::min);
    let x1 = boxes.iter().map(|b| b.x + b.width).fold(f32::MIN, f32::max);
    let y1 = boxes
        .iter()
        .map(|b| b.y + b.height)
        .fold(f32::MIN, f32::max);
    Rectangle {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    }
}

/// `b` is `a` reflected about the widget's own span: `a` is at `ua`, `b` at `ub` (both widgets are
/// the same width). A widget that hugs one side of its parent (a chip, a menu) sits where its
/// parent puts it under both directions; what it owes is its inside.
fn mirrored(a: Rectangle, ua: Rectangle, b: Rectangle, ub: Rectangle) -> bool {
    let rel_a = a.x - ua.x;
    let want = ub.x + (ua.width - rel_a - a.width);
    (b.x - want).abs() <= 1.0
        && (b.width - a.width).abs() <= 1.0
        && ((b.y - ub.y) - (a.y - ua.y)).abs() <= 1.0
        && (b.height - a.height).abs() <= 1.0
}

/// Draws `make` under both directions and checks that every box of the left-to-right picture has its
/// reflection, about the widget's own span, in the right-to-left one, and that **something moved
/// inside it**.
#[track_caller]
fn assert_a_mirror_image(
    what: &str,
    make: impl Fn(&'static Tokens, LayoutDirection) -> Element<'static, Msg>,
) {
    mirror_image(what, true, make);
}

/// [`assert_a_mirror_image`] for a widget whose boxes can all be symmetric: a progress bar at 0% or
/// 100% is a full-width label over a full-width track, and only its text's ink moves (which
/// `text_alignment.rs` reads). The boxes must still be reflections.
#[track_caller]
fn assert_a_mirror_image_without_motion(
    what: &str,
    make: impl Fn(&'static Tokens, LayoutDirection) -> Element<'static, Msg>,
) {
    mirror_image(what, false, make);
}

#[track_caller]
fn mirror_image(
    what: &str,
    must_move: bool,
    make: impl Fn(&'static Tokens, LayoutDirection) -> Element<'static, Msg>,
) {
    let t = leak(Tokens::light());
    let mut ltr = Rig::new(page(make(t, Ltr)));
    let mut rtl = Rig::new(page(make(t, Rtl)));
    let (a, b) = (boxes(&mut ltr), boxes(&mut rtl));
    assert_eq!(
        a.len(),
        b.len(),
        "{what}: {} boxes under LTR, {} under RTL",
        a.len(),
        b.len()
    );
    let (ua, ub) = (span(&a), span(&b));
    assert!(
        (ua.width - ub.width).abs() <= 1.0,
        "{what}: the widget is {:.0} wide under LTR and {:.0} under RTL",
        ua.width,
        ub.width
    );
    let mut used = vec![false; b.len()];
    let mut moved = 0;
    for r in &a {
        let Some(j) = (0..b.len()).find(|j| !used[*j] && mirrored(*r, ua, b[*j], ub)) else {
            panic!(
                "{what}: the box x {:.0}..{:.0} y {:.0}..{:.0} under LTR (the widget spans x {:.0}..{:.0}) has \
                 no reflection under RTL (the widget spans x {:.0}..{:.0}); the RTL boxes are {:?}",
                r.x,
                r.x + r.width,
                r.y,
                r.y + r.height,
                ua.x,
                ua.x + ua.width,
                ub.x,
                ub.x + ub.width,
                b.iter()
                    .map(|q| (q.x.round(), (q.x + q.width).round()))
                    .collect::<Vec<_>>()
            );
        };
        used[j] = true;
        if ((b[j].x - ub.x) - (r.x - ua.x)).abs() > 1.0 {
            moved += 1;
        }
    }
    assert!(
        !must_move || moved > 0,
        "{what}: nothing moved inside the widget between LTR and RTL: it ignores its direction"
    );
}

// ---- the five, one by one ---------------------------------------------------------------------------------------------------

#[test]
fn the_removable_chip_is_a_mirror_image() {
    assert_a_mirror_image("removable", |t, dir| {
        chip::removable(t, "Rust", false, Msg::Go, Msg::Go, dir)
    });
    assert_a_mirror_image("removable, selected", |t, dir| {
        chip::removable(t, "Rust", true, Msg::Go, Msg::Go, dir)
    });
    assert_a_mirror_image("removable_with_tooltip", |t, dir| {
        chip::removable_with_tooltip(t, "Rust", false, Msg::Go, Msg::Go, "Remove Rust", dir)
    });
}

#[test]
fn the_chips_label_is_first_in_tab_order_under_both_directions() {
    // The row is logical: Tab visits the label, then the ×, whichever way the window reads.
    for dir in [Ltr, Rtl] {
        let t = leak(Tokens::light());
        let mut rig = Rig::new(page(chip::removable(
            t,
            "Rust",
            false,
            Msg::Go,
            Msg::Go,
            dir,
        )));
        let targets = rig.targets();
        assert_eq!(targets.len(), 2, "{dir:?}");
        let (label, remove) = (targets[0].bounds, targets[1].bounds);
        match dir {
            Ltr => assert!(label.x < remove.x, "LTR: the label is left of the ×"),
            Rtl => assert!(label.x > remove.x, "RTL: the label is right of the ×"),
        }
    }
}

#[test]
fn the_progress_row_and_card_are_mirror_images() {
    // 30%: the fill is one box that moves. 0% and 100%: nothing is asymmetric but the label's ink.
    assert_a_mirror_image("progress::row, 30%", |t, dir| {
        progress::row(t, "Indexing files", Some(0.3), Tone::Accent, dir)
    });
    assert_a_mirror_image("progress::card, 30%", |t, dir| {
        progress::card(t, "Syncing", Some(0.3), Tone::Info, dir)
    });
    for (name, value) in [("none", None), ("100%", Some(1.0))] {
        assert_a_mirror_image_without_motion(&format!("progress::row, {name}"), |t, dir| {
            progress::row(t, "Indexing files", value, Tone::Accent, dir)
        });
        assert_a_mirror_image_without_motion(&format!("progress::card, {name}"), |t, dir| {
            progress::card(t, "Syncing", value, Tone::Info, dir)
        });
    }
}

#[test]
fn both_footers_put_their_content_at_the_start_edge() {
    assert_a_mirror_image("design app_footer", |t, dir| {
        dw::app_footer(t, text("Ready").into(), dir)
    });
    assert_a_mirror_image("unstyled app_footer", |_, dir| {
        app_footer(text("Ready").into(), dir)
    });
}

#[test]
fn a_bare_render_menu_lays_out_its_icon_and_label_for_the_direction() {
    for open in [false, true] {
        assert_a_mirror_image(&format!("render_menu, open={open}"), move |_, dir| {
            render_menu(menu(1, "File", 3), &ON_MENU, open, dir)
        });
    }
}

#[test]
fn a_notice_is_a_mirror_image_in_every_combination() {
    for (name, title, action, dismiss) in [
        ("bare", false, false, false),
        ("title, action, dismiss", true, true, true),
        ("dismiss only", false, false, true),
        ("action only", true, true, false),
    ] {
        assert_a_mirror_image(&format!("notice, {name}"), move |t, dir| {
            let mut n = Notice::new(t, Tone::Warning, "Index is out of date.", dir);
            if title {
                n = n.title("Stale index");
            }
            if action {
                n = n.action("Rebuild", Msg::Go);
            }
            if dismiss {
                n = n.dismiss(Msg::Go).dismiss_tooltip("Dismiss");
            }
            n.render()
        });
    }
}

#[test]
fn a_notices_controls_keep_their_tab_order_and_move_to_the_end_edge() {
    // Action first, dismiss after, under both; the controls are at the end edge (right under LTR).
    for dir in [Ltr, Rtl] {
        let t = leak(Tokens::light());
        let mut rig = Rig::new(page(
            Notice::new(t, Tone::Info, "Body", dir)
                .action("Act", Msg::Go)
                .dismiss(Msg::Go)
                .render(),
        ));
        let targets = rig.targets();
        assert_eq!(targets.len(), 2, "{dir:?}");
        let (action, dismiss) = (targets[0].bounds, targets[1].bounds);
        match dir {
            Ltr => {
                assert!(
                    action.x < dismiss.x,
                    "LTR: the action is left of the dismiss"
                );
                assert!(
                    action.x > W / 2.0,
                    "LTR: the controls are at the right edge"
                );
            }
            Rtl => {
                assert!(
                    action.x > dismiss.x,
                    "RTL: the action is right of the dismiss"
                );
                assert!(
                    dismiss.x + dismiss.width < W / 2.0,
                    "RTL: the controls are at the left edge"
                );
            }
        }
    }
}

// ---- the fill ---------------------------------------------------------------------------------------------------------------

/// How far the exactly-`bar`-coloured run may fall short of the fill, in logical pixels: the fill's
/// transparent border is a one-pixel rim that shows the track's own border through it on a GPU, and its
/// corners are anti-aliased. (That the fill *is* the bar iced draws is exact, in the test below.)
const RIM: f32 = 4.0;

/// The columns (in frame pixels) the colour `bar` fills on the row `y`, as `(first, last)`.
fn run_of(frame: &[u8], y: usize, bar: [u8; 3]) -> Option<(usize, usize)> {
    let at = |x: usize| {
        let i = (y * (W as usize) * SCALE + x) * 4;
        [frame[i], frame[i + 1], frame[i + 2]]
    };
    let hits: Vec<usize> = (0..(W as usize) * SCALE)
        .filter(|x| at(*x) == bar)
        .collect();
    Some((*hits.first()?, *hits.last()?))
}

fn bar_colour(t: &Tokens, tone: Tone) -> [u8; 3] {
    let c = match snora::design::style::progress::toned(t, tone).bar {
        iced::Background::Color(c) => c,
        other => panic!("the bar is not a flat colour: {other:?}"),
    };
    [
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
    ]
}

/// The row of pixels through the middle of the bar: where the track is, from its bounds.
fn bar_row(rig: &mut Rig<Msg>) -> (usize, Rectangle) {
    // The track is the widest container 8 px tall (the fill and the rest are inside it).
    let track = boxes(rig)
        .into_iter()
        .filter(|b| (b.height - 8.0).abs() < 0.5)
        .max_by(|a, b| a.width.total_cmp(&b.width))
        .expect("the track is in the tree");
    (
        ((track.y + track.height / 2.0) * SCALE as f32) as usize,
        track,
    )
}

#[test]
fn the_fill_starts_at_the_start_edge_and_is_the_value_wide() {
    let t = leak(Tokens::light());
    let theme = design_theme(t);
    let bar = bar_colour(t, Tone::Accent);
    for value in [0.1_f32, 0.3, 0.5, 0.9] {
        let mut spans = Vec::new();
        for dir in [Ltr, Rtl] {
            let mut rig = Rig::new(page(progress::row(
                t,
                "Label",
                Some(value),
                Tone::Accent,
                dir,
            )));
            let (y, track) = bar_row(&mut rig);
            let frame = rig.frame_scaled(&theme, SCALE as f32);
            let (first, last) =
                run_of(&frame, y, bar).unwrap_or_else(|| panic!("{value}: no fill under {dir:?}"));
            // In logical pixels. The fill is a fraction of the whole track, as iced's bar was.
            let (a, b) = (
                first as f32 / SCALE as f32,
                (last + 1) as f32 / SCALE as f32,
            );
            let inner = (track.x, track.x + track.width);
            let width = track.width;
            assert!(
                ((b - a) - value * width).abs() <= RIM,
                "{dir:?}, {value}: the fill is {:.1} px, wanted {:.1} of {width:.1}",
                b - a,
                value * width
            );
            match dir {
                Ltr => assert!(
                    (a - inner.0).abs() <= RIM,
                    "LTR: the fill starts at the left edge of the track ({a} vs {})",
                    inner.0
                ),
                Rtl => assert!(
                    (b - inner.1).abs() <= RIM,
                    "RTL: the fill ends at the right edge of the track ({b} vs {})",
                    inner.1
                ),
            }
            spans.push((a, b));
        }
        // And the one is the other reflected about the track.
        let (l, r) = (spans[0], spans[1]);
        assert!(
            ((W - l.1) - r.0).abs() <= 2.0 && ((W - l.0) - r.1).abs() <= 2.0,
            "{value}: LTR {l:?} and RTL {r:?} are not reflections of each other in a window of {W}"
        );
    }
}

#[test]
fn zero_percent_draws_no_fill_and_one_hundred_fills_the_track_under_both_directions() {
    // iced treats `FillPortion(0)` as `Fill`, so an empty end is left out of the row: this is the
    // test that a bar asked for 0% does not draw full, and one asked for 100% draws no gap.
    let t = leak(Tokens::light());
    let theme = design_theme(t);
    let bar = bar_colour(t, Tone::Success);
    for dir in [Ltr, Rtl] {
        for (value, label) in [(Some(0.0), "0%"), (None, "indeterminate")] {
            let mut rig = Rig::new(page(progress::row(t, "Label", value, Tone::Success, dir)));
            let (y, _) = bar_row(&mut rig);
            let frame = rig.frame_scaled(&theme, SCALE as f32);
            assert_eq!(
                run_of(&frame, y, bar),
                None,
                "{dir:?}, {label}: a fill was drawn"
            );
        }
        let mut rig = Rig::new(page(progress::row(
            t,
            "Label",
            Some(1.0),
            Tone::Success,
            dir,
        )));
        let (y, track) = bar_row(&mut rig);
        let frame = rig.frame_scaled(&theme, SCALE as f32);
        let (first, last) = run_of(&frame, y, bar).expect("100% draws a fill");
        let (a, b) = (
            first as f32 / SCALE as f32,
            (last + 1) as f32 / SCALE as f32,
        );
        assert!(
            (a - track.x).abs() <= RIM && (b - (track.x + track.width)).abs() <= RIM,
            "{dir:?}, 100%: the fill spans {a}..{b}, the track {}..{}",
            track.x,
            track.x + track.width
        );
        // And an over-range value clamps.
        let mut over = Rig::new(page(progress::row(
            t,
            "Label",
            Some(7.0),
            Tone::Success,
            dir,
        )));
        let (y, _) = bar_row(&mut over);
        let f = over.frame_scaled(&theme, SCALE as f32);
        assert_eq!(
            run_of(&f, y, bar),
            Some((first, last)),
            "{dir:?}: 700% is 100%"
        );
    }
}

/// What 0.63 drew: iced's own `progress_bar` under the label, in the same column.
fn iced_progress_content(
    t: &'static Tokens,
    label: &'static str,
    value: Option<f32>,
    tone: Tone,
) -> Element<'static, Msg> {
    let (shown, suffix) = match value {
        Some(v) => (
            v.clamp(0.0, 1.0),
            format!("{:.0}%", v.clamp(0.0, 1.0) * 100.0),
        ),
        None => (0.0, "…".to_owned()),
    };
    let tok = t.clone();
    column![
        text(format!("{label}  {suffix}"))
            .size(snora::design::style::text::body_size(t))
            .color(snora::design::style::color::to_iced_color(
                t.palette.text_primary
            )),
        progress_bar(0.0..=1.0, shown)
            .girth(8.0)
            .length(Length::Fill)
            .style(move |_| snora::design::style::progress::toned(&tok, tone)),
    ]
    .spacing(t.spacing.xs)
    .into()
}

/// **Left to right is what it was.** The fill is no longer drawn by iced's `progress_bar` (which has
/// no direction), so this draws both, for every preset, tone and value, and compares every pixel.
/// A fill that rounds its width differently, or a 0% bar that fills, shows here.
#[test]
fn the_fill_is_the_bar_iced_draws_under_ltr() {
    // Every preset with every tone (a mid value, row and card), and every preset with every value
    // (one tone, row and card): 132 pairs per renderer, where all 504 combinations took three
    // minutes on a GPU.
    let tones = [
        Tone::Info,
        Tone::Success,
        Tone::Warning,
        Tone::Danger,
        Tone::Accent,
        Tone::Neutral,
    ];
    let values = [None, Some(0.0), Some(0.03), Some(0.5), Some(1.0)];
    let mut cases: Vec<(Tone, Option<f32>, bool)> = Vec::new();
    for tone in tones {
        cases.push((tone, Some(0.3), false));
        cases.push((tone, Some(0.97), true));
    }
    for value in values {
        for card_variant in [false, true] {
            cases.push((Tone::Accent, value, card_variant));
        }
    }
    let mut compared = 0;
    for (preset, tokens) in Tokens::presets() {
        let t = leak(tokens);
        let theme = design_theme(t);
        for &(tone, value, card_variant) in &cases {
            let (new, old) = if card_variant {
                (
                    progress::card(t, "Progress", value, tone, Ltr),
                    card::surface(t, iced_progress_content(t, "Progress", value, tone)),
                )
            } else {
                (
                    progress::row(t, "Progress", value, tone, Ltr),
                    container(iced_progress_content(t, "Progress", value, tone))
                        .width(Length::Fill)
                        .into(),
                )
            };
            let a = Rig::new(page(new)).frame_scaled(&theme, SCALE as f32);
            let b = Rig::new(page(old)).frame_scaled(&theme, SCALE as f32);
            let differ = a.iter().zip(&b).filter(|(x, y)| x != y).count();
            assert_eq!(
                differ, 0,
                "{preset}, {tone:?}, {value:?}, card={card_variant}: {differ} bytes differ from iced's bar"
            );
            compared += 1;
        }
    }
    assert_eq!(compared, 6 * cases.len(), "every frame was compared");
    assert_eq!(compared, 6 * 22, "132 frames");
}
