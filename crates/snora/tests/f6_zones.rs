//! RFC-126: F6 and Shift+F6 move focus between the four zones, through `snora::keyboard::focus_zone`
//! on the runtime's own interface. One test per rule (the rules are in
//! `snora-interaction/src/zones.rs` and `focus_zone`'s rustdoc); the ring is read from pixels.
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

/// A control with no handler: disabled, so it is not a focus target.
fn disabled(label: &'static str) -> Element<'static, Msg> {
    snora::control(text(label)).into()
}

fn ctl_id(label: &'static str, id: &'static str) -> Element<'static, Msg> {
    snora::control(text(label))
        .on_press(Msg::Press(label))
        .id(id)
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
    /// The sidebar's controls have no handler: disabled, so no focus target (R-019).
    side_disabled: bool,
    /// Content in the `header_menu` slot, with `close` giving it its backdrop (R-025).
    header_menu: bool,
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
            side_disabled: false,
            header_menu: false,
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
            || (self.ctx_menu && self.close)
            || (self.header_menu && self.close);
        if !covered {
            if self.menu {
                v.push("trigger");
            } else {
                v.extend(["H1", "H2"]);
            }
            let body = ["B1", "B2"];
            let side: &[&str] = if self.side && !self.side_disabled {
                &["S1", "S2"]
            } else {
                &[]
            };
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
        if self.header_menu {
            v.push("hm");
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
        let body: Element<'static, Msg> = column![ctl("B1"), ctl_id("B2", "b2")].into();
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
            let items: Element<'static, Msg> = if self.side_disabled {
                column![disabled("S1"), disabled("S2")].into()
            } else {
                column![ctl("S1"), ctl("S2")].into()
            };
            l = l.side_bar(iced::widget::container(items).width(Length::Shrink).into());
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
        if self.header_menu {
            l = l.header_menu(ctl("hm"));
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
// The rules
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

/// Rule 1's "counted from where focus is": a focus a **task** put there is the starting point too.
#[test]
fn rule_1_a_focus_moved_by_a_task_is_where_f6_counts_from() {
    let scene = Scene::new();
    let names = scene.names();
    let mut rig = scene.rig();
    let _ = rig.task::<Msg>(focus("b2"));
    assert_eq!(held(&mut rig, &names), vec!["B2"]);
    press_f6(&mut rig, false);
    assert_eq!(
        held(&mut rig, &names),
        vec!["F1"],
        "from the body, the footer"
    );
    let _ = rig.task::<Msg>(focus("b2"));
    press_f6(&mut rig, true);
    assert_eq!(
        held(&mut rig, &names),
        vec!["S1"],
        "back from the body, the sidebar"
    );
}

/// Rule 3 and 4: an inline **context menu is in no zone** (the application opens it at a point), so
/// F6 from one of its controls is "focus in no zone": the first zone, Shift+F6 the last. (The dropdown
/// differs: it has a trigger, and the trigger's zone is known.)
#[test]
fn rule_3_a_context_menu_is_in_no_zone() {
    let scene = Scene {
        ctx_menu: true,
        ..Scene::new()
    };
    let names = scene.names();
    for (shift, want) in [(false, "H1"), (true, "F1")] {
        let mut rig = scene.rig();
        rig.focus_at(names.iter().position(|n| *n == "cm-item0").unwrap());
        assert_eq!(held(&mut rig, &names), vec!["cm-item0"]);
        press_f6(&mut rig, shift);
        println!(
            "context menu item focused, shift {shift} -> {:?}",
            held(&mut rig, &names)
        );
        assert_eq!(held(&mut rig, &names), vec![want], "shift {shift}");
    }
}

/// Rule 6: a held F6 arrives as one key event per repeat, and each is one move. The binding takes
/// the key and the modifiers, so a repeat is just another call.
#[test]
fn rule_6_a_held_f6_keeps_moving_one_zone_per_repeat() {
    use common::removal::key_event;
    let scene = Scene::new();
    let names = scene.names();
    let mut rig = scene.rig();
    let mut walk = Vec::new();
    for repeat in [false, true, true, true] {
        let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) =
            key_event(F6, repeat, Modifiers::empty())
        else {
            unreachable!()
        };
        let cycle = snora::keyboard::cycle_zones(key, modifiers).expect("F6 is claimed");
        let _ = rig.task::<Msg>(snora::keyboard::focus_zone(cycle));
        walk.push(held(&mut rig, &names)[0]);
    }
    println!("F6 held: {walk:?}");
    assert_eq!(walk, vec!["H1", "S1", "B1", "F1"]);
    assert!(rig.messages.is_empty(), "a held F6 pressed something");
}

// ---------------------------------------------------------------------------
// 0.62 (the audit's A-2): F6 after a click counts from Tab's starting point
// ---------------------------------------------------------------------------

/// A click on the target named `name`: it presses (its message is published) and focuses nothing
/// (RFC-121 R-1), and the control becomes Tab's starting point (R-2).
fn click(rig: &mut Rig<Msg>, names: &[&'static str], name: &'static str) {
    let i = names.iter().position(|n| *n == name).unwrap();
    let at = rig.targets()[i].bounds.center();
    rig.click_at(at);
    assert_eq!(
        rig.messages.last(),
        Some(&Msg::Press(name)),
        "premise: the click pressed {name}"
    );
    assert!(
        rig.focused().is_empty(),
        "premise: a click focuses nothing (RFC-121)"
    );
}

/// A press on a point where no control is: the body's empty right-hand side.
const NOWHERE: iced::Point = iced::Point::new(700.0, 450.0);

/// Rule 1: with nothing focused, F6 counts from the zone of the control last clicked, as Tab
/// carries on from it. Sidebar -> body; Shift+F6 from the same click -> the header.
#[test]
fn rule_1_after_a_click_f6_counts_from_the_clicked_controls_zone() {
    let scene = Scene::new();
    let names = scene.names();
    for (shift, want) in [(false, "B1"), (true, "H1")] {
        let mut rig = scene.rig();
        click(&mut rig, &names, "S2");
        press_f6(&mut rig, shift);
        println!("click S2, shift {shift} -> {:?}", held(&mut rig, &names));
        assert_eq!(held(&mut rig, &names), vec![want], "shift {shift}");
    }
    // From the body: the footer, and back, the sidebar.
    for (shift, want) in [(false, "F1"), (true, "S1")] {
        let mut rig = scene.rig();
        click(&mut rig, &names, "B2");
        press_f6(&mut rig, shift);
        assert_eq!(
            held(&mut rig, &names),
            vec![want],
            "from the body, shift {shift}"
        );
    }
}

/// Rule 3: a press that is not on a control clears the starting point, so F6 is "from no zone"
/// again: the header (the first zone with a control), as with no click at all.
#[test]
fn rule_3_a_press_elsewhere_clears_the_starting_point() {
    let scene = Scene::new();
    let names = scene.names();
    let mut rig = scene.rig();
    click(&mut rig, &names, "S2");
    let before = rig.messages.len();
    rig.click_at(NOWHERE);
    assert_eq!(
        rig.messages.len(),
        before,
        "premise: the press landed on no control"
    );
    press_f6(&mut rig, false);
    assert_eq!(held(&mut rig, &names), vec!["H1"]);
    // And backwards: the last zone.
    let mut rig = scene.rig();
    click(&mut rig, &names, "S2");
    rig.click_at(NOWHERE);
    press_f6(&mut rig, true);
    assert_eq!(held(&mut rig, &names), vec!["F1"]);
}

/// Rule 1: once F6 has moved focus the starting point is spent, as Tab spends it: if focus is lost
/// later, the next F6 does not count from the old click.
#[test]
fn rule_1_moving_focus_spends_the_starting_point() {
    let scene = Scene::new();
    let names = scene.names();
    let mut rig = scene.rig();
    click(&mut rig, &names, "S2");
    press_f6(&mut rig, false);
    assert_eq!(held(&mut rig, &names), vec!["B1"]);
    // Focus is lost without a press (an application's task, here the one that takes it away).
    rig.run(Box::new(Blur));
    assert!(rig.focused().is_empty(), "premise: focus was taken away");
    press_f6(&mut rig, false);
    assert_eq!(
        held(&mut rig, &names),
        vec!["H1"],
        "from no zone, not from the sidebar click that came before the last F6"
    );
}

/// Takes focus from every control, as an application's `unfocus` would.
struct Blur;
impl iced_runtime::core::widget::Operation<()> for Blur {
    fn traverse(
        &mut self,
        operate: &mut dyn FnMut(&mut dyn iced_runtime::core::widget::Operation<()>),
    ) {
        operate(self);
    }
    fn focusable(
        &mut self,
        _id: Option<&iced_runtime::core::widget::Id>,
        _bounds: iced::Rectangle,
        state: &mut dyn iced_runtime::core::widget::operation::Focusable,
    ) {
        state.unfocus();
    }
}

/// Rule 5: a click on the dialog's own control, then F6: nothing moves. The page under the dialog
/// has a starting point of its own from before it opened (a click on the sidebar), and it is
/// covered, so it names no target.
#[test]
fn rule_5_a_click_in_a_dialog_then_f6_moves_nothing() {
    let page = Scene::new();
    let page_names = page.names();
    let mut rig = page.rig();
    click(&mut rig, &page_names, "S2");
    let with_dialog = Scene {
        dialog: true,
        ..Scene::new()
    };
    let names = with_dialog.names();
    let mut rig = rig.rebuilt(with_dialog.build());
    assert_eq!(
        rig.targets().len(),
        names.len(),
        "premise: only the dialog's controls are targets"
    );
    click(&mut rig, &names, "D1");
    press_f6(&mut rig, false);
    assert!(
        rig.focused().is_empty(),
        "F6 moved focus under a dialog: {:?}",
        held(&mut rig, &names)
    );
    press_f6(&mut rig, true);
    assert!(
        rig.focused().is_empty(),
        "Shift+F6 moved focus under a dialog"
    );
}

// ---- the 0.62 audit, Unit 3 ---------------------------------------------------------------

/// **R-012: snora does not answer F6 by itself.** The key event, delivered to the rendered layout:
/// `Ignored`, nothing focused, nothing published, for F6 and Shift+F6. The application routes F6 (to
/// `cycle_zones` and `focus_zone`); the premise is that this scene does move when it does.
#[test]
fn the_engine_does_not_answer_f6() {
    let scene = Scene::new();
    let names = scene.names();
    let mut rig = scene.rig();
    for mods in [Modifiers::empty(), Modifiers::SHIFT] {
        let status = rig.press(iced::keyboard::Key::Named(F6), mods);
        assert_eq!(
            status,
            iced_runtime::core::event::Status::Ignored,
            "F6 with {mods:?}"
        );
        assert!(rig.focused().is_empty(), "F6 with {mods:?} moved focus");
        assert!(rig.messages.is_empty(), "{:?}", rig.messages);
    }
    press_f6(&mut rig, false);
    assert_eq!(
        held(&mut rig, &names),
        vec!["H1"],
        "the premise: routed by the application, F6 moves"
    );
}

/// **R-019: a zone whose only controls are disabled is skipped** (rule 2), like one with none: the
/// sidebar's controls have no handler, so F6 goes header, body, footer, header, and Shift+F6 from
/// the body goes to the header.
#[test]
fn rule_2_a_zone_whose_controls_are_all_disabled_is_skipped() {
    let scene = Scene {
        side_disabled: true,
        ..Scene::new()
    };
    let names = scene.names();
    assert!(
        !names.contains(&"S1"),
        "the premise: a disabled control is no target"
    );
    let mut rig = scene.rig();
    assert_eq!(
        rig.targets().len(),
        names.len(),
        "the tree reports exactly the enabled controls"
    );
    let mut walk = Vec::new();
    for _ in 0..4 {
        press_f6(&mut rig, false);
        walk.push(held(&mut rig, &names)[0]);
    }
    assert_eq!(walk, vec!["H1", "B1", "F1", "H1"]);
    let mut rig = scene.rig();
    rig.focus_at(names.iter().position(|n| *n == "B1").unwrap());
    press_f6(&mut rig, true);
    assert_eq!(held(&mut rig, &names), vec!["H1"], "Shift+F6 over it");
}

/// **R-025: rule 5 for a `header_menu` with its backdrop**, the cover `rule_5_under_a_cover_...`
/// does not run: the page is covered, F6 does nothing from nothing and from the menu's own control.
#[test]
fn rule_5_under_a_header_menu_with_its_backdrop_f6_does_nothing() {
    let scene = Scene {
        header_menu: true,
        close: true,
        ..Scene::new()
    };
    let names = scene.names();
    assert_eq!(names, vec!["hm"], "the premise: the page is covered");
    for shift in [false, true] {
        let mut rig = scene.rig();
        press_f6(&mut rig, shift);
        assert!(
            held(&mut rig, &names).is_empty(),
            "moved from nothing (shift {shift})"
        );
        let mut rig = scene.rig();
        rig.focus_at(0);
        press_f6(&mut rig, shift);
        assert_eq!(
            held(&mut rig, &names),
            vec!["hm"],
            "moved from the menu's control (shift {shift})"
        );
    }
}

/// **R-021: a control of a dialog or a sheet is in no zone, and F6 does not take it into one.**
/// With focus on the dialog's first control (or the sheet's), F6 and Shift+F6 leave it where it is,
/// and the skeleton's controls stay out of reach. (The rustdoc used to say "goes to the first zone";
/// rule 5 is what holds, and this names the dialog and the sheet.)
#[test]
fn a_dialogs_or_a_sheets_control_keeps_focus_through_f6() {
    for (what, scene, first) in [
        (
            "a dialog",
            Scene {
                dialog: true,
                ..Scene::new()
            },
            "D1",
        ),
        (
            "a sheet",
            Scene {
                sheet: true,
                ..Scene::new()
            },
            "Sheet1",
        ),
    ] {
        let names = scene.names();
        for shift in [false, true] {
            let mut rig = scene.rig();
            rig.focus_at(names.iter().position(|n| *n == first).unwrap());
            press_f6(&mut rig, shift);
            assert_eq!(
                held(&mut rig, &names),
                vec![first],
                "{what}: F6 (shift {shift}) took focus out of it"
            );
        }
    }
}
