//! The probe's subjects: every snora control, built the same way
//! `every_control.rs` builds them (RFC-121).
//!
//! **A copy of that file's builders**, kept apart so a second test file can
//! cover "every snora control" without including the first file's tests.
//! If a control is added to one list, add it to the other: `every_control.rs`
//! is untouched by RFC-121 on purpose (its assertions are the architect's to
//! rule on, see that RFC's handoff section 3).

use iced::widget::space;
use iced::{Element, Length, Point, Size};

use snora::{
    AppLayout, BreadcrumbAction, Crumb, LayoutDirection, Menu, MenuAction, MenuItem, Tab,
    TabAction, TabBar, Toast, ToastIntent, render,
};

pub const WINDOW: Size = Size::new(1024.0, 768.0);
pub const AWAY: Point = Point::new(1000.0, 760.0);

#[derive(Debug, Clone, PartialEq)]
pub struct Msg(pub String);

pub fn msg(text: &str) -> Msg {
    Msg(text.to_owned())
}

pub static ON_TAB: fn(TabAction<u8>) -> Msg = |a| Msg(format!("{a:?}"));
pub static ON_CRUMB: fn(BreadcrumbAction<u8>) -> Msg = |a| Msg(format!("{a:?}"));
pub static ON_MENU: fn(MenuAction<u8, u8>) -> Msg = |a| Msg(format!("{a:?}"));

/// One thing to probe: how to build it, and what each of its focus
/// targets, in tree order, must publish.
pub struct Subject {
    pub name: &'static str,
    pub build: Box<dyn Fn() -> Element<'static, Msg>>,
    pub publishes: Vec<Msg>,
}

impl Subject {
    /// The subject, inset from the window's edge. A ring is drawn outside
    /// its control; one at the edge of the window would have its outer side
    /// off the frame, which is the window clipping it and nothing to do
    /// with the control.
    pub fn element(&self) -> Element<'static, Msg> {
        iced::widget::container((self.build)()).padding(24).into()
    }
}

pub fn subject(
    name: &'static str,
    publishes: &[&str],
    build: impl Fn() -> Element<'static, Msg> + 'static,
) -> Subject {
    Subject {
        name,
        build: Box::new(build),
        publishes: publishes.iter().map(|m| msg(m)).collect(),
    }
}

pub fn tab_bar() -> TabBar<u8> {
    TabBar {
        tabs: ["One", "Two"]
            .iter()
            .enumerate()
            .map(|(i, l)| Tab {
                id: i as u8,
                label: (*l).to_owned(),
                icon: None,
            })
            .collect(),
        active: 0,
    }
}

pub fn crumbs() -> Vec<Crumb<u8>> {
    vec![
        Crumb::ancestor(0, "Home"),
        Crumb::ancestor(1, "Library"),
        // The leaf is plain text: not a control, not a target.
        Crumb::leaf(2, "Books"),
    ]
}

pub fn menu() -> Menu<u8, u8> {
    Menu {
        id: 0,
        label: "File".to_owned(),
        icon: None,
        items: [("Open", 7), ("Save", 8)]
            .iter()
            .map(|(l, id)| MenuItem {
                menu_id: 0,
                id: *id,
                label: (*l).to_owned(),
                icon: None,
            })
            .collect(),
    }
}

pub const MENU_MESSAGES: [&str; 3] = [
    "MenuPressed(0)",
    "MenuItemPressed { menu_id: 0, menu_item_id: 7 }",
    "MenuItemPressed { menu_id: 0, menu_item_id: 8 }",
];

pub fn toast_layout() -> AppLayout<Element<'static, Msg>, Msg> {
    let body: Element<'static, Msg> = space().width(Length::Fill).height(Length::Fill).into();
    AppLayout::new(body).toasts(vec![Toast::new(
        7,
        ToastIntent::Info,
        "Saved",
        "All good.",
        msg("dismiss"),
    )])
}

/// The unstyled controls.
pub fn unstyled() -> Vec<Subject> {
    vec![
        subject("tabs", &["Pressed(0)", "Pressed(1)"], || {
            snora::widget::app_tab_bar(tab_bar(), &ON_TAB, LayoutDirection::Ltr)
        }),
        subject(
            "breadcrumb ancestors",
            &["Pressed(0)", "Pressed(1)"],
            || snora::widget::app_breadcrumb(crumbs(), &ON_CRUMB, LayoutDirection::Ltr),
        ),
        subject("header menu (trigger and items)", &MENU_MESSAGES, || {
            snora::widget::app_header(
                "App",
                vec![menu()],
                &ON_MENU,
                Some(&0),
                None,
                LayoutDirection::Ltr,
            )
        }),
        subject("render_menu (trigger and items)", &MENU_MESSAGES, || {
            snora::widget::render_menu(menu(), &ON_MENU, true)
        }),
        subject("toast close", &["dismiss"], || render(toast_layout())),
    ]
}

/// The design-path controls, for one preset.
#[cfg(feature = "design")]
pub fn designed(tokens: &'static snora::design::Tokens) -> Vec<Subject> {
    use snora::design::{Tone, button, chip, notice::Notice};

    vec![
        subject("design tabs", &["Pressed(0)", "Pressed(1)"], move || {
            snora::design::widget::app_tab_bar(tokens, tab_bar(), &ON_TAB, LayoutDirection::Ltr)
        }),
        subject(
            "design breadcrumb ancestors",
            &["Pressed(0)", "Pressed(1)"],
            move || {
                snora::design::widget::app_breadcrumb(
                    tokens,
                    crumbs(),
                    &ON_CRUMB,
                    LayoutDirection::Ltr,
                )
            },
        ),
        subject("design header menu", &MENU_MESSAGES, move || {
            snora::design::widget::app_header(
                tokens,
                "App",
                vec![menu()],
                &ON_MENU,
                Some(&0),
                None,
                LayoutDirection::Ltr,
            )
        }),
        subject("button::primary", &["primary"], move || {
            button::primary(tokens, "Save", msg("primary"))
        }),
        subject("button::secondary", &["secondary"], move || {
            button::secondary(tokens, "Save", msg("secondary"))
        }),
        subject("button::ghost", &["ghost"], move || {
            button::ghost(tokens, "Save", msg("ghost"))
        }),
        subject("button::link", &["link"], move || {
            button::link(tokens, "Choose another folder", msg("link"))
        }),
        subject("button::danger", &["danger"], move || {
            button::danger(tokens, "Delete", msg("danger"))
        }),
        // A disabled button is not a focus target: Tab skips it.
        subject("button::primary_maybe(None)", &[], move || {
            button::primary_maybe(tokens, "Save", None)
        }),
        subject(
            "notice action and dismiss",
            &["act", "dismiss"],
            move || {
                Notice::new(tokens, Tone::Info, "Index rebuilt.")
                    .action("Undo", msg("act"))
                    .dismiss(msg("dismiss"))
                    .render()
            },
        ),
        subject("chip::filter", &["toggle"], move || {
            chip::filter(tokens, "Rust", false, msg("toggle"))
        }),
        subject("chip::removable", &["toggle", "remove"], move || {
            chip::removable(tokens, "Rust", false, msg("toggle"), msg("remove"))
        }),
        // **A control that is NOT reached** (RFC-119): the page's "page" control
        // is under an open dialog, so the tree reports one focus target, the
        // dialog's, and Tab, Enter and the ring are checked on that. A page
        // control that were reachable would make the count two and fail.
        subject(
            "a page control under an open dialog is not reached",
            &["dialog"],
            move || {
                let body = button::primary(tokens, "Delete", msg("page"));
                let ok = button::primary(tokens, "OK", msg("dialog"));
                snora::design::render(
                    snora::AppLayout::new(body).dialog(snora::Dialog::new(ok)),
                    tokens,
                )
            },
        ),
        subject("design toast close", &["dismiss"], move || {
            snora::design::render(toast_layout(), tokens)
        }),
    ]
}
