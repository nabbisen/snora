//! Structural tests for the tab bar's two edges (RFC-102).
//!
//! The contrast of those edges is asserted in `crate::contrast_tests`;
//! what this module asserts is that they are drawn with the **right
//! primitives**, which is the defect RFC-102 was raised for:
//!
//! - the active tab's underline was a `Shadow` on a `radius: 4.0` button,
//!   so it inherited the rounded corners and curled up at both ends;
//! - the bar's "bottom edge" was an all-sided `Border`, because iced 0.14
//!   has no per-side border width, so it outlined the whole bar;
//! - an inactive tab's hover fill used the same `radius: 4.0`, so it
//!   painted a rounded box over that outline.
//!
//! Each of those is a property of a style function, invisible to a
//! rendered-bounds test: a shadow and a radius are drawn, not laid out.
//! The rendered side — that the underline and the rule exist as elements
//! with the right bounds — is asserted in
//! `crates/snora/tests/tab_bar_edges.rs`.

use iced::widget::button;
use iced::{Shadow, Theme};

use super::{tab_bar_container_style, tab_button_style};

/// Both stock themes are enough here: none of these assertions is
/// colour-dependent, and the colour-dependent ones live in the contrast
/// suite, which sweeps all six theme contexts.
fn themes() -> [(&'static str, Theme); 2] {
    [("stock Light", Theme::Light), ("stock Dark", Theme::Dark)]
}

const ALL_STATUSES: [button::Status; 4] = [
    button::Status::Active,
    button::Status::Hovered,
    button::Status::Pressed,
    button::Status::Disabled,
];

/// The underline is an element now, so no tab button may draw a shadow —
/// in any state. A shadow here would be the old fake underline returning,
/// and it would curl at the corners again.
#[test]
fn tab_button_style_draws_no_shadow() {
    for (name, theme) in themes() {
        for is_active in [true, false] {
            for status in ALL_STATUSES {
                let style = tab_button_style(&theme, status, is_active);
                assert_eq!(
                    style.shadow,
                    Shadow::default(),
                    "{name}: tab_button_style(is_active={is_active}, {status:?}) draws a shadow; \
                     the underline is its own element (RFC-102 R-1)"
                );
            }
        }
    }
}

/// Square corners on the tab button (RFC-102 Q-1 (a)): the hover fill is
/// contained within the tab and sits above the bar's rule, so a rounded
/// pill over a straight rule is exactly the look that was reported.
#[test]
fn tab_button_style_is_square() {
    for (name, theme) in themes() {
        for is_active in [true, false] {
            for status in ALL_STATUSES {
                let radius = tab_button_style(&theme, status, is_active).border.radius;
                assert_eq!(
                    radius,
                    0.0.into(),
                    "{name}: tab_button_style(is_active={is_active}, {status:?}) has a corner \
                     radius; the tab's fill must be square (RFC-102 Q-1 (a))"
                );
            }
        }
    }
}

/// The bar's bottom edge is a rule element, so the bar's own container
/// must paint no border at all. iced 0.14 borders are all-sided: any
/// width here outlines the whole bar, which is the defect (RFC-102 R-2).
#[test]
fn tab_bar_container_style_paints_no_border() {
    for (name, theme) in themes() {
        let border = tab_bar_container_style(&theme).border;
        assert_eq!(
            border.width, 0.0,
            "{name}: the tab bar container paints a {}px border; iced borders are all-sided, \
             so the bottom edge must be an element instead (RFC-102 R-2)",
            border.width,
        );
    }
}

/// The underline's own style: a straight, square, solid line.
#[test]
fn tab_indicator_style_is_square_and_solid() {
    for (name, theme) in themes() {
        let style = super::tab_indicator_style(&theme);
        assert_eq!(
            style.border.radius,
            0.0.into(),
            "{name}: the indicator has a corner radius; it must be a straight line"
        );
        assert!(
            style.background.is_some(),
            "{name}: the indicator paints no background, so nothing is drawn"
        );
        assert_eq!(
            style.shadow,
            Shadow::default(),
            "{name}: the indicator draws a shadow"
        );
    }
}

/// The bar's bottom rule, same shape as the indicator.
#[test]
fn tab_bar_rule_style_is_square_and_solid() {
    for (name, theme) in themes() {
        let style = super::tab_bar_rule_style(&theme);
        assert_eq!(
            style.border.radius,
            0.0.into(),
            "{name}: the bar's bottom rule has a corner radius"
        );
        assert!(
            style.background.is_some(),
            "{name}: the bar's bottom rule paints no background, so nothing is drawn"
        );
    }
}
