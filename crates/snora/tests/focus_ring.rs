//! The focus ring, read from the pixels that were drawn (RFC-116 Unit 0).
//!
//! # How the ring is found
//!
//! Not from its geometry. The ring is **whatever appears when the item
//! gains focus**: two frames are rendered, one with nothing focused and
//! one after the item has been clicked, and the region that changed is
//! the ring. That is the same technique RFC-108 used for the tooltip
//! body, and it keeps working if the ring's offset or width changes.
//!
//! **The cursor is moved away before both snapshots.** `control`'s fill
//! follows `cursor.is_over(bounds)`, so leaving the pointer on the item
//! after the click would put the hover fill into the diff and the "ring"
//! would be the whole item.
//!
//! # Two backgrounds, because the ring sits on two things
//!
//! An inactive item sits on the rail; the active one sits on its own
//! `primary.strong` highlight. The ring is drawn **outside** the item's
//! bounds, so most of it is on the rail either way — but its inner edge
//! meets the item. Both pairings are measured, which is what the handoff
//! asked for.

#![cfg(feature = "widgets")]

use iced::widget::{row, space};
use iced::{Element, Length, Point, Rectangle, Theme};

use snora::widget::SideBarLabels;
use snora::{Icon, LayoutDirection, SideBar, SideBarItem};

mod common;
use common::tab_router::tab_router;
use common::{
    Frame, NON_TEXT_MIN, colour_of, contrast_ratio, frame_pixels, peak_colour, record_mark,
    simulator,
};

/// Where the cursor is parked for every snapshot, so no frame carries a
/// hover state.
const AWAY: Point = Point::new(900.0, 700.0);

const NAMES: [&str; 2] = ["Home", "Search"];
const GLYPHS: [&str; 2] = ["1", "2"];
/// The active item, whose ring sits on the highlight.
const ACTIVE: usize = 0;
/// A resting item, whose ring sits on the rail.
const RESTING: usize = 1;

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Go(u8),
}

fn rail() -> Element<'static, Msg> {
    let bar = SideBar {
        items: NAMES
            .iter()
            .enumerate()
            .map(|(i, name)| SideBarItem {
                view_id: i as u8,
                icon: Icon::Text(GLYPHS[i].to_owned()),
                tooltip: (*name).to_owned(),
                on_press: Msg::Go(i as u8),
            })
            .collect(),
        active: ACTIVE as u8,
    };
    row![
        snora::widget::app_side_bar_with(bar, LayoutDirection::Ltr, SideBarLabels::Beneath),
        space().width(Length::Fill)
    ]
    .into()
}

/// The design path's rail.
///
/// Gated, because `snora`'s default features do not include `design` —
/// and `cargo test -p snora` in CI's Rust quality job runs **default**
/// features, not `--all-features`. An ungated version of this compiled
/// fine under every `--all-features` run I had made locally and broke
/// that one job.
#[cfg(feature = "design")]
fn design_rail(tokens: &snora::design::Tokens) -> Element<'static, Msg> {
    let bar = SideBar {
        items: NAMES
            .iter()
            .enumerate()
            .map(|(i, name)| SideBarItem {
                view_id: i as u8,
                icon: Icon::Text(GLYPHS[i].to_owned()),
                tooltip: (*name).to_owned(),
                on_press: Msg::Go(i as u8),
            })
            .collect(),
        active: ACTIVE as u8,
    };
    row![
        snora::design::widget::app_side_bar_with(
            tokens,
            bar,
            LayoutDirection::Ltr,
            SideBarLabels::Beneath
        ),
        space().width(Length::Fill)
    ]
    .into()
}

/// Renders the rail, optionally focusing item `focus` by Tab (a click no
/// longer focuses, RFC-121), and returns the frame plus the bounds of item
/// `report`'s icon.
///
/// **`report` is a separate argument on purpose.** The first version
/// took one index and used it for both, so the unfocused frame reported
/// the *active* item's icon even when the resting item was the subject
/// — and "what the ring sits on" was then read from the active
/// highlight for a resting item. Tokyo Night Light duly came out at
/// 2.85:1 against a background the ring never touches.
fn frame(
    build: &dyn Fn() -> Element<'static, Msg>,
    theme: &Theme,
    focus: Option<usize>,
    report: usize,
    name: &str,
) -> (Frame, Rectangle) {
    let mut ui = simulator(tab_router(build()));
    let item_glyph = |ui: &mut iced_test::Simulator<'_, Msg>, i: usize| {
        ui.find(GLYPHS[i])
            .expect("the rail draws the item's icon")
            .visible_bounds()
            .expect("the icon is visible")
    };

    let glyph = item_glyph(&mut ui, report);

    if let Some(i) = focus {
        // Item `i` is the (i + 1)th focus target, so that many Tabs reach it.
        for _ in 0..=i {
            let _ = ui.tap_key(iced::keyboard::key::Named::Tab);
        }
    }
    // Park the cursor for both frames, so hover is never in the diff.
    ui.point_at(AWAY);

    let f = frame_pixels(&mut ui, theme, name);
    (f, glyph)
}

/// Whether any pixel inside `region` changed between the two frames.
///
/// Distinct from [`changed`], which returns a bounding box. **A ring
/// surrounds its item, so its bounding box always contains the item** —
/// asking "is the icon outside the changed box" can only ever answer
/// no, which is how the first version of
/// `the_ring_is_outside_the_item_it_marks` failed against a correct
/// ring.
fn any_changed_in(before: &Frame, after: &Frame, region: Rectangle) -> bool {
    let s = Frame::SCALE;
    let x0 = (region.x * s).floor().max(0.0) as u32;
    let x1 = ((region.x + region.width) * s).ceil() as u32;
    let y0 = (region.y * s).floor().max(0.0) as u32;
    let y1 = ((region.y + region.height) * s).ceil() as u32;

    for y in y0..y1.min(before.height.min(after.height)) {
        for x in x0..x1.min(before.width.min(after.width)) {
            if let (Some(a), Some(b)) = (before.pixel(x, y), after.pixel(x, y))
                && a[..3] != b[..3]
            {
                return true;
            }
        }
    }
    false
}

/// The bounding box of every physical pixel that differs between two
/// frames, in logical pixels.
fn changed(before: &Frame, after: &Frame) -> Option<Rectangle> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    let mut any = false;
    for y in 0..before.height.min(after.height) {
        for x in 0..before.width.min(after.width) {
            // `Frame::pixel` hands back the raw RGBA bytes, so the
            // comparison is exact: any differing channel is a changed
            // pixel. No tolerance is needed or wanted — a tolerance
            // here would let a faint ring count as "no change".
            let (a, b) = (before.pixel(x, y), after.pixel(x, y));
            if let (Some(a), Some(b)) = (a, b)
                && a[..3] != b[..3]
            {
                any = true;
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    if !any {
        return None;
    }
    let s = Frame::SCALE;
    Some(Rectangle {
        x: x0 as f32 / s,
        y: y0 as f32 / s,
        width: (x1 - x0 + 1) as f32 / s,
        height: (y1 - y0 + 1) as f32 / s,
    })
}

/// Measures the ring on one item against what it sits on.
fn assert_ring(
    failures: &mut Vec<String>,
    context: &str,
    build: &dyn Fn() -> Element<'static, Msg>,
    theme: &Theme,
    index: usize,
    what: &str,
) {
    let (plain, glyph) = frame(
        build,
        theme,
        None,
        index,
        &format!("ring-plain-{context}-{what}"),
    );
    let (focused, _) = frame(
        build,
        theme,
        Some(index),
        index,
        &format!("ring-focused-{context}-{what}"),
    );

    let Some(ring) = changed(&plain, &focused) else {
        failures.push(format!(
            "{context} / {what}: focusing the item changed no pixel at all — there is no ring"
        ));
        return;
    };

    // What the ring sits on, read from the frame that has no ring in
    // the way: a patch at the item's own centre, which is the
    // highlight for the active item and the rail for a resting one.
    let behind = colour_of(plain.modal_colour(Rectangle {
        x: glyph.center().x - 3.0,
        y: glyph.center().y - 3.0,
        width: 6.0,
        height: 6.0,
    }));

    // The ring is a thin mark, so the parity job records its
    // fully-covered pixel rather than the region's average — the mode
    // over a 2px ring's bounding box is whatever it surrounds.
    record_mark(
        &format!("focus-ring/{what}/{context}"),
        &focused,
        ring,
        behind,
    );

    let core = peak_colour(&focused, ring, behind);
    let r = contrast_ratio(core, behind);

    println!(
        "focus-ring | {context:<30} | {what:<18} | {r:6.2} (floor {NON_TEXT_MIN:.1}) | \
         ring {:.0}x{:.0} on {behind:?}",
        ring.width, ring.height,
    );

    if r < NON_TEXT_MIN {
        failures.push(format!(
            "{context} / {what}: the focus ring is {r:.2} against what it sits on, \
             under {NON_TEXT_MIN} (ring {core:?} vs {behind:?})"
        ));
    }
}

fn assert_no_failures(failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "{} focus-ring failure(s):\n  {}",
        failures.len(),
        failures.join("\n  "),
    );
}

/// Every theme iced ships.
///
/// The ring's colour goes through `readable_text`, whose fallback is at
/// least 4.58:1 against any opaque fill — so this sweep is here to
/// confirm that reasoning against drawn pixels, in the nineteen themes
/// a two-theme sweep would have said nothing about.
#[test]
fn the_focus_ring_clears_the_non_text_floor_unstyled() {
    let mut failures = Vec::new();
    for theme in Theme::ALL {
        for (index, what) in [(ACTIVE, "active highlight"), (RESTING, "resting rail")] {
            assert_ring(
                &mut failures,
                &format!("stock {theme}"),
                &rail,
                theme,
                index,
                what,
            );
        }
    }
    assert_no_failures(failures);
}

#[cfg(feature = "design")]
#[test]
fn the_focus_ring_clears_the_non_text_floor_design() {
    use snora::design::Tokens;

    let mut failures = Vec::new();
    for (preset, tokens) in [
        ("design light", Tokens::light()),
        ("design dark", Tokens::dark()),
        ("design high_contrast_light", Tokens::high_contrast_light()),
        ("design high_contrast_dark", Tokens::high_contrast_dark()),
    ] {
        let theme = snora::design::theme(&tokens);
        for (index, what) in [(ACTIVE, "active highlight"), (RESTING, "resting rail")] {
            let tokens = tokens.clone();
            assert_ring(
                &mut failures,
                preset,
                &move || design_rail(&tokens),
                &theme,
                index,
                what,
            );
        }
    }
    assert_no_failures(failures);
}

/// The ring is drawn **outside** the item, so it cannot cover the
/// item's own content.
///
/// Without this, a ring drawn inside the bounds would still clear the
/// contrast floor while sitting on top of the icon and the name.
#[test]
fn the_ring_is_outside_the_item_it_marks() {
    let theme = Theme::Light;
    let (plain, glyph) = frame(&rail, &theme, None, ACTIVE, "ring-outside-plain");
    let (focused, _) = frame(&rail, &theme, Some(ACTIVE), ACTIVE, "ring-outside-focused");
    let ring = changed(&plain, &focused).expect("focusing draws a ring");

    assert!(
        !any_changed_in(&plain, &focused, glyph),
        "focusing the item changed pixels inside its own icon {glyph:?} — the ring is \
         drawn over the content rather than around it (changed region {ring:?})",
    );
}

/// A sanity check on the instrument: with nothing focused, two frames
/// of the same rail are identical.
///
/// If this failed, `changed` would be reporting noise and every figure
/// above would be measuring it.
#[test]
fn two_unfocused_frames_are_identical() {
    let theme = Theme::Light;
    let (a, _) = frame(&rail, &theme, None, ACTIVE, "ring-control-a");
    let (b, _) = frame(&rail, &theme, None, ACTIVE, "ring-control-b");
    assert!(
        changed(&a, &b).is_none(),
        "two frames of the same unfocused rail differ, so the ring measurement is \
         reading noise: {:?}",
        changed(&a, &b),
    );
}

/// The ring is only drawn for the item that has focus.
#[test]
fn only_the_focused_item_gets_a_ring() {
    let theme = Theme::Light;
    let (plain, other) = frame(&rail, &theme, None, RESTING, "ring-one-plain");
    let (focused, active) = frame(&rail, &theme, Some(ACTIVE), ACTIVE, "ring-one-focused");

    // The premise, asserted: the item that was focused *has* a ring. Without
    // this the test passes just as well when focusing did nothing (as it did
    // when this set-up was a click, from RFC-121 on).
    let ring = changed(&plain, &focused)
        .expect("focusing the active item changed nothing, so this test would pass for any reason");
    assert!(
        ring.contains(active.center()),
        "the change {ring:?} is not around the active item's icon {active:?}",
    );

    // Grown by the ring's offset and width, so a ring drawn around the
    // *other* item would fall inside this region.
    let around_other = Rectangle {
        x: other.x - 6.0,
        y: other.y - 6.0,
        width: other.width + 12.0,
        height: other.height + 12.0,
    };
    assert!(
        !any_changed_in(&plain, &focused, around_other),
        "focusing one item changed pixels around the other item's icon {around_other:?}, \
         so more than one ring was drawn",
    );
}
