//! Chrome and toast label sizes follow their host (RFC-104).
//!
//! Every chrome label was a literal: 13 in the tab bar and breadcrumb, 14
//! in menus, 16 in the header title, 16/14 in toasts. An application that
//! sets iced's `default_text_size`, or that ships a `Tokens` bundle with
//! its own `Typography`, got those numbers anyway.
//!
//! # What these measure
//!
//! **Text, not boxes.** A parent can hand a text widget a box larger than
//! its glyphs, so a box measurement can agree with itself while the text
//! ignores the host (RFC-099). Each assertion here compares a label's
//! rendered height against the height of **the same string rendered
//! alone** under the same settings — the natural size for the size that
//! should apply. Where the expected size is known (the styled path), the
//! reference is rendered at that size explicitly.

#![cfg(feature = "widgets")]

use iced::{Element, Settings};
use iced_test::Simulator;

use snora::{Crumb, LayoutDirection, Menu, MenuAction, MenuItem, Tab, TabAction, TabBar};

/// Two host sizes far enough apart that no rounding could confuse them,
/// and neither equal to any literal the widgets used to carry.
const SMALL_HOST: f32 = 12.0;
const LARGE_HOST: f32 = 22.0;

const HEADER_TITLE: &str = "Workspace";
const TAB_LABEL: &str = "Overview";
const CRUMB_LABEL: &str = "Projects";
const MENU_LABEL: &str = "File";
const MENU_ITEM_LABEL: &str = "Open";

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Tab(TabAction<u8>),
    Crumb(snora::BreadcrumbAction<u8>),
    Menu(MenuAction<u8, u8>),
}

static ON_TAB: fn(TabAction<u8>) -> Msg = Msg::Tab;
static ON_CRUMB: fn(snora::BreadcrumbAction<u8>) -> Msg = Msg::Crumb;
static ON_MENU: fn(MenuAction<u8, u8>) -> Msg = Msg::Menu;

fn settings(default_text_size: f32) -> Settings {
    Settings {
        default_text_size: default_text_size.into(),
        ..Settings::default()
    }
}

/// The height of `label` as rendered inside `element`.
fn rendered_height(element: Element<'_, Msg>, label: &str, host: f32) -> f32 {
    let mut ui = Simulator::with_settings(settings(host), element);
    ui.find(label)
        .unwrap_or_else(|_| panic!("{label:?} is not rendered"))
        .visible_bounds()
        .unwrap_or_else(|| panic!("{label:?} is not visible"))
        .height
}

/// The height of `label` rendered on its own: at `size` when given, and
/// at whatever the host's default is when not.
fn natural_height(label: &str, size: Option<f32>, host: f32) -> f32 {
    let mut text = iced::widget::text(label.to_owned());
    if let Some(size) = size {
        text = text.size(size);
    }
    let element: Element<'_, Msg> = text.into();
    let mut ui = Simulator::with_settings(settings(host), element);
    ui.find(label)
        .expect("the reference string renders")
        .visible_bounds()
        .expect("the reference string is visible")
        .height
}

// ---------------------------------------------------------------------------
// Fixtures, built through the public API
// ---------------------------------------------------------------------------

fn tab_bar() -> TabBar<u8> {
    TabBar {
        tabs: vec![Tab {
            id: 0,
            label: TAB_LABEL.to_owned(),
            icon: None,
        }],
        active: 0,
    }
}

fn crumbs() -> Vec<Crumb<u8>> {
    vec![
        Crumb {
            id: 0,
            label: CRUMB_LABEL.to_owned(),
            is_leaf: false,
        },
        Crumb {
            id: 1,
            label: "Current".to_owned(),
            is_leaf: true,
        },
    ]
}

fn menu() -> Menu<u8, u8> {
    Menu {
        id: 0,
        label: MENU_LABEL.to_owned(),
        icon: None,
        items: vec![MenuItem {
            menu_id: 0,
            id: 0,
            label: MENU_ITEM_LABEL.to_owned(),
            icon: None,
        }],
    }
}

/// Every unstyled chrome label, as (what it is, the element, the string).
fn unstyled_labels() -> Vec<(&'static str, Element<'static, Msg>, &'static str)> {
    vec![
        (
            "header title",
            snora::widget::app_header(
                HEADER_TITLE,
                Vec::<Menu<u8, u8>>::new(),
                &ON_MENU,
                None,
                None,
                LayoutDirection::Ltr,
            ),
            HEADER_TITLE,
        ),
        (
            "tab label",
            snora::widget::app_tab_bar(tab_bar(), &ON_TAB, LayoutDirection::Ltr),
            TAB_LABEL,
        ),
        (
            "crumb label",
            snora::widget::app_breadcrumb(crumbs(), &ON_CRUMB, LayoutDirection::Ltr),
            CRUMB_LABEL,
        ),
        (
            "menu trigger",
            snora::widget::render_menu(menu(), &ON_MENU, false),
            MENU_LABEL,
        ),
        (
            "menu item",
            snora::widget::render_menu(menu(), &ON_MENU, true),
            MENU_ITEM_LABEL,
        ),
    ]
}

// ---------------------------------------------------------------------------
// (a) The unstyled path follows the host's default_text_size
// ---------------------------------------------------------------------------

#[test]
fn unstyled_chrome_labels_follow_the_host_text_size() {
    for (what, element, label) in unstyled_labels() {
        let rendered = rendered_height(element, label, LARGE_HOST);
        let inherited = natural_height(label, None, LARGE_HOST);
        assert!(
            (rendered - inherited).abs() <= 0.5,
            "{what}: rendered {rendered}px tall with default_text_size = {LARGE_HOST}, but the \
             same string inheriting that size is {inherited}px — the label carries a literal \
             size and ignores the host",
        );
    }
}

/// The same property from the other side: change the host's size and the
/// label must move with it. Guards against a label that happens to match
/// at one size.
#[test]
fn unstyled_chrome_labels_change_with_the_host_text_size() {
    for (what, element, label) in unstyled_labels() {
        let small = rendered_height(element, label, SMALL_HOST);
        let element = unstyled_labels()
            .into_iter()
            .find(|(w, _, _)| *w == what)
            .map(|(_, e, _)| e)
            .expect("the fixture is rebuilt for the second render");
        let large = rendered_height(element, label, LARGE_HOST);
        assert!(
            large > small + 1.0,
            "{what}: {small}px at default_text_size = {SMALL_HOST} and {large}px at \
             {LARGE_HOST} — the label does not follow the host",
        );
    }
}

// ---------------------------------------------------------------------------
// (c) The toast follows the host on the default path
// ---------------------------------------------------------------------------

const TOAST_TITLE: &str = "Saved";
const TOAST_MESSAGE: &str = "All good.";

fn toast_element() -> Element<'static, Msg> {
    let toast = snora::Toast::new(
        7,
        snora::ToastIntent::Info,
        TOAST_TITLE,
        TOAST_MESSAGE,
        Msg::Tab(TabAction::Pressed(0)),
    );
    let body: Element<'static, Msg> = iced::widget::text("body").into();
    snora::render(snora::AppLayout::new(body).toasts(vec![toast]))
}

#[test]
fn toast_text_follows_the_host_text_size() {
    for label in [TOAST_TITLE, TOAST_MESSAGE] {
        let rendered = rendered_height(toast_element(), label, LARGE_HOST);
        let inherited = natural_height(label, None, LARGE_HOST);
        assert!(
            (rendered - inherited).abs() <= 0.5,
            "toast {label:?}: rendered {rendered}px tall with default_text_size = {LARGE_HOST}, \
             but the same string inheriting that size is {inherited}px — the toast carries \
             literal sizes",
        );
    }
}

// ---------------------------------------------------------------------------
// (b), (c) The styled path follows the tokens' Typography
// ---------------------------------------------------------------------------

#[cfg(feature = "design")]
mod styled {
    use super::*;
    use snora::design::Tokens;
    use snora::design::style::text::{body_size, label_size, title_size};

    /// A token bundle whose typography is scaled, so "follows the
    /// tokens" cannot be satisfied by a literal that happens to match
    /// the default scale.
    fn scaled(factor: f32) -> Tokens {
        let mut tokens = Tokens::light();
        for role in [
            &mut tokens.typography.body,
            &mut tokens.typography.body_small,
            &mut tokens.typography.label,
            &mut tokens.typography.title,
            &mut tokens.typography.heading,
            &mut tokens.typography.display,
        ] {
            role.size *= factor;
        }
        tokens
    }

    /// Each styled chrome label, with the role it must follow.
    fn cases(tokens: &Tokens) -> Vec<(&'static str, Element<'_, Msg>, &'static str, f32)> {
        vec![
            (
                "header title",
                snora::design::widget::app_header(
                    tokens,
                    HEADER_TITLE,
                    Vec::<Menu<u8, u8>>::new(),
                    &ON_MENU,
                    None,
                    None,
                    LayoutDirection::Ltr,
                ),
                HEADER_TITLE,
                title_size(tokens).0,
            ),
            (
                "tab label",
                snora::design::widget::app_tab_bar(
                    tokens,
                    tab_bar(),
                    &ON_TAB,
                    LayoutDirection::Ltr,
                ),
                TAB_LABEL,
                label_size(tokens).0,
            ),
            (
                "crumb label",
                snora::design::widget::app_breadcrumb(
                    tokens,
                    crumbs(),
                    &ON_CRUMB,
                    LayoutDirection::Ltr,
                ),
                CRUMB_LABEL,
                label_size(tokens).0,
            ),
        ]
    }

    #[test]
    fn styled_chrome_labels_follow_the_tokens() {
        for factor in [1.0, 1.75] {
            let tokens = scaled(factor);
            for (what, element, label, expected) in cases(&tokens) {
                let rendered = rendered_height(element, label, LARGE_HOST);
                let reference = natural_height(label, Some(expected), LARGE_HOST);
                assert!(
                    (rendered - reference).abs() <= 0.5,
                    "{what} at typography x{factor}: rendered {rendered}px, but the same string \
                     at the role's {expected}px measures {reference}px — the styled label does \
                     not follow the tokens",
                );
            }
        }
    }

    #[test]
    fn styled_toast_text_follows_the_tokens() {
        for factor in [1.0, 1.75] {
            let tokens = scaled(factor);
            let toast = snora::Toast::new(
                7,
                snora::ToastIntent::Info,
                TOAST_TITLE,
                TOAST_MESSAGE,
                Msg::Tab(TabAction::Pressed(0)),
            );
            let body: Element<'_, Msg> = iced::widget::text("body").into();
            let element =
                snora::design::render(snora::AppLayout::new(body).toasts(vec![toast]), &tokens);

            let mut ui = Simulator::with_settings(settings(LARGE_HOST), element);
            for (label, expected) in [
                (TOAST_TITLE, title_size(&tokens).0),
                (TOAST_MESSAGE, body_size(&tokens).0),
            ] {
                let rendered = ui
                    .find(label)
                    .unwrap_or_else(|_| panic!("toast {label:?} renders"))
                    .visible_bounds()
                    .expect("visible")
                    .height;
                let reference = natural_height(label, Some(expected), LARGE_HOST);
                assert!(
                    (rendered - reference).abs() <= 0.5,
                    "toast {label:?} at typography x{factor}: rendered {rendered}px against the \
                     role's {reference}px — the styled toast does not follow the tokens",
                );
            }
        }
    }
}
