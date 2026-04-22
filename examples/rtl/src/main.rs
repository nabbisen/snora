//! # Example: rtl
//!
//! Flip the whole application between left-to-right and right-to-left
//! layout in one click. Demonstrates:
//!
//! * `LayoutDirection::{Ltr, Rtl}` as the single framework-level switch.
//! * `snora::direction::row_dir` for direction-aware custom rows inside
//!   your own widgets.
//! * That built-in snora widgets (`app_header`, `app_side_bar`) mirror
//!   automatically when the same direction is passed in.
//! * That toast anchor position flips with direction — the framework
//!   handles this without any application code change.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-rtl
//! ```

use std::time::Instant;

use iced::{
    Alignment::Center,
    Element, Length, Subscription, Task,
    widget::{button, column, container, space, text},
};
use snora::{
    AppLayout, LayoutDirection, SideBar, SideBarItem, Toast, ToastIntent,
    direction::row_dir,
    render,
    widget::{app_footer, app_header, app_side_bar},
};

#[derive(Debug, Clone)]
enum Message {
    Flip,
    ShowToast,
    Dismiss(u64),
    ToastTick,
    // Header has no menus here, so MenuAction uses `()` on both sides.
    // The payload is unused but the variant shape must match
    // `app_header`'s callback signature.
    #[allow(dead_code)]
    HeaderAction(snora::MenuAction<(), ()>),
}

struct App {
    direction: LayoutDirection,
    toasts: Vec<Toast<Message>>,
    next_id: u64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            direction: LayoutDirection::Ltr,
            toasts: Vec::new(),
            next_id: 0,
        }
    }
}

impl App {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Flip => self.direction = self.direction.flipped(),
            Message::ShowToast => {
                let id = self.next_id;
                self.next_id += 1;
                self.toasts.push(Toast::new(
                    id,
                    ToastIntent::Info,
                    "RTL demo",
                    "Toast anchor tracks the layout direction.",
                    Message::Dismiss(id),
                ));
            }
            Message::Dismiss(id) => self.toasts.retain(|t| t.id != id),
            Message::ToastTick => {
                snora::toast::sweep_expired(&mut self.toasts, Instant::now());
            }
            Message::HeaderAction(_) => {}
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        snora::toast::subscription(&self.toasts, || Message::ToastTick)
    }

    fn view(&self) -> Element<'_, Message> {
        // Header — title on the start edge, a "Flip" button on the end
        // edge. The start / end resolution is done inside `app_header`
        // based on the direction we pass.
        let end_controls: Element<'_, Message> = button(text("Flip LTR ↔ RTL").size(12))
            .on_press(Message::Flip)
            .into();

        let header = app_header(
            "snora — rtl",
            Vec::<snora::Menu<(), ()>>::new(),
            &Message::HeaderAction,
            None,
            Some(end_controls),
            self.direction,
        );

        // Sidebar — also direction-aware. Under Rtl it appears on the
        // right side of the body row automatically.
        let sidebar = app_side_bar(
            SideBar {
                items: vec![
                    SideBarItem {
                        view_id: (),
                        icon: "✦".into(),
                        tooltip: "Demo".into(),
                        on_press: Message::ShowToast,
                    },
                ],
                active: (),
            },
            self.direction,
        );

        // Body — use `row_dir` for a direction-aware two-column layout
        // inside the body. "Start" is where the reader begins; "End"
        // is where the reader ends.
        let start_card = card("START", "This block sits at the logical start edge.");
        let end_card = card("END", "This block sits at the logical end edge.");

        let body_row = row_dir(self.direction, start_card, end_card)
            .spacing(16)
            .align_y(Center);

        let body = container(
            column![
                text(format!("Current direction: {:?}", self.direction)).size(22),
                text(
                    "Click the header button (or the sidebar icon) to flip \
                     the layout. Header, sidebar, body row below, and toast \
                     anchor all mirror together.",
                )
                .size(13),
                space().height(Length::Fixed(16.0)),
                body_row,
                space().height(Length::Fixed(16.0)),
                button(text("Show a toast").size(13)).on_press(Message::ShowToast),
            ]
            .spacing(8),
        )
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill);

        // Footer — another direction-aware row, built by hand.
        let footer = app_footer(
            row_dir(
                self.direction,
                text(format!("direction = {:?}", self.direction)).size(12),
                text(format!("toasts queued: {}", self.toasts.len())).size(12),
            )
            .align_y(Center)
            .spacing(12)
            .into(),
        );

        let layout = AppLayout::new(body.into())
            .header(header)
            .side_bar(sidebar)
            .footer(footer)
            .direction(self.direction)
            .toasts(self.toasts.clone());

        render(layout)
    }
}

fn card(title: &str, body: &str) -> Element<'static, Message> {
    container(
        column![
            text(title.to_string()).size(16),
            text(body.to_string()).size(13),
        ]
        .spacing(6),
    )
    .padding(16)
    .width(Length::Fixed(260.0))
    .style(|theme: &iced::Theme| {
        let ep = theme.extended_palette();
        container::Style {
            background: Some(iced::Background::Color(ep.background.weak.color)),
            text_color: Some(ep.background.base.text),
            border: iced::Border {
                radius: 8.0.into(),
                width: 1.0,
                color: ep.background.strong.color,
            },
            ..Default::default()
        }
    })
    .into()
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("snora — rtl"))
        .subscription(App::subscription)
        .run()
}
