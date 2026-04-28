//! # Example: dialog
//!
//! A single centered modal dialog with dim backdrop. Demonstrates:
//!
//! * `Dialog::new(content)` — pure content, no close hook on the dialog
//!   itself.
//! * `AppLayout::on_close_modals` — the one place to wire
//!   click-outside-to-close.
//! * Graceful degradation — if you remove `.on_close_modals(...)` the
//!   dialog still renders, it just cannot be closed by clicking outside.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-dialog
//! ```

use iced::{
    Alignment::Center,
    Background, Border, Color, Element, Length, Padding,
    widget::{button, column, container, row, space, text},
};
use snora::{AppLayout, Dialog, render};

#[derive(Debug, Clone)]
enum Message {
    Open,
    CloseModals,
}

#[derive(Default)]
struct App {
    open: bool,
}

impl App {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::Open => self.open = true,
            Message::CloseModals => self.open = false,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let body = container(
            column![
                text("Dialog demo").size(28),
                text("Click the button to open a centered modal dialog.").size(14),
                space().height(Length::Fixed(16.0)),
                button(text("Open dialog").size(13)).on_press(Message::Open),
            ]
            .spacing(8),
        )
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill);

        let mut layout = AppLayout::new(body.into())
            // Wire the outside-click sink once. Both `Dialog` and
            // `Sheet` (if present) share this single channel.
            .on_close_modals(Message::CloseModals);

        if self.open {
            layout = layout.dialog(Dialog::new(dialog_card()));
        }

        render(layout)
    }
}

/// The dialog's inner content. Snora centers this and paints the dim
/// backdrop; we just draw the card surface and the close button.
fn dialog_card() -> Element<'static, Message> {
    container(
        column![
            text("About this example").size(20),
            text(
                "This dialog was opened by setting `AppLayout::dialog = \
                 Some(Dialog::new(...))`. The framework painted the dim \
                 backdrop and centered this card.\n\nYou can close it by \
                 clicking outside (because we set `on_close_modals`) or by \
                 pressing the button below.",
            )
            .size(14),
            space().height(Length::Fixed(8.0)),
            row![
                container(space()).width(Length::Fill),
                button(text("Close").size(13)).on_press(Message::CloseModals),
            ]
            .align_y(Center),
        ]
        .spacing(12),
    )
    .padding(Padding::from([20.0, 24.0]))
    .width(Length::Fixed(420.0))
    .style(|theme: &iced::Theme| {
        let ep = theme.extended_palette();
        container::Style {
            background: Some(Background::Color(ep.background.base.color)),
            text_color: Some(ep.background.base.text),
            border: Border {
                radius: 10.0.into(),
                width: 1.0,
                color: Color {
                    a: 0.15,
                    ..ep.background.strong.color
                },
            },
            ..Default::default()
        }
    })
    .into()
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("snora — dialog"))
        .run()
}
