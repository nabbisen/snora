//! An ancestor breadcrumb is underlined; the leaf is not (RFC-109).
//!
//! # What was wrong
//!
//! An ancestor and the leaf were the same colour by construction. An
//! ancestor is a `button` whose resting arm of `crumb_button_style`
//! sets no background and `text_color: background.base.text`; the leaf
//! is plain `text`, which inherits `theme.base().text_color` — the same
//! value. So the only thing distinguishing "you can click this" from
//! "you are here" was word order — not a distinction carried by colour
//! alone, but no drawn distinction at all: the affordance that 1.4.1's
//! sufficient techniques assume is there was missing. Hover gave a
//! fill, but hover is not available to a user who is not already
//! pointing at the right word.
//!
//! # Why this measures the frame
//!
//! The rule is an element, so it has bounds, and bounds are what this
//! asserts — the same choice RFC-102 made for the tab bar's two edges
//! and for the same reason: a style property that is drawn rather than
//! laid out cannot be measured, and that is how a curled underline and
//! an outlined bar both shipped unnoticed.
//!
//! Ancestors are found by **clicking**, not by index: what makes a
//! crumb an ancestor is that pressing it emits
//! `BreadcrumbAction::Pressed`, and that is what the test uses to tell
//! them from the leaf.

#![cfg(feature = "widgets")]

use iced::widget::{column, space};
use iced::{Element, Length, Rectangle, Theme};

use snora::{BreadcrumbAction, Crumb, LayoutDirection};

mod common;
use common::{
    NON_TEXT_MIN, TOLERANCE, colour_of, containers, frame_pixels, peak_row, pointer_target,
    simulator,
};

/// The rule's intended height, as an expectation rather than an input:
/// `snora_widgets::crumb`'s constant is private, and making it public
/// to serve a test would widen the surface (the `side_bar_fit.rs`
/// reasoning).
const RULE_HEIGHT: f32 = 1.0;

const ANCESTORS: [&str; 3] = ["Home", "Library", "Books"];
const LEAF: &str = "The Hobbit";

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Crumb(BreadcrumbAction<u8>),
}

static ON_ACTION: fn(BreadcrumbAction<u8>) -> Msg = Msg::Crumb;

fn crumbs() -> Vec<Crumb<u8>> {
    let mut v: Vec<Crumb<u8>> = ANCESTORS
        .iter()
        .enumerate()
        .map(|(i, label)| Crumb::ancestor(i as u8, (*label).to_owned()))
        .collect();
    v.push(Crumb::leaf(ANCESTORS.len() as u8, LEAF.to_owned()));
    v
}

fn trail() -> Element<'static, Msg> {
    snora::widget::app_breadcrumb(crumbs(), &ON_ACTION, LayoutDirection::Ltr)
}

fn label_bounds(label: &str) -> Rectangle {
    let mut ui = simulator(trail());
    ui.find(label)
        .unwrap_or_else(|_| panic!("{label} is rendered"))
        .visible_bounds()
        .unwrap_or_else(|| panic!("{label} is visible"))
}

/// The crumb's own clickable box, found by clicking.
fn ancestor_box(index: u8, label: &str) -> Rectangle {
    pointer_target(
        label,
        trail,
        label_bounds(label).center(),
        move |m: &Msg| *m == Msg::Crumb(BreadcrumbAction::Pressed(index)),
    )
}

/// Every rule-shaped element inside `host` and below `label`.
fn rules_under(host: Rectangle, label: Rectangle) -> Vec<Rectangle> {
    let mut ui = simulator(trail());
    containers(&mut ui)
        .into_iter()
        .filter(|r| {
            (r.height - RULE_HEIGHT).abs() <= TOLERANCE
                && r.x >= host.x - TOLERANCE
                && r.x + r.width <= host.x + host.width + TOLERANCE
                && r.y >= label.y + label.height - TOLERANCE
                && r.y + r.height <= host.y + host.height + TOLERANCE
        })
        .collect()
}

#[test]
fn every_ancestor_has_a_rule_under_its_label() {
    for (i, label) in ANCESTORS.iter().enumerate() {
        let host = ancestor_box(i as u8, label);
        let text = label_bounds(label);
        let rules = rules_under(host, text);
        assert_eq!(
            rules.len(),
            1,
            "{label}: expected exactly one rule under the label and inside the button \
             {host:?} (label {text:?}), found {}: {rules:?}",
            rules.len(),
        );
        let rule = rules[0];
        assert!(
            rule.height >= RULE_HEIGHT - TOLERANCE,
            "{label}: the rule is {}px tall, under the {RULE_HEIGHT}px minimum",
            rule.height,
        );
        assert!(
            rule.width >= text.width - TOLERANCE,
            "{label}: the rule is {}px wide against a {}px label — it must span it",
            rule.width,
            text.width,
        );
    }
}

#[test]
fn the_leaf_has_no_rule() {
    let text = label_bounds(LEAF);
    // The leaf is plain text, so it has no button to bound the search;
    // its own box plus the row's height is the region a rule would have
    // to be in to read as the leaf's underline.
    let region = Rectangle {
        x: text.x,
        y: text.y + text.height - TOLERANCE,
        width: text.width,
        height: 8.0,
    };
    let mut ui = simulator(trail());
    let found: Vec<Rectangle> = containers(&mut ui)
        .into_iter()
        .filter(|r| {
            (r.height - RULE_HEIGHT).abs() <= TOLERANCE
                && r.x + r.width > region.x
                && r.x < region.x + region.width
                && r.y >= region.y
                && r.y <= region.y + region.height
        })
        .collect();
    assert!(
        found.is_empty(),
        "the leaf is underlined, which makes it look like an ancestor: {found:?}",
    );
}

/// Mirrors `CrumbGeometry::unstyled`'s per-button padding, as
/// expectations rather than inputs (the `side_bar_fit.rs` reasoning).
const BTN_PAD_X: f32 = 4.0;
const BTN_PAD_Y: f32 = 2.0;

/// The pointer target must not shrink: the rule adds height inside the
/// button, and nothing about this change may make a crumb harder to
/// hit.
///
/// **The obvious form of this assertion cannot fail.** Written as
/// "the button is at least as big as its label", it passes however
/// much padding is removed, because a button is always at least its
/// content — it passed unchanged with the padding cut from `[2, 4]` to
/// `[0, 2]`. What can fail, and what a user actually loses when the
/// padding goes, is the margin of button *outside* the label on each
/// side, so that is what this measures.
#[test]
fn ancestor_pointer_targets_keep_their_padding() {
    for (i, label) in ANCESTORS.iter().enumerate() {
        let host = ancestor_box(i as u8, label);
        let text = label_bounds(label);

        let left = text.x - host.x;
        let right = (host.x + host.width) - (text.x + text.width);
        let top = text.y - host.y;
        let bottom = (host.y + host.height) - (text.y + text.height);

        assert!(
            left >= BTN_PAD_X - TOLERANCE && right >= BTN_PAD_X - TOLERANCE,
            "{label}: the button keeps {left:.2}px left and {right:.2}px right of its label, \
             under the {BTN_PAD_X}px each side a crumb is padded by",
        );
        assert!(
            top >= BTN_PAD_Y - TOLERANCE,
            "{label}: the button keeps {top:.2}px above its label, under {BTN_PAD_Y}px",
        );
        // Below the label the button holds the rule as well as the
        // padding, so there is strictly more room than above.
        assert!(
            bottom >= BTN_PAD_Y + RULE_HEIGHT - TOLERANCE,
            "{label}: the button keeps {bottom:.2}px below its label, under the \
             {BTN_PAD_Y}px padding plus the {RULE_HEIGHT}px rule",
        );
    }
}

/// Where the rule lands, for the record. A 1px rule at a fractional
/// offset is spread over three physical rows at partial coverage
/// (RFC-108 measured exactly that on the tab bar), so the offset is
/// part of the result, not an implementation detail.
#[test]
fn report_rule_geometry() {
    for (i, label) in ANCESTORS.iter().enumerate() {
        let host = ancestor_box(i as u8, label);
        let text = label_bounds(label);
        let rules = rules_under(host, text);
        for rule in rules {
            println!(
                "crumb-rule | {label:<8} | button y={:.2} h={:.2} | label y={:.2} h={:.2} | \
                 rule y={:.2} h={:.2} w={:.2} | whole-pixel top: {}",
                host.y,
                host.height,
                text.y,
                text.height,
                rule.y,
                rule.height,
                rule.width,
                (rule.y - rule.y.round()).abs() < 0.001,
            );
        }
        let _ = i;
    }
}

// ---------------------------------------------------------------------
// Unit 3 — the rule as drawn
// ---------------------------------------------------------------------

/// The trail with page below it, so there is an undimmed page to read
/// the rule against.
fn trail_page() -> Element<'static, Msg> {
    column![trail(), space().height(Length::Fill)].into()
}

/// A patch of page well below the crumb row.
const PAGE_PATCH: Rectangle = Rectangle {
    x: 400.0,
    y: 200.0,
    width: 120.0,
    height: 60.0,
};

fn stock_themes() -> [(&'static str, Theme); 2] {
    [("stock light", Theme::Light), ("stock dark", Theme::Dark)]
}

#[cfg(feature = "design")]
fn design_presets() -> [(&'static str, snora::design::Tokens); 4] {
    use snora::design::Tokens;
    [
        ("design light", Tokens::light()),
        ("design dark", Tokens::dark()),
        ("design high_contrast_light", Tokens::high_contrast_light()),
        ("design high_contrast_dark", Tokens::high_contrast_dark()),
    ]
}

/// Measures one context's rule: its contrast against the page, and
/// whether the renderer drew it as a solid row or spread it.
fn assert_rule_drawn(
    failures: &mut Vec<String>,
    context: &str,
    build: impl Fn() -> Element<'static, Msg>,
    theme: &Theme,
) {
    let mut probe = simulator(build());
    let bar = containers(&mut probe)
        .into_iter()
        .filter(|r| (r.height - RULE_HEIGHT).abs() <= TOLERANCE)
        .min_by(|a, b| a.x.total_cmp(&b.x))
        .expect("an ancestor rule exists");

    let mut ui = simulator(build());
    let frame = frame_pixels(&mut ui, theme, &format!("crumb-rule-{context}"));
    let page = colour_of(frame.modal_colour(PAGE_PATCH));

    // Inset horizontally so the label's own glyph edges above cannot
    // reach the sampled rows.
    let region = Rectangle {
        x: bar.x + 2.0,
        y: bar.y,
        width: bar.width - 4.0,
        height: bar.height,
    };

    let (y, colour, peak) = peak_row(&frame, region, page);
    let rows = frame.row_modes(region).len();
    println!(
        "crumb-drawn | {context:<28} | peak {peak:5.2} (floor {NON_TEXT_MIN:.1}) | \
         rule y={:.2} | physical rows {rows}, peak row {y} | {colour:?}",
        bar.y,
    );
    if peak < NON_TEXT_MIN {
        failures.push(format!(
            "{context}: the ancestor rule as drawn is {peak:.2} < {NON_TEXT_MIN} \
             against the page ({colour:?} vs {page:?})",
        ));
    }
}

#[test]
fn the_rule_clears_the_non_text_floor_as_drawn_unstyled() {
    let mut failures = Vec::new();
    for (context, theme) in stock_themes() {
        assert_rule_drawn(&mut failures, context, trail_page, &theme);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[cfg(feature = "design")]
#[test]
fn the_rule_clears_the_non_text_floor_as_drawn_design() {
    let mut failures = Vec::new();
    for (context, tokens) in design_presets() {
        let theme = snora::design::theme(&tokens);
        let build = {
            let tokens = tokens.clone();
            move || -> Element<'static, Msg> {
                let trail = snora::design::widget::app_breadcrumb(
                    &tokens,
                    crumbs(),
                    &ON_ACTION,
                    LayoutDirection::Ltr,
                );
                column![trail, space().height(Length::Fill)].into()
            }
        };
        assert_rule_drawn(&mut failures, context, build, &theme);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
