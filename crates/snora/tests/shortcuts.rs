//! The shortcuts router and the Escape field (RFC-116 R-2, Unit 1).
//!
//! > **A shortcut does exactly what a click on that control does.** Every binding
//! > has a modifier. A shortcut works unless the focused control uses that key; a
//! > focused text field uses every arrow key. The digit routes match the physical
//! > digit row, named keys the logical key.
//!
//! Driven through the runtime's own `UserInterface` (`common::removal::Rig`): a key
//! press returns the event's `Status`, the testable stand-in for "an application's
//! `keyboard::listen()` never receives it" (`listen()` passes only `Ignored`). The
//! scenes are the **engine's own** (`snora::render` with `AppLayout::shortcuts`
//! and `AppLayout::escape`), so the router is placed where the engine places it.
//!
//! Unit 0's prototype measurements (`rfc116-r2-unit0`) are the reference the
//! production router must still meet; each is a test here.
//!
//! The renderer follows `ICED_TEST_BACKEND` (default tiny-skia). A wgpu run must
//! use `--test-threads=1`.

#![cfg(all(feature = "widgets", feature = "design"))]

mod common;
use common::removal::Rig;

use iced::Element;
use iced::keyboard::key::{Code, Named, Physical};
use iced::keyboard::{Key, Modifiers};
use iced::widget::{column, text_editor, text_input};
use iced_runtime::core::event::Status;

use snora::design::{Tokens, button};
use snora::keyboard::{Digits, Escape, Platform, Shortcuts};
use snora::widget::SideBarLabels;
use snora::{
    AppLayout, BreadcrumbAction, Crumb, Dialog, Icon, LayoutDirection, Sheet, SideBar, SideBarItem,
    Tab, TabAction, TabBar, Toast, ToastIntent, render,
};

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    /// What a click on side bar item *n* publishes.
    Go(u8),
    /// What a click on tab *n* publishes.
    Tab(u8),
    /// What a click on crumb *n* publishes.
    Crumb(u8),
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
    EscapeModal,
    EscapeMenu,
    Typed,
    Edit,
    Menu(snora::MenuAction<u8, u8>),
}

static ON_TAB: fn(TabAction<u8>) -> Msg = |a| {
    let TabAction::Pressed(i) = a;
    Msg::Tab(i)
};
static ON_CRUMB: fn(BreadcrumbAction<u8>) -> Msg = |a| {
    let BreadcrumbAction::Pressed(i) = a;
    Msg::Crumb(i)
};

fn tokens() -> &'static Tokens {
    use std::sync::OnceLock;
    static T: OnceLock<Tokens> = OnceLock::new();
    T.get_or_init(Tokens::light)
}

fn btn(label: &str, msg: Msg) -> Element<'static, Msg> {
    button::primary(tokens(), label.to_owned(), msg)
}

fn leak<T>(v: T) -> &'static T {
    Box::leak(Box::new(v))
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Content {
    /// Two buttons whose names say which view they belong to.
    Buttons,
    TextInput,
    TextEditor,
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

#[derive(Clone)]
struct Scene {
    digits: Option<Digits>,
    platform: Platform,
    direction: LayoutDirection,
    /// The tab bar is drawn, with this many tabs; the router is told about
    /// `tabs_in_shortcuts` of them.
    tabs_drawn: usize,
    tabs_in_shortcuts: usize,
    active_tab: u8,
    crumbs: bool,
    /// The router is told about the crumbs (`crumbs` draws them).
    crumbs_routed: bool,
    side_bar: bool,
    view: u8,
    content: Content,
    open: Open,
    escape: Escape<Msg>,
    /// Hand the layout a router at all.
    shortcuts: bool,
}

impl Scene {
    fn tabs() -> Self {
        Self {
            digits: Some(Digits::Tabs),
            platform: Platform::Other,
            direction: LayoutDirection::Ltr,
            tabs_drawn: 4,
            tabs_in_shortcuts: 4,
            active_tab: 0,
            crumbs: true,
            crumbs_routed: true,
            side_bar: false,
            view: 0,
            content: Content::Buttons,
            open: Open::NOTHING,
            escape: Escape::default(),
            shortcuts: true,
        }
    }

    fn side_bar() -> Self {
        Self {
            digits: Some(Digits::SideBar),
            tabs_drawn: 0,
            tabs_in_shortcuts: 0,
            crumbs: false,
            side_bar: true,
            ..Self::tabs()
        }
    }
}

fn side_bar_data() -> SideBar<Msg, u8> {
    SideBar {
        items: (0..3u8)
            .map(|i| SideBarItem {
                view_id: i,
                icon: Icon::Text(format!("{}", i + 1)),
                tooltip: format!("Item {i}"),
                on_press: Msg::Go(i),
            })
            .collect(),
        active: 0,
    }
}

fn tab_bar_data(count: usize, active: u8) -> TabBar<u8> {
    TabBar {
        tabs: (0..count as u8)
            .map(|i| Tab {
                id: i,
                label: format!("T{i}"),
                icon: None,
            })
            .collect(),
        active,
    }
}

fn crumb_data() -> Vec<Crumb<u8>> {
    vec![
        Crumb::ancestor(0, "Root"),
        Crumb::ancestor(1, "Mid"),
        Crumb::leaf(2, "Here"),
    ]
}

fn window(sc: &Scene) -> Element<'static, Msg> {
    let mut body = column![].spacing(8);
    if sc.tabs_drawn > 0 {
        body = body.push(snora::widget::app_tab_bar(
            tab_bar_data(sc.tabs_drawn, sc.active_tab),
            &ON_TAB,
            sc.direction,
        ));
    }
    if sc.crumbs {
        body = body.push(snora::widget::app_breadcrumb(
            crumb_data(),
            &ON_CRUMB,
            sc.direction,
        ));
    }
    let content: Element<'static, Msg> = match sc.content {
        Content::Buttons if sc.view == 0 => column![btn("a1", Msg::A1), btn("a2", Msg::A2)]
            .spacing(8)
            .into(),
        Content::Buttons => column![btn("b1", Msg::B1), btn("b2", Msg::B2)]
            .spacing(8)
            .into(),
        Content::TextInput => text_input("", "hello").on_input(|_| Msg::Typed).into(),
        Content::TextEditor => {
            let content = leak(text_editor::Content::with_text("a\nb\nc"));
            text_editor(content).on_action(|_| Msg::Edit).into()
        }
    };
    body = body.push(content);
    let body: Element<'static, Msg> = body.into();

    let mut layout = AppLayout::new(body)
        .direction(sc.direction)
        .escape(sc.escape.clone());

    if sc.side_bar {
        layout = layout.side_bar(snora::widget::app_side_bar_with(
            side_bar_data(),
            sc.direction,
            SideBarLabels::Beneath,
        ));
    }
    if sc.shortcuts {
        let mut shortcuts = Shortcuts::new().platform(sc.platform);
        if sc.side_bar {
            shortcuts = shortcuts.side_bar(&side_bar_data());
        }
        if sc.tabs_in_shortcuts > 0 {
            shortcuts = shortcuts.tabs(&tab_bar_data(sc.tabs_in_shortcuts, sc.active_tab), &ON_TAB);
        }
        if sc.crumbs && sc.crumbs_routed {
            shortcuts = shortcuts.crumbs(&crumb_data(), &ON_CRUMB);
        }
        if let Some(digits) = sc.digits {
            shortcuts = shortcuts.digits(digits);
        }
        layout = layout.shortcuts(shortcuts);
    }

    match sc.open.menu {
        Menu::None => {}
        Menu::WithBackdrop => {
            layout = layout
                .header_menu(btn("Menu item", Msg::MenuItem))
                .on_close_menus(Msg::CloseMenus);
        }
        Menu::WithoutBackdrop => layout = layout.header_menu(btn("Menu item", Msg::MenuItem)),
    }
    if sc.open.dialog {
        layout = layout.dialog(Dialog::new(btn("OK", Msg::DialogOk)));
    }
    if sc.open.sheet {
        layout = layout.sheet(Sheet::new(btn("Sheet OK", Msg::SheetOk)));
    }
    if (sc.open.dialog || sc.open.sheet) && sc.open.close_modals {
        layout = layout.on_close_modals(Msg::CloseModals);
    }
    if sc.open.toast {
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

// ---- keys ------------------------------------------------------------------------

fn digit_code(n: u8) -> Code {
    [
        Code::Digit1,
        Code::Digit2,
        Code::Digit3,
        Code::Digit4,
        Code::Digit5,
        Code::Digit6,
        Code::Digit7,
        Code::Digit8,
        Code::Digit9,
    ][usize::from(n) - 1]
}

/// The digit key as a keyboard reports it (QWERTY: logical and physical agree).
fn digit(rig: &mut Rig<Msg>, n: u8, modifiers: Modifiers) -> Status {
    rig.messages.clear();
    rig.press_on(
        Key::Character(n.to_string().into()),
        Physical::Code(digit_code(n)),
        modifiers,
    )
}

fn named(rig: &mut Rig<Msg>, key: Named, modifiers: Modifiers) -> Status {
    rig.messages.clear();
    rig.press(Key::Named(key), modifiers)
}

/// A key on its physical position, with the logical character the layout makes of it.
fn physical(rig: &mut Rig<Msg>, logical: &str, code: Code, modifiers: Modifiers) -> Status {
    rig.messages.clear();
    rig.press_on(
        Key::Character(logical.into()),
        Physical::Code(code),
        modifiers,
    )
}

const CTRL: Modifiers = Modifiers::CTRL;

// ---- silent under overlays; captured when it answers ----------------------------------

/// **Silent under every overlay that blocks the pointer, and answers beneath a
/// menu with no backdrop; captured when it answers.** RFC-119's table.
#[test]
fn the_router_is_silent_under_an_overlay_and_captures_when_it_answers() {
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
            "a dialog",
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
    let mut off = Vec::new();
    for (name, open, answers) in rows {
        let mut rig = Rig::new(window(&Scene {
            open,
            ..Scene::tabs()
        }));
        let status = digit(&mut rig, 2, CTRL);
        let (want_messages, want_status) = if answers {
            (vec![Msg::Tab(1)], Status::Captured)
        } else {
            (vec![], Status::Ignored)
        };
        if rig.messages != want_messages || status != want_status {
            off.push(format!("{name}: {:?} {status:?}", rig.messages));
        }
    }
    assert!(off.is_empty(), "rows off: {off:?}");
}

// ---- a shortcut does what a click does ------------------------------------------------

/// **The same message a real click publishes**, for a route of each kind: a tab by
/// digit, next and previous tab, a side bar item, the parent crumb.
#[test]
fn a_shortcut_publishes_what_a_click_publishes() {
    // (scene, target focus index in Tab order, how to press the shortcut)
    let click = |sc: &Scene, at: usize| {
        let mut rig = Rig::new(window(sc));
        let centre = rig.targets()[at].bounds.center();
        rig.messages.clear();
        rig.click_at(centre);
        rig.messages
    };
    let tabs = Scene::tabs();
    // Tab order: tabs 0..3, then crumbs 0 and 1, then the content.
    for (n, want_target) in [(1u8, 0usize), (2, 1), (4, 3)] {
        let mut rig = Rig::new(window(&tabs));
        digit(&mut rig, n, CTRL);
        assert_eq!(
            rig.messages,
            click(&tabs, want_target),
            "Ctrl+{n} is not a click on tab {want_target}"
        );
    }
    // Next tab: from tab 0 it is tab 1. Previous wraps to the last.
    let mut rig = Rig::new(window(&tabs));
    named(&mut rig, Named::Tab, CTRL);
    assert_eq!(
        rig.messages,
        click(&tabs, 1),
        "Ctrl+Tab is not a click on tab 1"
    );
    let mut rig = Rig::new(window(&tabs));
    named(&mut rig, Named::Tab, CTRL | Modifiers::SHIFT);
    assert_eq!(
        rig.messages,
        click(&tabs, 3),
        "Ctrl+Shift+Tab is not a click on tab 3"
    );
    // The parent crumb: crumb 1 (index 5 after the four tabs and crumb 0).
    let mut rig = Rig::new(window(&tabs));
    named(&mut rig, Named::ArrowUp, Modifiers::ALT);
    assert_eq!(
        rig.messages,
        click(&tabs, 5),
        "Alt+↑ is not a click on the parent crumb"
    );
    // A side bar item.
    let side = Scene::side_bar();
    let mut rig = Rig::new(window(&side));
    digit(&mut rig, 3, CTRL);
    assert_eq!(
        rig.messages,
        click(&side, 2),
        "Ctrl+3 is not a click on item 2"
    );
}

/// **Focus is taken away and Tab starts from the target** (the view swap): focus on
/// `a2` in view A, `Ctrl+2`, view B. Nothing is focused, Enter presses nothing, and
/// Tab lands on the item after the one jumped to. (Without it, `b2` inherits the
/// focus and one Enter presses a control nobody chose: RFC-120's finding by
/// another route.)
#[test]
fn a_shortcut_takes_focus_away_and_tab_carries_on_from_the_target() {
    let mut rig = Rig::new(window(&Scene::side_bar()));
    // Three side bar items, then a1 and a2.
    rig.focus_at(4);
    assert_eq!(rig.focused(), vec![4], "premise: focus is on a2");
    digit(&mut rig, 2, CTRL);
    assert_eq!(rig.messages, vec![Msg::Go(1)], "premise: a click's message");

    let mut rig = rig.rebuilt(window(&Scene {
        view: 1,
        ..Scene::side_bar()
    }));
    assert_eq!(rig.focused(), Vec::<usize>::new(), "nothing is focused");
    rig.messages.clear();
    rig.enter();
    assert!(rig.messages.is_empty(), "Enter pressed {:?}", rig.messages);
    rig.tab(false);
    assert_eq!(
        rig.focused(),
        vec![2],
        "Tab goes to the item after the target"
    );
}

/// **A disabled or absent target does nothing**: no message, no focus change, and
/// the event is left `Ignored`. The router is told about four tabs and only two are
/// there.
#[test]
fn an_absent_target_does_nothing() {
    let sc = Scene {
        tabs_drawn: 2,
        tabs_in_shortcuts: 4,
        ..Scene::tabs()
    };
    let mut rig = Rig::new(window(&sc));
    let last = rig.targets().len() - 1;
    rig.focus_at(last);
    let status = digit(&mut rig, 4, CTRL);
    assert!(rig.messages.is_empty(), "published {:?}", rig.messages);
    assert_eq!(status, Status::Ignored);
    assert_eq!(rig.focused(), vec![last], "focus did not move");
}

/// **No target in this view: nothing.** Digits are set to tabs, in a view with no
/// tab bar.
#[test]
fn digits_set_to_tabs_do_nothing_in_a_view_without_tabs() {
    let sc = Scene {
        tabs_drawn: 0,
        tabs_in_shortcuts: 0,
        ..Scene::tabs()
    };
    let mut rig = Rig::new(window(&sc));
    for n in 1..=9 {
        let status = digit(&mut rig, n, CTRL);
        assert!(
            rig.messages.is_empty(),
            "Ctrl+{n} published {:?}",
            rig.messages
        );
        assert_eq!(status, Status::Ignored);
    }
}

// ---- a focused control that uses the key wins -----------------------------------------

/// **A focused control that uses the key wins**: a `text_editor` takes `Alt+↑`, the
/// router stays silent (the editor's own action is all that is published).
#[test]
fn a_focused_text_editor_keeps_alt_up() {
    let mut rig = Rig::new(window(&Scene {
        content: Content::TextEditor,
        ..Scene::tabs()
    }));
    let last = rig.targets().len() - 1;
    rig.focus_at(last);
    named(&mut rig, Named::ArrowUp, Modifiers::ALT);
    assert_eq!(
        rig.messages,
        vec![Msg::Edit],
        "the router answered over the editor"
    );
}

/// **A focused text field takes every arrow key, with any modifier.** `Alt+↑` on
/// Windows and Linux, `Cmd+↑` on macOS (where it means "start of text"): no parent
/// route fires.
#[test]
fn a_focused_text_field_takes_every_arrow_key() {
    for (platform, modifiers) in [
        (Platform::Other, Modifiers::ALT),
        (Platform::MacOs, Modifiers::LOGO),
    ] {
        let sc = Scene {
            content: Content::TextInput,
            platform,
            ..Scene::tabs()
        };
        let mut rig = Rig::new(window(&sc));
        let last = rig.targets().len() - 1;
        rig.focus_at(last);
        named(&mut rig, Named::ArrowUp, modifiers);
        assert!(
            !rig.messages.contains(&Msg::Crumb(1)),
            "{platform:?}: {modifiers:?}+↑ in a text field took the reader to the parent: {:?}",
            rig.messages
        );
        // And with nothing focused, the same key does go to the parent (the
        // premise: the route is there).
        let mut rig = Rig::new(window(&sc));
        named(&mut rig, Named::ArrowUp, modifiers);
        assert_eq!(rig.messages, vec![Msg::Crumb(1)], "{platform:?}: premise");
    }
}

// ---- which key matches how -------------------------------------------------------------

/// **Digits match the physical digit row.** An AZERTY-shaped event (logical `&`,
/// physical `Digit1`, Ctrl) reaches tab 1.
#[test]
fn a_digit_matches_the_physical_key_not_the_logical_one() {
    let mut rig = Rig::new(window(&Scene::tabs()));
    let status = physical(&mut rig, "&", Code::Digit1, CTRL);
    assert_eq!(
        rig.messages,
        vec![Msg::Tab(0)],
        "the AZERTY 1 key did not reach tab 1"
    );
    assert_eq!(status, Status::Captured);
}

/// **Named keys match the logical key**, and a binding needs its modifier exactly.
#[test]
fn named_keys_match_the_logical_key() {
    let tabs = Scene {
        active_tab: 1,
        ..Scene::tabs()
    };
    let mut rig = Rig::new(window(&tabs));
    named(&mut rig, Named::Tab, CTRL);
    assert_eq!(rig.messages, vec![Msg::Tab(2)], "Ctrl+Tab");
    named(&mut rig, Named::PageDown, CTRL);
    assert_eq!(rig.messages, vec![Msg::Tab(2)], "Ctrl+PageDown");
    named(&mut rig, Named::PageUp, CTRL);
    assert_eq!(rig.messages, vec![Msg::Tab(0)], "Ctrl+PageUp");
    named(&mut rig, Named::Tab, CTRL | Modifiers::SHIFT);
    assert_eq!(rig.messages, vec![Msg::Tab(0)], "Ctrl+Shift+Tab");
    named(&mut rig, Named::ArrowUp, Modifiers::ALT);
    assert_eq!(rig.messages, vec![Msg::Crumb(1)], "Alt+↑");
}

/// **Every binding has a modifier.** A plain `1`, an arrow or Tab fires no route; nor
/// does a binding with the wrong modifier.
#[test]
fn a_key_without_its_modifier_fires_no_route() {
    let mut rig = Rig::new(window(&Scene::tabs()));
    for (name, status) in [
        ("1", digit(&mut rig, 1, Modifiers::empty())),
        ("Tab", named(&mut rig, Named::Tab, Modifiers::empty())),
        ("↑", named(&mut rig, Named::ArrowUp, Modifiers::empty())),
        ("Shift+1", digit(&mut rig, 1, Modifiers::SHIFT)),
        ("Alt+1", digit(&mut rig, 1, Modifiers::ALT)),
        (
            "Ctrl+Alt+↑",
            named(&mut rig, Named::ArrowUp, CTRL | Modifiers::ALT),
        ),
    ] {
        assert!(
            rig.messages.is_empty(),
            "{name} published {:?}",
            rig.messages
        );
        assert_eq!(status, Status::Ignored, "{name}");
    }
}

/// **`9` is the last; a digit beyond the count does nothing.** Four tabs.
#[test]
fn nine_is_the_last_and_a_digit_beyond_the_count_does_nothing() {
    let mut rig = Rig::new(window(&Scene::tabs()));
    digit(&mut rig, 9, CTRL);
    assert_eq!(rig.messages, vec![Msg::Tab(3)], "Ctrl+9 is the last tab");
    for n in 5..=8 {
        let status = digit(&mut rig, n, CTRL);
        assert!(
            rig.messages.is_empty(),
            "Ctrl+{n} published {:?}",
            rig.messages
        );
        assert_eq!(status, Status::Ignored);
    }
    digit(&mut rig, 4, CTRL);
    assert_eq!(rig.messages, vec![Msg::Tab(3)], "Ctrl+4 is the fourth");
}

/// **No `.digits(…)`, no digit route**; and the digits go to one destination, so with
/// both a side bar and tabs the other is not touched.
#[test]
fn the_digits_go_to_the_destination_the_application_chose() {
    let none = Scene {
        digits: None,
        ..Scene::tabs()
    };
    let mut rig = Rig::new(window(&none));
    assert_eq!(digit(&mut rig, 2, CTRL), Status::Ignored);
    assert!(rig.messages.is_empty());

    // Both widgets drawn and told; the digits to the side bar.
    let both = |digits| Scene {
        digits: Some(digits),
        side_bar: true,
        ..Scene::tabs()
    };
    let mut rig = Rig::new(window(&both(Digits::SideBar)));
    digit(&mut rig, 2, CTRL);
    assert_eq!(rig.messages, vec![Msg::Go(1)], "Digits::SideBar");
    let mut rig = Rig::new(window(&both(Digits::Tabs)));
    digit(&mut rig, 2, CTRL);
    assert_eq!(rig.messages, vec![Msg::Tab(1)], "Digits::Tabs");
}

// ---- RTL ---------------------------------------------------------------------------------

/// The tab ids in the order Tab visits them.
fn tab_order(sc: &Scene) -> Vec<u8> {
    let mut order = Vec::new();
    for k in 0..sc.tabs_drawn {
        let mut rig = Rig::new(window(sc));
        for _ in 0..=k {
            rig.tab(false);
        }
        rig.messages.clear();
        rig.enter();
        if let Some(Msg::Tab(i)) = rig.messages.first() {
            order.push(*i);
        }
    }
    order
}

/// **RTL: the tab row is mirrored, and next tab is the one Tab calls next.**
#[test]
fn right_to_left_next_tab_follows_tab_order() {
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl] {
        let sc = Scene {
            direction,
            active_tab: 1,
            ..Scene::tabs()
        };
        let order = tab_order(&sc);
        assert_eq!(order.len(), 4, "premise: four tabs in Tab order {order:?}");
        let at = order.iter().position(|t| *t == 1).unwrap();
        let next = order[(at + 1) % 4];
        let previous = order[(at + 3) % 4];

        let mut rig = Rig::new(window(&sc));
        named(&mut rig, Named::Tab, CTRL);
        assert_eq!(
            rig.messages,
            vec![Msg::Tab(next)],
            "{direction:?}: Ctrl+Tab"
        );
        named(&mut rig, Named::Tab, CTRL | Modifiers::SHIFT);
        assert_eq!(
            rig.messages,
            vec![Msg::Tab(previous)],
            "{direction:?}: Ctrl+Shift+Tab"
        );
        // A tab by digit is the declared one, whichever way the row is mirrored.
        digit(&mut rig, 3, CTRL);
        assert_eq!(rig.messages, vec![Msg::Tab(2)], "{direction:?}: Ctrl+3");
        // The parent does not depend on direction.
        named(&mut rig, Named::ArrowUp, Modifiers::ALT);
        assert_eq!(rig.messages, vec![Msg::Crumb(1)], "{direction:?}: Alt+↑");
    }
    // And the two directions really differ, so this can fail.
    let ltr = tab_order(&Scene {
        active_tab: 1,
        ..Scene::tabs()
    });
    let rtl = tab_order(&Scene {
        direction: LayoutDirection::Rtl,
        active_tab: 1,
        ..Scene::tabs()
    });
    assert_ne!(ltr, rtl, "the mirrored row has a different Tab order");
}

// ---- both platform tables --------------------------------------------------------------------

/// **Both platform tables.** Windows and Linux: Ctrl, and Ctrl+PageDown/PageUp, Alt+↑.
/// macOS: Cmd for the digits, Control+Tab and Shift+Cmd+]/[ for tabs, Cmd+↑ for the
/// parent. Each table answers its own keys and not the other's.
#[test]
fn each_platform_answers_its_own_table() {
    let other = Scene {
        active_tab: 1,
        platform: Platform::Other,
        ..Scene::tabs()
    };
    let mac = Scene {
        active_tab: 1,
        platform: Platform::MacOs,
        ..Scene::tabs()
    };
    let cmd = Modifiers::LOGO;

    let mut rig = Rig::new(window(&other));
    digit(&mut rig, 3, CTRL);
    assert_eq!(rig.messages, vec![Msg::Tab(2)], "Windows/Linux: Ctrl+3");
    digit(&mut rig, 3, cmd);
    assert!(rig.messages.is_empty(), "Windows/Linux: Cmd+3 is not bound");
    named(&mut rig, Named::PageDown, CTRL);
    assert_eq!(
        rig.messages,
        vec![Msg::Tab(2)],
        "Windows/Linux: Ctrl+PageDown"
    );
    named(&mut rig, Named::PageUp, CTRL);
    assert_eq!(
        rig.messages,
        vec![Msg::Tab(0)],
        "Windows/Linux: Ctrl+PageUp"
    );
    named(&mut rig, Named::ArrowUp, Modifiers::ALT);
    assert_eq!(rig.messages, vec![Msg::Crumb(1)], "Windows/Linux: Alt+↑");
    named(&mut rig, Named::ArrowUp, cmd);
    assert!(rig.messages.is_empty(), "Windows/Linux: Cmd+↑ is not bound");

    let mut rig = Rig::new(window(&mac));
    digit(&mut rig, 3, cmd);
    assert_eq!(rig.messages, vec![Msg::Tab(2)], "macOS: Cmd+3");
    digit(&mut rig, 3, CTRL);
    assert!(rig.messages.is_empty(), "macOS: Ctrl+3 is not bound");
    named(&mut rig, Named::Tab, CTRL);
    assert_eq!(rig.messages, vec![Msg::Tab(2)], "macOS: Control+Tab");
    named(&mut rig, Named::Tab, CTRL | Modifiers::SHIFT);
    assert_eq!(rig.messages, vec![Msg::Tab(0)], "macOS: Control+Shift+Tab");
    physical(&mut rig, "}", Code::BracketRight, cmd | Modifiers::SHIFT);
    assert_eq!(rig.messages, vec![Msg::Tab(2)], "macOS: Shift+Cmd+]");
    physical(&mut rig, "{", Code::BracketLeft, cmd | Modifiers::SHIFT);
    assert_eq!(rig.messages, vec![Msg::Tab(0)], "macOS: Shift+Cmd+[");
    named(&mut rig, Named::ArrowUp, cmd);
    assert_eq!(rig.messages, vec![Msg::Crumb(1)], "macOS: Cmd+↑");
    named(&mut rig, Named::ArrowUp, Modifiers::ALT);
    assert!(rig.messages.is_empty(), "macOS: Alt+↑ is not bound");
    named(&mut rig, Named::PageDown, CTRL);
    assert!(rig.messages.is_empty(), "macOS: Ctrl+PageDown is not bound");
}

// ---- the Escape field --------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum Overlay {
    Dialog,
    Sheet,
    Menu,
}

fn escape_scene(overlay: Overlay, field: bool, click_outside: bool) -> Scene {
    let escape = Escape {
        modal: field.then_some(Msg::EscapeModal),
        menu: field.then_some(Msg::EscapeMenu),
    };
    let open = match overlay {
        Overlay::Dialog => Open {
            dialog: true,
            close_modals: click_outside,
            ..Open::NOTHING
        },
        Overlay::Sheet => Open {
            sheet: true,
            close_modals: click_outside,
            ..Open::NOTHING
        },
        Overlay::Menu => Open {
            menu: if click_outside {
                Menu::WithBackdrop
            } else {
                Menu::WithoutBackdrop
            },
            ..Open::NOTHING
        },
    };
    Scene {
        escape,
        open,
        shortcuts: false,
        ..Scene::tabs()
    }
}

/// **The Escape field, on the overlay's own layer.** Escape publishes the overlay's
/// escape message if one is set, and otherwise its click-outside message; with
/// neither it publishes nothing and is left `Ignored` for the application. Four rows
/// for a dialog, a sheet and a menu.
#[test]
fn the_escape_field_is_answered_on_the_overlay_layer() {
    let mut off = Vec::new();
    for overlay in [Overlay::Dialog, Overlay::Sheet, Overlay::Menu] {
        let (field_message, close_message) = match overlay {
            Overlay::Menu => (Msg::EscapeMenu, Msg::CloseMenus),
            _ => (Msg::EscapeModal, Msg::CloseModals),
        };
        let rows: [(bool, bool, Vec<Msg>, Status); 4] = [
            // The field alone: it answers, and captures.
            (true, false, vec![field_message.clone()], Status::Captured),
            // Both: the field's message, once.
            (true, true, vec![field_message.clone()], Status::Captured),
            // Only the click-outside message: RFC-119, unchanged.
            (false, true, vec![close_message], Status::Captured),
            // Neither: nothing, and the application's own handling still sees it.
            (false, false, vec![], Status::Ignored),
        ];
        for (field, click_outside, want_messages, want_status) in rows {
            let mut rig = Rig::new(window(&escape_scene(overlay, field, click_outside)));
            rig.messages.clear();
            let status = rig.press(Key::Named(Named::Escape), Modifiers::empty());
            if rig.messages != want_messages || status != want_status {
                off.push(format!(
                    "{overlay:?} field={field} click_outside={click_outside}: {:?} {status:?}",
                    rig.messages
                ));
            }
        }
    }
    assert!(off.is_empty(), "rows off: {off:?}");
}

/// **A menu covered by a modal is silent** (its layer is inside the `Inert` a modal
/// switches on): Escape goes to the modal, once.
#[test]
fn escape_goes_to_the_modal_and_not_the_menu_it_covers() {
    let escape = Escape {
        modal: Some(Msg::EscapeModal),
        menu: Some(Msg::EscapeMenu),
    };
    let sc = Scene {
        escape,
        open: Open {
            menu: Menu::WithoutBackdrop,
            dialog: true,
            ..Open::NOTHING
        },
        shortcuts: false,
        ..Scene::tabs()
    };
    let mut rig = Rig::new(window(&sc));
    rig.messages.clear();
    rig.press(Key::Named(Named::Escape), Modifiers::empty());
    assert_eq!(rig.messages, vec![Msg::EscapeModal]);
}

// ---- the 0.62 audit, Unit 3: what the router guides say, row by row -----------------------------

fn press_key(rig: &mut Rig<Msg>, key: Named, modifiers: Modifiers) -> Status {
    named(rig, key, modifiers)
}

/// **K-011, R-059: next and previous tab wrap, both ways, on both key pairs.** From the last tab,
/// `Ctrl+Tab` and `Ctrl+PageDown` go to the first; from the first, `Ctrl+Shift+Tab` and
/// `Ctrl+PageUp` go to the last; and in the middle they step. (The first-to-last half of
/// `Ctrl+Shift+Tab` is also in `a_shortcut_publishes_what_a_click_publishes`.)
#[test]
fn next_and_previous_tab_wrap_from_the_ends_and_step_in_the_middle() {
    let at = |active_tab: u8| Scene {
        active_tab,
        ..Scene::tabs()
    };
    for (what, active, key, mods, want) in [
        ("Ctrl+Tab from the last", 3, Named::Tab, CTRL, 0),
        ("Ctrl+PageDown from the last", 3, Named::PageDown, CTRL, 0),
        (
            "Ctrl+Shift+Tab from the first",
            0,
            Named::Tab,
            CTRL | Modifiers::SHIFT,
            3,
        ),
        ("Ctrl+PageUp from the first", 0, Named::PageUp, CTRL, 3),
        ("Ctrl+Tab in the middle", 1, Named::Tab, CTRL, 2),
        ("Ctrl+PageDown in the middle", 1, Named::PageDown, CTRL, 2),
        (
            "Ctrl+Shift+Tab in the middle",
            2,
            Named::Tab,
            CTRL | Modifiers::SHIFT,
            1,
        ),
        ("Ctrl+PageUp in the middle", 2, Named::PageUp, CTRL, 1),
    ] {
        let mut rig = Rig::new(window(&at(active)));
        let status = press_key(&mut rig, key, mods);
        assert_eq!(rig.messages, vec![Msg::Tab(want)], "{what}");
        assert_eq!(status, Status::Captured, "{what}");
    }
}

/// **K-003, R-045: a route the builder does not name answers nothing.** Without `.digits(..)` no
/// Ctrl+digit key does anything (all nine, `Ignored`); without `.tabs(..)` neither tab key pair
/// moves a tab while the tab bar is drawn; without `.crumbs(..)` `Alt+↑` does nothing while the
/// trail is drawn and the other routes are named.
#[test]
fn a_route_the_builder_does_not_name_answers_nothing() {
    let none = Scene {
        digits: None,
        ..Scene::tabs()
    };
    for n in 1..=9u8 {
        let mut rig = Rig::new(window(&none));
        let status = digit(&mut rig, n, CTRL);
        assert!(
            rig.messages.is_empty() && status == Status::Ignored,
            "no .digits: Ctrl+{n}: {:?} {status:?}",
            rig.messages
        );
    }

    let no_tabs = Scene {
        tabs_in_shortcuts: 0,
        ..Scene::tabs()
    };
    for (key, mods) in [
        (Named::Tab, CTRL),
        (Named::Tab, CTRL | Modifiers::SHIFT),
        (Named::PageDown, CTRL),
        (Named::PageUp, CTRL),
    ] {
        let mut rig = Rig::new(window(&no_tabs));
        let status = press_key(&mut rig, key, mods);
        assert!(
            rig.messages.is_empty() && status == Status::Ignored,
            "no .tabs: {key:?}+{mods:?}: {:?} {status:?}",
            rig.messages
        );
    }

    // The premise for the crumbs: with them named, Alt+Up goes to the ancestor before the leaf.
    let mut rig = Rig::new(window(&Scene::tabs()));
    press_key(&mut rig, Named::ArrowUp, Modifiers::ALT);
    assert_eq!(rig.messages, vec![Msg::Crumb(1)], "the premise");
    let no_crumbs = Scene {
        crumbs_routed: false,
        ..Scene::tabs()
    };
    let mut rig = Rig::new(window(&no_crumbs));
    let status = press_key(&mut rig, Named::ArrowUp, Modifiers::ALT);
    assert!(
        rig.messages.is_empty() && status == Status::Ignored,
        "no .crumbs: Alt+Up: {:?} {status:?}",
        rig.messages
    );
}

/// **R-067: a trail with no ancestor has no parent route.** A single crumb, the leaf, is drawn and
/// named: `Alt+↑` does nothing.
#[test]
fn a_trail_with_no_ancestor_has_no_parent_route() {
    let view = |crumbs: Vec<Crumb<u8>>| {
        let body: Element<'static, Msg> =
            snora::widget::app_breadcrumb(crumbs.clone(), &ON_CRUMB, LayoutDirection::Ltr);
        render(
            AppLayout::new(body).shortcuts(
                Shortcuts::new()
                    .platform(Platform::Other)
                    .crumbs(&crumbs, &ON_CRUMB),
            ),
        )
    };
    let mut rig = Rig::new(view(crumb_data()));
    press_key(&mut rig, Named::ArrowUp, Modifiers::ALT);
    assert_eq!(
        rig.messages,
        vec![Msg::Crumb(1)],
        "the premise: with ancestors it routes"
    );

    let mut rig = Rig::new(view(vec![Crumb::leaf(0, "Only")]));
    let status = press_key(&mut rig, Named::ArrowUp, Modifiers::ALT);
    assert!(
        rig.messages.is_empty() && status == Status::Ignored,
        "a leaf only: {:?} {status:?}",
        rig.messages
    );
}

/// **R-071: with fewer than two tabs there is nothing to go to.** One tab: none of the four tab keys
/// publishes, and each is `Ignored`.
#[test]
fn with_fewer_than_two_tabs_next_and_previous_do_nothing() {
    for count in [1usize, 0] {
        let sc = Scene {
            tabs_drawn: count,
            tabs_in_shortcuts: count,
            ..Scene::tabs()
        };
        for (key, mods) in [
            (Named::Tab, CTRL),
            (Named::Tab, CTRL | Modifiers::SHIFT),
            (Named::PageDown, CTRL),
            (Named::PageUp, CTRL),
        ] {
            let mut rig = Rig::new(window(&sc));
            let status = press_key(&mut rig, key, mods);
            assert!(
                rig.messages.is_empty() && status == Status::Ignored,
                "{count} tab(s): {key:?}+{mods:?}: {:?} {status:?}",
                rig.messages
            );
        }
    }
}

/// **R-048: a disabled target does nothing.** A route names a tab; the control that is that tab is
/// there but has no handler (disabled), so it reports no tag and cannot be found. Ctrl+2 publishes
/// nothing and is `Ignored`. With the handler, the same scene publishes (the premise).
#[test]
fn a_disabled_target_does_nothing() {
    use snora_interaction::{Control, Ring, Role};
    let view = |enabled: bool| {
        let mut controls = column![].spacing(8);
        for i in 0..4u8 {
            let c = Control::new(
                iced::widget::text(format!("T{i}")),
                Box::new(iced::widget::button::primary),
                Ring {
                    color: Box::new(|_| iced::Color::BLACK),
                    width: 2.0,
                    offset: 2.0,
                },
            )
            .tag(Role::Tab, usize::from(i));
            let c = if i != 1 || enabled {
                c.on_press(Msg::Tab(i))
            } else {
                c
            };
            controls = controls.push(c);
        }
        let body: Element<'static, Msg> = controls.into();
        render(
            AppLayout::new(body).shortcuts(
                Shortcuts::new()
                    .platform(Platform::Other)
                    .digits(Digits::Tabs)
                    .tabs(&tab_bar_data(4, 0), &ON_TAB),
            ),
        )
    };
    let mut rig = Rig::new(view(true));
    digit(&mut rig, 2, CTRL);
    assert_eq!(
        rig.messages,
        vec![Msg::Tab(1)],
        "the premise: enabled, it answers"
    );

    let mut rig = Rig::new(view(false));
    let status = digit(&mut rig, 2, CTRL);
    assert!(
        rig.messages.is_empty() && status == Status::Ignored,
        "a disabled target: {:?} {status:?}",
        rig.messages
    );
    // And its neighbours still answer.
    digit(&mut rig, 3, CTRL);
    assert_eq!(rig.messages, vec![Msg::Tab(2)]);
}

/// **R-005: snora installs no other key binding.** On an unfocused page with the router and the
/// Escape field set, F6, Shift+F6, Tab, Shift+Tab, Enter, Space and an arrow are `Ignored`, publish
/// nothing and focus nothing.
#[test]
fn the_engine_answers_no_key_that_was_not_routed() {
    let mut rig = Rig::new(window(&Scene {
        escape: Escape {
            modal: Some(Msg::EscapeModal),
            menu: Some(Msg::EscapeMenu),
        },
        ..Scene::tabs()
    }));
    let mut off = Vec::new();
    for (key, mods) in [
        (Named::F6, Modifiers::empty()),
        (Named::F6, Modifiers::SHIFT),
        (Named::Tab, Modifiers::empty()),
        (Named::Tab, Modifiers::SHIFT),
        (Named::Enter, Modifiers::empty()),
        (Named::Space, Modifiers::empty()),
        (Named::ArrowDown, Modifiers::empty()),
    ] {
        let status = press_key(&mut rig, key, mods);
        let focused = rig.focused();
        if status != Status::Ignored || !rig.messages.is_empty() || !focused.is_empty() {
            off.push(format!(
                "{key:?}+{mods:?}: {status:?} {:?} focused {focused:?}",
                rig.messages
            ));
        }
    }
    assert!(
        off.is_empty(),
        "keys the engine answered by itself: {off:?}"
    );
}

/// **K-024, R-073, L-067 row 4: the Escape field and the click-outside message, composed.**
/// A dialog or a sheet with only an escape message is not closed by a click outside, and closes on
/// Escape (captured); with both, a click still publishes the click-outside message and Escape the
/// escape message; with nothing open, Escape is `Ignored` and publishes nothing.
#[test]
fn the_escape_message_and_the_click_outside_message_compose() {
    let corner = iced::Point::new(4.0, 4.0);
    let far = iced::Point::new(790.0, 590.0);
    for (overlay, spot) in [(Overlay::Dialog, far), (Overlay::Sheet, corner)] {
        // Escape only: a click outside does nothing.
        let mut rig = Rig::new(window(&escape_scene(overlay, true, false)));
        rig.click_at(spot);
        assert!(
            rig.messages.is_empty(),
            "{overlay:?} with only an escape message: a click outside published {:?}",
            rig.messages
        );
        let status = rig.press(Key::Named(Named::Escape), Modifiers::empty());
        assert_eq!(rig.messages, vec![Msg::EscapeModal], "{overlay:?}");
        assert_eq!(status, Status::Captured, "{overlay:?}");

        // Both: the click still closes.
        let mut rig = Rig::new(window(&escape_scene(overlay, true, true)));
        rig.click_at(spot);
        assert_eq!(
            rig.messages,
            vec![Msg::CloseModals],
            "{overlay:?} with both: a click outside still publishes the close message"
        );
    }
    // Nothing open: Escape is left to the application.
    let mut rig = Rig::new(window(&Scene {
        escape: Escape {
            modal: Some(Msg::EscapeModal),
            menu: Some(Msg::EscapeMenu),
        },
        shortcuts: false,
        ..Scene::tabs()
    }));
    let status = rig.press(Key::Named(Named::Escape), Modifiers::empty());
    assert!(rig.messages.is_empty(), "{:?}", rig.messages);
    assert_eq!(status, Status::Ignored);
}

/// **R-076: Escape does not dismiss a toast.** With only a toast on screen, and with the toast's
/// close control focused, Escape publishes nothing and is `Ignored`; with a dialog also open, only
/// the dialog's message.
#[test]
fn escape_does_not_dismiss_a_toast() {
    let escape = Escape {
        modal: Some(Msg::EscapeModal),
        menu: Some(Msg::EscapeMenu),
    };
    let only_toast = Scene {
        escape: escape.clone(),
        open: Open {
            toast: true,
            ..Open::NOTHING
        },
        shortcuts: false,
        ..Scene::tabs()
    };
    let mut rig = Rig::new(window(&only_toast));
    let status = rig.press(Key::Named(Named::Escape), Modifiers::empty());
    assert!(
        rig.messages.is_empty() && status == Status::Ignored,
        "{:?} {status:?}",
        rig.messages
    );

    // With the toast's own × focused (the last target in tree order).
    let mut rig = Rig::new(window(&only_toast));
    let last = rig.targets().len() - 1;
    rig.focus_at(last);
    assert_eq!(rig.focused(), vec![last], "the premise: the × has focus");
    let status = rig.press(Key::Named(Named::Escape), Modifiers::empty());
    assert!(
        rig.messages.is_empty() && status == Status::Ignored,
        "{:?} {status:?}",
        rig.messages
    );

    // A dialog and a toast: only the dialog answers.
    let both = Scene {
        escape,
        open: Open {
            toast: true,
            dialog: true,
            ..Open::NOTHING
        },
        shortcuts: false,
        ..Scene::tabs()
    };
    let mut rig = Rig::new(window(&both));
    rig.press(Key::Named(Named::Escape), Modifiers::empty());
    assert_eq!(rig.messages, vec![Msg::EscapeModal]);
}

/// **R-133: the router is silent under every cover that blocks the pointer**, including the two the
/// table above does not run: a `context_menu` with its backdrop, and an open `app_header` dropdown
/// with `on_close_menus`. Ctrl+2 goes to tab 2 with nothing open (the premise) and to no one under
/// either; neither captures it.
#[test]
fn the_router_is_silent_under_a_context_menu_and_an_open_dropdown() {
    static ON_MENU: fn(snora::MenuAction<u8, u8>) -> Msg = Msg::Menu;
    fn menu() -> snora::Menu<u8, u8> {
        snora::Menu {
            id: 0,
            label: "File".into(),
            icon: None,
            items: (0..2)
                .map(|i| snora::MenuItem {
                    menu_id: 0,
                    id: i,
                    label: "Open".into(),
                    icon: None,
                })
                .collect(),
        }
    }
    let base = || {
        let body: Element<'static, Msg> =
            snora::widget::app_tab_bar(tab_bar_data(4, 0), &ON_TAB, LayoutDirection::Ltr);
        AppLayout::new(body).shortcuts(
            Shortcuts::new()
                .platform(Platform::Other)
                .digits(Digits::Tabs)
                .tabs(&tab_bar_data(4, 0), &ON_TAB),
        )
    };
    let mut rig = Rig::new(render(base()));
    let status = digit(&mut rig, 2, CTRL);
    assert_eq!(
        rig.messages,
        vec![Msg::Tab(1)],
        "the premise: nothing open, it answers"
    );
    assert_eq!(status, Status::Captured);

    let covers: [(&str, AppLayout<Element<'static, Msg>, Msg>); 2] = [
        (
            "a context menu with its backdrop",
            base()
                .context_menu(snora::widget::render_menu(menu(), &ON_MENU, true))
                .on_close_menus(Msg::CloseMenus),
        ),
        (
            "an open dropdown with on_close_menus",
            base()
                .header(snora::widget::app_header(
                    "App",
                    vec![menu()],
                    &ON_MENU,
                    Some(&0u8),
                    None,
                    LayoutDirection::Ltr,
                ))
                .on_close_menus(Msg::CloseMenus),
        ),
    ];
    for (what, layout) in covers {
        let mut rig = Rig::new(render(layout));
        let status = digit(&mut rig, 2, CTRL);
        assert!(
            rig.messages.is_empty() && status == Status::Ignored,
            "{what}: the router answered: {:?} {status:?}",
            rig.messages
        );
    }
}

/// **R-050: a focused text field is taken to use every arrow key, with any modifier.** Every arrow,
/// with each modifier set a binding could have, on both platform tables: the router publishes no
/// route (the field's own handling is all that happens). Nothing focused, the same `Alt+↑`
/// (`Cmd+↑` on macOS) reaches the parent, which is the premise that the route is there.
#[test]
fn a_focused_text_field_takes_every_arrow_with_every_modifier() {
    let arrows = [
        Named::ArrowUp,
        Named::ArrowDown,
        Named::ArrowLeft,
        Named::ArrowRight,
    ];
    let modifier_sets = [
        Modifiers::empty(),
        Modifiers::SHIFT,
        Modifiers::CTRL,
        Modifiers::ALT,
        Modifiers::LOGO,
        Modifiers::CTRL | Modifiers::SHIFT,
        Modifiers::ALT | Modifiers::SHIFT,
        Modifiers::LOGO | Modifiers::SHIFT,
    ];
    for platform in [Platform::Other, Platform::MacOs] {
        let sc = Scene {
            content: Content::TextInput,
            platform,
            ..Scene::tabs()
        };
        for arrow in arrows {
            for mods in modifier_sets {
                let mut rig = Rig::new(window(&sc));
                let last = rig.targets().len() - 1;
                rig.focus_at(last);
                assert_eq!(
                    rig.focused(),
                    vec![last],
                    "the premise: the field has focus"
                );
                named(&mut rig, arrow, mods);
                assert!(
                    !rig.messages
                        .iter()
                        .any(|m| matches!(m, Msg::Crumb(_) | Msg::Tab(_) | Msg::Go(_))),
                    "{platform:?}: {arrow:?}+{mods:?} in a text field was routed: {:?}",
                    rig.messages
                );
            }
        }
    }
}

/// **R-049: a shortcut works unless the focused control uses that key, and no more is "used".** In
/// a focused text field `Ctrl+2` still goes to tab 2 (the field does not use it), and with a snora
/// control focused (which uses Enter and Space only) `Alt+↑` still goes to the parent.
#[test]
fn a_shortcut_works_in_a_field_that_does_not_use_its_key() {
    let mut rig = Rig::new(window(&Scene {
        content: Content::TextInput,
        ..Scene::tabs()
    }));
    let last = rig.targets().len() - 1;
    rig.focus_at(last);
    assert_eq!(
        rig.focused(),
        vec![last],
        "the premise: the field has focus"
    );
    digit(&mut rig, 2, CTRL);
    assert_eq!(rig.messages, vec![Msg::Tab(1)], "Ctrl+2 in a text field");

    let mut rig = Rig::new(window(&Scene::tabs()));
    let last = rig.targets().len() - 1;
    rig.focus_at(last);
    assert_eq!(
        rig.focused(),
        vec![last],
        "the premise: a control has focus"
    );
    named(&mut rig, Named::ArrowUp, Modifiers::ALT);
    assert_eq!(
        rig.messages,
        vec![Msg::Crumb(1)],
        "Alt+↑ with a snora control focused"
    );
}
