//! # Example: toast
//!
//! Covers the full toast UX vocabulary in one screen:
//!
//! * All five [`ToastIntent`] values (Debug / Info / Success / Warning /
//!   Error) and how they map to theme colors.
//! * All three lifetime policies:
//!     - Default (4 s auto-dismiss),
//!     - Custom duration (per-toast override),
//!     - Persistent (manual dismiss only).
//! * The framework's TTL subscription + `sweep_expired` pair — the
//!   application only writes two one-liners and stores a `Vec<Toast<_>>`.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-toast
//! ```

use std::time::Instant;

use iced::{
    Alignment::Center,
    Element, Length, Subscription, Task,
    widget::{button, column, container, row, space, text},
};
use snora::{AppLayout, Toast, ToastIntent, ToastLifetime, render};

#[derive(Debug, Clone)]
enum Message {
    /// Show a default-duration toast with the given intent.
    ShowDefault(ToastIntent),
    /// Show a toast whose duration is overridden at call time.
    ShowWithDuration(ToastIntent, u64), // millis
    /// Show a persistent toast — dismissable only by the close button.
    ShowPersistent(ToastIntent),
    /// User clicked a toast's close button.
    Dismiss(u64),
    /// Periodic tick from the framework's toast subscription.
    ToastTick,
}

#[derive(Default)]
struct App {
    toasts: Vec<Toast<Message>>,
    next_id: u64,
}

impl App {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::ShowDefault(intent) => {
                let id = self.issue_id();
                self.toasts.push(
                    Toast::new(
                        id,
                        intent,
                        format!("{intent}"),
                        "Default lifetime — auto-dismisses in 4 seconds.",
                        Message::Dismiss(id),
                    ),
                );
            }
            Message::ShowWithDuration(intent, millis) => {
                let id = self.issue_id();
                self.toasts.push(
                    Toast::new(
                        id,
                        intent,
                        format!("{intent}"),
                        format!("Custom lifetime — {millis} ms."),
                        Message::Dismiss(id),
                    )
                    .with_lifetime(ToastLifetime::millis(millis)),
                );
            }
            Message::ShowPersistent(intent) => {
                let id = self.issue_id();
                self.toasts.push(
                    Toast::new(
                        id,
                        intent,
                        format!("{intent}"),
                        "Persistent — close manually.",
                        Message::Dismiss(id),
                    )
                    .persistent(),
                );
            }
            Message::Dismiss(id) => {
                self.toasts.retain(|t| t.id != id);
            }
            Message::ToastTick => {
                // This is the entire TTL bookkeeping — framework-owned.
                snora::toast::sweep_expired(&mut self.toasts, Instant::now());
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Subscribe to sweep ticks only while transient toasts exist.
        // When the queue is empty or all-persistent, this returns
        // `Subscription::none()` automatically.
        snora::toast::subscription(&self.toasts, || Message::ToastTick)
    }

    fn view(&self) -> Element<'_, Message> {
        let body = container(
            column![
                text("Toast gallery").size(28),
                text(
                    "Click any button to enqueue a toast. The bottom-right \
                     stack is painted by the framework; lifetime is managed \
                     by the framework's subscription + sweep helpers.",
                )
                .size(13),
                space().height(Length::Fixed(12.0)),
                section_label("Default lifetime (4 s auto-dismiss)"),
                intent_row(Message::ShowDefault),
                space().height(Length::Fixed(12.0)),
                section_label("Custom lifetime"),
                row![
                    labeled_button("1 s Info", Message::ShowWithDuration(ToastIntent::Info, 1_000)),
                    labeled_button(
                        "10 s Success",
                        Message::ShowWithDuration(ToastIntent::Success, 10_000),
                    ),
                ]
                .spacing(8),
                space().height(Length::Fixed(12.0)),
                section_label("Persistent (manual dismiss)"),
                row![
                    labeled_button(
                        "Warning",
                        Message::ShowPersistent(ToastIntent::Warning),
                    ),
                    labeled_button("Error", Message::ShowPersistent(ToastIntent::Error)),
                ]
                .spacing(8),
                space().height(Length::Fixed(16.0)),
                text(format!(
                    "Currently queued: {} toast(s)",
                    self.toasts.len()
                ))
                .size(12),
            ]
            .spacing(8),
        )
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill);

        // Hand the vec straight to the framework. snora does not mutate it.
        let layout = AppLayout::new(body.into()).toasts(self.toasts.clone());
        render(layout)
    }

    fn issue_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

fn section_label<'a>(s: &'a str) -> Element<'a, Message> {
    text(s).size(16).into()
}

fn intent_row<F>(make_msg: F) -> Element<'static, Message>
where
    F: Fn(ToastIntent) -> Message,
{
    row![
        labeled_button("Debug", make_msg(ToastIntent::Debug)),
        labeled_button("Info", make_msg(ToastIntent::Info)),
        labeled_button("Success", make_msg(ToastIntent::Success)),
        labeled_button("Warning", make_msg(ToastIntent::Warning)),
        labeled_button("Error", make_msg(ToastIntent::Error)),
    ]
    .spacing(8)
    .align_y(Center)
    .into()
}

fn labeled_button(label: &str, msg: Message) -> Element<'static, Message> {
    button(text(label.to_string()).size(13)).on_press(msg).into()
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("snora — toast"))
        .subscription(App::subscription)
        .run()
}
