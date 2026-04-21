//! Chrome: the app's frame — header, sidebar, footer.
//!
//! These are plain `Element`-producing functions that are stitched into the
//! `AppLayout` via `Section`. They lean on snora's pre-built `app_header`,
//! `app_side_bar`, and `app_footer` where it's useful and drop down to raw iced
//! widgets for controls specific to the showcase.

use iced::{
    Alignment::Center,
    Element, Length,
    widget::{button, container, row, space, text},
};
use snora::{
    AppSideBar, AppSideBarItem, Icon, LayoutDirection, Menu, MenuAction, MenuItem,
    components::app::{footer::app_footer, header::app_header, side_bar::app_side_bar},
    icons,
};

use super::{
    App, FileMenuItemId, HelpMenuItemId, MenuId, MenuItemId, Message, ViewId, ViewMenuItemId,
};

// A function *item* (not a closure value) has its own zero-sized 'static type, so
// `&header_action` can be passed as `&'a F` for any lifetime `'a` — which is exactly
// what `app_header` asks for. Using a closure local would inherit the caller's
// lifetime and fight the borrow checker here.
fn header_action(a: MenuAction<MenuId, MenuItemId>) -> Message {
    Message::HeaderAction(a)
}

pub fn header(app: &App) -> Element<'_, Message> {
    let menus = vec![
        Menu {
            id: MenuId::File,
            label: "File".into(),
            icon: Some(Icon::Lucide(icons::FileText)),
            items: vec![
                MenuItem {
                    menu_id: MenuId::File,
                    id: MenuItemId::File(FileMenuItemId::New),
                    label: "New".into(),
                    icon: None,
                },
                MenuItem {
                    menu_id: MenuId::File,
                    id: MenuItemId::File(FileMenuItemId::Open),
                    label: "Open…".into(),
                    icon: None,
                },
                MenuItem {
                    menu_id: MenuId::File,
                    id: MenuItemId::File(FileMenuItemId::Quit),
                    label: "Quit".into(),
                    icon: None,
                },
            ],
        },
        Menu {
            id: MenuId::View,
            label: "View".into(),
            icon: None,
            items: vec![
                MenuItem {
                    menu_id: MenuId::View,
                    id: MenuItemId::View(ViewMenuItemId::ToggleLogs),
                    label: "Toggle Logs Panel".into(),
                    icon: None,
                },
                MenuItem {
                    menu_id: MenuId::View,
                    id: MenuItemId::View(ViewMenuItemId::FlipDirection),
                    label: "Flip LTR / RTL".into(),
                    icon: None,
                },
            ],
        },
        Menu {
            id: MenuId::Help,
            label: "Help".into(),
            icon: None,
            items: vec![
                MenuItem {
                    menu_id: MenuId::Help,
                    id: MenuItemId::Help(HelpMenuItemId::Documentation),
                    label: "Documentation".into(),
                    icon: None,
                },
                MenuItem {
                    menu_id: MenuId::Help,
                    id: MenuItemId::Help(HelpMenuItemId::About),
                    label: "About".into(),
                    icon: None,
                },
            ],
        },
    ];

    // Right-side controls: a direction indicator + the LTR/RTL flip button.
    let direction_label = match app.direction {
        LayoutDirection::Ltr => "LTR",
        LayoutDirection::Rtl => "RTL",
    };
    let right_controls: Element<'_, Message> = row![
        text(format!("Direction: {}", direction_label)).size(12),
        button(text("Flip LTR ↔ RTL").size(12)).on_press(Message::ToggleDirection),
    ]
    .spacing(12)
    .align_y(Center)
    .into();

    app_header(
        "Snora Showcase",
        menus,
        &header_action,
        app.active_menu_id.as_ref(),
        Some(right_controls),
    )
}

pub fn sidebar(app: &App) -> Element<'_, Message> {
    let items = vec![
        AppSideBarItem {
            view_id: ViewId::Home,
            icon: Icon::Lucide(icons::Home),
            tooltip: "Home".into(),
            action: Message::SelectView(ViewId::Home),
        },
        AppSideBarItem {
            view_id: ViewId::Search,
            icon: Icon::Lucide(icons::Search),
            tooltip: "Search".into(),
            action: Message::SelectView(ViewId::Search),
        },
        AppSideBarItem {
            view_id: ViewId::Settings,
            icon: Icon::Lucide(icons::Settings),
            tooltip: "Settings".into(),
            action: Message::SelectView(ViewId::Settings),
        },
    ];
    app_side_bar(AppSideBar {
        items,
        view_id: app.active_view_id.clone(),
    })
}

pub fn footer(app: &App) -> Element<'_, Message> {
    let content: Element<'_, Message> = row![
        text(format!(
            "View: {}  ·  Logs: {}  ·  Toasts: {}",
            app.active_view_id,
            app.logs.len(),
            app.toasts.len()
        ))
        .size(12),
        container(space()).width(Length::Fill),
        button(text("Add Log").size(12)).on_press(Message::AddLog),
        button(
            text(if app.show_bottom_sheet {
                "Hide Logs ▲"
            } else {
                "Show Logs ▼"
            })
            .size(12)
        )
        .on_press(Message::ToggleSheet),
    ]
    .spacing(12)
    .align_y(Center)
    .into();

    app_footer(content)
}
