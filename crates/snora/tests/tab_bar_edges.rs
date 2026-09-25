//! Rendered-layout tests for the tab bar's two edges (RFC-102).
//!
//! # Why these measure layout
//!
//! Both edges used to be drawn by a style property rather than by an
//! element: the active tab's underline was a `Shadow` offset under the
//! button, and the bar's "bottom edge" was an all-sided `Border`. Neither
//! is a thing that exists in the layout, so neither could be measured,
//! and the defects — a curled underline and an outlined bar — were
//! invisible to every test the crate had. RFC-102 makes both of them
//! elements. These tests assert the elements are there, with the bounds
//! they claim.
//!
//! The style side of the same change (no shadow, square corners, no
//! container border) is asserted in `snora-widgets`'
//! `crate::tab::tests`, because a shadow and a radius are drawn rather
//! than laid out, so they have no rendered bounds at all.
//!
//! # Choosing `container` over `rule`
//!
//! `iced::widget::rule::horizontal` would draw either line. It
//! implements no `operate`, though, so it is invisible to
//! `Simulator::find` and its rendered bounds cannot be asserted — which
//! is the one thing this file exists to do. A `container` with a
//! background at a fixed height draws the same pixels and reports its
//! bounds, so both edges are containers.

#![cfg(feature = "widgets")]

use iced::{Element, Point, Rectangle};
use iced_test::Simulator;
use iced_test::selector::Candidate;

use snora::{LayoutDirection, Tab, TabAction, TabBar};

/// The active tab's underline, and the bar's bottom rule.
const INDICATOR_HEIGHT: f32 = 2.0;
const BAR_RULE_HEIGHT: f32 = 1.0;

/// Layout here is float arithmetic on exact inputs; half a pixel absorbs
/// sub-pixel rounding and is far below any misplacement worth catching.
const TOLERANCE: f32 = 0.5;

/// How far inside a candidate rectangle's corners the click probes land.
const CORNER_INSET: f32 = 0.5;

const LABELS: [&str; 3] = ["Alpha", "Beta", "Gamma"];

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Tab(TabAction<u8>),
}

/// `app_tab_bar` borrows its mapping function for the element's lifetime.
static ON_ACTION: fn(TabAction<u8>) -> Msg = Msg::Tab;

fn tab_bar(active: u8) -> TabBar<u8> {
    TabBar {
        tabs: LABELS
            .iter()
            .enumerate()
            .map(|(i, label)| Tab {
                id: i as u8,
                label: (*label).to_owned(),
                icon: None,
            })
            .collect(),
        active,
    }
}

fn unstyled(active: u8) -> Element<'static, Msg> {
    snora::widget::app_tab_bar(tab_bar(active), &ON_ACTION, LayoutDirection::Ltr)
}

/// Every container-like widget's bounds, in tree order. iced reports
/// buttons, containers and stacks alike through `Operation::container`.
fn containers(ui: &mut Simulator<'_, Msg>) -> Vec<Rectangle> {
    let mut bounds = Vec::new();
    let _ = ui.find(|candidate: Candidate<'_>| {
        if matches!(candidate, Candidate::Container { .. }) {
            bounds.push(candidate.bounds());
        }
        None::<()>
    });
    bounds
}

/// The one element of the given height, or a failure naming what was
/// found instead.
fn sole_element_of_height(bounds: &[Rectangle], height: f32, what: &str, label: &str) -> Rectangle {
    let matches: Vec<Rectangle> = bounds
        .iter()
        .copied()
        .filter(|r| (r.height - height).abs() <= TOLERANCE)
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "{label}: expected exactly one {height}px element ({what}), found {}: {matches:?}",
        matches.len(),
    );
    matches[0]
}

/// Whether a click at `point` presses the tab with the given id.
fn click_presses<'a>(build: &impl Fn() -> Element<'a, Msg>, point: Point, id: u8) -> bool {
    let mut ui = Simulator::new(build());
    ui.point_at(point);
    let _ = ui.simulate(iced_test::simulator::click());
    ui.into_messages()
        .any(|m| m == Msg::Tab(TabAction::Pressed(id)))
}

/// The tab's own box: the **largest** container holding the label whose
/// four corners all press that tab.
///
/// Measured by clicking rather than by picking a container out of the
/// tree, because several containers hold the label — the button, the
/// stack the underline is layered into, the button's content row, the
/// row of tabs, the bar. Only the button and its stack answer a click at
/// every corner with that tab's message, and they share one rectangle,
/// which is also the thing a user can actually hit. The tabs row and the
/// bar fail at their corners, which are padding; the content row passes
/// but is smaller.
fn tab_bounds<'a>(build: &impl Fn() -> Element<'a, Msg>, label: &str, id: u8) -> Rectangle {
    let mut ui = Simulator::new(build());
    let text = ui
        .find(label)
        .unwrap_or_else(|_| panic!("tab label {label:?} is not rendered"))
        .visible_bounds()
        .unwrap_or_else(|| panic!("tab label {label:?} is not visible"));
    let centre = text.center();

    let mut holding: Vec<Rectangle> = containers(&mut ui)
        .into_iter()
        .filter(|r| r.contains(centre))
        .collect();
    holding.sort_by(|a, b| (b.width * b.height).total_cmp(&(a.width * a.height)));

    holding
        .into_iter()
        .find(|r| {
            [
                Point::new(r.x + CORNER_INSET, r.y + CORNER_INSET),
                Point::new(r.x + r.width - CORNER_INSET, r.y + CORNER_INSET),
                Point::new(r.x + CORNER_INSET, r.y + r.height - CORNER_INSET),
                Point::new(r.x + r.width - CORNER_INSET, r.y + r.height - CORNER_INSET),
            ]
            .into_iter()
            .all(|p| click_presses(build, p, id))
        })
        .unwrap_or_else(|| panic!("no clickable box around the tab label {label:?}"))
}

fn assert_underline_spans_active_tab<'a>(label: &str, build: impl Fn() -> Element<'a, Msg>) {
    let tab = tab_bounds(&build, LABELS[0], 0);
    let mut ui = Simulator::new(build());
    let all = containers(&mut ui);
    let underline =
        sole_element_of_height(&all, INDICATOR_HEIGHT, "the active tab's underline", label);

    assert!(
        (underline.x - tab.x).abs() <= TOLERANCE
            && (underline.width - tab.width).abs() <= TOLERANCE,
        "{label}: the underline spans x {}..{} but its tab spans {}..{} — it must span the tab",
        underline.x,
        underline.x + underline.width,
        tab.x,
        tab.x + tab.width,
    );
    assert!(
        ((underline.y + underline.height) - (tab.y + tab.height)).abs() <= TOLERANCE,
        "{label}: the underline's bottom edge is at {} but the tab's is at {} — the underline \
         must sit at the tab's bottom edge",
        underline.y + underline.height,
        tab.y + tab.height,
    );
}

fn assert_bar_has_bottom_rule(label: &str, element: Element<'_, Msg>) {
    let mut ui = Simulator::new(element);
    let all = containers(&mut ui);
    let bar = *all
        .iter()
        .max_by(|a, b| (a.width * a.height).total_cmp(&(b.width * b.height)))
        .expect("the bar renders at least one container");
    let rule = sole_element_of_height(&all, BAR_RULE_HEIGHT, "the bar's bottom rule", label);

    assert!(
        (rule.x - bar.x).abs() <= TOLERANCE && (rule.width - bar.width).abs() <= TOLERANCE,
        "{label}: the bottom rule spans x {}..{} but the bar spans {}..{} — the rule must span \
         the whole bar, outside its horizontal padding",
        rule.x,
        rule.x + rule.width,
        bar.x,
        bar.x + bar.width,
    );
    assert!(
        ((rule.y + rule.height) - (bar.y + bar.height)).abs() <= TOLERANCE,
        "{label}: the bottom rule ends at y {} but the bar ends at {} — the rule must be the \
         bar's bottom edge",
        rule.y + rule.height,
        bar.y + bar.height,
    );
}

/// Labels must not move when the active tab changes: whichever tab is
/// active, every label sits at the same y as every other, in every
/// arrangement.
fn assert_labels_share_one_y(label: &str, build: impl Fn(u8) -> Element<'static, Msg>) {
    let mut seen: Vec<(String, f32)> = Vec::new();
    for active in 0..LABELS.len() as u8 {
        let mut ui = Simulator::new(build(active));
        for name in LABELS {
            let y = ui
                .find(name)
                .unwrap_or_else(|_| panic!("tab label {name:?} is not rendered"))
                .visible_bounds()
                .expect("label is visible")
                .y;
            seen.push((format!("active={active} {name}"), y));
        }
    }
    let (first_case, first_y) = seen[0].clone();
    for (case, y) in &seen {
        assert!(
            (y - first_y).abs() <= TOLERANCE,
            "{label}: label y drifts between arrangements — {case} sits at {y} but {first_case} \
             sits at {first_y}; the active tab's underline must not change the row's layout",
        );
    }
}

#[test]
fn unstyled_active_tab_underline_spans_the_tab() {
    assert_underline_spans_active_tab("unstyled", || unstyled(0));
}

#[test]
fn unstyled_bar_has_a_bottom_rule() {
    assert_bar_has_bottom_rule("unstyled", unstyled(0));
}

#[test]
fn unstyled_labels_share_one_y() {
    assert_labels_share_one_y("unstyled", unstyled);
}

#[cfg(feature = "design")]
mod styled {
    use super::*;
    use snora::design::Tokens;

    fn styled(tokens: &Tokens, active: u8) -> Element<'_, Msg> {
        snora::design::widget::app_tab_bar(
            tokens,
            tab_bar(active),
            &ON_ACTION,
            LayoutDirection::Ltr,
        )
    }

    /// The four built-in presets. `Element` borrows the tokens, so each
    /// case owns its bundle for the length of the assertion.
    fn presets() -> [(&'static str, Tokens); 4] {
        [
            ("styled light", Tokens::light()),
            ("styled dark", Tokens::dark()),
            ("styled high_contrast_light", Tokens::high_contrast_light()),
            ("styled high_contrast_dark", Tokens::high_contrast_dark()),
        ]
    }

    #[test]
    fn styled_active_tab_underline_spans_the_tab() {
        for (label, tokens) in presets() {
            let build = || styled(&tokens, 0);
            let tab = tab_bounds(&build, LABELS[0], 0);
            let mut ui = Simulator::new(build());
            let all = containers(&mut ui);
            let underline =
                sole_element_of_height(&all, INDICATOR_HEIGHT, "the active tab's underline", label);
            assert!(
                (underline.x - tab.x).abs() <= TOLERANCE
                    && (underline.width - tab.width).abs() <= TOLERANCE
                    && ((underline.y + underline.height) - (tab.y + tab.height)).abs() <= TOLERANCE,
                "{label}: underline {underline:?} does not span the bottom of its tab {tab:?}",
            );
        }
    }

    #[test]
    fn styled_bar_has_a_bottom_rule() {
        for (label, tokens) in presets() {
            let mut ui = Simulator::new(styled(&tokens, 0));
            let all = containers(&mut ui);
            let bar = *all
                .iter()
                .max_by(|a, b| (a.width * a.height).total_cmp(&(b.width * b.height)))
                .expect("the bar renders at least one container");
            let rule =
                sole_element_of_height(&all, BAR_RULE_HEIGHT, "the bar's bottom rule", label);
            assert!(
                (rule.width - bar.width).abs() <= TOLERANCE
                    && ((rule.y + rule.height) - (bar.y + bar.height)).abs() <= TOLERANCE,
                "{label}: bottom rule {rule:?} is not the bottom edge of the bar {bar:?}",
            );
        }
    }

    #[test]
    fn styled_labels_share_one_y() {
        for (label, tokens) in presets() {
            let mut seen: Vec<(String, f32)> = Vec::new();
            for active in 0..LABELS.len() as u8 {
                let mut ui = Simulator::new(styled(&tokens, active));
                for name in LABELS {
                    let y = ui
                        .find(name)
                        .expect("label renders")
                        .visible_bounds()
                        .expect("label is visible")
                        .y;
                    seen.push((format!("active={active} {name}"), y));
                }
            }
            let (first_case, first_y) = seen[0].clone();
            for (case, y) in &seen {
                assert!(
                    (y - first_y).abs() <= TOLERANCE,
                    "{label}: {case} sits at y {y} but {first_case} sits at {first_y}",
                );
            }
        }
    }
}
