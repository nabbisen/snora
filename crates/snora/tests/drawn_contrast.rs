//! Contrast read from the pixels that were actually drawn (RFC-108 R-2).
//!
//! # Why this exists beside the style suites
//!
//! Every contrast assertion snora had before this one reads a **style
//! function's return value**: `tab_bar_rule_style(&theme).background`,
//! `sidebar_active_color(&theme)`, and so on. That answers "is the
//! colour we intend to use sufficient?" It cannot answer "is the colour
//! that reached the screen sufficient?", and the two differ whenever
//! something between the style and the frame changes the result — a
//! translucent layer composited underneath, an element laid out at a
//! fractional offset so the renderer spreads it over two rows at partial
//! coverage, or a style that is computed and then never used.
//!
//! These tests read the frame. Each one finds its element the way a user
//! reaches it — by clicking, or by what the element does — takes the
//! colour that element actually painted, and asserts it against **the
//! same floor the element's style test uses**, so the two suites cannot
//! disagree about what the requirement is.
//!
//! # Thin elements and peak coverage
//!
//! `Simulator::snapshot` fixes the scale factor at 2.0, so a 1 logical
//! pixel rule is two physical rows — if it lands on a whole pixel. At a
//! fractional offset the renderer spreads it over three rows at partial
//! coverage, and the region's modal colour is then a blend that is
//! weaker than the colour the style asked for. For any element under
//! two logical pixels this file asserts **both**: the region's modal
//! colour, and the strongest single row, which at scale 2 is the solid
//! core row if there is one. The second is what a user's eye picks out;
//! the first is what the element averages to.
//!
//! # Text is out of scope, with one named exception
//!
//! Glyph edges are antialiased by design, so most pixels of a glyph are
//! a blend of the text colour and whatever is behind it. The floor
//! applies to the style colour, and the style suites already assert it.
//! The one exception is the toast, whose row here needs a text colour to
//! assert its fill against and whose style function is private to the
//! crate; [`peak_colour`] reads it from fully-covered core pixels rather
//! than from the blend. See that test for the argument that this is
//! exact rather than approximate.

#![cfg(feature = "widgets")]

use iced::widget::{column, row, space};
use iced::{Color, Element, Length, Point, Rectangle, Theme};

mod common;
use common::{
    AA_TEXT, Frame, NON_TEXT_MIN, colour_of, containers, contrast_ratio, frame_pixels, peak_colour,
    peak_row, pointer_target, simulator,
};

#[test]
fn contrast_ratio_matches_wcag_reference_identity() {
    let black = Color::BLACK;
    let white = Color::WHITE;
    assert!(
        (contrast_ratio(black, white) - 21.0).abs() < 0.01,
        "black on white must be 21:1, got {}",
        contrast_ratio(black, white),
    );
    assert!((contrast_ratio(white, white) - 1.0).abs() < 0.01);
}

/// Both stock themes. The unstyled path's own two contexts, matching
/// `theme_contexts`' stock half in `snora-widgets`' contrast suite.
fn stock_themes() -> [(&'static str, Theme); 2] {
    [("stock light", Theme::Light), ("stock dark", Theme::Dark)]
}

/// Prints one measured row. `cargo test -- --nocapture` turns this file
/// into the measurement table the review asks for, from the same numbers
/// the assertions use rather than from a second pass that could drift.
fn report(context: &str, element: &str, ratio: f32, floor: f32, detail: &str) {
    println!(
        "drawn-contrast | {context:<28} | {element:<34} | {ratio:6.2} (floor {floor:.1}) | {detail}"
    );
}

/// Collects every failure and reports them together, so one run names
/// every context that is short rather than only the first.
fn assert_no_failures(failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "{} contrast failure(s) read from drawn pixels:\n  {}",
        failures.len(),
        failures.join("\n  "),
    );
}

// ---------------------------------------------------------------------
// The tab bar's two thin elements
// ---------------------------------------------------------------------

const TAB_LABELS: [&str; 3] = ["Alpha", "Beta", "Gamma"];
const ACTIVE_TAB: u8 = 1;

/// The bar's bottom rule and the active tab's underline, as RFC-102 drew
/// them: one logical pixel and two.
const BAR_RULE_HEIGHT: f32 = 1.0;
const INDICATOR_HEIGHT: f32 = 2.0;

#[derive(Debug, Clone, PartialEq)]
enum TabMsg {
    Tab(snora::TabAction<u8>),
}

static ON_TAB: fn(snora::TabAction<u8>) -> TabMsg = TabMsg::Tab;

fn tab_bar_page() -> Element<'static, TabMsg> {
    let bar = snora::widget::app_tab_bar(
        snora::TabBar {
            tabs: TAB_LABELS
                .iter()
                .enumerate()
                .map(|(i, label)| snora::Tab {
                    id: i as u8,
                    label: (*label).to_owned(),
                    icon: None,
                })
                .collect(),
            active: ACTIVE_TAB,
        },
        &ON_TAB,
        snora::LayoutDirection::Ltr,
    );
    column![bar, space().height(Length::Fill)].into()
}

/// The sole container of the given height — the rule or the underline.
///
/// Identified by what it is rather than by where it sits: these are the
/// only elements in the bar with these heights, which is the same
/// identification `tab_bar_edges.rs` makes and asserts to be unique.
fn sole_element_of_height(bounds: &[Rectangle], height: f32, what: &str) -> Rectangle {
    let matches: Vec<Rectangle> = bounds
        .iter()
        .copied()
        .filter(|r| (r.height - height).abs() <= 0.5)
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "expected exactly one {height}px element ({what}), found {}: {matches:?}",
        matches.len(),
    );
    matches[0]
}

/// A patch of page well below the bar, in logical coordinates. The bar is
/// about 38px tall at the default text size; 200 is far clear of it.
const PAGE_PATCH: Rectangle = Rectangle {
    x: 400.0,
    y: 200.0,
    width: 120.0,
    height: 60.0,
};

#[test]
fn tab_bar_bottom_rule_meets_non_text_floor_as_drawn() {
    let mut failures = Vec::new();
    for (context, theme) in stock_themes() {
        let mut ui = simulator(tab_bar_page());
        let rule = sole_element_of_height(&containers(&mut ui), BAR_RULE_HEIGHT, "the bar's rule");

        let frame = frame_pixels(&mut ui, &theme, &format!("tab-rule-{context}"));
        let page = colour_of(frame.modal_colour(PAGE_PATCH));

        let drawn = colour_of(frame.modal_colour(rule));
        let r = contrast_ratio(drawn, page);
        report(
            context,
            "tab bar bottom rule (modal)",
            r,
            NON_TEXT_MIN,
            &format!("y={:.1}", rule.y),
        );
        if r < NON_TEXT_MIN {
            failures.push(format!(
                "{context}: tab bar bottom rule as drawn {r:.2} < {NON_TEXT_MIN} \
                 (rule {drawn:?} vs page {page:?}, bounds {rule:?})",
            ));
        }

        // Under two logical pixels: the solid core row must clear it too.
        let (y, colour, peak) = peak_row(&frame, rule, page);
        report(
            context,
            "tab bar bottom rule (peak row)",
            peak,
            NON_TEXT_MIN,
            &format!("physical y={y}"),
        );
        if peak < NON_TEXT_MIN {
            failures.push(format!(
                "{context}: tab bar rule's strongest row (physical y={y}) {peak:.2} \
                 < {NON_TEXT_MIN} ({colour:?} vs page {page:?})",
            ));
        }
    }
    assert_no_failures(failures);
}

#[test]
fn tab_indicator_meets_non_text_floor_as_drawn() {
    let mut failures = Vec::new();
    for (context, theme) in stock_themes() {
        let mut ui = simulator(tab_bar_page());
        let indicator = sole_element_of_height(
            &containers(&mut ui),
            INDICATOR_HEIGHT,
            "the active underline",
        );

        let frame = frame_pixels(&mut ui, &theme, &format!("tab-indicator-{context}"));
        let page = colour_of(frame.modal_colour(PAGE_PATCH));

        let drawn = colour_of(frame.modal_colour(indicator));
        let r = contrast_ratio(drawn, page);
        report(
            context,
            "active tab underline (modal)",
            r,
            NON_TEXT_MIN,
            &format!("y={:.1}", indicator.y),
        );
        if r < NON_TEXT_MIN {
            failures.push(format!(
                "{context}: active tab underline as drawn {r:.2} < {NON_TEXT_MIN} \
                 (underline {drawn:?} vs page {page:?}, bounds {indicator:?})",
            ));
        }

        let (y, colour, peak) = peak_row(&frame, indicator, page);
        report(
            context,
            "active tab underline (peak row)",
            peak,
            NON_TEXT_MIN,
            &format!("physical y={y}"),
        );
        if peak < NON_TEXT_MIN {
            failures.push(format!(
                "{context}: underline's strongest row (physical y={y}) {peak:.2} \
                 < {NON_TEXT_MIN} ({colour:?} vs page {page:?})",
            ));
        }
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// The sidebar's active highlight, against the rail it sits on
// ---------------------------------------------------------------------

/// Mirrors `snora_widgets::sidebar`'s private geometry, as
/// `side_bar_fit.rs` does and for the same reason: making a constant
/// public to serve a test widens the surface. These are expectations.
const RAIL_WIDTH: f32 = 64.0;
const BUTTON_SIZE: f32 = 48.0;
const RAIL_VERTICAL_PADDING: f32 = 16.0;

#[derive(Debug, Clone, PartialEq)]
enum RailMsg {
    First,
    Second,
}

fn side_bar_page() -> Element<'static, RailMsg> {
    let rail = snora::widget::app_side_bar(
        snora::SideBar {
            items: vec![
                snora::SideBarItem {
                    view_id: 0,
                    icon: snora::Icon::Text("A".into()),
                    tooltip: "first view".into(),
                    on_press: RailMsg::First,
                },
                snora::SideBarItem {
                    view_id: 1,
                    icon: snora::Icon::Text("B".into()),
                    tooltip: "second view".into(),
                    on_press: RailMsg::Second,
                },
            ],
            active: 0,
        },
        snora::LayoutDirection::Ltr,
    );
    row![rail, space().width(Length::Fill)].into()
}

#[test]
fn sidebar_active_highlight_meets_non_text_floor_against_rail_as_drawn() {
    let mut failures = Vec::new();
    for (context, theme) in stock_themes() {
        // The active button is found by clicking, not by position: what
        // is highlighted is whatever answers `RailMsg::First`.
        let inside = Point::new(RAIL_WIDTH / 2.0, RAIL_VERTICAL_PADDING + BUTTON_SIZE / 2.0);
        let button = pointer_target(context, side_bar_page, inside, |m: &RailMsg| {
            *m == RailMsg::First
        });

        let mut ui = simulator(side_bar_page());
        let frame = frame_pixels(&mut ui, &theme, &format!("rail-{context}"));

        // The rail's own background, read from a strip above the first
        // button — inside the rail, outside every button.
        let rail_patch = Rectangle {
            x: 2.0,
            y: 2.0,
            width: RAIL_WIDTH - 4.0,
            height: RAIL_VERTICAL_PADDING - 4.0,
        };
        let rail = colour_of(frame.modal_colour(rail_patch));

        // The highlight, read from the button's own box. Its modal
        // colour is the fill: the glyph covers a small fraction of it.
        let fill = colour_of(frame.modal_colour(button));
        let r = contrast_ratio(fill, rail);
        report(
            context,
            "sidebar active fill vs rail",
            r,
            NON_TEXT_MIN,
            &format!("button {:.0}x{:.0}", button.width, button.height),
        );
        if r < NON_TEXT_MIN {
            failures.push(format!(
                "{context}: active sidebar fill as drawn {r:.2} < {NON_TEXT_MIN} \
                 (fill {fill:?} vs rail {rail:?}, button {button:?})",
            ));
        }
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// The chrome container's border
// ---------------------------------------------------------------------

const HEADER_TITLE: &str = "Chrome title";

#[derive(Debug, Clone, PartialEq)]
enum HeaderMsg {
    Menu(snora::MenuAction<u8, u8>),
}

static ON_MENU: fn(snora::MenuAction<u8, u8>) -> HeaderMsg = HeaderMsg::Menu;

fn header_page() -> Element<'static, HeaderMsg> {
    let header = snora::widget::app_header(
        HEADER_TITLE,
        Vec::new(),
        &ON_MENU,
        None,
        None,
        snora::LayoutDirection::Ltr,
    );
    column![header, space().height(Length::Fill)].into()
}

/// The chrome container: the full-width container that holds the title.
///
/// Identified by what it contains and how far it spans, not by its index
/// in the tree — several containers hold the title, and only the chrome
/// one runs the width of the window.
fn chrome_container(ui: &mut iced_test::Simulator<'_, HeaderMsg>) -> Rectangle {
    let title = ui
        .find(HEADER_TITLE)
        .expect("the header draws its title")
        .visible_bounds()
        .expect("the title is visible");

    let mut holding: Vec<Rectangle> = containers(ui)
        .into_iter()
        .filter(|r| r.width >= 1000.0 && r.contains(title.center()))
        .collect();
    holding.sort_by(|a, b| (a.height).total_cmp(&b.height));
    *holding
        .first()
        .expect("a full-width container holds the title")
}

#[test]
fn chrome_container_border_meets_non_text_floor_as_drawn() {
    let mut failures = Vec::new();
    for (context, theme) in stock_themes() {
        let mut ui = simulator(header_page());
        let chrome = chrome_container(&mut ui);

        let frame = frame_pixels(&mut ui, &theme, &format!("chrome-{context}"));
        let page = colour_of(frame.modal_colour(PAGE_PATCH));

        // The bottom border: the last logical pixel of the container,
        // away from the corners so a radius cannot be sampled.
        let border = Rectangle {
            x: chrome.x + 40.0,
            y: chrome.y + chrome.height - 1.0,
            width: chrome.width - 80.0,
            height: 1.0,
        };

        let drawn = colour_of(frame.modal_colour(border));
        let r = contrast_ratio(drawn, page);
        report(
            context,
            "chrome container border (modal)",
            r,
            NON_TEXT_MIN,
            &format!("y={:.1}", border.y),
        );
        if r < NON_TEXT_MIN {
            failures.push(format!(
                "{context}: chrome container border as drawn {r:.2} < {NON_TEXT_MIN} \
                 (border {drawn:?} vs page {page:?}, chrome {chrome:?})",
            ));
        }

        // One logical pixel wide: the solid core row must clear it too.
        let (y, colour, peak) = peak_row(&frame, border, page);
        report(
            context,
            "chrome container border (peak row)",
            peak,
            NON_TEXT_MIN,
            &format!("physical y={y}"),
        );
        if peak < NON_TEXT_MIN {
            failures.push(format!(
                "{context}: chrome border's strongest row (physical y={y}) {peak:.2} \
                 < {NON_TEXT_MIN} ({colour:?} vs page {page:?})",
            ));
        }
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// The tooltip body's border — located by what hovering changes
// ---------------------------------------------------------------------

/// The bounding box, in **logical** coordinates, of every pixel that
/// differs between two frames of the same size.
///
/// This is how the tooltip is found. A tooltip overlay takes no part in
/// widget operations in iced 0.14, so `Simulator::find` cannot see it at
/// all (RFC-100 §4) and it has no bounds to ask for. What it does have
/// is an effect: hovering makes it appear. The region that changed when
/// the pointer arrived *is* the tooltip, identified by its behaviour
/// rather than by any position this test chose.
fn changed_bounds(before: &Frame, after: &Frame) -> Rectangle {
    assert_eq!(
        (before.width, before.height),
        (after.width, after.height),
        "frames must be the same size to diff"
    );

    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    for y in 0..before.height {
        for x in 0..before.width {
            if before.pixel(x, y) != after.pixel(x, y) {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x + 1);
                y1 = y1.max(y + 1);
            }
        }
    }
    assert!(
        x0 != u32::MAX,
        "nothing changed between the two frames; the tooltip did not open"
    );

    Rectangle {
        x: x0 as f32 / Frame::SCALE,
        y: y0 as f32 / Frame::SCALE,
        width: (x1 - x0) as f32 / Frame::SCALE,
        height: (y1 - y0) as f32 / Frame::SCALE,
    }
}

/// Renders the rail beside an empty page, optionally with the pointer
/// resting on the first button.
fn rail_frame(
    build: impl Fn() -> Element<'static, RailMsg>,
    theme: &Theme,
    hovered: bool,
    name: &str,
) -> Frame {
    let mut ui = simulator(build());
    if hovered {
        let position = Point::new(RAIL_WIDTH / 2.0, RAIL_VERTICAL_PADDING + BUTTON_SIZE / 2.0);
        ui.point_at(position);
        let _ = ui.simulate([iced::Event::Mouse(iced::mouse::Event::CursorMoved {
            position,
        })]);
    }
    frame_pixels(&mut ui, theme, name)
}

/// Asserts a hovered rail's tooltip body is outlined against the page.
///
/// The body's border is its outermost logical pixel. Both the ring's
/// modal colour and its strongest row are asserted, because the border
/// is one logical pixel and so is subject to the same coverage question
/// as the tab bar's rule.
fn assert_tooltip_border(
    failures: &mut Vec<String>,
    context: &str,
    build: impl Fn() -> Element<'static, RailMsg>,
    theme: &Theme,
) {
    let plain = rail_frame(&build, theme, false, &format!("tooltip-plain-{context}"));
    let hovered = rail_frame(&build, theme, true, &format!("tooltip-hover-{context}"));
    let body = changed_bounds(&plain, &hovered);

    // The page the body floats over, read beside it from the frame that
    // has no tooltip in the way.
    let page = colour_of(plain.modal_colour(Rectangle {
        x: body.x + 4.0,
        y: body.y + body.height + 20.0,
        width: 40.0,
        height: 16.0,
    }));

    // The outline, found relative to the fill rather than assumed to be
    // the first thing that appeared.
    //
    // **The first row of the changed region is not the border.** On the
    // design path the body carries a shadow, and the shadow appears
    // first: measured on design light, physical rows 45-52 are a
    // 254->248 gradient, row 54 is the border at (137,140,143), and row
    // 56 onward is the fill at 255. Reading row 45 as the border
    // reported 1.01:1 in three presets and looked exactly like a
    // defect.
    //
    // So: walk up from the body's centre to where the fill stops, and
    // take the strongest row between there and the top of what
    // appeared. That is the outline — a shadow is a weaker gradient by
    // construction, and the fill is excluded by where the search ends.
    let centre_x = body.x + body.width / 2.0;
    let interior = colour_of(hovered.modal_colour(Rectangle {
        x: centre_x - 4.0,
        y: body.y + body.height / 2.0 - 2.0,
        width: 8.0,
        height: 4.0,
    }));

    let px = (centre_x * Frame::SCALE) as u32;
    let centre_row = ((body.y + body.height / 2.0) * Frame::SCALE) as u32;
    let top_row = (body.y * Frame::SCALE) as u32;
    let mut fill_top = centre_row;
    while fill_top > top_row
        && colour_of(hovered.pixel(px, fill_top - 1).expect("inside the frame")) == interior
    {
        fill_top -= 1;
    }
    assert!(
        fill_top > top_row,
        "{context}: the fill reaches the top of the changed region, so there is no outline          between them to measure"
    );

    let top = Rectangle {
        x: body.x + 6.0,
        y: top_row as f32 / Frame::SCALE,
        width: (body.width - 12.0).max(4.0),
        height: (fill_top - top_row) as f32 / Frame::SCALE,
    };

    let (peak_y, drawn, _) = peak_row(&hovered, top, page);
    let r = contrast_ratio(drawn, page);
    report(
        context,
        "tooltip body border",
        r,
        NON_TEXT_MIN,
        &format!(
            "body {:.0}x{:.0}, outline rows {top_row}..{fill_top}, peak y={peak_y}",
            body.width, body.height
        ),
    );
    if r < NON_TEXT_MIN {
        failures.push(format!(
            "{context}: tooltip body border as drawn {r:.2} < {NON_TEXT_MIN} \
             (border {drawn:?} vs page {page:?}, body {body:?})",
        ));
    }
}

#[test]
fn unstyled_tooltip_body_border_meets_non_text_floor_as_drawn() {
    let mut failures = Vec::new();
    for (context, theme) in stock_themes() {
        assert_tooltip_border(&mut failures, context, side_bar_page, &theme);
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// The design path: four presets, the token-derived theme
// ---------------------------------------------------------------------

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

/// The iced `Theme` a preset's tokens derive, so the frame is rendered
/// on the same page the application would see.
#[cfg(feature = "design")]
fn design_theme(tokens: &snora::design::Tokens) -> Theme {
    snora::design::theme(tokens)
}

#[cfg(feature = "design")]
#[test]
fn styled_tooltip_body_border_meets_non_text_floor_as_drawn() {
    let mut failures = Vec::new();
    for (context, tokens) in design_presets() {
        let theme = design_theme(&tokens);
        let build = {
            let tokens = tokens.clone();
            move || -> Element<'static, RailMsg> {
                let rail = snora::design::widget::app_side_bar(
                    &tokens,
                    snora::SideBar {
                        items: vec![
                            snora::SideBarItem {
                                view_id: 0,
                                icon: snora::Icon::Text("A".into()),
                                tooltip: "first view".into(),
                                on_press: RailMsg::First,
                            },
                            snora::SideBarItem {
                                view_id: 1,
                                icon: snora::Icon::Text("B".into()),
                                tooltip: "second view".into(),
                                on_press: RailMsg::Second,
                            },
                        ],
                        active: 0,
                    },
                    snora::LayoutDirection::Ltr,
                );
                row![rail, space().width(Length::Fill)].into()
            }
        };
        assert_tooltip_border(&mut failures, context, build, &theme);
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// The toast's fill, against the text drawn on it
// ---------------------------------------------------------------------

const TOAST_TITLE: &str = "Saved";
const TOAST_ID: u64 = 7;

#[derive(Debug, Clone, PartialEq)]
enum ToastMsg {
    Body,
    Dismiss(u64),
}

fn toast_intents() -> [(&'static str, snora::ToastIntent); 5] {
    use snora::ToastIntent;
    [
        ("info", ToastIntent::Info),
        ("success", ToastIntent::Success),
        ("warning", ToastIntent::Warning),
        ("error", ToastIntent::Error),
        ("debug", ToastIntent::Debug),
    ]
}

fn toast_layout(
    intent: snora::ToastIntent,
) -> snora::AppLayout<Element<'static, ToastMsg>, ToastMsg> {
    let toast = snora::Toast::new(
        TOAST_ID,
        intent,
        TOAST_TITLE,
        "All good.",
        ToastMsg::Dismiss(TOAST_ID),
    );
    let body: Element<'static, ToastMsg> = iced::widget::button(iced::widget::text("body"))
        .on_press(ToastMsg::Body)
        .into();
    snora::AppLayout::new(body).toasts(vec![toast])
}

/// The unstyled engine path, as an application with no tokens gets it.
fn toast_page(intent: snora::ToastIntent) -> Element<'static, ToastMsg> {
    snora::render(toast_layout(intent))
}

/// The `design` engine path, which takes the tokens and styles the
/// toast from them (RFC-104). A different renderer, not the unstyled
/// one under a token-derived theme.
#[cfg(feature = "design")]
fn design_toast_page(
    intent: snora::ToastIntent,
    tokens: &snora::design::Tokens,
) -> Element<'static, ToastMsg> {
    snora::design::render(toast_layout(intent), tokens)
}

/// The toast surface: the smallest container that holds the title and is
/// wider than the title itself — the card, not the text's own box.
fn toast_surface(ui: &mut iced_test::Simulator<'_, ToastMsg>) -> (Rectangle, Rectangle) {
    let title = ui
        .find(TOAST_TITLE)
        .expect("the toast draws its title")
        .visible_bounds()
        .expect("the title is visible");

    let mut holding: Vec<Rectangle> = containers(ui)
        .into_iter()
        .filter(|r| r.contains(title.center()) && r.width > title.width + 20.0)
        .collect();
    holding.sort_by(|a, b| (a.width * a.height).total_cmp(&(b.width * b.height)));
    (
        *holding.first().expect("a card holds the toast title"),
        title,
    )
}

/// Asserts a toast's drawn fill clears AA against the text drawn on it.
///
/// **This is the one place this file reads text pixels, and it is exact
/// rather than approximate.** Most pixels of a glyph are a blend, which
/// is why text is otherwise out of scope here. [`peak_row`] does not
/// take a blend: it takes the row furthest from the fill, and at scale 2
/// a 16px glyph's stems are several physical pixels wide, so that row
/// contains pixels the renderer covered *fully* — which carry the style's
/// own text colour, unblended. The alternative was to read the colour
/// from `snora::toast`'s style function, which is private to the crate
/// and which RFC-101 Q-3 declined to make public for a test's sake.
fn assert_toast_contrast(
    failures: &mut Vec<String>,
    context: &str,
    page: Element<'static, ToastMsg>,
    theme: &Theme,
) {
    let mut ui = simulator(page);
    let (surface, title) = toast_surface(&mut ui);
    let frame = frame_pixels(&mut ui, theme, &format!("toast-{context}"));

    // The fill: the surface's modal colour. Text and the close glyph
    // cover a small fraction of a 340px-wide card.
    let fill = colour_of(frame.modal_colour(surface));

    // The text: the fully-covered core of the title's own glyphs.
    let text = peak_colour(&frame, title, fill);

    let r = contrast_ratio(text, fill);
    report(
        context,
        "toast text on its drawn fill",
        r,
        AA_TEXT,
        &format!("fill {fill:?}"),
    );
    if r < AA_TEXT {
        failures.push(format!(
            "{context}: toast text on its drawn fill {r:.2} < {AA_TEXT} \
             (text {text:?} vs fill {fill:?})",
        ));
    }
}

#[test]
fn toast_text_meets_aa_on_its_drawn_fill_unstyled() {
    let mut failures = Vec::new();
    for (theme_name, theme) in stock_themes() {
        for (intent_name, intent) in toast_intents() {
            assert_toast_contrast(
                &mut failures,
                &format!("{theme_name} / {intent_name}"),
                toast_page(intent),
                &theme,
            );
        }
    }
    assert_no_failures(failures);
}

#[cfg(feature = "design")]
#[test]
fn toast_text_meets_aa_on_its_drawn_fill_design() {
    let mut failures = Vec::new();
    for (preset, tokens) in design_presets() {
        let theme = design_theme(&tokens);
        for (intent_name, intent) in toast_intents() {
            assert_toast_contrast(
                &mut failures,
                &format!("{preset} / {intent_name}"),
                design_toast_page(intent, &tokens),
                &theme,
            );
        }
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// The chip's fills: selected against unselected, disabled against
// enabled (RFC-103's state register, RFC-106's fix)
// ---------------------------------------------------------------------

/// The register's floor for "a selected chip differs from an unselected
/// one": `State::ChipSelected => Cue::Luminance { min: 3.0, .. }`.
#[cfg(feature = "design")]
const CHIP_SELECTED_MIN: f32 = 3.0;

/// The register's floor for "a disabled selected chip differs from an
/// enabled one": `State::ChipSelectedDisabled => Cue::Luminance { min: 2.6, .. }`.
#[cfg(feature = "design")]
const CHIP_DISABLED_MIN: f32 = 2.6;

#[cfg(feature = "design")]
const CHIP_LABEL: &str = "Filter";

#[cfg(feature = "design")]
#[derive(Debug, Clone, PartialEq)]
enum ChipMsg {
    Toggle,
}

#[cfg(feature = "design")]
fn chip_page(
    tokens: &snora::design::Tokens,
    selected: bool,
    enabled: bool,
) -> Element<'static, ChipMsg> {
    let on_toggle = enabled.then_some(ChipMsg::Toggle);
    let chip = snora::design::chip::filter(tokens, CHIP_LABEL, selected, on_toggle);
    column![chip, space().height(Length::Fill)].into()
}

/// The chip's own box: the smallest container holding its label that is
/// wider than the label — the button, not the text's own rectangle.
///
/// A disabled chip answers no click, so it cannot be found with
/// `pointer_target` the way an enabled control is. It is identified
/// instead by the label it carries, which is the thing a user reads.
#[cfg(feature = "design")]
fn chip_box(ui: &mut iced_test::Simulator<'_, ChipMsg>) -> Rectangle {
    let label = ui
        .find(CHIP_LABEL)
        .expect("the chip draws its label")
        .visible_bounds()
        .expect("the label is visible");

    let mut holding: Vec<Rectangle> = containers(ui)
        .into_iter()
        .filter(|r| r.contains(label.center()) && r.width > label.width + 4.0)
        .collect();
    holding.sort_by(|a, b| (a.width * a.height).total_cmp(&(b.width * b.height)));
    *holding.first().expect("a button holds the chip label")
}

/// The chip's drawn fill in one state.
///
/// Read from a strip inside the button but clear of its label, so the
/// glyphs cannot drag the measurement: the chip's left padding.
#[cfg(feature = "design")]
fn chip_fill(
    tokens: &snora::design::Tokens,
    theme: &Theme,
    selected: bool,
    enabled: bool,
    name: &str,
) -> Color {
    let mut ui = simulator(chip_page(tokens, selected, enabled));
    let chip = chip_box(&mut ui);
    let frame = frame_pixels(&mut ui, theme, name);
    colour_of(frame.modal_colour(Rectangle {
        x: chip.x + 1.0,
        y: chip.y + chip.height / 2.0 - 2.0,
        width: 4.0,
        height: 4.0,
    }))
}

#[cfg(feature = "design")]
#[test]
fn chip_fills_meet_their_register_floors_as_drawn() {
    let mut failures = Vec::new();
    for (context, tokens) in design_presets() {
        let theme = design_theme(&tokens);

        let unselected = chip_fill(&tokens, &theme, false, true, &format!("chip-un-{context}"));
        let selected = chip_fill(&tokens, &theme, true, true, &format!("chip-sel-{context}"));
        let selected_disabled = chip_fill(
            &tokens,
            &theme,
            true,
            false,
            &format!("chip-sel-dis-{context}"),
        );

        let r = contrast_ratio(selected, unselected);
        report(
            context,
            "chip selected vs unselected fill",
            r,
            CHIP_SELECTED_MIN,
            "",
        );
        if r < CHIP_SELECTED_MIN {
            failures.push(format!(
                "{context}: selected chip fill as drawn {r:.2} < {CHIP_SELECTED_MIN} \
                 (selected {selected:?} vs unselected {unselected:?})",
            ));
        }

        let r = contrast_ratio(selected_disabled, selected);
        report(
            context,
            "chip disabled vs enabled fill",
            r,
            CHIP_DISABLED_MIN,
            "",
        );
        if r < CHIP_DISABLED_MIN {
            failures.push(format!(
                "{context}: disabled selected chip fill as drawn {r:.2} < {CHIP_DISABLED_MIN} \
                 (disabled {selected_disabled:?} vs enabled {selected:?})",
            ));
        }
    }
    assert_no_failures(failures);
}

// ---------------------------------------------------------------------
// The unstyled modal dim, as drawn (RFC-110's optional check, which
// RFC-108 R-1 made possible)
// ---------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum DimMsg {
    Body,
    Ok,
}

fn dim_page(dialog: bool) -> Element<'static, DimMsg> {
    let body: Element<'static, DimMsg> = iced::widget::button(iced::widget::text("body"))
        .on_press(DimMsg::Body)
        .into();
    let layout = snora::AppLayout::new(body);
    if dialog {
        let content: Element<'static, DimMsg> = iced::widget::button(iced::widget::text("OK"))
            .on_press(DimMsg::Ok)
            .into();
        snora::render(layout.dialog(snora::Dialog::new(content)))
    } else {
        snora::render(layout)
    }
}

/// The dim's derivation reaches the screen.
///
/// `render.rs`'s own unit tests assert the rule and the ratio from the
/// colour the style produces. This asserts the separate thing a style
/// test cannot: that the colour is actually painted over the page. The
/// dim is translucent, so what is measured is the page with the dim on
/// it against the same page without — which is the pairing RFC-110 is
/// about, and the only one a user ever sees.
#[test]
fn the_unstyled_dim_is_drawn_over_the_page() {
    let mut failures = Vec::new();
    for (context, theme) in stock_themes() {
        // A corner, far from the centred dialog card.
        let corner = Rectangle {
            x: 8.0,
            y: 700.0,
            width: 60.0,
            height: 40.0,
        };

        let mut plain = simulator(dim_page(false));
        let undimmed = colour_of(
            frame_pixels(&mut plain, &theme, &format!("dim-plain-{context}")).modal_colour(corner),
        );

        let mut modal = simulator(dim_page(true));
        let dimmed = colour_of(
            frame_pixels(&mut modal, &theme, &format!("dim-modal-{context}")).modal_colour(corner),
        );

        let r = contrast_ratio(dimmed, undimmed);
        report(
            context,
            "modal dim, page vs dimmed page",
            r,
            NON_TEXT_MIN,
            &format!("{undimmed:?} -> {dimmed:?}"),
        );
        if r < NON_TEXT_MIN {
            failures.push(format!(
                "{context}: the drawn dim separates the page from itself by only {r:.2} \
                 < {NON_TEXT_MIN} ({undimmed:?} -> {dimmed:?})",
            ));
        }
    }
    assert_no_failures(failures);
}

/// The coverage profile of the two thin tab elements, row by row.
///
/// Both ask for `snap`, and whether that is honoured depends on the
/// renderer: `iced_wgpu` reads `Quad::snap`, `iced_tiny_skia` ignores it
/// in iced 0.14. The harness is pinned to tiny-skia, so this prints
/// rather than asserts — run it under `ICED_TEST_BACKEND=wgpu` and the
/// same output says whether snapping changed anything.
#[test]
fn report_thin_element_coverage() {
    let mut ui = simulator(tab_bar_page());
    let bounds = containers(&mut ui);
    let rule = sole_element_of_height(&bounds, BAR_RULE_HEIGHT, "the bar's rule");
    let indicator = sole_element_of_height(&bounds, INDICATOR_HEIGHT, "the active underline");

    let frame = frame_pixels(&mut ui, &Theme::Light, "thin-coverage");
    for (name, region, x) in [
        ("tab bar rule", rule, 500u32),
        ("tab underline", indicator, (indicator.x + 10.0) as u32),
    ] {
        let first = (region.y * Frame::SCALE).floor() as u32;
        let rows: Vec<String> = (first..first + (region.height * Frame::SCALE) as u32 + 2)
            .map(|y| {
                let p = frame.pixel(x * 2, y).unwrap_or([0, 0, 0, 0]);
                format!("{y}:{}", p[0])
            })
            .collect();
        println!(
            "thin-coverage | renderer={} | {name:<14} | y={:.2} h={:.1} | rows (physical:red) {}",
            frame.renderer,
            region.y,
            region.height,
            rows.join(" "),
        );
    }
}
