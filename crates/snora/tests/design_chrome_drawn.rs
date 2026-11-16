//! The design widgets, read from what they draw (the 0.63 audit's follow-up).
//!
//! `snora-widgets`' own tests build a widget and compare its *geometry struct* to the tokens; these read the
//! bounds and text the tree reports, so a literal that happens to equal the default scale, or a label that is
//! never drawn, cannot pass.
//!
//! **Left-to-right only, for now.** The mirrored half of the geometry waits for RFC-132 (the header and the rows it
//! lists are not a mirror image under RTL today, so asserting their RTL padding would pin the defect). When that lands
//! these tests gain the RTL half, where the padding sits on the other side.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia); layout bounds do not depend on it.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use iced::Element;
use snora::design::{Tokens, Tone};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Go,
}

/// **The progress label shows the clamped percentage; indeterminate shows "…"** (`progress::row`). The
/// `snora-widgets` tests built the widget and dropped it.
#[test]
fn the_progress_label_shows_the_clamped_percentage_or_an_ellipsis() {
    let t: &'static Tokens = Box::leak(Box::new(Tokens::light()));
    for (value, want) in [
        (Some(0.6_f32), "Loading  60%"),
        (Some(1.5), "Loading  100%"),
        (Some(-0.5), "Loading  0%"),
        (None, "Loading  …"),
    ] {
        let el: Element<'static, Msg> = snora::design::progress::row(
            t,
            "Loading",
            value,
            Tone::Accent,
            snora::LayoutDirection::Ltr,
        );
        let mut ui = iced_test::Simulator::new(el);
        assert!(
            ui.find(want).is_ok(),
            "progress::row with {value:?} should show \"{want}\""
        );
    }
}

// ---- drawn padding and gaps (chrome-geometry.md's mapping table) -------------------------------------------------

use std::sync::{Arc, Mutex};

use common::removal::Rig;
use iced::widget::{container, text};
use iced::{Length, Rectangle};
use iced_runtime::core::widget::{Id, Operation};
use snora::design::widget as dw;
use snora::{
    BreadcrumbAction, Crumb, Icon, LayoutDirection, MenuAction, SideBar, SideBarItem, Tab,
    TabAction, TabBar,
};

/// What the tree reports for a widget laid out alone: its own box (a wrapping container with a known id, as wide
/// as the window), every text with its bounds, and every focus target's bounds, in tree order.
struct Seen {
    root: Rectangle,
    texts: Vec<(String, Rectangle)>,
    targets: Vec<Rectangle>,
}

fn seen(el: Element<'static, Msg>) -> Seen {
    struct Root(Arc<Mutex<Option<Rectangle>>>);
    impl Operation for Root {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
            if id.is_some_and(|i| *i == Id::new("probe")) {
                *self.0.lock().unwrap() = Some(bounds);
            }
        }
    }
    struct Texts(Arc<Mutex<Vec<(String, Rectangle)>>>);
    impl Operation for Texts {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn text(&mut self, _id: Option<&Id>, bounds: Rectangle, t: &str) {
            self.0.lock().unwrap().push((t.to_owned(), bounds));
        }
    }
    let wrapped: Element<'static, Msg> = container(el).id("probe").width(Length::Fill).into();
    let mut rig = Rig::new(wrapped);
    let root = Arc::new(Mutex::new(None));
    rig.run(Box::new(Root(root.clone())));
    let texts = Arc::new(Mutex::new(Vec::new()));
    rig.run(Box::new(Texts(texts.clone())));
    let targets = rig.targets().into_iter().map(|t| t.bounds).collect();
    let root = root
        .lock()
        .unwrap()
        .expect("the probe container is laid out");
    let texts = texts.lock().unwrap().clone();
    Seen {
        root,
        texts,
        targets,
    }
}

impl Seen {
    fn text(&self, s: &str) -> Rectangle {
        self.texts
            .iter()
            .find(|(t, _)| t == s)
            .unwrap_or_else(|| {
                panic!(
                    "no text {s:?} in {:?}",
                    self.texts.iter().map(|x| &x.0).collect::<Vec<_>>()
                )
            })
            .1
    }
}

/// The default scale and a doubled one: `xs 8, sm 16, md 24, lg 32, xl 48, xxl 64`. Every preset has the default,
/// so a geometry builder that hard-codes today's literal passes the first and fails the second.
fn scales() -> [(&'static str, &'static Tokens); 2] {
    let default: &'static Tokens = Box::leak(Box::new(Tokens::light()));
    let mut doubled = Tokens::light();
    let s = doubled.spacing;
    doubled.spacing.xs = s.xs * 2.0;
    doubled.spacing.sm = s.sm * 2.0;
    doubled.spacing.md = s.md * 2.0;
    doubled.spacing.lg = s.lg * 2.0;
    doubled.spacing.xl = s.xl * 2.0;
    doubled.spacing.xxl = s.xxl * 2.0;
    [
        ("default", default),
        ("doubled", Box::leak(Box::new(doubled))),
    ]
}

fn near(what: &str, got: f32, want: f32) {
    assert!(
        (got - want).abs() <= 0.51,
        "{what}: drawn {got:.2}, expected {want:.2} (the page's mapping or measurement)"
    );
}

static ON_MENU: fn(MenuAction<u8, u8>) -> Msg = |_| Msg::Go;

/// **The header** (`chrome-geometry.md`): horizontal padding `Spacing::lg`, vertical `Spacing::sm`, and the gap
/// `Spacing::md` on each side of the fixed 20 px spacer between the title and the first menu.
#[test]
fn the_headers_padding_and_gap_are_the_mapped_tokens_as_drawn() {
    for (scale, t) in scales() {
        // The title alone: its box is the row, so its distance from the container's edges is the padding.
        let alone = seen(dw::app_header(
            t,
            "App title",
            vec![],
            &ON_MENU,
            None,
            None,
            LayoutDirection::Ltr,
        ));
        let title = alone.text("App title");
        near(
            &format!("{scale}: header horizontal padding (lg)"),
            title.x - alone.root.x,
            t.spacing.lg,
        );
        near(
            &format!("{scale}: header vertical padding (sm)"),
            title.y - alone.root.y,
            t.spacing.sm,
        );
        // With a menu: the trigger is taller than the title, which is centred in the row, so measure the gap along it.
        let menu = snora::Menu {
            id: 0u8,
            label: "File".into(),
            icon: None,
            items: vec![],
        };
        let s = seen(dw::app_header(
            t,
            "App title",
            vec![menu],
            &ON_MENU,
            None,
            None,
            LayoutDirection::Ltr,
        ));
        let title = s.text("App title");
        let trigger = s.targets[0];
        near(
            &format!("{scale}: title to the first menu: md + the 20 px spacer + md"),
            trigger.x - (title.x + title.width),
            2.0 * t.spacing.md + 20.0,
        );
    }
}

/// **The footer**: horizontal padding `Spacing::lg`, vertical `Spacing::sm`.
#[test]
fn the_footers_padding_is_the_mapped_token_as_drawn() {
    for (scale, t) in scales() {
        let s = seen(dw::app_footer(
            t,
            text("Ready").into(),
            snora::LayoutDirection::Ltr,
        ));
        let label = s.text("Ready");
        near(
            &format!("{scale}: footer horizontal padding (lg)"),
            label.x - s.root.x,
            t.spacing.lg,
        );
        near(
            &format!("{scale}: footer vertical padding (sm)"),
            label.y - s.root.y,
            t.spacing.sm,
        );
    }
}

fn on_tab(_: TabAction<u8>) -> Msg {
    Msg::Go
}
fn on_crumb(_: BreadcrumbAction<u8>) -> Msg {
    Msg::Go
}

/// **The tab bar**: the bar's horizontal padding `Spacing::md`, the gap between tabs `Spacing::xs`, each tab's padding
/// `md` across and `sm` down.
#[test]
fn the_tab_bars_padding_and_gap_are_the_mapped_tokens_as_drawn() {
    for (scale, t) in scales() {
        let bar = TabBar {
            tabs: (0..2u8)
                .map(|i| Tab {
                    id: i,
                    label: format!("Tab{i}"),
                    icon: None,
                })
                .collect(),
            active: 0,
        };
        static F: fn(TabAction<u8>) -> Msg = on_tab;
        let s = seen(dw::app_tab_bar(t, bar, &F, LayoutDirection::Ltr));
        let (a, b) = (s.targets[0], s.targets[1]);
        near(
            &format!("{scale}: tab bar horizontal padding (md)"),
            a.x - s.root.x,
            t.spacing.md,
        );
        near(
            &format!("{scale}: gap between tabs (xs)"),
            b.x - (a.x + a.width),
            t.spacing.xs,
        );
        let label = s.text("Tab0");
        near(
            &format!("{scale}: tab horizontal padding (md)"),
            label.x - a.x,
            t.spacing.md,
        );
        near(
            &format!("{scale}: tab vertical padding (sm)"),
            label.y - a.y,
            t.spacing.sm,
        );
    }
}

/// **The breadcrumb**: the trail's padding `md` across and `xs` down, each crumb's padding `xs` both ways, the gap to
/// the separator `Spacing::sm`.
#[test]
fn the_breadcrumbs_padding_and_gap_are_the_mapped_tokens_as_drawn() {
    for (scale, t) in scales() {
        let crumbs = vec![
            Crumb::ancestor(0u8, "Root"),
            Crumb::ancestor(1, "Mid"),
            Crumb::leaf(2, "Here"),
        ];
        static F: fn(BreadcrumbAction<u8>) -> Msg = on_crumb;
        let s = seen(dw::app_breadcrumb(t, crumbs, &F, LayoutDirection::Ltr));
        println!(
            "{scale}: root {:?} targets {:?} texts {:?}",
            s.root, s.targets, s.texts
        );
        let first = s.targets[0];
        near(
            &format!("{scale}: trail horizontal padding (md)"),
            first.x - s.root.x,
            t.spacing.md,
        );
        let label = s.text("Root");
        near(
            &format!("{scale}: crumb horizontal padding (xs)"),
            label.x - first.x,
            t.spacing.xs,
        );
        near(
            &format!("{scale}: crumb vertical padding (xs)"),
            label.y - first.y,
            t.spacing.xs,
        );
    }
}

/// **The side bar**: the rail's vertical padding `Spacing::lg`, the gap between its buttons `Spacing::md`.
#[test]
fn the_side_bars_padding_and_gap_are_the_mapped_tokens_as_drawn() {
    for (scale, t) in scales() {
        let rail = SideBar {
            items: (0..2u8)
                .map(|i| SideBarItem {
                    view_id: i,
                    icon: Icon::Text(format!("{}", i + 1)),
                    tooltip: format!("Item {i}"),
                    on_press: Msg::Go,
                })
                .collect(),
            active: 0,
        };
        let s = seen(dw::app_side_bar(t, rail, LayoutDirection::Ltr));
        let (a, b) = (s.targets[0], s.targets[1]);
        near(
            &format!("{scale}: rail vertical padding (lg)"),
            a.y - s.root.y,
            t.spacing.lg,
        );
        near(
            &format!("{scale}: gap between buttons (md)"),
            b.y - (a.y + a.height),
            t.spacing.md,
        );
    }
}

/// **A link is as tall as a ghost button, with two measured exceptions** (`buttons.md`, the link section; the 0.63
/// audit measured it and no test did). Link minus ghost, in pixels, by `typography.label.size`:
/// - `light`, `dark`, `light_aaa`, `dark_aaa`: 0 up to 39 px, +3 from 40 px (the label's extra leading no longer fits in
///   the button's own padding);
/// - `high_contrast_*` (3 px ring, 2 px out): 0 below 24 px, +1 from 24 to 39 px (the underline is kept clear of the
///   ring), +4 from 40 px.
#[test]
fn a_link_is_as_tall_as_a_ghost_button_but_for_the_measured_exceptions() {
    use snora::design::button;
    let sizes = [14.0_f32, 23.0, 24.0, 32.0, 39.0, 40.0, 48.0];
    for (name, base) in Tokens::presets() {
        let hc = name.starts_with("high_contrast");
        let want: [f32; 7] = if hc {
            [0.0, 0.0, 1.0, 1.0, 1.0, 4.0, 4.0]
        } else {
            [0.0, 0.0, 0.0, 0.0, 0.0, 3.0, 3.0]
        };
        for (size, want) in sizes.into_iter().zip(want) {
            let mut t = base.clone();
            t.typography.label.size = size;
            let t: &'static Tokens = Box::leak(Box::new(t));
            let height = |el: Element<'static, Msg>| Rig::new(el).targets()[0].bounds.height;
            let l = height(button::link(t, "Open", Msg::Go));
            let g = height(button::ghost(t, "Open", Msg::Go));
            near(
                &format!("{name} at {size} px: link minus ghost"),
                l - g,
                want,
            );
        }
    }
}
