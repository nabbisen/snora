//! # Example: header_menu
//!
//! Classic File / View / Help drop-down menu bar. Demonstrates:
//!
//! * `Menu` / `MenuItem` / `MenuAction` vocabulary from `snora-core`.
//! * The `snora::widget::app_header` helper consuming `Vec<Menu<_, _>>`.
//! * How to install the transparent click-outside backdrop for the
//!   header dropdown by populating `AppLayout::header_menu` whenever any
//!   menu is open.
//! * Application-defined `MenuId` and `MenuItemId` enums.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-header-menu
//! ```

use iced::{
    Element, Length,
    widget::{container, space, text},
};
use snora::{
    AppLayout, LayoutDirection, Menu, MenuAction, MenuItem,
    render, widget::app_header,
};

// ---------------------------------------------------------------------
// Application-defined menu identities.
// ---------------------------------------------------------------------
//
// These are your types, not snora's. snora threads them through the menu
// widget via generics and echoes them back on `MenuAction`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuId {
    File,
    View,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuItemId {
    New,
    Open,
    Quit,
    ToggleStatus,
    About,
}

#[derive(Debug, Clone)]
enum Message {
    HeaderAction(MenuAction<MenuId, MenuItemId>),
    CloseMenus,
}

#[derive(Default)]
struct App {
    /// Which menu is currently open, if any.
    active: Option<MenuId>,
    show_status: bool,
    last_action: String,
}

impl App {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::HeaderAction(action) => match action {
                MenuAction::MenuPressed(id) => {
                    // Same menu → toggle closed; different menu → switch.
                    self.active = if self.active == Some(id) {
                        None
                    } else {
                        Some(id)
                    };
                }
                MenuAction::MenuItemPressed { menu_item_id, .. } => {
                    self.active = None;
                    match menu_item_id {
                        MenuItemId::New => self.last_action = "File / New".into(),
                        MenuItemId::Open => self.last_action = "File / Open".into(),
                        MenuItemId::Quit => self.last_action = "File / Quit".into(),
                        MenuItemId::ToggleStatus => {
                            self.show_status = !self.show_status;
                            self.last_action = "View / Toggle status".into();
                        }
                        MenuItemId::About => self.last_action = "Help / About".into(),
                    }
                }
            },
            Message::CloseMenus => self.active = None,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Build the menu definitions. The list is application-owned —
        // snora sees plain data.
        let menus = vec![
            Menu {
                id: MenuId::File,
                label: "File".into(),
                icon: None,
                items: vec![
                    item(MenuId::File, MenuItemId::New, "New"),
                    item(MenuId::File, MenuItemId::Open, "Open…"),
                    item(MenuId::File, MenuItemId::Quit, "Quit"),
                ],
            },
            Menu {
                id: MenuId::View,
                label: "View".into(),
                icon: None,
                items: vec![item(
                    MenuId::View,
                    MenuItemId::ToggleStatus,
                    "Toggle status line",
                )],
            },
            Menu {
                id: MenuId::Help,
                label: "Help".into(),
                icon: None,
                items: vec![item(MenuId::Help, MenuItemId::About, "About")],
            },
        ];

        let header = app_header(
            "snora — header menu",
            menus,
            &Message::HeaderAction,
            self.active.as_ref(),
            None,
            LayoutDirection::Ltr,
        );

        let body_column = iced::widget::column![
            text("Header menu demo").size(28),
            text(
                "Open any menu from the bar above. Picking an item updates \
                 the status line. Clicking outside an open menu closes it — \
                 that happens because we populated `header_menu` and \
                 `on_close_menus` on the AppLayout.",
            )
            .size(13),
            space().height(Length::Fixed(12.0)),
            text(format!("Last action: {}", if self.last_action.is_empty() {
                "(none)"
            } else {
                self.last_action.as_str()
            }))
            .size(13),
            if self.show_status {
                text("Status line is ON.").size(12).into()
            } else {
                Element::<Message>::from(space().height(Length::Fixed(0.0)))
            },
        ]
        .spacing(8);

        let body = container(body_column)
            .padding(32)
            .width(Length::Fill)
            .height(Length::Fill);

        let mut layout = AppLayout::new(body.into())
            .header(header)
            .on_close_menus(Message::CloseMenus);

        // Installing the transparent click-outside backdrop is snora's job,
        // but snora only installs it when `header_menu` is populated. The
        // actual dropdown items are drawn inline by `render_menu` inside the
        // header bar; all we need here is an opt-in signal that a menu is
        // open. An empty element suffices.
        if self.active.is_some() {
            layout = layout.header_menu(space().into());
        }

        render(layout)
    }
}

fn item(menu_id: MenuId, id: MenuItemId, label: &str) -> MenuItem<MenuId, MenuItemId> {
    MenuItem {
        menu_id,
        id,
        label: label.into(),
        icon: None,
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("snora — header menu"))
        .run()
}
