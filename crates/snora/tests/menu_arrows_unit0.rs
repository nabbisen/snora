//! RFC-123 Unit 0: arrow keys in snora's menus, on a PROTOTYPE
//! (`snora-interaction/src/menu_keys.rs`, wired in `render.rs` and `menu.rs`).
//! Prints the table (`--nocapture`) and asserts each rule.
//!
//! Renderer follows `ICED_TEST_BACKEND` (default tiny-skia); wgpu needs `--test-threads=1`.
#![cfg(feature = "widgets")]
mod common;
use common::removal::Rig;
use iced::keyboard::key::Named::{ArrowDown, ArrowUp, End, Enter, Home};
use iced::widget::{column, text, text_input};
use iced_runtime::core::event::Status;
use snora::{AppLayout, LayoutDirection, Menu, MenuAction, MenuItem};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Menu(MenuAction<u8, u8>),
    Close,
    Page,
    Typed(String),
}
static ON_MENU: fn(MenuAction<u8, u8>) -> Msg = Msg::Menu;

fn menu() -> Menu<u8, u8> {
    Menu {
        id: 0,
        label: "File".into(),
        icon: None,
        items: (0..3)
            .map(|i| MenuItem {
                menu_id: 0,
                id: i,
                label: format!("Item {i}"),
                icon: None,
            })
            .collect(),
    }
}

fn page() -> iced::Element<'static, Msg> {
    column![
        snora::control(text("page")).on_press(Msg::Page),
        text_input("field", "").on_input(Msg::Typed),
    ]
    .into()
}

/// The header's dropdown (items inline in the header, as `app_header` draws them).
fn header(backdrop: bool, dir: LayoutDirection) -> iced::Element<'static, Msg> {
    let header = snora::widget::app_header("App", vec![menu()], &ON_MENU, Some(&0), None, dir);
    let mut l = AppLayout::new(page())
        .header(header)
        .header_menu(iced::widget::space().into())
        .direction(dir);
    if backdrop {
        l = l.on_close_menus(Msg::Close);
    }
    snora::render(l)
}

/// A context menu built from `render_menu`, opened by the application.
fn context(backdrop: bool, dir: LayoutDirection) -> iced::Element<'static, Msg> {
    let m = snora::widget::render_menu(menu(), &ON_MENU, true);
    let mut l = AppLayout::new(page()).context_menu(m).direction(dir);
    if backdrop {
        l = l.on_close_menus(Msg::Close);
    }
    snora::render(l)
}

/// Which focus target is focused, as an index into the Tab order, or `None`.
fn at(rig: &mut Rig<Msg>) -> Option<usize> {
    rig.focused().first().copied()
}

const KEYS: [(&str, iced::keyboard::key::Named); 4] = [
    ("↓", ArrowDown),
    ("↑", ArrowUp),
    ("Home", Home),
    ("End", End),
];

#[test]
fn the_premise_what_is_reachable() {
    println!("\n## focus targets in an open menu (trigger + 3 items = 4 for a menu alone)\n");
    println!("| scene | targets |");
    println!("|---|---|");
    for (name, el) in [
        (
            "header menu, backdrop (the default wiring)",
            header(true, LayoutDirection::Ltr),
        ),
        (
            "header menu, no backdrop",
            header(false, LayoutDirection::Ltr),
        ),
        (
            "context menu, backdrop",
            context(true, LayoutDirection::Ltr),
        ),
        (
            "context menu, no backdrop",
            context(false, LayoutDirection::Ltr),
        ),
    ] {
        let mut rig = Rig::new(el);
        println!("| {name} | {} |", rig.targets().len());
    }
}

/// Rules 1 and 2 (context menu, with backdrop: the page is out of reach, so targets are
/// `[trigger, item 0, item 1, item 2]`): wrap, Home, End.
#[test]
fn arrows_move_between_items_and_wrap() {
    let mut rig = Rig::new(context(true, LayoutDirection::Ltr));
    assert_eq!(rig.targets().len(), 4);
    assert_eq!(at(&mut rig), None, "opening moved focus");
    println!(
        "\n## context menu: ↓ from nothing, then ↓ ↓ ↓ ↑ Home End (target index, 1..3 = items)\n"
    );
    let mut seen = vec![];
    for (label, key) in [
        ("↓", ArrowDown),
        ("↓", ArrowDown),
        ("↓", ArrowDown),
        ("↓ (wraps)", ArrowDown),
        ("↑ (wraps)", ArrowUp),
        ("Home", Home),
        ("End", End),
    ] {
        let status = rig.named(key, false);
        seen.push((label, at(&mut rig), status));
    }
    for (label, i, s) in &seen {
        println!("- {label}: {i:?} {s:?}");
    }
    let got: Vec<_> = seen.iter().map(|s| s.1).collect();
    assert_eq!(
        got,
        vec![
            Some(1),
            Some(2),
            Some(3),
            Some(1),
            Some(3),
            Some(1),
            Some(3)
        ]
    );
    assert!(
        seen.iter().all(|s| s.2 == Status::Captured),
        "an arrow reached the application"
    );
}

/// Rule 3: from nothing focused, ↓ starts at the first item and ↑ at the last; and
/// **no focus at all before the key**, however the menu was opened.
#[test]
fn from_nothing_down_is_first_and_up_is_last() {
    for (name, key, want) in [
        ("↓", ArrowDown, 1),
        ("↑", ArrowUp, 3),
        ("Home", Home, 1),
        ("End", End, 3),
    ] {
        let mut rig = Rig::new(context(true, LayoutDirection::Ltr));
        assert_eq!(
            at(&mut rig),
            None,
            "opened by the application: focus before the key"
        );
        rig.named(key, false);
        assert_eq!(at(&mut rig), Some(want), "{name} from nothing");
    }
    // Opened by the pointer: a click on the trigger of a closed menu, then rebuilt open.
    // (The menu is the application's state; the click publishes and the rebuild opens it.)
    let closed = || {
        let m = snora::widget::render_menu(menu(), &ON_MENU, false);
        snora::render(AppLayout::new(page()).header(m).on_close_menus(Msg::Close))
    };
    let mut rig = Rig::new(closed());
    let trigger = rig.targets()[0].bounds.center();
    rig.click_at(trigger);
    assert!(
        rig.messages
            .contains(&Msg::Menu(MenuAction::MenuPressed(0)))
    );
    assert_eq!(at(&mut rig), None, "a click focused the trigger");
    let mut rig = rig.rebuilt(header(true, LayoutDirection::Ltr));
    println!(
        "\nafter a pointer click and the rebuild, the header-with-backdrop menu has {} targets",
        rig.targets().len()
    );
}

/// Opened by Enter on the trigger, **no backdrop** (the page stays reachable, so the
/// trigger keeps focus): ↓ and ↑ from the trigger.
#[test]
fn from_the_trigger_after_enter() {
    let closed = || {
        let m = snora::widget::render_menu(menu(), &ON_MENU, false);
        snora::render(AppLayout::new(page()).header(m))
    };
    let mut rig = Rig::new(closed());
    // page control, text field, then the trigger? Find the trigger by pressing Tab to it.
    let n = rig.targets().len();
    println!(
        "\nclosed menu, no backdrop: {n} targets (page control, field, trigger in tree order)"
    );
    let trigger_ix = n - 1; // the header is laid out first; see print below
    let _ = trigger_ix;
    for i in 0..n {
        rig.focus_at(i);
        let _ = i;
    }
    // Focus the menu's trigger: it is the first target (header comes before the body).
    rig.focus_at(0);
    rig.enter();
    assert!(
        rig.messages
            .contains(&Msg::Menu(MenuAction::MenuPressed(0))),
        "Enter on the trigger did not press it: {:?}",
        rig.messages
    );
    let mut rig = rig.rebuilt({
        let m = snora::widget::render_menu(menu(), &ON_MENU, true);
        snora::render(
            AppLayout::new(page())
                .header(m)
                .header_menu(iced::widget::space().into()),
        )
    });
    println!(
        "open, no backdrop: {} targets, focused {:?} (the trigger keeps focus)",
        rig.targets().len(),
        rig.focused()
    );
    let before = at(&mut rig);
    let status = rig.named(ArrowDown, false);
    println!(
        "↓ from the trigger: {before:?} -> {:?} ({status:?})",
        at(&mut rig)
    );
}

/// Rule 4: a focused text field, or a page control, under a menu **without** a
/// backdrop keeps its arrows, and the menu does not move.
#[test]
fn a_focused_field_or_page_control_keeps_its_arrows() {
    for (what, names) in [("page control", 0usize), ("text field", 1)] {
        let mut rig = Rig::new(context(false, LayoutDirection::Ltr));
        let n = rig.targets().len();
        // Tab order in the stack: the page first (control, field), then the menu.
        println!("\ncontext menu, no backdrop: {n} targets");
        rig.focus_at(names);
        assert_eq!(at(&mut rig), Some(names));
        for (label, key) in KEYS {
            let status = rig.named(key, false);
            println!(
                "{what} focused, {label}: now {:?} ({status:?})",
                at(&mut rig)
            );
            assert_eq!(
                at(&mut rig),
                Some(names),
                "{what}: {label} moved focus to the menu"
            );
        }
    }
}

/// A held arrow keeps moving, and Enter after arrows presses once, publishing what a
/// click publishes.
#[test]
fn a_held_arrow_moves_each_time_and_enter_presses_once() {
    let mut rig = Rig::new(context(true, LayoutDirection::Ltr));
    rig.named(ArrowDown, false);
    for want in [2, 3, 1, 2] {
        let status = rig.named(ArrowDown, true);
        assert_eq!(at(&mut rig), Some(want), "held ↓");
        assert_eq!(status, Status::Captured);
    }
    rig.messages.clear();
    rig.named(Enter, false);
    assert_eq!(
        rig.messages,
        vec![Msg::Menu(MenuAction::MenuItemPressed {
            menu_id: 0,
            menu_item_id: 1
        })],
        "Enter after the arrows"
    );
}

/// RTL does not mirror a vertical list.
#[test]
fn rtl_does_not_mirror_the_arrows() {
    let mut rig = Rig::new(context(true, LayoutDirection::Rtl));
    rig.named(ArrowDown, false);
    assert_eq!(at(&mut rig), Some(1));
    rig.named(ArrowDown, false);
    assert_eq!(at(&mut rig), Some(2));
}

/// A key that is not ours is not captured; a closed menu answers nothing; a dialog
/// over the menu makes it silent.
#[test]
fn only_the_four_keys_and_only_when_open_and_uncovered() {
    let mut rig = Rig::new(context(true, LayoutDirection::Ltr));
    let s = rig.named(iced::keyboard::key::Named::ArrowLeft, false);
    assert_eq!(s, Status::Ignored, "← was captured");
    assert_eq!(at(&mut rig), None);

    let mut rig = Rig::new(snora::render(AppLayout::new(page())));
    let s = rig.named(ArrowDown, false);
    assert_eq!(s, Status::Ignored, "no menu open: ↓ was captured");

    let m = snora::widget::render_menu(menu(), &ON_MENU, true);
    let dialog = snora::Dialog::new(snora::control(text("OK")).on_press(Msg::Page).into());
    let mut rig = Rig::new(snora::render(
        AppLayout::new(page())
            .context_menu(m)
            .on_close_menus(Msg::Close)
            .on_close_modals(Msg::Close)
            .dialog(dialog),
    ));
    let s = rig.named(ArrowDown, false);
    println!(
        "\nmenu under a dialog: ↓ {s:?}, focused {:?}",
        rig.focused()
    );
    assert_eq!(rig.focused(), Vec::<usize>::new());
}

/// The design path's header dropdown draws through the same `build_menu`, so it carries
/// the same marks; with no backdrop its items are reachable and answer the arrows.
#[cfg(feature = "design")]
#[test]
fn the_design_headers_dropdown_answers_the_arrows_too() {
    let tokens: &'static snora::design::Tokens =
        Box::leak(Box::new(snora::design::Tokens::light()));
    let header = snora::design::widget::app_header(
        tokens,
        "App",
        vec![menu()],
        &ON_MENU,
        Some(&0),
        None,
        LayoutDirection::Ltr,
    );
    let l = AppLayout::new(page())
        .header(header)
        .header_menu(iced::widget::space().into());
    let mut rig = Rig::new(snora::design::render(l, tokens));
    let n = rig.targets().len();
    assert_eq!(rig.focused(), Vec::<usize>::new());
    rig.named(ArrowUp, false);
    let got = at(&mut rig);
    println!(
        "\ndesign header, no backdrop: {n} targets; ↑ from nothing -> {got:?} (the last item; the order is trigger, 3 items, page, field)"
    );
    assert_eq!(got, Some(3));
}
