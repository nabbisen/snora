//! # Example: context_menu
//!
//! Floating context menu (right-click-style) with click-outside dismissal.
//! Demonstrates:
//!
//! * `AppLayout::context_menu` — a light overlay above the skeleton.
//! * `AppLayout::on_close_menus` — the transparent backdrop's close sink.
//!   Separate from `on_close_modals` so that menus and modals can close
//!   independently.
//! * Manual positioning inside the overlay, since iced 0.14 has no
//!   absolute-position primitive for overlays.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-context-menu
//! ```

use iced::{
    Background, Border, Color, Element, Length, Point,
    widget::{button, column, container, row, space, text},
};
use snora::{AppLayout, render};

#[derive(Debug, Clone)]
enum Message {
    Open(Point),
    PickItem(&'static str),
    CloseMenus,
}

#[derive(Default)]
struct App {
    open_at: Option<Point>,
    last_pick: Option<&'static str>,
}

impl App {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::Open(p) => self.open_at = Some(p),
            Message::PickItem(label) => {
                self.last_pick = Some(label);
                self.open_at = None;
            }
            Message::CloseMenus => self.open_at = None,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let body = container(
            column![
                text("Context menu demo").size(28),
                text(
                    "iced 0.14 does not report the pointer coordinate \
                     alongside a button press, so this demo pins the menu \
                     at a fixed point. In a real app you would hook into a \
                     `mouse::Event` via a subscription or the advanced \
                     widget API to get the real click position.",
                )
                .size(13),
                space().height(Length::Fixed(16.0)),
                row![
                    button(text("Open here").size(13))
                        .on_press(Message::Open(Point::new(240.0, 220.0))),
                    button(text("Open elsewhere").size(13))
                        .on_press(Message::Open(Point::new(480.0, 160.0))),
                ]
                .spacing(8),
                space().height(Length::Fixed(12.0)),
                text(match self.last_pick {
                    Some(p) => format!("Last pick: {p}"),
                    None => "Last pick: (nothing yet)".into(),
                })
                .size(13),
            ]
            .spacing(8),
        )
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill);

        let mut layout = AppLayout::new(body.into())
            // Any outside click dismisses the menu. snora installs the
            // transparent backdrop for you when `context_menu` is `Some`.
            .on_close_menus(Message::CloseMenus);

        if let Some(p) = self.open_at {
            layout = layout.context_menu(menu_at(p));
        }

        render(layout)
    }
}

fn menu_at(p: Point) -> Element<'static, Message> {
    let card = container(
        column![
            menu_button("Copy"),
            menu_button("Paste"),
            menu_button("Delete"),
        ]
        .spacing(2),
    )
    .padding(8)
    .width(Length::Fixed(160.0))
    .style(|theme: &iced::Theme| {
        let ep = theme.extended_palette();
        container::Style {
            background: Some(Background::Color(ep.background.base.color)),
            text_color: Some(ep.background.base.text),
            border: Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color {
                    a: 0.25,
                    ..ep.background.strong.color
                },
            },
            ..Default::default()
        }
    });

    // Position the card with Space padding — iced 0.14 has no absolute
    // positioning primitive in overlay layers.
    column![
        space().height(Length::Fixed(p.y)),
        row![space().width(Length::Fixed(p.x)), card],
    ]
    .into()
}

fn menu_button(label: &'static str) -> Element<'static, Message> {
    button(text(label).size(13))
        .width(Length::Fill)
        .on_press(Message::PickItem(label))
        .into()
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("snora — context menu"))
        .run()
}
