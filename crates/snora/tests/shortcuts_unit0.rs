//! RFC-116 R-2, Unit 0: the shortcuts router, measured before it is ruled.
//! **Nothing here ships.** The router is `common/router_proto.rs`, a prototype.
//!
//! Each test answers one row of the handoff (`implementation-handoff-r2-unit0.md`)
//! and **prints what it measured** (`--nocapture`), then asserts it, so a row can
//! be shown failing. Driven through the runtime's own `UserInterface`, as
//! `modal_keyboard.rs` is: `update` returns each event's `Status`, the testable
//! stand-in for "an application's `keyboard::listen()` never receives it"
//! (`listen()` passes only `Ignored`).
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia). A wgpu run must
//! use `--test-threads=1`.

#![cfg(all(feature = "widgets", feature = "design"))]

mod common;
use common::removal::Rig;
use common::router_proto::{B1, Captures, Order, Route, Router, Target};

use iced::Element;
use iced::keyboard::{Key, Modifiers, key::Named};
use iced::widget::{column, container, row, text_editor, text_input};
use iced_runtime::core::event::Status;
use iced_runtime::core::widget::Id;

use snora::design::{Tokens, button};
use snora::widget::SideBarLabels;
use snora::{
    AppLayout, Dialog, Icon, LayoutDirection, Sheet, SideBar, SideBarItem, Toast, ToastIntent,
    render,
};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    /// What a click on sidebar item *n* publishes.
    Go(u8),
    Page,
    A1,
    A2,
    B1,
    B2,
    MenuItem,
    DialogOk,
    SheetOk,
    ToastDismiss,
    CloseMenus,
    CloseModals,
    Escape,
    Parent,
    Typed,
    Edit,
}

fn tokens() -> &'static Tokens {
    use std::sync::OnceLock;
    static T: OnceLock<Tokens> = OnceLock::new();
    T.get_or_init(Tokens::light)
}

fn btn(label: &str, msg: Msg) -> Element<'static, Msg> {
    button::primary(tokens(), label.to_owned(), msg)
}

const CTRL: Modifiers = Modifiers::CTRL;
const SIDEBAR: &str = "snora-sidebar";

/// The sidebar a click would use, with its three items.
fn sidebar() -> Element<'static, Msg> {
    let bar = SideBar {
        items: (0..3u8)
            .map(|i| SideBarItem {
                view_id: i,
                icon: Icon::Text(format!("{}", i + 1)),
                tooltip: format!("Item {i}"),
                on_press: Msg::Go(i),
            })
            .collect(),
        active: 0,
    };
    container(snora::widget::app_side_bar_with(
        bar,
        LayoutDirection::Ltr,
        SideBarLabels::Beneath,
    ))
    // The id the engine's own sidebar slot carries.
    .id(Id::new(SIDEBAR))
    .into()
}

/// Ctrl+1..3: each emits **exactly what a click on that item emits**.
fn routes() -> Vec<Route<Msg>> {
    (0..3u8)
        .map(|i| Route {
            key: Key::Character(format!("{}", i + 1).into()),
            modifiers: CTRL,
            message: Msg::Go(i),
            target: Target {
                region: SIDEBAR,
                index: usize::from(i),
            },
        })
        .collect()
}

fn page_body(view: u8) -> Element<'static, Msg> {
    match view {
        0 => column![btn("a1", Msg::A1), btn("a2", Msg::A2)]
            .spacing(8)
            .into(),
        _ => column![btn("b1", Msg::B1), btn("b2", Msg::B2)]
            .spacing(8)
            .into(),
    }
}

/// The page: sidebar and body, with the router around both, inside the page.
fn page(view: u8, order: Order, b1: B1) -> Element<'static, Msg> {
    Router::new(row![sidebar(), page_body(view)].spacing(12), routes())
        .order(order)
        .b1(b1)
        .into()
}

#[derive(Clone, Copy, Debug)]
enum Menu {
    None,
    WithBackdrop,
    WithoutBackdrop,
}

#[derive(Clone, Copy, Debug)]
struct Open {
    menu: Menu,
    dialog: bool,
    sheet: bool,
    toast: bool,
    close_modals: bool,
}

impl Open {
    const NOTHING: Self = Self {
        menu: Menu::None,
        dialog: false,
        sheet: false,
        toast: false,
        close_modals: true,
    };
}

fn window(open: Open) -> Element<'static, Msg> {
    let mut layout = AppLayout::new(page(0, Order::AfterChildren, B1::On));
    match open.menu {
        Menu::None => {}
        Menu::WithBackdrop => {
            layout = layout
                .header_menu(btn("Menu item", Msg::MenuItem))
                .on_close_menus(Msg::CloseMenus);
        }
        Menu::WithoutBackdrop => layout = layout.header_menu(btn("Menu item", Msg::MenuItem)),
    }
    if open.dialog {
        layout = layout.dialog(Dialog::new(btn("OK", Msg::DialogOk)));
    }
    if open.sheet {
        layout = layout.sheet(Sheet::new(btn("Sheet OK", Msg::SheetOk)));
    }
    if (open.dialog || open.sheet) && open.close_modals {
        layout = layout.on_close_modals(Msg::CloseModals);
    }
    if open.toast {
        layout = layout.toasts(vec![Toast::new(
            1,
            ToastIntent::Info,
            "Saved",
            "All good.",
            Msg::ToastDismiss,
        )]);
    }
    render(layout)
}

fn key(c: &str) -> Key {
    Key::Character(c.into())
}

/// Ctrl+2 into a fresh interface, and what came of it.
fn ctrl2(open: Open) -> (Vec<Msg>, Status) {
    let mut rig = Rig::new(window(open));
    rig.messages.clear();
    let status = rig.press(key("2"), CTRL);
    (rig.messages, status)
}

// ---- rows 1 and 2: silent under an overlay; captured when it answers -----------

/// **1. Silent under overlays, and 2. captured when it answers.** A router inside
/// the page's `Inert`, for each row of RFC-119's table.
#[test]
fn row_1_and_2_the_router_is_silent_under_an_overlay_and_captures_when_it_answers() {
    let rows: [(&str, Open, bool); 8] = [
        ("nothing open", Open::NOTHING, true),
        (
            "toasts only",
            Open {
                toast: true,
                ..Open::NOTHING
            },
            true,
        ),
        (
            "a menu WITHOUT a backdrop (the page stays pressable)",
            Open {
                menu: Menu::WithoutBackdrop,
                ..Open::NOTHING
            },
            true,
        ),
        (
            "a menu WITH its backdrop",
            Open {
                menu: Menu::WithBackdrop,
                ..Open::NOTHING
            },
            false,
        ),
        (
            "a dialog, closing on a click outside",
            Open {
                dialog: true,
                ..Open::NOTHING
            },
            false,
        ),
        (
            "a dialog with NO click-outside close",
            Open {
                dialog: true,
                close_modals: false,
                ..Open::NOTHING
            },
            false,
        ),
        (
            "a sheet",
            Open {
                sheet: true,
                ..Open::NOTHING
            },
            false,
        ),
        (
            "a dialog and toasts",
            Open {
                dialog: true,
                toast: true,
                ..Open::NOTHING
            },
            false,
        ),
    ];

    println!("\n## Ctrl+2 with the router inside the page's Inert\n");
    println!("| what is open | published | status | the router should answer |");
    println!("|---|---|---|---|");
    let mut off = Vec::new();
    for (name, open, answers) in rows {
        let (messages, status) = ctrl2(open);
        println!("| {name} | {messages:?} | {status:?} | {answers} |");
        let (want_messages, want_status) = if answers {
            (vec![Msg::Go(1)], Status::Captured)
        } else {
            (vec![], Status::Ignored)
        };
        if messages != want_messages || status != want_status {
            off.push(name);
        }
    }
    assert!(off.is_empty(), "rows off: {off:?}");
}

// ---- row 3: B2, precedence --------------------------------------------------------

fn parent_routes() -> Vec<Route<Msg>> {
    vec![Route {
        key: Key::Named(Named::ArrowUp),
        modifiers: Modifiers::ALT,
        message: Msg::Parent,
        target: Target {
            region: SIDEBAR,
            index: 0,
        },
    }]
}

fn leak<T>(v: T) -> &'static T {
    Box::leak(Box::new(v))
}

fn editor_page(order: Order) -> Element<'static, Msg> {
    let content: &'static text_editor::Content = leak(text_editor::Content::with_text("a\nb\nc"));
    Router::new(
        row![sidebar(), text_editor(content).on_action(|_| Msg::Edit),],
        parent_routes(),
    )
    .order(order)
    .into()
}

fn input_page(order: Order) -> Element<'static, Msg> {
    Router::new(
        row![sidebar(), text_input("", "hello").on_input(|_| Msg::Typed)],
        routes(),
    )
    .order(order)
    .into()
}

fn captures_page(order: Order) -> Element<'static, Msg> {
    Router::new(
        row![sidebar(), Captures::new(btn("inside", Msg::Page))],
        routes(),
    )
    .order(order)
    .into()
}

/// The last focus target of the page, focused: the text input, the editor, ...
fn focus_last(rig: &mut Rig<Msg>) {
    let last = rig.targets().len() - 1;
    rig.focus_at(last);
}

/// **3. B2: the router sees a key only after the focused widget has had it.**
/// Measured with the three kinds of focused content, in both orders.
#[test]
fn row_3_the_router_answers_only_what_the_focused_widget_left() {
    println!("\n## B2: a focused widget that uses the key itself wins\n");
    println!("| focused widget | key | router after the widget | router before the widget |");
    println!("|---|---|---|---|");

    type Page = fn(Order) -> Element<'static, Msg>;
    let cases: [(&str, Page, Key, Modifiers, Vec<Msg>); 3] = [
        // A one-line text input does not use Ctrl+2: the shortcut works.
        ("text_input", input_page, key("2"), CTRL, vec![Msg::Go(1)]),
        // A multi-line editor uses Alt+Up (it moves the cursor and publishes its
        // own action): it wins, and the router stays silent.
        (
            "text_editor",
            editor_page,
            Key::Named(Named::ArrowUp),
            Modifiers::ALT,
            vec![Msg::Edit],
        ),
        // A widget that captures every key: it wins.
        ("a capturing widget", captures_page, key("2"), CTRL, vec![]),
    ];
    let mut off = Vec::new();
    for (name, page, k, m, expect_after) in cases {
        let mut got = Vec::new();
        for order in [Order::AfterChildren, Order::BeforeChildren] {
            let mut rig = Rig::new(page(order));
            focus_last(&mut rig);
            rig.messages.clear();
            let status = rig.press(k.clone(), m);
            got.push((rig.messages.clone(), status));
        }
        println!(
            "| {name} | {k:?}+{m:?} | {:?} {:?} | {:?} {:?} |",
            got[0].0, got[0].1, got[1].0, got[1].1
        );
        // After: the widget's own use wins (nothing from the router), unless it
        // does not use the key (the router answers).
        let want_after = expect_after;
        if got[0].0 != want_after {
            off.push(format!(
                "{name}: after-children published {:?}, want {want_after:?}",
                got[0].0
            ));
        }
    }
    assert!(off.is_empty(), "rows off: {off:?}");
}

// ---- row 4: B1, a shortcut as a click ---------------------------------------------

/// What an application does with a view change: rebuild.
fn swap(rig: Rig<Msg>, view: u8, b1: B1) -> Rig<Msg> {
    rig.rebuilt(render(AppLayout::new(page(view, Order::AfterChildren, b1))))
}

/// **4. B1: a shortcut does what a click does, so focus is not inherited.**
/// Focus on a2 in view A, Ctrl+2, view B. With B1 nothing is focused and Tab
/// carries on from the item that was jumped to; **without it, b2 inherits the
/// focus** and one Enter presses a control nobody chose (RFC-120's finding by
/// another route).
#[test]
fn row_4_a_shortcut_does_what_a_click_does() {
    println!("\n## B1: the view swap\n");
    println!("| router | published | focused after | Enter publishes | Tab then focuses |");
    println!("|---|---|---|---|---|");

    let mut results = Vec::new();
    for b1 in [B1::On, B1::Off] {
        let mut rig = Rig::new(render(AppLayout::new(page(0, Order::AfterChildren, b1))));
        // a2 is the fifth focus target: three sidebar items, then a1, a2.
        rig.focus_at(4);
        assert_eq!(rig.focused(), vec![4], "premise: focus is on a2");
        rig.messages.clear();
        let status = rig.press(key("2"), CTRL);
        assert_eq!(status, Status::Captured, "premise: the router answered");
        let published = rig.messages.clone();
        assert_eq!(
            published,
            vec![Msg::Go(1)],
            "premise: exactly the click's message"
        );

        // The application's update: view 1.
        let mut rig = swap(rig, 1, b1);
        let focused = rig.focused();
        rig.messages.clear();
        rig.enter();
        let enter = rig.messages.clone();
        // Tab, through `traverse`, and where it lands.
        let mut again = swap(
            {
                let mut r = Rig::new(render(AppLayout::new(page(0, Order::AfterChildren, b1))));
                r.focus_at(4);
                r.press(key("2"), CTRL);
                r
            },
            1,
            b1,
        );
        again.tab(false);
        let tab = again.focused();
        println!("| {b1:?} | {published:?} | {focused:?} | {enter:?} | {tab:?} |");
        results.push((b1, focused, enter, tab));
    }

    // B1 on: nothing inherited; Tab starts from the clicked item (index 1), so
    // lands on the item after it (index 2).
    assert_eq!(
        results[0].1,
        Vec::<usize>::new(),
        "B1 on: nothing is focused"
    );
    assert!(results[0].2.is_empty(), "B1 on: Enter presses nothing");
    assert_eq!(
        results[0].3,
        vec![2],
        "B1 on: Tab carries on after the jumped-to item"
    );
    // B1 off: the control that moved into a2's place inherited the focus.
    assert_eq!(
        results[1].1,
        vec![4],
        "B1 off: b2 inherits the focus, which is the bug"
    );
    assert_eq!(
        results[1].2,
        vec![Msg::B2],
        "B1 off: Enter presses b2, which nobody chose"
    );
}

/// **A target that is not there does nothing** (a disabled control reports no
/// focus target): no message, no focus change, and the event is left `Ignored`.
#[test]
fn row_4_a_missing_target_does_nothing() {
    let routes = vec![Route {
        key: key("9"),
        modifiers: CTRL,
        message: Msg::Go(8),
        // Only three items: there is no ninth to jump to.
        target: Target {
            region: SIDEBAR,
            index: 8,
        },
    }];
    let view: Element<'static, Msg> = Router::new(row![sidebar(), page_body(0)], routes).into();
    let mut rig = Rig::new(render(AppLayout::new(view)));
    rig.focus_at(4);
    rig.messages.clear();
    let status = rig.press(key("9"), CTRL);
    assert_eq!(rig.messages, Vec::<Msg>::new(), "nothing is published");
    assert_eq!(status, Status::Ignored);
    assert_eq!(rig.focused(), vec![4], "focus did not move");
}

// ---- row 5: the Escape field ------------------------------------------------------

/// The escape field, prototyped as the engine would: `OnEscape` on the overlay
/// layer's element, so it answers while the overlay is open.
fn escape_dialog(field: bool, close_modals: bool) -> Element<'static, Msg> {
    let dialog: Element<'static, Msg> = btn("OK", Msg::DialogOk);
    let dialog = if field {
        snora_interaction::OnEscape::new(dialog, Msg::Escape).into()
    } else {
        dialog
    };
    let mut layout =
        AppLayout::new(page(0, Order::AfterChildren, B1::On)).dialog(Dialog::new(dialog));
    if close_modals {
        layout = layout.on_close_modals(Msg::CloseModals);
    }
    render(layout)
}

/// **5. The Escape field, on the overlay layer.** For a dialog with no
/// click-outside close, it answers Escape while open and captures it; with both it
/// and `on_close_modals` set, the top layer (the dialog's) answers first, so the
/// field wins and the message is published **once**.
#[test]
fn row_5_an_escape_field_answers_on_the_overlay_layer() {
    println!("\n## the Escape field\n");
    println!("| field | on_close_modals | Escape publishes | status |");
    println!("|---|---|---|---|");
    let mut rows = Vec::new();
    for (field, close) in [(true, false), (true, true), (false, true), (false, false)] {
        let mut rig = Rig::new(escape_dialog(field, close));
        rig.messages.clear();
        let status = rig.press(Key::Named(Named::Escape), Modifiers::empty());
        println!("| {field} | {close} | {:?} | {status:?} |", rig.messages);
        rows.push((field, close, rig.messages, status));
    }
    // Field alone: answers, captured. The router has no Escape route.
    assert_eq!(rows[0].2, vec![Msg::Escape]);
    assert_eq!(rows[0].3, Status::Captured);
    // Both: once, and the field's.
    assert_eq!(
        rows[1].2,
        vec![Msg::Escape],
        "both set: the field's message, once"
    );
    // Only the close message: RFC-119's rule, unchanged.
    assert_eq!(rows[2].2, vec![Msg::CloseModals]);
    // Neither: nothing, and the application's own `dismiss_on_escape` still sees it.
    assert!(rows[3].2.is_empty());
    assert_eq!(rows[3].3, Status::Ignored);
}

// ---- row 5, continued: a menu ---------------------------------------------------

fn escape_menu(field: bool, close_menus: bool) -> Element<'static, Msg> {
    let menu: Element<'static, Msg> = btn("Menu item", Msg::MenuItem);
    let menu = if field {
        snora_interaction::OnEscape::new(menu, Msg::Escape).into()
    } else {
        menu
    };
    let mut layout = AppLayout::new(page(0, Order::AfterChildren, B1::On)).header_menu(menu);
    if close_menus {
        layout = layout.on_close_menus(Msg::CloseMenus);
    }
    render(layout)
}

/// The same for a menu: the field answers on the menu's layer, whether or not it
/// has a backdrop, and **wins over the backdrop's own Escape** when both are set.
#[test]
fn row_5_an_escape_field_answers_for_a_menu_too() {
    println!("\n## the Escape field, for a menu\n");
    println!("| field | on_close_menus | Escape publishes | status |");
    println!("|---|---|---|---|");
    let mut rows = Vec::new();
    for (field, close) in [(true, false), (true, true), (false, true), (false, false)] {
        let mut rig = Rig::new(escape_menu(field, close));
        rig.messages.clear();
        let status = rig.press(Key::Named(Named::Escape), Modifiers::empty());
        println!("| {field} | {close} | {:?} | {status:?} |", rig.messages);
        rows.push((rig.messages, status));
    }
    assert_eq!(rows[0], (vec![Msg::Escape], Status::Captured));
    assert_eq!(
        rows[1],
        (vec![Msg::Escape], Status::Captured),
        "both: the field, once"
    );
    assert_eq!(rows[2], (vec![Msg::CloseMenus], Status::Captured));
    assert_eq!(rows[3], (vec![], Status::Ignored));
}

// ---- row 1, the control: where the router is placed is what silences it -----------

/// **The control for row 1, and the finding F-1 as a measurement.** The same
/// router, placed **outside** the page (as the application-side router of R-2 as
/// accepted effectively is: fed by `listen()`, which sees every key nothing
/// captured), answers **beneath an open dialog**. So the silence in row 1 comes
/// from where the router sits, not from the router.
#[test]
fn row_1_control_a_router_outside_the_page_fires_beneath_a_dialog() {
    let body: Element<'static, Msg> = row![sidebar(), page_body(0)].into();
    let layout = AppLayout::new(body).dialog(Dialog::new(btn("OK", Msg::DialogOk)));
    let outside: Element<'static, Msg> = Router::new(render(layout), routes()).b1(B1::Off).into();
    let mut rig = Rig::new(outside);
    rig.messages.clear();
    let status = rig.press(key("2"), CTRL);
    assert_eq!(
        rig.messages,
        vec![Msg::Go(1)],
        "a router outside the page answers beneath a dialog (F-1)"
    );
    assert_eq!(status, Status::Captured);
}

/// **R-2's own rule, measured and not assumed:** a shortcut emits exactly what a
/// click on that control emits. The click is a real one, on the item's centre.
#[test]
fn row_4_a_shortcut_publishes_what_a_click_publishes() {
    for item in 0..3u8 {
        let mut click = Rig::new(render(AppLayout::new(page(
            0,
            Order::AfterChildren,
            B1::On,
        ))));
        let centre = click.targets()[usize::from(item)].bounds.center();
        click.messages.clear();
        click.click_at(centre);
        let clicked = click.messages.clone();

        let mut keys = Rig::new(render(AppLayout::new(page(
            0,
            Order::AfterChildren,
            B1::On,
        ))));
        keys.messages.clear();
        keys.press(key(&format!("{}", item + 1)), CTRL);
        assert_eq!(
            clicked,
            vec![Msg::Go(item)],
            "premise: the click published it"
        );
        assert_eq!(
            keys.messages,
            clicked,
            "Ctrl+{} is not what a click publishes",
            item + 1
        );
    }
}
