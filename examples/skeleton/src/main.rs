//! # Example: skeleton
//!
//! Demonstrates the four skeleton slots (header / side_bar / body / footer)
//! and shows how each slot is just a plain `iced::Element` that snora
//! stitches together.
//!
//! Each slot here is a different kind of element to drive home that snora
//! places no constraint on slot content:
//!
//! * `header` uses the prefab `snora::widget::app_header` helper.
//! * `side_bar` uses the prefab `snora::widget::app_side_bar` helper.
//! * `footer` uses the prefab `snora::widget::app_footer` helper.
//! * `body` is hand-written with raw iced widgets.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-skeleton
//! ```

use iced::{
    Alignment::Center,
    Element, Length,
    widget::{column, container, row, space, text},
};
use snora::{
    AppLayout, LayoutDirection, SideBar, SideBarItem,
    render,
    widget::{app_footer, app_header, app_side_bar},
};

#[derive(Debug, Clone)]
enum Message {
    SelectView(ViewId),
    // Header has an empty menu list here, so no MenuAction variant is
    // needed. See the `header_menu` example for the menu-driven case.
    //
    // The payload is unused in this example but the variant shape must
    // match `app_header`'s `on_menu_action` callback signature, so we
    // allow the dead-code lint locally.
    #[allow(dead_code)]
    HeaderAction(snora::MenuAction<(), ()>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewId {
    Home,
    Profile,
    Settings,
}

struct App {
    active: ViewId,
}

impl Default for App {
    fn default() -> Self {
        Self { active: ViewId::Home }
    }
}

impl App {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::SelectView(v) => self.active = v,
            Message::HeaderAction(_) => {}
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Header — bold title on the start edge, no menus, no end controls.
        let header = app_header(
            "snora — skeleton",
            Vec::<snora::Menu<(), ()>>::new(),
            &Message::HeaderAction,
            None,
            None,
            LayoutDirection::Ltr,
        );

        // Sidebar — three icon buttons. `Icon::Text` keeps the example
        // feature-free (no lucide-icons dependency needed).
        let sidebar = app_side_bar(
            SideBar {
                items: vec![
                    SideBarItem {
                        view_id: ViewId::Home,
                        icon: "🏠".into(),
                        tooltip: "Home".into(),
                        on_press: Message::SelectView(ViewId::Home),
                    },
                    SideBarItem {
                        view_id: ViewId::Profile,
                        icon: "👤".into(),
                        tooltip: "Profile".into(),
                        on_press: Message::SelectView(ViewId::Profile),
                    },
                    SideBarItem {
                        view_id: ViewId::Settings,
                        icon: "⚙".into(),
                        tooltip: "Settings".into(),
                        on_press: Message::SelectView(ViewId::Settings),
                    },
                ],
                active: self.active,
            },
            LayoutDirection::Ltr,
        );

        // Body — varies with the active view. Just a heading and a blurb.
        let body: Element<'_, Message> = {
            let (title, blurb) = match self.active {
                ViewId::Home => ("Home", "The default landing view."),
                ViewId::Profile => ("Profile", "Imagine user settings here."),
                ViewId::Settings => ("Settings", "Imagine app configuration here."),
            };
            container(
                column![
                    text(title).size(28),
                    text(blurb).size(14),
                ]
                .spacing(8),
            )
            .padding(32)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        };

        // Footer — a simple status bar.
        let footer = app_footer(
            row![
                text(format!("Active view: {:?}", self.active)).size(12),
                container(space()).width(Length::Fill),
                text("Snora skeleton demo").size(12),
            ]
            .align_y(Center)
            .spacing(12)
            .into(),
        );

        // Assemble. Each slot is a plain Element — no trait, no wrapper enum.
        let layout = AppLayout::new(body)
            .header(header)
            .side_bar(sidebar)
            .footer(footer);

        render(layout)
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("snora — skeleton"))
        .run()
}
