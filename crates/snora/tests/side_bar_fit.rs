//! Rendered-layout tests for the sidebar rail (RFC-099).
//!
//! # Why these measure layout rather than geometry numbers
//!
//! The sidebar shipped buttons that did not fit its rail for at least forty
//! minor releases, while `side_bar_geometry_matches_mapping_all_presets`
//! passed on every one of them. That test asserted that each geometry field
//! was wired to the right token. It never asked whether the result fit.
//!
//! The fix derives the rail's horizontal padding as
//! `(RAIL_WIDTH - BUTTON_SIZE) / 2`. That makes
//! `2 * padding + BUTTON_SIZE <= RAIL_WIDTH` true by arithmetic for any
//! constants at all, so a test built from the same derivation would pass
//! before a regression and after every one. These tests instead ask iced
//! where things actually landed.
//!
//! # Mirrored constants
//!
//! `RAIL_WIDTH` and `BUTTON_SIZE` are private to `snora-widgets`, and making
//! them public to serve a test would widen the public surface. They are
//! mirrored here as **expectations** — the documented intent that the rail
//! is 64px and its buttons 48px square — not as inputs to anything under
//! test. If the real constants change, these tests fail and must be updated
//! deliberately, which is the correct outcome for a layout change.

#![cfg(feature = "widgets")]

use iced::{Element, Point};
use iced_test::simulator;

use snora::{Icon, LayoutDirection, SideBar, SideBarItem};

/// Mirrors `snora_widgets::sidebar::RAIL_WIDTH`. See module doc.
const RAIL_WIDTH: f32 = 64.0;
/// Mirrors `snora_widgets::sidebar::BUTTON_SIZE`. See module doc.
const BUTTON_SIZE: f32 = 48.0;

/// Vertical padding of the unstyled rail — the source that keeps its value
/// under RFC-099 Q-1 (a). The styled variant reads `tokens.spacing.lg`.
const UNSTYLED_VERTICAL_PADDING: f32 = 16.0;

/// How far inside each intended button edge the probe clicks land.
///
/// The intended button spans `[(RAIL - BTN) / 2, (RAIL + BTN) / 2]`, i.e.
/// `[8, 56]`. A rail that pads all four sides by 16 leaves the button only
/// `[16, 48]`. An inset of 4 puts the probes at `x = 12` and `x = 52`:
/// inside the intended button, outside the constrained one. Both edges are
/// probed so that a button widening toward only one side cannot pass.
const EDGE_INSET: f32 = 4.0;

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    First,
    Second,
}

fn side_bar() -> SideBar<Msg, u8> {
    SideBar {
        items: vec![
            SideBarItem {
                view_id: 0,
                icon: Icon::Text("A".into()),
                tooltip: "first view".into(),
                on_press: Msg::First,
            },
            SideBarItem {
                view_id: 1,
                icon: Icon::Text("B".into()),
                tooltip: "second view".into(),
                on_press: Msg::Second,
            },
        ],
        active: 0,
    }
}

/// Clicks at `x` on the vertical centre of the first button and reports
/// whether that button's message was produced.
///
/// The first button's top edge sits at the rail's vertical padding, so its
/// vertical centre is `vertical_padding + BUTTON_SIZE / 2` regardless of
/// the gap between buttons.
fn first_button_receives_click_at(
    element: impl Fn() -> Element<'static, Msg>,
    vertical_padding: f32,
    x: f32,
) -> bool {
    let y = vertical_padding + BUTTON_SIZE / 2.0;
    let mut ui = simulator(element());
    ui.point_at(Point::new(x, y));
    let _ = ui.simulate(iced_test::simulator::click());
    ui.into_messages().any(|m| m == Msg::First)
}

/// Asserts the first button is clickable across its full intended width,
/// probing just inside both edges.
fn assert_button_spans_intended_width(
    label: &str,
    element: impl Fn() -> Element<'static, Msg>,
    vertical_padding: f32,
) {
    let intended_left = (RAIL_WIDTH - BUTTON_SIZE) / 2.0;
    let intended_right = (RAIL_WIDTH + BUTTON_SIZE) / 2.0;
    let left_probe = intended_left + EDGE_INSET;
    let right_probe = intended_right - EDGE_INSET;

    assert!(
        first_button_receives_click_at(&element, vertical_padding, left_probe),
        "{label}: a click at x = {left_probe} (inside the intended {BUTTON_SIZE}px button, \
         {EDGE_INSET}px from its left edge) did not reach the button — \
         the button is narrower than it claims, or does not fit its {RAIL_WIDTH}px rail",
    );
    assert!(
        first_button_receives_click_at(&element, vertical_padding, right_probe),
        "{label}: a click at x = {right_probe} (inside the intended {BUTTON_SIZE}px button, \
         {EDGE_INSET}px from its right edge) did not reach the button — \
         the button is narrower than it claims, or does not fit its {RAIL_WIDTH}px rail",
    );
}

#[test]
fn unstyled_side_bar_buttons_span_their_full_width() {
    assert_button_spans_intended_width(
        "unstyled",
        || snora::widget::app_side_bar(side_bar(), LayoutDirection::Ltr),
        UNSTYLED_VERTICAL_PADDING,
    );
}

#[cfg(feature = "design")]
mod styled {
    use super::*;
    use snora::design::Tokens;

    fn check(label: &str, tokens: fn() -> Tokens) {
        let vertical_padding = tokens().spacing.lg;
        assert_button_spans_intended_width(
            label,
            move || {
                let t = tokens();
                snora::design::widget::app_side_bar(&t, side_bar(), LayoutDirection::Ltr)
            },
            vertical_padding,
        );
    }

    #[test]
    fn styled_light_side_bar_buttons_span_their_full_width() {
        check("styled light", Tokens::light);
    }

    #[test]
    fn styled_dark_side_bar_buttons_span_their_full_width() {
        check("styled dark", Tokens::dark);
    }

    #[test]
    fn styled_high_contrast_light_side_bar_buttons_span_their_full_width() {
        check("styled high_contrast_light", Tokens::high_contrast_light);
    }

    #[test]
    fn styled_high_contrast_dark_side_bar_buttons_span_their_full_width() {
        check("styled high_contrast_dark", Tokens::high_contrast_dark);
    }
}

// ---------------------------------------------------------------------------
// Icon centring (RFC-099 Unit 3)
// ---------------------------------------------------------------------------

/// Allowed difference, in logical pixels, between two measured positions or
/// sizes that should coincide.
///
/// Layout here is pure float arithmetic on exact inputs; the centred build
/// measures an offset of `0.000` on both axes. Half a pixel absorbs any
/// sub-pixel rounding a future text shaper might introduce, and is still
/// twenty times smaller than the defect it guards against (the uncentred
/// glyph sat ~10px left of and ~10px above the button's centre).
const TOLERANCE: f32 = 0.5;

/// The icon's own size, laid out with nothing around it.
fn natural_icon_size() -> iced::Size {
    let mut ui = simulator(snora::widget::icon_element::<Msg>(&Icon::Text("A".into())));
    let bounds = ui
        .find("A")
        .expect("standalone icon renders its text")
        .visible_bounds()
        .expect("standalone icon is visible");
    bounds.size()
}

/// Asserts the first button's icon is centred in the button.
///
/// # Why two assertions, and why the size one matters
///
/// Comparing only the centre of the icon's reported box with the centre of
/// the button **cannot fail on the uncentred code.** Without a centring
/// container the fixed-size button hands its content fixed limits, so the
/// icon's text widget *fills* the button's content area — its box is
/// centred by construction — while the glyph is drawn at that box's
/// top-left. Measured before the fix: box offset `(0.000, 0.000)`, glyph
/// offset `(-9.989, -9.900)`.
///
/// So this first asserts the icon's box is its **natural** size (the box is
/// the glyph, not the space around it), and only then that the box is
/// centred. Remove the centring container and the size assertion fails.
fn assert_icon_centred(label: &str, element: Element<'static, Msg>) {
    let natural = natural_icon_size();

    let mut ui = simulator(element);
    let icon = ui
        .find("A")
        .expect("sidebar renders the first icon")
        .visible_bounds()
        .expect("first icon is visible");
    let icon_centre = icon.center();

    // The button reports itself as a container. Select the 48px square that
    // contains the icon — test-only selection, no production identifier
    // (RFC-047).
    let button = ui
        .find(|candidate: iced_test::selector::Candidate<'_>| {
            let bounds = candidate.bounds();
            let is_container =
                matches!(candidate, iced_test::selector::Candidate::Container { .. });
            (is_container
                && (bounds.width - BUTTON_SIZE).abs() <= TOLERANCE
                && (bounds.height - BUTTON_SIZE).abs() <= TOLERANCE
                && bounds.contains(icon_centre))
            .then_some(bounds)
        })
        .expect("a 48px button contains the first icon");
    let button_centre = button.center();

    assert!(
        (icon.width - natural.width).abs() <= TOLERANCE
            && (icon.height - natural.height).abs() <= TOLERANCE,
        "{label}: the icon's box is {}x{} but the glyph's natural size is {}x{} — \
         the icon fills the button instead of being centred in it \
         (is the centring container missing?)",
        icon.width,
        icon.height,
        natural.width,
        natural.height,
    );
    assert!(
        (icon_centre.x - button_centre.x).abs() <= TOLERANCE
            && (icon_centre.y - button_centre.y).abs() <= TOLERANCE,
        "{label}: icon centre {icon_centre:?} is not within {TOLERANCE}px of \
         button centre {button_centre:?} (button {button:?}, icon {icon:?})",
    );
}

#[test]
fn unstyled_side_bar_icon_is_centred_in_its_button() {
    assert_icon_centred(
        "unstyled",
        snora::widget::app_side_bar(side_bar(), LayoutDirection::Ltr),
    );
}

#[cfg(feature = "design")]
mod styled_centring {
    use super::*;
    use snora::design::Tokens;

    fn check(label: &str, tokens: Tokens) {
        assert_icon_centred(
            label,
            snora::design::widget::app_side_bar(&tokens, side_bar(), LayoutDirection::Ltr),
        );
    }

    #[test]
    fn styled_light_side_bar_icon_is_centred_in_its_button() {
        check("styled light", Tokens::light());
    }

    #[test]
    fn styled_dark_side_bar_icon_is_centred_in_its_button() {
        check("styled dark", Tokens::dark());
    }

    #[test]
    fn styled_high_contrast_light_side_bar_icon_is_centred_in_its_button() {
        check("styled high_contrast_light", Tokens::high_contrast_light());
    }

    #[test]
    fn styled_high_contrast_dark_side_bar_icon_is_centred_in_its_button() {
        check("styled high_contrast_dark", Tokens::high_contrast_dark());
    }
}
