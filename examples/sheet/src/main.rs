//! # Example: sheet
//!
//! Demonstrates the [`Sheet`] modal across:
//!
//! * All four [`SheetEdge`] anchors — `Top`, `Bottom`, `Start`, `End`.
//!   `Start` / `End` mirror under [`LayoutDirection::Rtl`].
//! * Every [`SheetSize`] variant — `OneThird` / `Half` / `TwoThirds`,
//!   plus an arbitrary `Ratio` and a fixed `Pixels`.
//!
//! The sheet shares `AppLayout::on_close_modals` with dialogs — there is
//! one close sink for both kinds of modal.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-sheet
//! ```

use iced::{
    Element, Length, Padding,
    widget::{button, column, container, row, space, text},
};
use snora::{AppLayout, LayoutDirection, Sheet, SheetEdge, SheetSize, render};

#[derive(Debug, Clone)]
enum Message {
    Open(SheetEdge, SheetSize),
    SetDirection(LayoutDirection),
    CloseModals,
}

#[derive(Default)]
struct App {
    open: Option<(SheetEdge, SheetSize)>,
    direction: LayoutDirection,
}

impl App {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::Open(edge, size) => self.open = Some((edge, size)),
            Message::SetDirection(d) => self.direction = d,
            Message::CloseModals => self.open = None,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let body = container(
            column![
                text("Sheet gallery").size(28),
                text(
                    "Each button opens a sheet at a different edge and \
                     size. Click outside the sheet (on the dim area) to \
                     close. Toggle the direction to see Start / End mirror."
                )
                .size(14),
                space().height(Length::Fixed(12.0)),
                section_label(format!("Direction: {:?}", self.direction)),
                row![
                    labeled_button("LTR", Message::SetDirection(LayoutDirection::Ltr)),
                    labeled_button("RTL", Message::SetDirection(LayoutDirection::Rtl)),
                ]
                .spacing(8),
                space().height(Length::Fixed(16.0)),
                section_label("Bottom edge"),
                row![
                    edge_button("1/3", SheetEdge::Bottom, SheetSize::OneThird),
                    edge_button("1/2", SheetEdge::Bottom, SheetSize::Half),
                    edge_button("2/3", SheetEdge::Bottom, SheetSize::TwoThirds),
                ]
                .spacing(8),
                space().height(Length::Fixed(8.0)),
                section_label("Top edge"),
                row![
                    edge_button("1/3", SheetEdge::Top, SheetSize::OneThird),
                    edge_button("1/2", SheetEdge::Top, SheetSize::Half),
                ]
                .spacing(8),
                space().height(Length::Fixed(8.0)),
                section_label("Start edge (LTR=left, RTL=right)"),
                row![
                    edge_button("1/3", SheetEdge::Start, SheetSize::OneThird),
                    edge_button("280 px", SheetEdge::Start, SheetSize::Pixels(280.0)),
                ]
                .spacing(8),
                space().height(Length::Fixed(8.0)),
                section_label("End edge (LTR=right, RTL=left)"),
                row![
                    edge_button("Ratio(0.4)", SheetEdge::End, SheetSize::Ratio(0.4)),
                    edge_button("280 px", SheetEdge::End, SheetSize::Pixels(280.0)),
                ]
                .spacing(8),
            ]
            .spacing(8),
        )
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill);

        let mut layout = AppLayout::new(body.into())
            .direction(self.direction)
            .on_close_modals(Message::CloseModals);

        if let Some((edge, size)) = self.open {
            layout =
                layout.sheet(Sheet::new(sheet_content(edge, size)).at(edge).with_size(size));
        }

        render(layout)
    }
}

fn section_label(s: impl Into<String>) -> Element<'static, Message> {
    text(s.into()).size(16).into()
}

fn edge_button(label: &str, edge: SheetEdge, size: SheetSize) -> Element<'static, Message> {
    labeled_button(label, Message::Open(edge, size))
}

fn labeled_button(label: &str, msg: Message) -> Element<'static, Message> {
    button(text(label.to_string()).size(13)).on_press(msg).into()
}

fn sheet_content(edge: SheetEdge, size: SheetSize) -> Element<'static, Message> {
    container(
        column![
            text(format!("Sheet @ {edge:?} / {size:?}")).size(20),
            text(
                "This sheet is rendered by the framework. The edge \
                 controls which side it slides from; the size controls \
                 its extent along the perpendicular axis. Inside-facing \
                 corners are rounded by the engine."
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
        .title(|_: &App| String::from("snora — sheet"))
        .run()
}
