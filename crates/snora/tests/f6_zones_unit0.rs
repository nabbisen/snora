//! RFC-126 Unit 0: F6 moves focus between the four zones, measured on a PROTOTYPE
//! (`snora_interaction::zone_focus`, `snora::keyboard::focus_zone`). Prints each row
//! (`--nocapture`) and asserts it.
//!
//! The first section measures the finding on what `main` has: `next_zone` and the
//! application's own variable. The second prototypes the rules.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia); wgpu needs `--test-threads=1`.
#![cfg(feature = "widgets")]
mod common;

use common::removal::{Rig, ring_on};
use iced::keyboard::Modifiers;
use iced::keyboard::key::Named::F6;
use iced::widget::operation::focus;
use iced::widget::{column, row, text};
use iced::{Element, Length};
use snora::focus::{Cycle, FocusZone, ZonePresence, next_zone};
use snora::{
    AppLayout, Dialog, LayoutDirection, Menu, MenuAction, MenuItem, Sheet, Toast, ToastIntent,
};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Press(&'static str),
    Menu(MenuAction<u8, u8>),
    Close,
    Dismiss(u64),
}
static ON_MENU: fn(MenuAction<u8, u8>) -> Msg = Msg::Menu;

fn ctl(label: &'static str) -> Element<'static, Msg> {
    snora::control(text(label))
        .on_press(Msg::Press(label))
        .into()
}

fn menu() -> Menu<u8, u8> {
    Menu {
        id: 0,
        label: "File".into(),
        icon: None,
        items: (0..2)
            .map(|i| MenuItem {
                menu_id: 0,
                id: i,
                label: format!("Item {i}"),
                icon: None,
            })
            .collect(),
    }
}

/// What each zone holds.
#[derive(Clone)]
struct Scene {
    dir: LayoutDirection,
    /// `app_header` with a menu (its trigger is the header's control) instead of plain controls.
    menu: bool,
    open: bool,
    close: bool,
    side: bool,
    /// The footer: a control, plain text (no control), or none.
    footer: Footer,
    dialog: bool,
    sheet: bool,
    toast: bool,
    ctx_menu: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Footer {
    Control,
    Text,
    None,
}

impl Scene {
    fn new() -> Self {
        Self {
            dir: LayoutDirection::Ltr,
            menu: false,
            open: false,
            close: false,
            side: true,
            footer: Footer::Control,
            dialog: false,
            sheet: false,
            toast: false,
            ctx_menu: false,
        }
    }

    /// The names of the focus targets, in the order an operation visits them (the tree, then an
    /// open dropdown, then the toast).
    fn names(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        // The covers hide the whole skeleton from every operation (RFC-119).
        let covered = self.dialog
            || self.sheet
            || (self.menu && self.open && self.close)
            || (self.ctx_menu && self.close);
        if !covered {
            if self.menu {
                v.push("trigger");
            } else {
                v.extend(["H1", "H2"]);
            }
            let body = ["B1", "B2"];
            let side: &[&str] = if self.side { &["S1", "S2"] } else { &[] };
            if self.dir == LayoutDirection::Ltr {
                v.extend(side.iter().copied());
                v.extend(body);
            } else {
                v.extend(body);
                v.extend(side.iter().copied());
            }
            if self.footer == Footer::Control {
                v.push("F1");
            }
        }
        if self.ctx_menu {
            v.extend(["cm-trigger", "cm-item0", "cm-item1"]);
        }
        if self.dialog {
            v.extend(["D1", "D2"]);
        }
        if self.sheet {
            v.push("Sheet1");
        }
        if self.menu && self.open {
            v.extend(["item0", "item1"]);
        }
        if self.toast {
            v.push("toast×");
        }
        v
    }

    fn build(&self) -> Element<'static, Msg> {
        let body: Element<'static, Msg> = column![ctl("B1"), ctl("B2")].into();
        let mut l = AppLayout::new(body).direction(self.dir);
        l = l.header(if self.menu {
            snora::widget::app_header(
                "App",
                vec![menu()],
                &ON_MENU,
                self.open.then_some(&0u8),
                None,
                self.dir,
            )
        } else {
            row![ctl("H1"), ctl("H2")].into()
        });
        if self.side {
            l = l.side_bar(column![ctl("S1"), ctl("S2")].width(Length::Shrink).into());
        }
        match self.footer {
            Footer::Control => l = l.footer(ctl("F1")),
            Footer::Text => l = l.footer(text("Ready").into()),
            Footer::None => {}
        }
        if self.close {
            l = l.on_close_menus(Msg::Close);
        }
        if self.dialog {
            l = l
                .dialog(Dialog::new(column![ctl("D1"), ctl("D2")].into()))
                .on_close_modals(Msg::Close);
        }
        if self.sheet {
            l = l.sheet(Sheet::new(ctl("Sheet1")));
        }
        if self.toast {
            l = l.toasts(vec![Toast::new(
                7,
                ToastIntent::Info,
                "T",
                "b",
                Msg::Dismiss(7),
            )]);
        }
        if self.ctx_menu {
            l = l.context_menu(snora::widget::render_menu(menu(), &ON_MENU, true));
        }
        snora::render(l)
    }

    fn rig(&self) -> Rig<Msg> {
        Rig::new(self.build())
    }
}

fn press_f6(rig: &mut Rig<Msg>, shift: bool) {
    let mods = if shift {
        Modifiers::SHIFT
    } else {
        Modifiers::empty()
    };
    let cycle = snora::keyboard::cycle_zones(iced::keyboard::Key::Named(F6), mods).unwrap();
    let _ = rig.task::<Msg>(snora::keyboard::focus_zone(cycle));
}

/// The names of the focused targets.
fn held(rig: &mut Rig<Msg>, names: &[&'static str]) -> Vec<&'static str> {
    rig.focused().into_iter().map(|i| names[i]).collect()
}

// ---------------------------------------------------------------------------
// On what main has: the finding
// ---------------------------------------------------------------------------

/// Finding 1: following the book, F6 changes a variable and moves no focus. And what an
/// application could write to move focus into a zone cannot reach a prefab control.
#[test]
fn m1_an_application_cannot_move_focus_into_a_zone() {
    let scene = Scene {
        menu: true,
        ..Scene::new()
    };
    let names = scene.names();
    let mut rig = scene.rig();
    // The book's example, to the letter: `next_zone` says where to go, and the move is "application-owned".
    let mut var = FocusZone::Body;
    let cycle =
        snora::keyboard::cycle_zones(iced::keyboard::Key::Named(F6), Modifiers::empty()).unwrap();
    let next = next_zone(var, cycle, ZonePresence::all(), false, false);
    if let Some(zone) = next {
        var = zone;
    }
    println!(
        "following the book: F6 from {:?} -> {next:?}; variable now {var:?}; focus now {:?}",
        FocusZone::Body,
        held(&mut rig, &names)
    );
    assert_eq!(
        held(&mut rig, &names),
        Vec::<&str>::new(),
        "nothing moved focus"
    );

    // The nearest thing the application can write: focus by the region's own container id.
    for region in [
        "snora-header",
        "snora-sidebar",
        "snora-body",
        "snora-footer",
    ] {
        let _ = rig.task::<Msg>(focus(region));
        println!("focus({region:?}) -> {:?}", held(&mut rig, &names));
        assert_eq!(held(&mut rig, &names), Vec::<&str>::new(), "{region}");
    }
    // The header's trigger is a snora control the application did not build: no id to name.
    println!(
        "targets: {names:?}; the trigger's id is made inside the widget (RFC-127 Q-3: no id on prefab controls)"
    );
}

/// Finding 2: it does not know where focus is. The variable says the body; the reader Tabbed
/// into the sidebar.
#[test]
fn m2_next_zone_counts_from_a_variable_not_from_where_focus_is() {
    let scene = Scene::new();
    let names = scene.names();
    let mut rig = scene.rig();
    // The reader Tabs: H1 H2 S1 -> the sidebar.
    for _ in 0..3 {
        rig.tab(false);
    }
    println!("the reader Tabbed into: {:?}", held(&mut rig, &names));
    let var = FocusZone::Body; // the application's variable, changed only by F6
    let cycle = Cycle::Forward;
    let from_variable = next_zone(var, cycle, ZonePresence::all(), false, false);
    let from_focus = next_zone(FocusZone::SideBar, cycle, ZonePresence::all(), false, false);
    println!(
        "next_zone from the variable ({var:?}): {from_variable:?}; from where focus is (SideBar): {from_focus:?}"
    );
    assert_eq!(from_variable, Some(FocusZone::Footer));
    assert_eq!(from_focus, Some(FocusZone::Body));
}

/// Finding 3: it contradicts "covered": a menu with its backdrop covers the page, and
/// `next_zone` answers as if none were open.
#[test]
fn m3_next_zone_ignores_a_covering_menu() {
    let without = next_zone(
        FocusZone::Body,
        Cycle::Forward,
        ZonePresence::all(),
        false,
        false,
    );
    let with_menu = next_zone(
        FocusZone::Body,
        Cycle::Forward,
        ZonePresence::all(),
        false,
        true,
    );
    println!("menu closed {without:?}; menu open {with_menu:?}");
    assert_eq!(without, with_menu);
}

/// Finding 4: a zone without a control is a stop.
#[test]
fn m4_a_footer_of_plain_text_is_a_stop() {
    let next = next_zone(
        FocusZone::Body,
        Cycle::Forward,
        ZonePresence::all(),
        false,
        false,
    );
    println!("a footer of plain text: F6 from the body -> {next:?} (it has no control)");
    assert_eq!(next, Some(FocusZone::Footer));
}

// ---------------------------------------------------------------------------
// The prototype
// ---------------------------------------------------------------------------

/// Rules 1 and 2, with the ring from pixels: all four zones, wrapping, both directions.
#[test]
fn rules_1_and_3_all_four_zones_wrap_both_ways_and_the_ring_is_drawn() {
    let scene = Scene::new();
    let names = scene.names();
    let mut rig = scene.rig();
    let mut clean = scene.rig();
    let mut seen = Vec::new();
    // Nothing focused: F6 goes to the first zone with a control, then on.
    for _ in 0..5 {
        press_f6(&mut rig, false);
        let h = held(&mut rig, &names);
        let ring = ring_on(&mut rig, &mut clean).map(|i| names[i]);
        seen.push((h, ring));
    }
    println!("F6 x5: {seen:?}");
    let want = ["H1", "S1", "B1", "F1", "H1"];
    for (i, (h, ring)) in seen.iter().enumerate() {
        assert_eq!(h, &vec![want[i]], "F6 #{i}");
        assert_eq!(ring.as_deref(), Some(want[i]), "ring #{i}");
    }
    // Shift+F6 goes back.
    let mut back = Vec::new();
    for _ in 0..4 {
        press_f6(&mut rig, true);
        back.push(held(&mut rig, &names)[0]);
    }
    println!("Shift+F6 x4: {back:?}");
    assert_eq!(back, vec!["F1", "B1", "S1", "H1"]);
}

/// A Tab into another zone, then F6: counted from where focus now is.
#[test]
fn rule_1_after_a_tab_into_another_zone_f6_counts_from_there() {
    let scene = Scene::new();
    let names = scene.names();
    let mut rig = scene.rig();
    press_f6(&mut rig, false); // H1
    for _ in 0..3 {
        rig.tab(false); // H2 S1 S2
    }
    assert_eq!(held(&mut rig, &names), vec!["S2"]);
    press_f6(&mut rig, false);
    println!("Tabbed to S2, F6 -> {:?}", held(&mut rig, &names));
    assert_eq!(
        held(&mut rig, &names),
        vec!["B1"],
        "from the sidebar, the body"
    );
    rig.tab(false); // B2
    press_f6(&mut rig, true);
    println!("Tabbed to B2, Shift+F6 -> {:?}", held(&mut rig, &names));
    assert_eq!(
        held(&mut rig, &names),
        vec!["S1"],
        "back from the body, the sidebar"
    );
}

/// Rule 2: a zone with no control is skipped: a footer of plain text, an absent sidebar.
#[test]
fn rule_2_a_zone_without_a_control_is_skipped() {
    for (name, scene, want) in [
        (
            "footer is plain text",
            Scene {
                footer: Footer::Text,
                ..Scene::new()
            },
            vec!["H1", "S1", "B1", "H1"],
        ),
        (
            "no footer",
            Scene {
                footer: Footer::None,
                ..Scene::new()
            },
            vec!["H1", "S1", "B1", "H1"],
        ),
        (
            "no sidebar",
            Scene {
                side: false,
                ..Scene::new()
            },
            vec!["H1", "B1", "F1", "H1"],
        ),
    ] {
        let names = scene.names();
        let mut rig = scene.rig();
        let mut walk = Vec::new();
        for _ in 0..4 {
            press_f6(&mut rig, false);
            walk.push(held(&mut rig, &names)[0]);
        }
        println!("{name}: F6 x4 -> {walk:?}");
        assert_eq!(walk, want, "{name}");
    }
    // Backwards, from the body, over the skipped one.
    let scene = Scene {
        footer: Footer::Text,
        ..Scene::new()
    };
    let names = scene.names();
    let mut rig = scene.rig();
    rig.focus_at(names.iter().position(|n| *n == "H1").unwrap());
    press_f6(&mut rig, true);
    println!(
        "H1, Shift+F6 over a plain-text footer -> {:?}",
        held(&mut rig, &names)
    );
    assert_eq!(
        held(&mut rig, &names),
        vec!["B1"],
        "the footer has no control: back to the body"
    );
}

/// Rule 3: with focus in no zone (nothing focused, or a toast): F6 goes to the first zone with a
/// control, Shift+F6 to the last.
#[test]
fn rule_3_with_focus_in_no_zone() {
    let scene = Scene {
        toast: true,
        ..Scene::new()
    };
    let names = scene.names();
    // Nothing focused.
    let mut rig = scene.rig();
    press_f6(&mut rig, false);
    assert_eq!(held(&mut rig, &names), vec!["H1"], "F6 from nothing");
    let mut rig = scene.rig();
    press_f6(&mut rig, true);
    assert_eq!(held(&mut rig, &names), vec!["F1"], "Shift+F6 from nothing");
    // A toast's close control focused.
    let toast = names.iter().position(|n| *n == "toast×").unwrap();
    let mut rig = scene.rig();
    rig.focus_at(toast);
    assert_eq!(held(&mut rig, &names), vec!["toast×"]);
    press_f6(&mut rig, false);
    println!("a toast focused, F6 -> {:?}", held(&mut rig, &names));
    assert_eq!(held(&mut rig, &names), vec!["H1"]);
    let mut rig = scene.rig();
    rig.focus_at(toast);
    press_f6(&mut rig, true);
    assert_eq!(held(&mut rig, &names), vec!["F1"]);
}

/// Rule 5, measured and not assumed: under a cover, F6 does nothing. From nothing focused and
/// from a control of the cover focused.
#[test]
fn rule_5_under_a_cover_f6_does_nothing() {
    let covers = [
        (
            "a dialog",
            Scene {
                dialog: true,
                ..Scene::new()
            },
        ),
        (
            "a sheet",
            Scene {
                sheet: true,
                ..Scene::new()
            },
        ),
        (
            "a context menu with its backdrop",
            Scene {
                ctx_menu: true,
                close: true,
                ..Scene::new()
            },
        ),
        (
            "an open app_header dropdown with a close message",
            Scene {
                menu: true,
                open: true,
                close: true,
                ..Scene::new()
            },
        ),
    ];
    for (name, scene) in covers {
        let names = scene.names();
        for shift in [false, true] {
            // From nothing focused.
            let mut rig = scene.rig();
            press_f6(&mut rig, shift);
            let from_nothing = held(&mut rig, &names);
            // From the cover's own first control focused.
            let mut rig = scene.rig();
            rig.focus_at(0);
            let before = held(&mut rig, &names);
            press_f6(&mut rig, shift);
            let after = held(&mut rig, &names);
            println!(
                "{name}, shift {shift}: from nothing {from_nothing:?}; from {before:?} -> {after:?}; targets {names:?}"
            );
            assert_eq!(
                from_nothing,
                Vec::<&str>::new(),
                "{name}: moved from nothing"
            );
            assert_eq!(before, after, "{name}: moved");
        }
    }
}

/// Rule 4: a dropdown item focused (a menu **without** a close message), then F6: the sidebar.
/// With a close message the page is covered: nothing moves.
#[test]
fn rule_4_a_dropdown_belongs_to_its_triggers_zone() {
    let scene = Scene {
        menu: true,
        open: true,
        close: false,
        ..Scene::new()
    };
    let names = scene.names();
    let item = names.iter().position(|n| *n == "item0").unwrap();
    let mut rig = scene.rig();
    rig.focus_at(item);
    assert_eq!(held(&mut rig, &names), vec!["item0"]);
    press_f6(&mut rig, false);
    println!(
        "item0 focused (no close message), F6 -> {:?}",
        held(&mut rig, &names)
    );
    assert_eq!(
        held(&mut rig, &names),
        vec!["S1"],
        "the header's dropdown is the header's zone: next is the sidebar"
    );
    // Backwards from the dropdown: the footer (the header's previous zone, wrapping).
    let mut rig = scene.rig();
    rig.focus_at(item);
    press_f6(&mut rig, true);
    println!("item0 focused, Shift+F6 -> {:?}", held(&mut rig, &names));
    assert_eq!(held(&mut rig, &names), vec!["F1"]);
    // F6 from the body *into* the header lands on the trigger, never in the dropdown.
    let mut rig = scene.rig();
    let b1 = names.iter().position(|n| *n == "B1").unwrap();
    rig.focus_at(b1);
    press_f6(&mut rig, false); // footer
    press_f6(&mut rig, false); // header
    println!("body -> footer -> header: {:?}", held(&mut rig, &names));
    assert_eq!(held(&mut rig, &names), vec!["trigger"]);
    // With a close message: covered.
    let scene = Scene {
        menu: true,
        open: true,
        close: true,
        ..Scene::new()
    };
    let names = scene.names();
    let mut rig = scene.rig();
    rig.focus_at(names.iter().position(|n| *n == "item0").unwrap());
    press_f6(&mut rig, false);
    println!(
        "item0 focused (close message), F6 -> {:?}",
        held(&mut rig, &names)
    );
    assert_eq!(held(&mut rig, &names), vec!["item0"]);
}

/// RTL: the same logical order.
#[test]
fn rtl_keeps_the_logical_order() {
    let scene = Scene {
        dir: LayoutDirection::Rtl,
        ..Scene::new()
    };
    let names = scene.names();
    let mut rig = scene.rig();
    let mut walk = Vec::new();
    for _ in 0..4 {
        press_f6(&mut rig, false);
        walk.push(held(&mut rig, &names)[0]);
    }
    println!("RTL (tree order {names:?}): F6 x4 -> {walk:?}");
    assert_eq!(walk, vec!["H1", "S1", "B1", "F1"]);
}

/// Rule 6: F6 never presses anything; a held F6 moves once per press.
#[test]
fn rule_6_f6_never_presses_and_a_held_f6_moves_once_per_press() {
    let scene = Scene::new();
    let names = scene.names();
    let mut rig = scene.rig();
    for _ in 0..6 {
        press_f6(&mut rig, false);
    }
    println!("six presses -> {:?}", held(&mut rig, &names));
    assert_eq!(
        held(&mut rig, &names),
        vec!["S1"],
        "six presses from nothing: H1 S1 B1 F1 H1 S1"
    );
    assert!(
        rig.messages.is_empty(),
        "F6 pressed something: {:?}",
        rig.messages
    );
}

/// The design path: the same walk.
#[cfg(feature = "design")]
#[test]
fn the_design_path_walks_the_same_zones() {
    let t: &'static snora::design::Tokens = Box::leak(Box::new(snora::design::Tokens::light()));
    let dc = |label: &'static str| -> Element<'static, Msg> {
        snora::design::control(t, text(label))
            .on_press(Msg::Press(label))
            .into()
    };
    let layout = AppLayout::new(Element::from(column![dc("B1")]))
        .header(dc("H1"))
        .side_bar(dc("S1"))
        .footer(dc("F1"));
    let mut rig = Rig::new(snora::design::render(layout, t));
    let names = ["H1", "S1", "B1", "F1"];
    let mut walk = Vec::new();
    for _ in 0..4 {
        press_f6(&mut rig, false);
        walk.push(names[rig.focused()[0]]);
    }
    println!("design path: {walk:?}");
    assert_eq!(walk, vec!["H1", "S1", "B1", "F1"]);
}
