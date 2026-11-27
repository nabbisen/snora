//! Rendered-layout tests for the toast's close button (RFC-101).
//!
//! The close button is **the only interactive control the engine renders
//! itself**, and it was built from literals rather than from tokens:
//! `button(text("×").size(18)).padding([0, 8])`. That padding is zero
//! vertically, so the button's height was the line box — 18 x 1.3 = 23.4 —
//! which is under the 24 x 24 pointer target snora's accessibility
//! checklist mandates (WCAG 2.5.8). snora's height assertions covered
//! token-derived controls, and this one derives from neither.
//!
//! What these tests measure, and why each is shaped the way it is:
//!
//! - **The target** is measured by clicking its corners
//!   ([`common::pointer_target`]), not by reading a rectangle out of the
//!   tree, because what matters is what a pointer can hit.
//! - **Centring** is asserted natural-size-then-centre (RFC-099): a
//!   container that hands its child fixed limits gets a text box that
//!   fills it and a glyph drawn at that box's top-left, so comparing box
//!   centres alone cannot fail.
//! - **The close mark is "×" in every build** (RFC-139: a Cargo feature never
//!   changes how a mark snora draws itself is drawn), so nothing here depends on
//!   the Lucide font. What the mark draws with no font loaded is pinned by
//!   `lucide_marks_in_the_starting_state.rs`.

#![cfg(feature = "widgets")]

use iced::{Element, Rectangle, Size};
use iced_test::Simulator;

use snora::{AppLayout, Toast, ToastIntent, render};

mod common;
use common::removal::Rig;
use common::{MIN_TARGET, TOLERANCE, containers, pointer_target, simulator};

/// The width the message column had at 0.50.0: the toast's fixed 340,
/// less its 12px padding on each side, less the 4px row spacing, less the
/// 25px the close button was then. The close button may not take more
/// room than it did, or long messages wrap earlier and the toast grows
/// taller — on a surface whose width is fixed, the button's size and the
/// message's room are the same number seen from two ends.
const MESSAGE_COLUMN_MIN: f32 = 287.0;

/// The close glyph, and the one it must not be: "×" in every build (RFC-139), never Lucide's `X` (U+E1B2), whether or
/// not the feature is on.
///
/// Test-local, and deliberately not shared with the implementation: an
/// integration test sees only snora's public API, and adding a public item
/// to carry a glyph for a test's convenience is what RFC-099's review
/// declined (RFC-101 Q-3).
const GLYPH: &str = "×";
const NOT_GLYPH: &str = "\u{e1b2}";

/// The size the engine draws the close glyph at.
const GLYPH_SIZE: f32 = 18.0;

const TOAST_ID: u64 = 7;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Msg {
    Body,
    Dismiss(u64),
}

/// Two intents, because `close_button_style` resolves its colour per
/// intent and the button's geometry must not follow it.
fn intents() -> [(&'static str, ToastIntent); 2] {
    [
        ("info", ToastIntent::Info),
        ("warning", ToastIntent::Warning),
    ]
}

fn body() -> Element<'static, Msg> {
    iced::widget::button(iced::widget::text("body"))
        .on_press(Msg::Body)
        .into()
}

/// A toast rendered through the public engine path, as an application
/// would get it.
fn toast(intent: ToastIntent) -> Element<'static, Msg> {
    toast_with(intent, "All good.")
}

/// The same with the message `message`.
fn toast_with(intent: ToastIntent, message: &str) -> Element<'static, Msg> {
    let toast = Toast::new(TOAST_ID, intent, "Saved", message, Msg::Dismiss(TOAST_ID));
    render(AppLayout::new(body()).toasts(vec![toast]))
}

fn glyph_bounds(ui: &mut Simulator<'_, Msg>, label: &str) -> Rectangle {
    ui.find(GLYPH)
        .unwrap_or_else(|_| panic!("{label}: the close glyph {GLYPH:?} is not rendered"))
        .visible_bounds()
        .unwrap_or_else(|| panic!("{label}: the close glyph {GLYPH:?} is not visible"))
}

/// The close button's pointer target.
fn close_target(intent: ToastIntent, label: &str) -> Rectangle {
    let inside = {
        let mut ui = simulator(toast(intent));
        glyph_bounds(&mut ui, label).center()
    };
    pointer_target(
        label,
        || toast(intent),
        inside,
        |m: &Msg| *m == Msg::Dismiss(TOAST_ID),
    )
}

/// The glyph rendered alone at the engine's size, in the text's font — built here rather than taken from the
/// engine, so a wrong glyph there cannot also be the reference.
fn reference_glyph_size() -> iced::Size {
    let element: Element<'static, Msg> = iced::widget::text(GLYPH).size(GLYPH_SIZE).into();

    let mut ui = simulator(element);
    ui.find(GLYPH)
        .expect("the reference glyph renders")
        .visible_bounds()
        .expect("the reference glyph is visible")
        .size()
}

// ---------------------------------------------------------------------------
// (a) The pointer target
// ---------------------------------------------------------------------------

/// The target a pointer reaches is **the floor, not merely above it**, read by clicking the corners.
///
/// `>=` would pass just as well with the `[0, 8]` padding this RFC
/// removed, which made the button 40 x 24: over the floor, and 15px of it
/// taken from the message column. On a fixed-width toast those are the
/// same 15px. Asserting equality is what stops padding creeping back.
///
/// **This is the pointer reading, and it keeps its tolerance** (`TOLERANCE`, half a pixel): it is
/// what says a click reaches the corners, and it cannot tell 23.75 from 24. The laid-out size is
/// asserted exactly by `close_control_is_laid_out_at_exactly_the_floor` below.
#[test]
fn close_button_target_is_exactly_the_floor() {
    for (label, intent) in intents() {
        let target = close_target(intent, label);
        assert!(
            (target.width - MIN_TARGET).abs() <= TOLERANCE
                && (target.height - MIN_TARGET).abs() <= TOLERANCE,
            "{label}: the toast close button's pointer target (the corner-click reading, within \
             {TOLERANCE} px) is {}x{}, not {MIN_TARGET}x{MIN_TARGET} — below the floor is a WCAG \
             2.5.8 defect, and above it is room taken from the message column of a fixed-width toast",
            target.width,
            target.height,
        );
    }
}

/// **The laid-out size of the close control is exactly the floor, with no tolerance** (0.66
/// housekeeping H-1). logolig measured 23.75 x 23.75 at a fractional `y`; that was the edge's
/// antialiasing counted in whole pixels, and the laid-out target is 24.0 x 24.0. But the reading
/// above allows half a pixel either way, so a real 23.75 would have passed it and nothing else
/// would catch it. This reads the focus target's own bounds from the layout (what
/// `mirrored_layout.rs` reads), and compares them with `==`.
///
/// Three intents, three message lengths (a short one, none, and one that wraps) and two window
/// widths, as the architect's probe covered. If a case is ever a fraction, that is a layout that is
/// not 24, and the message names it: do not widen the comparison.
#[test]
fn close_control_is_laid_out_at_exactly_the_floor() {
    let long =
        "A message long enough that it wraps onto more than one line in a toast of this width, \
                so that the close control sits beside a taller column. "
            .repeat(2);
    let messages: [(&str, &str); 3] = [("short", "All good."), ("empty", ""), ("wrapping", &long)];
    let mut off = Vec::new();
    let mut cases = 0;
    // Where the centred close control sits, by (intent, width, length): the "wrapping" message must make the toast taller than
    // the short one, or the case is a second "short" one and says nothing (the architect's H-1 review).
    let mut centre_y = std::collections::BTreeMap::new();
    for intent in [ToastIntent::Info, ToastIntent::Debug, ToastIntent::Error] {
        for (length, message) in messages {
            for width in [800.0f32, 320.0] {
                cases += 1;
                let mut rig = Rig::sized(toast_with(intent, message), Size::new(width, 600.0));
                let targets = rig.targets();
                let what = format!("{intent:?}, {length} message, {width} px wide");
                if targets.len() != 1 {
                    off.push(format!(
                        "{what}: expected the close control to be the one focus target, found {}",
                        targets.len()
                    ));
                    continue;
                }
                let b = targets[0].bounds;
                centre_y.insert((format!("{intent:?}"), width as u32, length), b.center().y);
                if b.width != MIN_TARGET || b.height != MIN_TARGET {
                    off.push(format!(
                        "{what}: the close control is laid out {}x{}, not exactly \
                         {MIN_TARGET}x{MIN_TARGET} (the layout's own bounds, compared with ==)",
                        b.width, b.height
                    ));
                }
            }
        }
    }
    for intent in [ToastIntent::Info, ToastIntent::Debug, ToastIntent::Error] {
        for width in [800u32, 320] {
            let at = |length: &'static str| {
                centre_y
                    .get(&(format!("{intent:?}"), width, length))
                    .copied()
            };
            let (short, wrapping) = (at("short"), at("wrapping"));
            assert!(
                matches!((short, wrapping), (Some(s), Some(w)) if w > s + 1.0),
                "premise: {intent:?} at {width} px: the wrapping message should make the toast taller than the short one \
                 (its centred close control lower), got {short:?} and {wrapping:?}"
            );
        }
    }
    assert_eq!(
        cases, 18,
        "premise: three intents, three lengths, two widths"
    );
    assert!(
        off.is_empty(),
        "{} of {cases} case(s) are not exactly the floor:\n  {}",
        off.len(),
        off.join("\n  ")
    );
}

/// The message column keeps the room it had at 0.50.0.
///
/// Found by structure rather than by arithmetic on the toast's literals:
/// the message column is the largest container that holds the message and
/// **not** the close glyph. The toast surface and the body row hold both;
/// the text column inside it is smaller.
#[test]
fn message_column_keeps_its_width() {
    for (label, intent) in intents() {
        let mut ui = simulator(toast(intent));
        let message = ui
            .find("All good.")
            .expect("the toast renders its message")
            .visible_bounds()
            .expect("the message is visible")
            .center();
        let glyph = glyph_bounds(&mut ui, label).center();

        let column = containers(&mut ui)
            .into_iter()
            .filter(|r| r.contains(message) && !r.contains(glyph))
            .max_by(|a, b| (a.width * a.height).total_cmp(&(b.width * b.height)))
            .unwrap_or_else(|| panic!("{label}: no container holds the message alone"));

        assert!(
            column.width >= MESSAGE_COLUMN_MIN,
            "{label}: the message column is {}px wide, under 0.50.0's {MESSAGE_COLUMN_MIN}px — \
             the close button has taken room from it, so longer messages wrap earlier and the \
             toast grows taller",
            column.width,
        );
    }
}

// ---------------------------------------------------------------------------
// (b) Centring
// ---------------------------------------------------------------------------

#[test]
fn close_glyph_is_natural_size_and_centred_in_its_target() {
    let natural = reference_glyph_size();
    for (label, intent) in intents() {
        let target = close_target(intent, label);
        let glyph = {
            let mut ui = simulator(toast(intent));
            glyph_bounds(&mut ui, label)
        };

        assert!(
            (glyph.width - natural.width).abs() <= TOLERANCE
                && (glyph.height - natural.height).abs() <= TOLERANCE,
            "{label}: the close glyph's box is {}x{} but the glyph's natural size is {}x{} — \
             it fills its container instead of being centred in it",
            glyph.width,
            glyph.height,
            natural.width,
            natural.height,
        );

        let (g, t) = (glyph.center(), target.center());
        assert!(
            (g.x - t.x).abs() <= TOLERANCE && (g.y - t.y).abs() <= TOLERANCE,
            "{label}: the close glyph's centre {g:?} is not within {TOLERANCE}px of its \
             button's centre {t:?} (button {target:?}, glyph {glyph:?})",
        );
    }
}

// ---------------------------------------------------------------------------
// (c) The glyph
// ---------------------------------------------------------------------------

#[test]
fn close_button_draws_the_text_glyph_in_every_build() {
    for (label, intent) in intents() {
        let mut ui = simulator(toast(intent));
        assert!(
            ui.find(GLYPH).is_ok(),
            "{label}: the close button does not draw {GLYPH:?}"
        );
        assert!(
            ui.find(NOT_GLYPH).is_err(),
            "{label}: the close button draws {NOT_GLYPH:?} (Lucide's X), but it must draw {GLYPH:?} in \
             every build (RFC-139)"
        );
    }
}
