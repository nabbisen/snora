//! RFC-124: the header's dropdown is an overlay anchored to its trigger. The geometry, the
//! cover (pointer, keyboard, Escape, focus), the engine learning "a menu is open" from the
//! tree, the ring gap, and a dropdown taller than the window. **Pixels, not only targets**:
//! a dropdown that is reachable and not drawn passes every target check.
//! Prints the tables (`--nocapture`).
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
    /// What the application set for Escape on a menu (`AppLayout::escape`).
    EscapeMenu,
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
    header_scene_escaping(open, wiring, dir, items, None)
}

/// The same, with the application's own Escape message for menus, if any.
fn header_scene_escaping(
    open: bool,
    wiring: Wiring,
    dir: LayoutDirection,
    items: u8,
    escape_menu: Option<Msg>,
) -> Element<'static, Msg> {
    let active = open.then_some(&0u8);
    let header = snora::widget::app_header("App", vec![menu(items)], &ON_MENU, active, None, dir);
    let mut l =
        AppLayout::new(page())
            .header(header)
            .direction(dir)
            .escape(snora::keyboard::Escape {
                menu: escape_menu,
                ..Default::default()
            });
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

/// R-1: below the trigger, at its start edge, one ring's reach lower; mirrored under RTL.
/// The item is inset from the card by the same reach (a scrollable clips what it holds, and
/// the ring is outside the item), so the card's edges are the item's moved out by it.
#[test]
fn the_dropdown_is_below_its_trigger_at_the_start_edge() {
    const REACH: f32 = 2.0;
    println!("\n## the card against its trigger (card = item 0 moved out by the reach, {REACH})\n");
    println!("| direction | trigger | card | edge that lines up | card top - trigger bottom |");
    println!("|---|---|---|---|---|");
    for dir in [LTR, RTL] {
        let mut closed = Rig::new(header_scene(false, Wiring::SinkOnly, dir, 3));
        let mut open = Rig::new(header_scene(true, Wiring::SinkOnly, dir, 3));
        let trigger = closed.targets()[0].bounds;
        let item = open.targets()[0].bounds;
        let card = Rectangle {
            x: item.x - REACH,
            y: item.y - REACH,
            width: item.width + 2.0 * REACH,
            height: item.height,
        };
        let (edge, aligned) = if dir == LTR {
            ("left", (card.x - trigger.x).abs() < 0.5)
        } else {
            (
                "right",
                ((card.x + card.width) - (trigger.x + trigger.width)).abs() < 0.5,
            )
        };
        let below = card.y - (trigger.y + trigger.height);
        println!(
            "| {dir:?} | x {:.1} w {:.1} y {:.1} h {:.1} | x {:.1} w {:.1} y {:.1} | {edge}: {aligned} | {below:.1} |",
            trigger.x, trigger.width, trigger.y, trigger.height, card.x, card.width, card.y
        );
        assert!(
            aligned,
            "[{dir:?}] the card's {edge} edge is not the trigger's"
        );
        assert!(
            (below - REACH).abs() < 0.5,
            "[{dir:?}] the card begins {below} below the trigger, not one ring's reach ({REACH})"
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

// ---------------------------------------------------------------------------
// Law 7: Escape publishes the escape message if one is set, else the close message
// ---------------------------------------------------------------------------

#[test]
fn escape_publishes_the_applications_escape_message_when_it_set_one() {
    for wiring in COVERED {
        let mut rig = Rig::new(header_scene_escaping(
            true,
            wiring,
            LTR,
            3,
            Some(Msg::EscapeMenu),
        ));
        rig.press(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            iced::keyboard::Modifiers::empty(),
        );
        assert_eq!(
            rig.messages,
            vec![Msg::EscapeMenu],
            "[{wiring:?}] Escape must publish the application's escape message, once, and not the close message"
        );
    }
}

/// And a click outside still publishes the close message: the escape message is Escape's only.
#[test]
fn a_click_outside_still_publishes_the_close_message_when_an_escape_message_is_set() {
    let mut rig = Rig::new(header_scene_escaping(
        true,
        Wiring::SinkOnly,
        LTR,
        3,
        Some(Msg::EscapeMenu),
    ));
    rig.click_at(PAGE_POINT);
    assert_eq!(rig.messages, vec![Msg::Close]);
}

// ---------------------------------------------------------------------------
// A gap equal to the ring's reach: the trigger's focus ring is never covered
// ---------------------------------------------------------------------------

/// The pixels of a region of a frame (logical == physical at scale 1).
fn region(frame: &[u8], r: Rectangle) -> Vec<[u8; 4]> {
    let (x0, y0) = (r.x.floor().max(0.0) as usize, r.y.floor().max(0.0) as usize);
    // The far edge rounds down: a quad that ends at 40.8 is drawn to row 40, which the next
    // quad (starting at 40.8, snapped to 41) shares only when they overlap.
    let (x1, y1) = (
        ((r.x + r.width).floor() as usize).min(800),
        ((r.y + r.height).floor() as usize).min(600),
    );
    let mut out = Vec::new();
    for y in y0..y1 {
        for x in x0..x1 {
            let i = (y * 800 + x) * 4;
            out.push([frame[i], frame[i + 1], frame[i + 2], frame[i + 3]]);
        }
    }
    out
}

/// With the trigger focused, the ring is the same whether the menu is open or closed. Only
/// the no-cover case can have both: with a cover the trigger's focus is set aside.
fn ring_is_unobscured(closed: Element<'static, Msg>, open: Element<'static, Msg>, reach: f32) {
    let mut closed = Rig::new(closed);
    closed.tab(false);
    assert_eq!(
        closed.focused(),
        vec![0],
        "premise: Tab focuses the trigger"
    );
    let trigger = closed.targets()[0].bounds;
    let mut open = Rig::new(open);
    open.tab(false);
    assert_eq!(
        open.focused(),
        vec![0],
        "premise: the trigger is focused under the open menu"
    );
    // The ring lies within `reach` of the control. Its lower edge is where the card begins.
    let band = Rectangle {
        x: trigger.x - reach,
        y: trigger.y - reach,
        width: trigger.width + 2.0 * reach,
        height: trigger.height + 2.0 * reach,
    };
    let (a, b) = (region(&closed.frame(), band), region(&open.frame(), band));
    let differing = a.iter().zip(&b).filter(|(x, y)| x != y).count();
    assert_eq!(
        differing, 0,
        "the open menu covers {differing} pixels of the trigger's focus ring"
    );
}

#[test]
fn the_card_does_not_cover_the_triggers_ring_on_the_engine_path() {
    ring_is_unobscured(
        header_scene(false, Wiring::Bare, LTR, 3),
        header_scene(true, Wiring::Bare, LTR, 3),
        2.0,
    );
}

#[cfg(feature = "design")]
#[test]
fn the_card_does_not_cover_the_triggers_ring_in_any_preset() {
    println!("\n## ring reach per preset (Tokens::focus.ring_offset)\n");
    // Every preset reaches 2 px; a token that reaches further is what tells the gap follows
    // the token and not a literal.
    let mut wide = snora::design::Tokens::light();
    wide.focus.ring_offset = 6.0;
    let all = snora::design::Tokens::presets()
        .into_iter()
        .chain([("light, ring_offset 6", wide)]);
    for (name, tokens) in all {
        let reach = tokens.focus.ring_offset;
        println!("- {name}: {reach}");
        let scene = |open: bool| -> Element<'static, Msg> {
            let tokens = Box::leak(Box::new(tokens.clone()));
            let header = snora::design::widget::app_header(
                tokens,
                "App",
                vec![menu(3)],
                &ON_MENU,
                open.then_some(&0u8),
                None,
                LTR,
            );
            snora::design::render(AppLayout::new(page()).header(header), tokens)
        };
        ring_is_unobscured(scene(false), scene(true), reach);
    }
}

// ---------------------------------------------------------------------------
// A dropdown taller than the window scrolls, and the focused item is kept in view
// ---------------------------------------------------------------------------

/// Where the changed pixels between two frames are: their centre, and the height of the
/// box around them. The ring is what changed when the same view is drawn focused and not.
fn changed_box(shown: &[u8], plain: &[u8]) -> Option<(iced::Point, usize)> {
    let (mut n, mut sx, mut sy) = (0usize, 0usize, 0usize);
    let (mut top, mut bottom) = (usize::MAX, 0usize);
    for (i, (a, b)) in shown
        .as_chunks::<4>()
        .0
        .iter()
        .zip(plain.as_chunks::<4>().0)
        .enumerate()
    {
        if a != b {
            let (x, y) = (i % 800, i / 800);
            n += 1;
            sx += x;
            sy += y;
            top = top.min(y);
            bottom = bottom.max(y);
        }
    }
    (n > 0).then(|| {
        (
            iced::Point::new((sx / n) as f32, (sy / n) as f32),
            bottom - top + 1,
        )
    })
}

fn ring_centre(shown: &[u8], plain: &[u8]) -> Option<iced::Point> {
    changed_box(shown, plain).map(|(at, _)| at)
}

#[test]
fn a_tall_dropdown_scrolls_and_the_focused_item_is_in_view() {
    const ITEMS: u8 = 40;
    let scene = || header_scene(true, Wiring::SinkOnly, LTR, ITEMS);
    let mut rig = Rig::new(scene());
    println!("\n## {ITEMS} items in a 600 px window: where the ring is after each Tab\n");
    let mut last = None;
    let mut whole_ring: Option<usize> = None;
    for k in 0..usize::from(ITEMS) {
        rig.tab(false);
        assert_eq!(rig.focused(), vec![k], "Tab {k} did not reach item {k}");
        let shown = rig.frame();
        // The same view, same scroll, nothing focused: what the ring is drawn over.
        rig.focus_at(usize::MAX);
        let plain = rig.frame();
        rig.focus_at(k);
        let (at, height) = changed_box(&shown, &plain).unwrap_or_else(|| {
            panic!("item {k} is focused and no ring is drawn: it is out of view")
        });
        // The whole ring is in view, not only part of it: the same height as item 0's,
        // which is not near an edge (a ring cut by the card's edge is shorter).
        let whole = *whole_ring.get_or_insert(height);
        assert!(
            height.abs_diff(whole) <= 1,
            "item {k}: the ring is {height} px tall, item 0's is {whole}: part of it is cut off"
        );
        assert!(
            (40.0..600.0).contains(&at.y),
            "item {k}: the ring is at {at:?}, outside the dropdown's room"
        );
        if k % 10 == 0 || k == usize::from(ITEMS) - 1 {
            println!("- item {k}: ring at y = {:.0}", at.y);
        }
        last = Some(at);
    }
    // The last item is at the bottom of the window, drawn, and pressable.
    let at = last.expect("40 Tabs");
    assert!(at.y > 500.0, "the last item is not at the bottom: {at:?}");
    let mut by_key = Rig::new(scene());
    for _ in 0..ITEMS {
        by_key.tab(false);
    }
    by_key.enter();
    assert_eq!(by_key.messages, vec![item_pressed(ITEMS - 1)]);
    // And by the pointer, where it is drawn.
    rig.click_at(at);
    assert_eq!(rig.messages, vec![item_pressed(ITEMS - 1)]);
}

/// A dropdown that fits is not scrolled: item 0 stays where it opened.
#[test]
fn a_dropdown_that_fits_does_not_scroll() {
    let mut open = Rig::new(header_scene(true, Wiring::SinkOnly, LTR, 3));
    let before = open.frame();
    for _ in 0..3 {
        open.tab(false);
    }
    open.focus_at(0);
    open.redraw();
    let mut clean = Rig::new(header_scene(true, Wiring::SinkOnly, LTR, 3));
    let plain = clean.frame();
    assert_eq!(plain, before, "premise: same view, same frame");
    let at = ring_centre(&open.frame(), &plain).expect("a ring on item 0");
    assert!(at.y < 80.0, "item 0 moved: ring at {at:?}");
}

// ---------------------------------------------------------------------------
// Measured for the architect: does a page under a modal's dim show hover today?
// ---------------------------------------------------------------------------

#[test]
fn does_a_page_under_a_modal_dim_show_hover() {
    let scene = |dialog: bool| -> Element<'static, Msg> {
        let mut layout = AppLayout::new(
            iced::widget::button(text("page button"))
                .on_press(Msg::Page)
                .into(),
        );
        if dialog {
            layout = layout
                .dialog(snora::Dialog::new(text("dialog").into()))
                .on_close_modals(Msg::Close);
        }
        snora::render(layout)
    };
    let differing = |dialog: bool| {
        let mut away = Rig::new(scene(dialog));
        away.point_at(Some(iced::Point::new(700.0, 500.0)));
        // iced 0.14 widgets take their hover status on the redraw request the runtime
        // sends before a draw, not on the pointer move.
        away.redraw();
        let mut over = Rig::new(scene(dialog));
        over.point_at(Some(iced::Point::new(20.0, 15.0)));
        over.redraw();
        let (a, b) = (away.frame(), over.frame());
        a.as_chunks::<4>()
            .0
            .iter()
            .zip(b.as_chunks::<4>().0)
            .filter(|(x, y)| x != y)
            .count()
    };
    // The premise: with no dim, hovering the button changes pixels at all.
    let without = differing(false);
    assert!(
        without > 0,
        "premise: a hovered iced button shows hover on this page"
    );
    let under = differing(true);
    println!(
        "\npage button, pointer over it vs away: {without} px differ with no dialog; \
         {under} px differ under a dialog's dim ({})",
        if under == 0 {
            "no hover shows under the dim"
        } else {
            "HOVER SHOWS under the dim"
        }
    );
}

/// Ruling 3: a page covered by a menu gets no pointer. A hover tint on a control a click
/// cannot reach would promise a click that does not happen.
#[test]
fn a_page_under_an_open_menu_shows_no_hover() {
    let scene = |open: bool| -> Element<'static, Msg> {
        let header = snora::widget::app_header(
            "App",
            vec![menu(3)],
            &ON_MENU,
            open.then_some(&0u8),
            None,
            LTR,
        );
        snora::render(
            AppLayout::new(
                iced::widget::button(text("page button"))
                    .on_press(Msg::Page)
                    .into(),
            )
            .header(header)
            .on_close_menus(Msg::Close),
        )
    };
    let differing = |open: bool| {
        let frame = |at: iced::Point| {
            let mut rig = Rig::new(scene(open));
            rig.point_at(Some(at));
            rig.redraw();
            rig.frame()
        };
        let (a, b) = (
            frame(iced::Point::new(700.0, 500.0)),
            frame(iced::Point::new(20.0, 60.0)),
        );
        a.as_chunks::<4>()
            .0
            .iter()
            .zip(b.as_chunks::<4>().0)
            .filter(|(x, y)| x != y)
            .count()
    };
    assert!(
        differing(false) > 0,
        "premise: with the menu closed, hovering the page button shows hover"
    );
    assert_eq!(
        differing(true),
        0,
        "the page shows hover under an open menu"
    );
}
