//! RFC-132: **the order is logical, and the positions mirror.**
//!
//! Under both directions a row's children sit in reading order in the tree, and under RTL the row
//! places them from the right edge. So a picture under RTL is the left-to-right picture reflected
//! (R-1 to R-3), Tab goes start to end under both, a change of direction keeps every widget's state,
//! and the one directional glyph snora draws flips (R-4).
//!
//! **Everything here is read from what the tree reports** (bounds from `Operation`s, the pixels that
//! change on a hover, the focus target after a Tab), never from how the widgets are built: a row that
//! reorders its children and a row that places them can be told apart only by looking.
//!
//! The mirror law, used throughout: an element at `x`, `w` wide, inside `[lo, hi]` under LTR is at
//! `lo + hi - x - w` under RTL, within a pixel.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia); layout bounds do not depend on it.
#![cfg(all(feature = "widgets", feature = "design"))]
mod common;

use std::sync::{Arc, Mutex};

use common::removal::Rig;
use iced::keyboard::Modifiers;
use iced::keyboard::key::Named;
use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length, Rectangle};
use iced_runtime::core::widget::{Id, Operation};
use snora::design::{Tokens, widget as dw};
use snora::widget::SideBarLabels;
use snora::{
    AppLayout, BreadcrumbAction, Crumb, Icon, LayoutDirection, Menu, MenuAction, MenuItem, SideBar,
    SideBarItem, Tab, TabAction, TabBar, Toast, ToastIntent, ToastPosition,
};
use snora_interaction::MirroredRow;

const W: f32 = 800.0;
const DIRS: [LayoutDirection; 2] = [LayoutDirection::Ltr, LayoutDirection::Rtl];

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Go,
    Menu(MenuAction<u8, u8>),
    Tab(TabAction<u8>),
    Crumb(BreadcrumbAction<u8>),
}
static ON_MENU: fn(MenuAction<u8, u8>) -> Msg = Msg::Menu;
static ON_TAB: fn(TabAction<u8>) -> Msg = Msg::Tab;
static ON_CRUMB: fn(BreadcrumbAction<u8>) -> Msg = Msg::Crumb;

fn tokens() -> &'static Tokens {
    Box::leak(Box::new(Tokens::light()))
}

// ---- readers ----------------------------------------------------------------------------------------------------------

fn container_bounds(rig: &mut Rig<Msg>, wanted: &'static str) -> Rectangle {
    struct Find(&'static str, Arc<Mutex<Option<Rectangle>>>);
    impl Operation for Find {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
            if id.is_some_and(|i| *i == Id::new(self.0)) {
                *self.1.lock().unwrap() = Some(bounds);
            }
        }
    }
    let out = Arc::new(Mutex::new(None));
    rig.run(Box::new(Find(wanted, out.clone())));
    let found = *out.lock().unwrap();
    found.unwrap_or_else(|| panic!("no container {wanted:?} in the tree"))
}

/// Every text in the tree, with its bounds, in tree order.
fn texts(rig: &mut Rig<Msg>) -> Vec<(String, Rectangle)> {
    struct Collect(Arc<Mutex<Vec<(String, Rectangle)>>>);
    impl Operation for Collect {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn text(&mut self, _id: Option<&Id>, bounds: Rectangle, text: &str) {
            self.0.lock().unwrap().push((text.to_owned(), bounds));
        }
    }
    let out = Arc::new(Mutex::new(Vec::new()));
    rig.run(Box::new(Collect(out.clone())));
    out.lock().unwrap().clone()
}

fn bounds_of(rig: &mut Rig<Msg>, wanted: &str) -> Rectangle {
    texts(rig)
        .into_iter()
        .find(|(s, _)| s == wanted)
        .unwrap_or_else(|| panic!("no text {wanted:?}"))
        .1
}

/// The text a focus target holds: every text whose centre is inside it.
fn label_of(target: Rectangle, tx: &[(String, Rectangle)]) -> String {
    tx.iter()
        .filter(|(_, b)| {
            let (cx, cy) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
            cx >= target.x
                && cx <= target.x + target.width
                && cy >= target.y
                && cy <= target.y + target.height
        })
        .map(|(s, _)| s.as_str())
        .collect::<Vec<_>>()
        .join("+")
}

/// Press Tab once per target and name each target focused, in the order Tab visited them.
fn tab_walk(rig: &mut Rig<Msg>) -> Vec<String> {
    let tx = texts(rig);
    let n = rig.targets().len();
    (0..n)
        .map(|_| {
            rig.tab(false);
            let focused = rig.focused();
            let targets = rig.targets();
            focused
                .first()
                .map(|i| label_of(targets[*i].bounds, &tx))
                .unwrap_or_else(|| "(nothing)".into())
        })
        .collect()
}

// ---- the law ----------------------------------------------------------------------------------------------------------

/// `ltr` and `rtl` are the same element in the two frames; both frames' span is `[lo, hi]`.
#[track_caller]
fn assert_mirrored(what: &str, ltr: Rectangle, rtl: Rectangle, lo: f32, hi: f32) {
    let want = lo + hi - ltr.x - ltr.width;
    assert!(
        (rtl.x - want).abs() <= 1.0 && (rtl.width - ltr.width).abs() <= 1.0,
        "{what}: x {:.0}..{:.0} under LTR should be x {want:.0}..{:.0} under RTL (the mirror about {lo:.0}..{hi:.0}), \
         but it is x {:.0}..{:.0}",
        ltr.x,
        ltr.x + ltr.width,
        want + ltr.width,
        rtl.x,
        rtl.x + rtl.width,
    );
}

/// Every text the two frames share, by string (a reordered tree has another order, which is not the
/// question here), mirrored about `[lo, hi]`. Repeated strings pair by occurrence.
#[track_caller]
fn assert_all_texts_mirrored(
    what: &str,
    ltr: &[(String, Rectangle)],
    rtl: &[(String, Rectangle)],
    skip: &[&str],
    lo: f32,
    hi: f32,
) {
    let mut used = vec![false; rtl.len()];
    let mut checked = 0;
    for (s, a) in ltr {
        if skip.contains(&s.as_str()) {
            continue;
        }
        let j = (0..rtl.len())
            .find(|j| !used[*j] && rtl[*j].0 == *s)
            .unwrap_or_else(|| {
                panic!("{what}: {s:?} is in the left-to-right frame and not the other")
            });
        used[j] = true;
        assert_mirrored(&format!("{what}: {s:?}"), *a, rtl[j].1, lo, hi);
        checked += 1;
    }
    assert!(checked > 0, "{what}: nothing was compared");
}

// ---- scenes -----------------------------------------------------------------------------------------------------------

fn menu(id: u8, label: &str, items: u8) -> Menu<u8, u8> {
    Menu {
        id,
        label: label.into(),
        icon: None,
        items: (0..items)
            .map(|i| MenuItem {
                menu_id: id,
                id: i,
                label: format!("{label}{i}"),
                icon: None,
            })
            .collect(),
    }
}

fn flip_button() -> Element<'static, Msg> {
    snora::control(text("Flip")).on_press(Msg::Go).into()
}

fn header(
    dir: LayoutDirection,
    design: bool,
    menus: usize,
    end: bool,
    open: Option<u8>,
) -> Element<'static, Msg> {
    let ms: Vec<_> = (0..menus as u8)
        .map(|i| menu(i, &format!("Menu{i}"), 2))
        .collect();
    let end = end.then(flip_button);
    if design {
        dw::app_header(tokens(), "App title", ms, &ON_MENU, open.as_ref(), end, dir)
    } else {
        snora::widget::app_header("App title", ms, &ON_MENU, open.as_ref(), end, dir)
    }
}

fn page() -> Element<'static, Msg> {
    column![
        snora::control(text("Body A")).on_press(Msg::Go),
        snora::control(text("Body B")).on_press(Msg::Go)
    ]
    .into()
}

fn rail(n: u8) -> SideBar<Msg, u8> {
    SideBar {
        items: (0..n)
            .map(|i| SideBarItem {
                view_id: i,
                icon: Icon::Text(format!("{}", i + 1)),
                tooltip: format!("Item{i}"),
                on_press: Msg::Go,
            })
            .collect(),
        active: 0,
    }
}

/// The whole skeleton: a header with a menu and an end control, a rail, a body of two controls, and a footer control.
fn skeleton(dir: LayoutDirection, labels: SideBarLabels) -> Element<'static, Msg> {
    let l = AppLayout::new(page())
        .header(header(dir, false, 1, true, None))
        .side_bar(snora::widget::app_side_bar_with(rail(2), dir, labels))
        .footer(snora::widget::app_footer(
            snora::control(text("Foot")).on_press(Msg::Go).into(),
        ))
        .direction(dir);
    snora::render(l)
}

fn tab_bar(dir: LayoutDirection, design: bool, icon: bool) -> Element<'static, Msg> {
    let bar = TabBar {
        tabs: (0..4u8)
            .map(|i| Tab {
                id: i,
                label: format!("Tab{i}"),
                icon: (icon && i == 1).then(|| Icon::Text("★".into())),
            })
            .collect(),
        active: 1,
    };
    if design {
        dw::app_tab_bar(tokens(), bar, &ON_TAB, dir)
    } else {
        snora::widget::app_tab_bar(bar, &ON_TAB, dir)
    }
}

fn crumbs(dir: LayoutDirection) -> Element<'static, Msg> {
    let trail = vec![
        Crumb::ancestor(0u8, "Root"),
        Crumb::ancestor(1, "Mid"),
        Crumb::leaf(2, "Here"),
    ];
    snora::widget::app_breadcrumb(trail, &ON_CRUMB, dir)
}

fn scene_rig(make: impl Fn(LayoutDirection) -> Element<'static, Msg>) -> [Rig<Msg>; 2] {
    [Rig::new(make(DIRS[0])), Rig::new(make(DIRS[1]))]
}

// ---- R-1: the row ------------------------------------------------------------------------------------------------------

/// **R-1: a mirrored row places its direct children from the right, and keeps their order in the tree.**
/// Spacing, padding and a `Fill` child are the cases a reflection could get wrong.
#[test]
fn a_mirrored_row_places_its_children_from_the_right_and_keeps_their_order() {
    let build = |mirrored: bool| -> Element<'static, Msg> {
        let r = row![
            container(text("first"))
                .width(Length::Fixed(60.0))
                .id("first"),
            container(text("second")).width(Length::Fill).id("second"),
            container(text("third"))
                .width(Length::Fixed(90.0))
                .id("third"),
        ]
        .spacing(7)
        .padding([0, 11]);
        container(MirroredRow::new(r, mirrored))
            .width(Length::Fill)
            .into()
    };
    let mut ltr = Rig::new(build(false));
    let mut rtl = Rig::new(build(true));
    // The tree order is the same: reflection moves positions only.
    let order = |rig: &mut Rig<Msg>| texts(rig).into_iter().map(|x| x.0).collect::<Vec<_>>();
    assert_eq!(
        order(&mut ltr),
        order(&mut rtl),
        "the children stay in logical order"
    );
    // The cells are reflected (a cell's own text is not: it is the cell's content).
    for id in ["first", "second", "third"] {
        let (a, b) = (
            container_bounds(&mut ltr, id),
            container_bounds(&mut rtl, id),
        );
        assert_mirrored(&format!("the {id} cell"), a, b, 0.0, W);
    }
    // And it is a reflection, not a no-op: the first cell is at the left under one, the right under the other.
    let (a, b) = (
        container_bounds(&mut ltr, "first"),
        container_bounds(&mut rtl, "first"),
    );
    assert!(a.x < 100.0 && b.x > 600.0, "first cell: {a:?} / {b:?}");
}

/// **A row that is not mirrored is the row it wraps**, to the pixel: this is what keeps every left-to-right
/// frame as it was. (The before-and-after table for the widgets themselves is in the request's `evidence/`.)
#[test]
fn an_unmirrored_row_is_the_row_it_wraps_to_the_pixel() {
    let plain = || {
        row![
            container(text("a")).width(Length::Fixed(60.0)),
            container(text("b")).width(Length::Fill),
            snora::control(text("c")).on_press(Msg::Go),
        ]
        .spacing(5)
        .padding(9)
    };
    let mut wrapped: Rig<Msg> = Rig::new(MirroredRow::new(plain(), false).into());
    let mut bare: Rig<Msg> = Rig::new(plain().into());
    assert_eq!(wrapped.frame(), bare.frame(), "the pixels");
    let (a, b) = (texts(&mut wrapped), texts(&mut bare));
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert_eq!((&x.0, x.1), (&y.0, y.1), "the bounds");
    }
    // `row_dir` under LTR is a plain row too.
    let two = |wrapped: bool| -> Element<'static, Msg> {
        if wrapped {
            snora::direction::row_dir(
                LayoutDirection::Ltr,
                text("start"),
                container(text("end")).width(Length::Fill),
            )
            .spacing(3)
            .into()
        } else {
            row![text("start"), container(text("end")).width(Length::Fill)]
                .spacing(3)
                .into()
        }
    };
    assert_eq!(
        Rig::new(two(true)).frame(),
        Rig::new(two(false)).frame(),
        "row_dir under LTR"
    );
}

/// `row_dir` and `row_dir_three`: the order in the tree is `start`, `end` (and the centre between) under both
/// directions, and the positions mirror.
#[test]
fn row_dir_and_row_dir_three_keep_logical_order_and_mirror_the_positions() {
    use snora::direction::{row_dir, row_dir_three};
    let two = |dir| -> Element<'static, Msg> {
        container(row_dir(dir, text("start"), text("end")).spacing(10))
            .width(Length::Fill)
            .into()
    };
    let three = |dir| -> Element<'static, Msg> {
        container(row_dir_three(dir, text("start"), text("middle"), text("end")).spacing(10))
            .width(Length::Fill)
            .into()
    };
    for (what, make) in [
        ("row_dir", &two as &dyn Fn(_) -> _),
        ("row_dir_three", &three),
    ] {
        let [mut ltr, mut rtl] = scene_rig(make);
        let (l, r) = (texts(&mut ltr), texts(&mut rtl));
        assert_eq!(
            l.iter().map(|x| &x.0).collect::<Vec<_>>(),
            r.iter().map(|x| &x.0).collect::<Vec<_>>(),
            "{what}: the tree order"
        );
        // A `Shrink` row is reflected about its own width, then sits where its container puts it. The container here is
        // `Fill`, and the row shrinks: both frames put it at the left, so compare the order of positions instead.
        let xs = |t: &[(String, Rectangle)]| t.iter().map(|x| x.1.x).collect::<Vec<_>>();
        assert!(
            xs(&l).windows(2).all(|p| p[0] < p[1]),
            "{what}: LTR left to right {:?}",
            xs(&l)
        );
        assert!(
            xs(&r).windows(2).all(|p| p[0] > p[1]),
            "{what}: RTL right to left {:?}",
            xs(&r)
        );
    }
}

// ---- R-1, R-3: each place snora mirrors ---------------------------------------------------------------------------------

/// **The skeleton**: the sidebar and the body swap sides, and the rail's own contents mirror with it.
#[test]
fn the_skeleton_is_a_mirror_image() {
    let [mut ltr, mut rtl] = scene_rig(|d| skeleton(d, SideBarLabels::Beneath));
    for id in [
        "snora-sidebar",
        "snora-body",
        "snora-header",
        "snora-footer",
    ] {
        let (a, b) = (
            container_bounds(&mut ltr, id),
            container_bounds(&mut rtl, id),
        );
        assert_mirrored(id, a, b, 0.0, W);
    }
    // The header and the rail are whole pictures, mirrored. (The body's own content and the footer's are the application's.)
    let (l, r) = (texts(&mut ltr), texts(&mut rtl));
    let keep = ["App title", "Menu0", "Flip", "Item0", "Item1", "1", "2"];
    let only = |t: &[(String, Rectangle)]| {
        t.iter()
            .filter(|x| keep.contains(&x.0.as_str()))
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_all_texts_mirrored(
        "the skeleton's header and rail",
        &only(&l),
        &only(&r),
        &[],
        0.0,
        W,
    );
}

/// **The header, in every variant** (R-2): the title at the start edge, the menus after it in logical order, the end
/// controls at the end edge. Design and unstyled; no, one and two menus; with and without an end control.
#[test]
fn the_header_is_a_mirror_image_in_every_variant() {
    for design in [true, false] {
        for menus in 0..=2usize {
            for end in [false, true] {
                let what = format!(
                    "{} header, {menus} menu(s), end control {end}",
                    if design { "design" } else { "unstyled" }
                );
                let [mut ltr, mut rtl] = scene_rig(|d| header(d, design, menus, end, None));
                let (l, r) = (texts(&mut ltr), texts(&mut rtl));
                assert_all_texts_mirrored(&what, &l, &r, &[], 0.0, W);
                // Each focus target too (a trigger is bigger than its label).
                let targets = |rig: &mut Rig<Msg>, tx: &[(String, Rectangle)]| {
                    rig.targets()
                        .into_iter()
                        .map(|t| (label_of(t.bounds, tx), t.bounds))
                        .collect::<Vec<_>>()
                };
                let (tl, tr) = (targets(&mut ltr, &l), targets(&mut rtl, &r));
                assert_eq!(tl.len(), menus + usize::from(end), "{what}: premise");
                if !tl.is_empty() {
                    assert_all_texts_mirrored(&format!("{what} (targets)"), &tl, &tr, &[], 0.0, W);
                }
            }
        }
    }
}

/// **The sidebar's tooltip opens on the end side of the rail**: right of the item under LTR, left under RTL. An overlay is
/// not reported by `operate`, so this reads the pixels that change when the item is hovered.
#[test]
fn the_sidebars_tooltip_opens_on_the_end_side() {
    for dir in DIRS {
        let l = AppLayout::new(page())
            .side_bar(snora::widget::app_side_bar_with(
                rail(2),
                dir,
                SideBarLabels::Tooltip,
            ))
            .direction(dir);
        let mut rig = Rig::new(snora::render(l));
        let tx = texts(&mut rig);
        let item = rig
            .targets()
            .into_iter()
            .map(|t| t.bounds)
            .find(|b| label_of(*b, &tx).contains('1'))
            .expect("the first rail item");
        rig.redraw();
        let clean = rig.frame();
        rig.point_at(Some(iced::Point::new(
            item.x + item.width / 2.0,
            item.y + item.height / 2.0,
        )));
        rig.redraw();
        let hot = rig.frame();
        let (mut x0, mut x1) = (i32::MAX, -1i32);
        for y in 0..600usize {
            for x in 0..800usize {
                let k = (y * 800 + x) * 4;
                if clean[k..k + 4] != hot[k..k + 4] {
                    x0 = x0.min(x as i32);
                    x1 = x1.max(x as i32);
                }
            }
        }
        assert!(x1 >= 0, "{dir:?}: hovering the item changed nothing");
        // **On the end side, clear of the item.** Hovering changes nothing but the tooltip, so the changed pixels are the tooltip.
        // iced snaps a tooltip into the window instead of flipping it, so one asked for on the wrong side at the window's edge
        // still lands partly over the item: "it reaches past the item on the end side" is not enough, it must not touch it.
        let (left, right) = (item.x, item.x + item.width);
        match dir {
            LayoutDirection::Ltr => assert!(
                x0 as f32 >= right,
                "LTR: the tooltip should be drawn clear of the item ({left:.0}..{right:.0}), on its right; changed pixels span x {x0}..{x1}"
            ),
            LayoutDirection::Rtl => assert!(
                x1 as f32 <= left,
                "RTL: the tooltip should be drawn clear of the item ({left:.0}..{right:.0}), on its left; changed pixels span x {x0}..{x1}"
            ),
        }
    }
}

/// **The rail's labelled forms**: icon and name, mirrored about the rail's own bounds.
#[test]
fn the_labelled_rail_mirrors_inside_itself() {
    for labels in [SideBarLabels::Beneath, SideBarLabels::Beside] {
        let shrink = |d| -> Element<'static, Msg> {
            container(snora::widget::app_side_bar_with(rail(2), d, labels))
                .width(Length::Shrink)
                .id("probe")
                .into()
        };
        let [mut ltr, mut rtl] = scene_rig(shrink);
        let (bl, br) = (
            container_bounds(&mut ltr, "probe"),
            container_bounds(&mut rtl, "probe"),
        );
        assert_eq!(
            (bl.x, bl.width),
            (br.x, br.width),
            "{labels:?}: the rail is the same box"
        );
        let (l, r) = (texts(&mut ltr), texts(&mut rtl));
        assert_all_texts_mirrored(
            &format!("{labels:?} rail"),
            &l,
            &r,
            &[],
            bl.x,
            bl.x + bl.width,
        );
        if labels == SideBarLabels::Beside {
            // The name is on the end side of its icon under both directions' reading: right under LTR, left under RTL.
            let (i, n) = (bounds_of(&mut ltr, "1"), bounds_of(&mut ltr, "Item0"));
            assert!(i.x < n.x, "LTR: the icon is before the name");
            let (i, n) = (bounds_of(&mut rtl, "1"), bounds_of(&mut rtl, "Item0"));
            assert!(
                i.x > n.x,
                "RTL: the icon is before the name, which is to its right"
            );
        }
    }
}

/// **The tab bar** (R-1): the first declared tab at the start edge, and **inside a tab** the icon at the start edge of its label.
#[test]
fn the_tab_bar_is_a_mirror_image_and_so_is_each_tab() {
    for design in [false, true] {
        let [mut ltr, mut rtl] = scene_rig(|d| tab_bar(d, design, true));
        let (l, r) = (texts(&mut ltr), texts(&mut rtl));
        assert_all_texts_mirrored(
            &format!("{} tab bar", if design { "design" } else { "unstyled" }),
            &l,
            &r,
            &[],
            0.0,
            W,
        );
        // Inside the tab that has an icon: the icon is before the label in reading order, so on the right under RTL.
        for (dir, rig) in [(DIRS[0], &mut ltr), (DIRS[1], &mut rtl)] {
            let (icon, label) = (bounds_of(rig, "★"), bounds_of(rig, "Tab1"));
            match dir {
                LayoutDirection::Ltr => assert!(icon.x < label.x, "LTR: icon left of the label"),
                LayoutDirection::Rtl => assert!(icon.x > label.x, "RTL: icon right of the label"),
            }
        }
    }
}

/// **The breadcrumb** (R-1): the trail starts at the start edge (it used to be reversed and still start at the left), and
/// **R-4: the separator is a directional glyph and flips**, `›` under LTR and `‹` under RTL, at every join.
#[test]
fn the_breadcrumb_is_a_mirror_image_and_its_separator_flips() {
    let [mut ltr, mut rtl] = scene_rig(crumbs);
    let (l, r) = (texts(&mut ltr), texts(&mut rtl));
    let seps = |t: &[(String, Rectangle)], glyph: &str| t.iter().filter(|x| x.0 == glyph).count();
    assert_eq!((seps(&l, "›"), seps(&l, "‹")), (2, 0), "LTR separators");
    assert_eq!((seps(&r, "›"), seps(&r, "‹")), (0, 2), "RTL separators");
    // Positions: the glyphs differ between the frames, so compare them as "a separator" and the crumbs by name.
    let norm = |t: &[(String, Rectangle)]| {
        t.iter()
            .map(|x| {
                (
                    if x.0 == "‹" {
                        "›".to_owned()
                    } else {
                        x.0.clone()
                    },
                    x.1,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_all_texts_mirrored("breadcrumb", &norm(&l), &norm(&r), &[], 0.0, W);
    let (root, here) = (bounds_of(&mut rtl, "Root"), bounds_of(&mut rtl, "Here"));
    assert!(
        root.x > here.x && root.x + root.width > W - 40.0,
        "RTL: Root is at the right edge, got {root:?}"
    );
}

/// **Toasts** at all six positions: the box (done in 0.62) **and the inside** (R-1): the close mark is at the end edge of
/// its toast, which is the left under RTL. **R-4's other half: the close mark is not directional, and does not change.**
#[test]
fn a_toast_mirrors_at_every_position_and_its_close_mark_does_not_change() {
    for pos in [
        ToastPosition::TopStart,
        ToastPosition::TopCenter,
        ToastPosition::TopEnd,
        ToastPosition::BottomStart,
        ToastPosition::BottomCenter,
        ToastPosition::BottomEnd,
    ] {
        let mut seen = Vec::new();
        for dir in DIRS {
            let l = AppLayout::new(text("page").into())
                .toasts(vec![Toast::new(
                    7,
                    ToastIntent::Info,
                    "TTitle",
                    "message",
                    Msg::Go,
                )])
                .toast_position(pos)
                .direction(dir);
            let mut rig = Rig::new(snora::render(l));
            let b = container_bounds(&mut rig, "snora-toast-7");
            let tx = texts(&mut rig);
            let close = rig.targets().last().expect("the close control").bounds;
            let glyph = label_of(close, &tx);
            seen.push((b, bounds_of(&mut rig, "TTitle"), close, glyph));
        }
        let [(bl, tl, cl, gl), (br, tr, cr, gr)] = [seen[0].clone(), seen[1].clone()];
        assert_mirrored(&format!("{pos:?}: the toast"), bl, br, 0.0, W);
        // Inside its own box: the box moves, and the close mark is reflected within it.
        let local = |r: Rectangle, of: Rectangle| Rectangle { x: r.x - of.x, ..r };
        assert_mirrored(
            &format!("{pos:?}: its close control, inside the toast"),
            local(cl, bl),
            local(cr, br),
            0.0,
            bl.width,
        );
        // The title is the near side of the close control: before it under LTR, after it under RTL.
        assert!(
            tl.x + tl.width <= cl.x,
            "{pos:?} LTR: the title is left of the close control"
        );
        assert!(
            tr.x >= cr.x + cr.width,
            "{pos:?} RTL: the title is right of the close control"
        );
        assert!(
            !gl.is_empty() && gl == gr,
            "{pos:?}: the close mark is {gl:?} under LTR and {gr:?} under RTL"
        );
    }
}

/// **A menu's trigger and its items**: the icon is at the start edge of the label.
#[test]
fn a_menu_triggers_and_items_icon_sits_at_the_start_edge_of_its_label() {
    let mk = |dir: LayoutDirection| -> Element<'static, Msg> {
        let mut m = menu(0, "File", 2);
        m.icon = Some(Icon::Text("#".into()));
        m.items[0].icon = Some(Icon::Text("@".into()));
        let h = snora::widget::app_header("App title", vec![m], &ON_MENU, Some(&0u8), None, dir);
        snora::render(AppLayout::new(page()).header(h).direction(dir))
    };
    let [mut ltr, mut rtl] = scene_rig(mk);
    ltr.redraw();
    rtl.redraw();
    for (dir, rig) in [(DIRS[0], &mut ltr), (DIRS[1], &mut rtl)] {
        for (icon, label) in [("#", "File"), ("@", "File0")] {
            let (i, l) = (bounds_of(rig, icon), bounds_of(rig, label));
            match dir {
                LayoutDirection::Ltr => assert!(i.x < l.x, "LTR: {icon} is left of {label}"),
                LayoutDirection::Rtl => assert!(i.x > l.x, "RTL: {icon} is right of {label}"),
            }
        }
    }
}

// ---- Tab order -------------------------------------------------------------------------------------------------------

/// **Tab goes start to end under both directions** (Q-1): the same sequence of controls, which is right to left on screen under
/// RTL. Measured by pressing Tab and reading which control holds focus.
/// A name, a builder for either direction, and the controls in the order Tab should visit them.
type Scene = (
    &'static str,
    Box<dyn Fn(LayoutDirection) -> Element<'static, Msg>>,
    Vec<&'static str>,
);

#[test]
fn tab_visits_the_same_controls_in_the_same_order_under_both_directions() {
    let scenes: Vec<Scene> = vec![
        (
            "skeleton",
            Box::new(|d| skeleton(d, SideBarLabels::Beneath)),
            vec![
                "Menu0", "Flip", "1+Item0", "2+Item1", "Body A", "Body B", "Foot",
            ],
        ),
        (
            "header",
            Box::new(|d| header(d, false, 2, true, None)),
            vec!["Menu0", "Menu1", "Flip"],
        ),
        (
            "tab bar",
            Box::new(|d| tab_bar(d, false, true)),
            vec!["Tab0", "★+Tab1", "Tab2", "Tab3"],
        ),
        ("breadcrumb", Box::new(crumbs), vec!["Root", "Mid"]),
        (
            "labelled rail",
            Box::new(|d| snora::widget::app_side_bar_with(rail(3), d, SideBarLabels::Beside)),
            vec!["1+Item0", "2+Item1", "3+Item2"],
        ),
    ];
    for (what, make, want) in scenes {
        for dir in DIRS {
            let mut rig = Rig::new(make(dir));
            let walk = tab_walk(&mut rig);
            // The labelled rail names its content in tree order, icon first under LTR and name first under RTL if reordered.
            assert_eq!(walk, want, "{what}, {dir:?}: Tab order");
        }
    }
}

// ---- the flip --------------------------------------------------------------------------------------------------------

fn flip_scene(dir: LayoutDirection, open: Option<u8>) -> Element<'static, Msg> {
    let body: Element<'static, Msg> =
        scrollable(column((0..80).map(|i| text(format!("row {i}")).into())))
            .id("scroll")
            .height(Length::Fill)
            .into();
    let l = AppLayout::new(body)
        .header(header(dir, false, 1, true, open))
        .side_bar(snora::widget::app_side_bar_with(
            rail(2),
            dir,
            SideBarLabels::Beneath,
        ))
        .direction(dir);
    snora::render(l)
}

fn scroll_offsets(rig: &mut Rig<Msg>) -> Vec<f32> {
    struct Offs(Arc<Mutex<Vec<f32>>>);
    impl Operation for Offs {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn scrollable(
            &mut self,
            _id: Option<&Id>,
            _b: Rectangle,
            _c: Rectangle,
            translation: iced::Vector,
            _s: &mut dyn iced_runtime::core::widget::operation::Scrollable,
        ) {
            self.0.lock().unwrap().push(translation.y);
        }
    }
    let out = Arc::new(Mutex::new(Vec::new()));
    rig.run(Box::new(Offs(out.clone())));
    out.lock().unwrap().clone()
}

/// **A live flip keeps focus** (R-1): whichever control holds it (the header's menu and end control, the rail's two
/// items), in both directions of change. The view is rebuilt around the kept widget state, as `view` does after `update`.
#[test]
fn a_flip_keeps_focus_on_the_same_control() {
    for (from, to) in [(DIRS[0], DIRS[1]), (DIRS[1], DIRS[0])] {
        let n = Rig::new(flip_scene(from, None)).targets().len();
        assert_eq!(n, 4, "premise: four controls");
        for at in 0..n {
            let mut rig = Rig::new(flip_scene(from, None));
            rig.focus_at(at);
            let tx = texts(&mut rig);
            let name = |rig: &mut Rig<Msg>, tx: &[(String, Rectangle)]| {
                let focused = rig.focused();
                let targets = rig.targets();
                focused.first().map(|i| label_of(targets[*i].bounds, tx))
            };
            let before = name(&mut rig, &tx);
            assert!(
                before.is_some(),
                "{from:?}: premise, target {at} is focused"
            );
            let mut rig = rig.rebuilt(flip_scene(to, None));
            let tx = texts(&mut rig);
            assert_eq!(
                name(&mut rig, &tx),
                before,
                "{from:?} -> {to:?}: focus on target {at} survives the flip"
            );
        }
    }
}

/// **A live flip keeps a scroll offset**, and an open menu stays open and follows its trigger to the other side.
#[test]
fn a_flip_keeps_the_scroll_offset_and_the_open_menu() {
    for (from, to) in [(DIRS[0], DIRS[1]), (DIRS[1], DIRS[0])] {
        let mut rig = Rig::new(flip_scene(from, None));
        rig.run(Box::new(
            iced_runtime::core::widget::operation::scrollable::scroll_to::<()>(
                Id::new("scroll"),
                iced_runtime::core::widget::operation::scrollable::AbsoluteOffset {
                    x: Some(0.0),
                    y: Some(200.0),
                },
            ),
        ));
        rig.redraw();
        assert_eq!(scroll_offsets(&mut rig), vec![200.0], "premise: scrolled");
        let mut rig = rig.rebuilt(flip_scene(to, None));
        assert_eq!(
            scroll_offsets(&mut rig),
            vec![200.0],
            "{from:?} -> {to:?}: the offset"
        );

        // The menu: the application holds its id, so it stays open; its dropdown follows the trigger.
        let mut rig = Rig::new(flip_scene(from, Some(0)));
        let item = |rig: &mut Rig<Msg>| bounds_of(rig, "Menu00");
        let trigger = |rig: &mut Rig<Msg>| bounds_of(rig, "Menu0");
        let (i0, t0) = (item(&mut rig), trigger(&mut rig));
        let mut rig = rig.rebuilt(flip_scene(to, Some(0)));
        let (i1, t1) = (item(&mut rig), trigger(&mut rig));
        assert!(
            (i0.x - t0.x).abs() < 40.0 && (i1.x - t1.x).abs() < 40.0,
            "the dropdown is at its trigger"
        );
        assert!(
            (t1.x - t0.x).abs() > 100.0,
            "premise: the trigger moved across ({t0:?} -> {t1:?})"
        );
    }
}

// ---- R-5 -------------------------------------------------------------------------------------------------------------

/// **R-5: snora answers no ← or →** (today), so the rule for when it does ("they follow the screen: → moves right under both
/// directions"; Tab, F6 and the tab shortcuts stay logical) has nothing to bind yet. This fails the day something answers
/// one, and its message says where the rule is written: `snora_interaction::Router`'s rules.
#[test]
fn snora_answers_no_horizontal_arrow() {
    let scenes: Vec<(&str, Element<'static, Msg>)> = vec![
        ("tab bar", tab_bar(DIRS[1], false, true)),
        ("skeleton", skeleton(DIRS[1], SideBarLabels::Beside)),
        ("open menu", flip_scene(DIRS[1], Some(0))),
        ("breadcrumb", crumbs(DIRS[1])),
    ];
    for (what, el) in scenes {
        let mut rig = Rig::new(el);
        let n = rig.targets().len();
        for at in 0..n {
            rig.focus_at(at);
            for key in [Named::ArrowLeft, Named::ArrowRight] {
                for mods in [
                    Modifiers::empty(),
                    Modifiers::SHIFT,
                    Modifiers::CTRL,
                    Modifiers::ALT,
                ] {
                    let status = rig.press(iced::keyboard::Key::Named(key), mods);
                    assert_eq!(
                        status,
                        iced_runtime::core::event::Status::Ignored,
                        "{what}, target {at}: snora answered {key:?} with {mods:?}. RFC-132 R-5: horizontal arrows follow the \
                         SCREEN under both directions (→ moves right); Tab, F6 and the tab shortcuts stay logical. \
                         Read the rule in `snora_interaction::Router`'s documentation before answering one."
                    );
                }
            }
        }
        assert!(
            rig.messages.is_empty(),
            "{what}: an arrow published {:?}",
            rig.messages
        );
    }
}
