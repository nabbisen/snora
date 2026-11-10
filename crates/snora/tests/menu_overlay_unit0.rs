//! RFC-124 Unit 0: the header's dropdown as an overlay anchored to its trigger, on a
//! PROTOTYPE. Prints the table (`--nocapture`) and asserts each row.
//!
//! Renderer follows `ICED_TEST_BACKEND` (default tiny-skia); wgpu needs `--test-threads=1`.
#![cfg(feature = "widgets")]
mod common;
use std::sync::{Arc, Mutex};

use common::removal::Rig;
use iced::widget::{column, text};
use iced::{Element, Rectangle};
use iced_runtime::core::widget::{Id, Operation};
use snora::{AppLayout, LayoutDirection, Menu, MenuAction, MenuItem};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Menu(MenuAction<u8, u8>),
    Close,
    Page,
}
static ON_MENU: fn(MenuAction<u8, u8>) -> Msg = Msg::Menu;

fn menu(items: u8) -> Menu<u8, u8> {
    Menu {
        id: 0,
        label: "File".into(),
        icon: None,
        items: (0..items)
            .map(|i| MenuItem {
                menu_id: 0,
                id: i,
                label: format!("Item {i}"),
                icon: None,
            })
            .collect(),
    }
}

fn page() -> Element<'static, Msg> {
    column![snora::control(text("page")).on_press(Msg::Page)].into()
}

/// How `menus.md` wires a header menu: `app_header`, `on_close_menus`, and an empty
/// `Space` in `header_menu` while a menu is open.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Wiring {
    /// `menus.md`'s: the backdrop sink and the empty `Space`.
    Documented,
    /// The sink only: the empty `Space` is left out.
    SinkOnly,
    /// Neither: no backdrop at all.
    Bare,
}

fn header_scene(
    open: bool,
    wiring: Wiring,
    dir: LayoutDirection,
    items: u8,
) -> Element<'static, Msg> {
    let active = open.then_some(&0u8);
    let header = snora::widget::app_header("App", vec![menu(items)], &ON_MENU, active, None, dir);
    let mut l = AppLayout::new(page()).header(header).direction(dir);
    if wiring != Wiring::Bare {
        l = l.on_close_menus(Msg::Close);
    }
    if wiring == Wiring::Documented && open {
        l = l.header_menu(iced::widget::space().into());
    }
    snora::render(l)
}

/// The bounds of the container with this id, as `operate` reports them.
fn bounds_of(rig: &mut Rig<Msg>, wanted: &'static str) -> Option<Rectangle> {
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
    *out.lock().unwrap()
}

#[test]
fn problem_3_the_headers_height_closed_and_open() {
    println!("\n## the header region and the page's first control, menu closed and open\n");
    println!(
        "| wiring | header height closed | header height open | body top closed | body top open |"
    );
    println!("|---|---|---|---|---|");
    for wiring in [Wiring::Bare, Wiring::SinkOnly, Wiring::Documented] {
        let mut closed = Rig::new(header_scene(false, wiring, LayoutDirection::Ltr, 3));
        let mut open = Rig::new(header_scene(true, wiring, LayoutDirection::Ltr, 3));
        let hc = bounds_of(&mut closed, "snora-header")
            .expect("header region")
            .height;
        let ho = bounds_of(&mut open, "snora-header")
            .expect("header region")
            .height;
        // Where the page begins: the body region's top edge.
        let yc = bounds_of(&mut closed, "snora-body").expect("body region").y;
        let yo = bounds_of(&mut open, "snora-body").expect("body region").y;
        println!("| {wiring:?} | {hc} | {ho} | {yc} | {yo} |");
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const LTR: LayoutDirection = LayoutDirection::Ltr;
const RTL: LayoutDirection = LayoutDirection::Rtl;

fn item_pressed(id: u8) -> Msg {
    Msg::Menu(MenuAction::MenuItemPressed {
        menu_id: 0,
        menu_item_id: id,
    })
}

/// The trigger's centre: read from the **closed** scene, because opening moves nothing.
fn trigger_center(dir: LayoutDirection) -> iced::Point {
    let mut closed = Rig::new(header_scene(false, Wiring::SinkOnly, dir, 3));
    closed.targets()[0].bounds.center()
}

/// A point on the page, clear of the header and of the dropdown.
const PAGE_POINT: iced::Point = iced::Point::new(500.0, 400.0);
/// On the page's own control (the top-left of the body).
const PAGE_CONTROL: iced::Point = iced::Point::new(10.0, 60.0);

// ---------------------------------------------------------------------------
// Problem 3, and R-1
// ---------------------------------------------------------------------------

#[test]
fn opening_a_menu_moves_nothing() {
    for wiring in [Wiring::Bare, Wiring::SinkOnly, Wiring::Documented] {
        let mut closed = Rig::new(header_scene(false, wiring, LTR, 3));
        let mut open = Rig::new(header_scene(true, wiring, LTR, 3));
        for id in ["snora-header", "snora-body"] {
            assert_eq!(
                bounds_of(&mut closed, id),
                bounds_of(&mut open, id),
                "[{wiring:?}] {id} moved when the menu opened"
            );
        }
    }
}

/// R-1: below the trigger, at its start edge; mirrored under RTL.
#[test]
fn the_dropdown_is_below_its_trigger_at_the_start_edge() {
    println!("\n## the dropdown against its trigger (item 0's bounds, trigger's bounds)\n");
    println!("| direction | trigger | item 0 | edge that lines up |");
    println!("|---|---|---|---|");
    for dir in [LTR, RTL] {
        let mut closed = Rig::new(header_scene(false, Wiring::SinkOnly, dir, 3));
        let mut open = Rig::new(header_scene(true, Wiring::SinkOnly, dir, 3));
        let trigger = closed.targets()[0].bounds;
        let item = open.targets()[0].bounds;
        let (edge, aligned) = if dir == LTR {
            ("left", (item.x - trigger.x).abs() < 0.5)
        } else {
            (
                "right",
                ((item.x + item.width) - (trigger.x + trigger.width)).abs() < 0.5,
            )
        };
        println!(
            "| {dir:?} | x {:.1} w {:.1} y {:.1} h {:.1} | x {:.1} w {:.1} y {:.1} | {edge}: {aligned} |",
            trigger.x, trigger.width, trigger.y, trigger.height, item.x, item.width, item.y
        );
        assert!(
            aligned,
            "[{dir:?}] the dropdown's {edge} edge is not the trigger's"
        );
        assert!(
            (item.y - (trigger.y + trigger.height)).abs() < 8.0,
            "[{dir:?}] the dropdown does not begin just below the trigger"
        );
    }
}

/// R-1: kept inside the window. A label wider than the room beside the trigger.
#[test]
fn the_dropdown_stays_inside_the_window() {
    for dir in [LTR, RTL] {
        let wide = Menu {
            id: 0u8,
            label: "File".into(),
            icon: None,
            items: vec![MenuItem {
                menu_id: 0u8,
                id: 0u8,
                label: "W".repeat(70),
                icon: None,
            }],
        };
        let header = snora::widget::app_header("App", vec![wide], &ON_MENU, Some(&0u8), None, dir);
        let mut rig = Rig::new(snora::render(
            AppLayout::new(page())
                .header(header)
                .direction(dir)
                .on_close_menus(Msg::Close),
        ));
        let item = rig.targets()[0].bounds;
        println!(
            "[{dir:?}] 70-character item: x {:.1} .. {:.1} (window 0 .. 800)",
            item.x,
            item.x + item.width
        );
        assert!(
            item.x >= 0.0 && item.x + item.width <= 800.5,
            "[{dir:?}] the dropdown is outside the window: {item:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// R-2, end to end, under `snora::render` and the documented wiring
// ---------------------------------------------------------------------------

/// Every wiring that has a sink (`menus.md`'s with the empty `Space`, and without it).
const COVERED: [Wiring; 2] = [Wiring::Documented, Wiring::SinkOnly];

#[test]
fn a_click_on_an_item_presses_it() {
    for wiring in COVERED {
        let mut rig = Rig::new(header_scene(true, wiring, LTR, 3));
        let at = rig.targets()[1].bounds.center();
        rig.click_at(at);
        assert_eq!(
            rig.messages,
            vec![item_pressed(1)],
            "[{wiring:?}] a click on an item must press it, and not close the menu"
        );
    }
}

#[test]
fn a_key_on_an_item_presses_it() {
    for wiring in COVERED {
        let mut rig = Rig::new(header_scene(true, wiring, LTR, 3));
        assert_eq!(
            rig.targets().len(),
            3,
            "[{wiring:?}] the items, and only the items, are focus targets"
        );
        assert!(rig.focused().is_empty(), "opening moved focus");
        rig.tab(false);
        assert_eq!(
            rig.focused(),
            vec![0],
            "[{wiring:?}] Tab did not reach item 0"
        );
        rig.enter();
        assert_eq!(rig.messages, vec![item_pressed(0)], "[{wiring:?}]");
    }
}

#[test]
fn a_click_outside_closes_and_presses_nothing() {
    for wiring in COVERED {
        for at in [PAGE_POINT, PAGE_CONTROL, trigger_center(LTR)] {
            let mut rig = Rig::new(header_scene(true, wiring, LTR, 3));
            rig.click_at(at);
            assert_eq!(
                rig.messages,
                vec![Msg::Close],
                "[{wiring:?}] a click at {at:?} (page, page control, trigger) must only close"
            );
        }
    }
}

#[test]
fn escape_closes_once_and_is_captured() {
    for wiring in COVERED {
        let mut rig = Rig::new(header_scene(true, wiring, LTR, 3));
        let status = rig.press(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            iced::keyboard::Modifiers::empty(),
        );
        assert_eq!(rig.messages, vec![Msg::Close], "[{wiring:?}] once");
        assert_eq!(
            status,
            iced_runtime::core::event::Status::Captured,
            "[{wiring:?}]"
        );
    }
}

/// The trigger's focus is set aside while the menu is open and comes back after.
#[test]
fn focus_returns_to_the_trigger() {
    for wiring in COVERED {
        let mut rig = Rig::new(header_scene(false, wiring, LTR, 3));
        rig.tab(false);
        assert_eq!(
            rig.focused(),
            vec![0],
            "[{wiring:?}] Tab reaches the trigger"
        );
        rig.enter();
        assert_eq!(rig.messages, vec![Msg::Menu(MenuAction::MenuPressed(0))]);
        // The application opens the menu; the runtime rebuilds, then redraws.
        let mut rig = rig.rebuilt(header_scene(true, wiring, LTR, 3));
        // No ring under the open menu: the frame is the one of the same menu opened with
        // nothing ever focused. (`focused()` cannot say: a covered layer reports no target.)
        let mut never_focused = Rig::new(header_scene(true, wiring, LTR, 3));
        assert!(
            rig.frame() == never_focused.frame(),
            "[{wiring:?}] the trigger kept its ring under the open menu"
        );
        assert!(
            rig.focused().is_empty(),
            "[{wiring:?}] the trigger kept its ring under the open menu: {:?}",
            rig.focused()
        );
        rig.tab(false);
        assert_eq!(rig.focused(), vec![0], "[{wiring:?}] Tab reaches item 0");
        // Escape; the application closes the menu.
        rig.press(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            iced::keyboard::Modifiers::empty(),
        );
        let mut rig = rig.rebuilt(header_scene(false, wiring, LTR, 3));
        let targets = rig.targets();
        assert_eq!(targets.len(), 2, "[{wiring:?}] trigger and page control");
        assert_eq!(
            rig.focused(),
            vec![0],
            "[{wiring:?}] focus did not return to the trigger"
        );
    }
}

// ---------------------------------------------------------------------------
// R-3: the engine learns it from the tree
// ---------------------------------------------------------------------------

/// With only the sink set (no `Space`), the layer is covered exactly while a menu is
/// open: the same view code, three states.
#[test]
fn the_cover_follows_what_the_tree_holds() {
    let mut rig = Rig::new(header_scene(false, Wiring::SinkOnly, LTR, 3));
    rig.click_at(PAGE_CONTROL);
    assert_eq!(rig.messages, vec![Msg::Page], "closed: the page is live");
    rig.messages.clear();

    let mut rig = rig.rebuilt(header_scene(true, Wiring::SinkOnly, LTR, 3));
    rig.click_at(PAGE_CONTROL);
    assert_eq!(rig.messages, vec![Msg::Close], "open: the page is covered");
    rig.messages.clear();

    let mut rig = rig.rebuilt(header_scene(false, Wiring::SinkOnly, LTR, 3));
    rig.click_at(PAGE_CONTROL);
    assert_eq!(
        rig.messages,
        vec![Msg::Page],
        "closed again: the page is live"
    );
}

/// Without a sink there is no cover, as today: the page stays pressable and reachable.
#[test]
fn with_no_sink_the_page_stays_live() {
    let mut rig = Rig::new(header_scene(true, Wiring::Bare, LTR, 3));
    assert_eq!(rig.targets().len(), 5, "trigger, page control, three items");
    rig.click_at(PAGE_CONTROL);
    assert_eq!(rig.messages, vec![Msg::Page]);
}

/// `menus.md`'s empty `Space` is harmless: both wirings draw the same pixels and offer
/// the same targets.
#[test]
fn the_empty_space_is_harmless() {
    let mut a = Rig::new(header_scene(true, Wiring::Documented, LTR, 3));
    let mut b = Rig::new(header_scene(true, Wiring::SinkOnly, LTR, 3));
    let (fa, fb) = (a.frame(), b.frame());
    let differing: Vec<usize> = fa
        .as_chunks::<4>()
        .0
        .iter()
        .zip(fb.as_chunks::<4>().0)
        .enumerate()
        .filter(|(_, (x, y))| x != y)
        .map(|(i, _)| i)
        .collect();
    println!(
        "pixels differing with and without the Space: {} (first at {:?})",
        differing.len(),
        differing.first().map(|i| (i % 800, i / 800))
    );
    if let (Some(i), Some(j)) = (differing.first(), differing.last()) {
        println!(
            "  first {:?} {:?} vs {:?}",
            (i % 800, i / 800),
            &fa[i * 4..i * 4 + 4],
            &fb[i * 4..i * 4 + 4]
        );
        println!(
            "  last {:?} {:?} vs {:?}",
            (j % 800, j / 800),
            &fa[j * 4..j * 4 + 4],
            &fb[j * 4..j * 4 + 4]
        );
        let xs: Vec<usize> = differing.iter().map(|i| i % 800).collect();
        let ys: Vec<usize> = differing.iter().map(|i| i / 800).collect();
        println!(
            "  x {}..{} y {}..{}",
            xs.iter().min().unwrap(),
            xs.iter().max().unwrap(),
            ys.iter().min().unwrap(),
            ys.iter().max().unwrap()
        );
    }
    assert!(differing.is_empty(), "the Space changed what is drawn");
    let ta: Vec<_> = a.targets().iter().map(|t| t.bounds).collect();
    let tb: Vec<_> = b.targets().iter().map(|t| t.bounds).collect();
    assert_eq!(ta, tb, "the Space changed what is reachable");
}

/// The starter's wiring: the menu's content is the application's own, in `header_menu`,
/// and `app_header` gets an empty `items` list.
#[test]
fn the_starters_wiring_still_works() {
    let header = snora::widget::app_header("App", vec![menu(0)], &ON_MENU, Some(&0), None, LTR);
    let content: Element<'static, Msg> = snora::control(text("Open…"))
        .on_press(Msg::Menu(MenuAction::MenuItemPressed {
            menu_id: 0,
            menu_item_id: 9,
        }))
        .into();
    let scene = |content| {
        snora::render(
            AppLayout::new(page())
                .header(header_clone(&ON_MENU))
                .header_menu(content)
                .on_close_menus(Msg::Close),
        )
    };
    drop(header);
    let mut rig = Rig::new(scene(content));
    let n = rig.targets().len();
    assert!(
        n >= 1,
        "the application's own menu content is reachable ({n})"
    );
    let at = rig.targets().last().unwrap().bounds.center();
    rig.click_at(at);
    assert_eq!(rig.messages, vec![item_pressed(9)]);
    rig.messages.clear();
    rig.click_at(PAGE_POINT);
    assert_eq!(rig.messages, vec![Msg::Close]);
}

fn header_clone(on: &'static fn(MenuAction<u8, u8>) -> Msg) -> Element<'static, Msg> {
    snora::widget::app_header("App", vec![menu(0)], on, Some(&0), None, LTR)
}

// ---------------------------------------------------------------------------
// Q-3: the open menu's overlay passes `Inert`, and nothing else does
// ---------------------------------------------------------------------------

fn dialog_scene(menu_open: bool) -> Element<'static, Msg> {
    let header = snora::widget::app_header(
        "App",
        vec![menu(3)],
        &ON_MENU,
        menu_open.then_some(&0u8),
        None,
        LTR,
    );
    snora::render(
        AppLayout::new(page())
            .header(header)
            .on_close_menus(Msg::Close)
            .dialog(snora::Dialog::new(text("dialog").into()))
            .on_close_modals(Msg::Close),
    )
}

/// A menu open under a dialog is covered by the dim like everything else: its overlay is
/// not drawn above it, and its items are not reachable.
#[test]
fn a_modal_covers_the_open_menu_too() {
    let mut open = Rig::new(dialog_scene(true));
    let mut closed = Rig::new(dialog_scene(false));
    assert_eq!(
        open.frame(),
        closed.frame(),
        "the dropdown is drawn above the dim"
    );
    let n = open.targets().len();
    let m = closed.targets().len();
    assert_eq!(n, m, "the dropdown's items are reachable under the dim");
    // The premise: without the dialog, the same two scenes do differ.
    let mut a = Rig::new(header_scene(true, Wiring::SinkOnly, LTR, 3));
    let mut b = Rig::new(header_scene(false, Wiring::SinkOnly, LTR, 3));
    assert_ne!(
        a.frame(),
        b.frame(),
        "premise: an open dropdown draws something"
    );
    // And when the dialog goes, the open menu is there again.
    let mut rig = open.rebuilt(header_scene(true, Wiring::SinkOnly, LTR, 3));
    assert_eq!(rig.targets().len(), 3);
}

#[test]
fn every_wiring_draws_the_dropdown_on_the_first_frame() {
    let mut closed = Rig::new(header_scene(false, Wiring::SinkOnly, LTR, 3));
    let plain = closed.frame();
    for wiring in [Wiring::Bare, Wiring::SinkOnly, Wiring::Documented] {
        let mut open = Rig::new(header_scene(true, wiring, LTR, 3));
        let shown = open.frame();
        let differing = shown
            .as_chunks::<4>()
            .0
            .iter()
            .zip(plain.as_chunks::<4>().0)
            .filter(|(x, y)| x != y)
            .count();
        assert!(
            differing > 500,
            "[{wiring:?}] the open menu drew {differing} pixels beyond the closed one: the dropdown is not on the screen"
        );
    }
}

/// A page overlay that is already showing when a menu opens. `Inert` cannot tell one
/// overlay in a group from another, so a covered page is given no pointer instead, and a
/// tooltip that needs one goes. Without that, the bubble floats above the open menu.
#[test]
fn a_page_tooltip_showing_when_a_menu_opens() {
    use iced::widget::tooltip;
    let scene = |open: bool, tip: bool| -> Element<'static, Msg> {
        let body: Element<'static, Msg> = if tip {
            tooltip(
                text("hover me"),
                text("TOOLTIP BUBBLE"),
                tooltip::Position::Bottom,
            )
            .into()
        } else {
            text("hover me").into()
        };
        let header = snora::widget::app_header(
            "App",
            vec![menu(3)],
            &ON_MENU,
            open.then_some(&0u8),
            None,
            LTR,
        );
        snora::render(
            AppLayout::new(body)
                .header(header)
                .on_close_menus(Msg::Close),
        )
    };
    let hover = iced::Point::new(10.0, 56.0);
    let mut with = Rig::new(scene(false, true));
    with.point_at(Some(hover));
    let mut without = Rig::new(scene(false, false));
    without.point_at(Some(hover));
    assert!(
        with.frame() != without.frame(),
        "premise: hovering showed no tooltip"
    );

    let mut with = with.rebuilt(scene(true, true));
    let mut without = without.rebuilt(scene(true, false));
    let differing = |a: &[u8], b: &[u8]| {
        a.as_chunks::<4>()
            .0
            .iter()
            .zip(b.as_chunks::<4>().0)
            .filter(|(x, y)| x != y)
            .count()
    };
    let (a, b) = (with.frame(), without.frame());
    assert_eq!(
        differing(&a, &b),
        0,
        "the page's tooltip is still drawn above the open menu"
    );
    // The pointer moves: the next event.
    with.point_at(Some(iced::Point::new(400.0, 400.0)));
    without.point_at(Some(iced::Point::new(400.0, 400.0)));
    let (a, b) = (with.frame(), without.frame());
    println!(
        "after the pointer moves away: {} px differ",
        differing(&a, &b)
    );
    assert_eq!(
        differing(&a, &b),
        0,
        "the tooltip is still showing after the pointer left"
    );
}
