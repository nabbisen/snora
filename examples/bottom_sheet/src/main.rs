//! # Example: bottom_sheet
//!
//! Walks through every [`SheetHeight`] variant:
//!
//! * `OneThird`, `Half`, `TwoThirds` — canonical ratio presets.
//! * `Ratio(f32)` — arbitrary fractions (clamped to 0.0..=1.0).
//! * `Pixels(f32)` — fixed height, ignores window size.
//!
//! The sheet shares `AppLayout::on_close_modals` with the dialog example
//! — there is one close sink for both kinds of modal.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-bottom-sheet
//! ```

use iced::{
    Element, Length, Padding,
    widget::{button, column, container, row, space, text},
};
use snora::{AppLayout, BottomSheet, SheetHeight, render};

#[derive(Debug, Clone)]
enum Message {
    Open(SheetHeight),
    CloseModals,
}

#[derive(Default)]
struct App {
    sheet: Option<SheetHeight>,
}

impl App {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::Open(h) => self.sheet = Some(h),
            Message::CloseModals => self.sheet = None,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let body = container(
            column![
                text("Bottom sheet heights").size(28),
                text(
                    "Each button opens a bottom sheet at a different \
                     height. Click outside the sheet (on the dim area) to \
                     close.",
                )
                .size(14),
                space().height(Length::Fixed(16.0)),
                row![
                    open_button("1/3", SheetHeight::OneThird),
                    open_button("1/2", SheetHeight::Half),
                    open_button("2/3", SheetHeight::TwoThirds),
                ]
                .spacing(8),
                row![
                    open_button("Ratio(0.2)", SheetHeight::Ratio(0.2)),
                    open_button("Ratio(0.75)", SheetHeight::Ratio(0.75)),
                    open_button("Pixels(180)", SheetHeight::Pixels(180.0)),
                ]
                .spacing(8),
            ]
            .spacing(8),
        )
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill);

        let mut layout = AppLayout::new(body.into()).on_close_modals(Message::CloseModals);

        if let Some(h) = self.sheet {
            layout = layout.bottom_sheet(BottomSheet::new(sheet_content(h)).with_height(h));
        }

        render(layout)
    }
}

fn open_button(label: &str, h: SheetHeight) -> Element<'static, Message> {
    button(text(label.to_string()).size(13))
        .on_press(Message::Open(h))
        .into()
}

fn sheet_content(h: SheetHeight) -> Element<'static, Message> {
    container(
        column![
            text(format!("Sheet @ {h:?}")).size(20),
            text(
                "This sheet is rendered by the framework. Its height comes \
                 from the `SheetHeight` vocabulary — a ratio or pixel value \
                 chosen by your application, resolved to physical size by \
                 the engine.",
            )
            .size(13),
            space().height(Length::Fixed(12.0)),
            button(text("Close from inside").size(13)).on_press(Message::CloseModals),
        ]
        .spacing(8),
    )
    .padding(Padding::from([20.0, 24.0]))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("snora — bottom sheet"))
        .run()
}
