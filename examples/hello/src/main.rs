//! # Example: hello
//!
//! The absolute minimum snora application. Demonstrates:
//!
//! * How to wrap an `iced::Element` in an `AppLayout` and hand it to
//!   `snora::render`.
//! * That no trait, wrapper type, or overlay plumbing is required for a
//!   "just a body" application.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-hello
//! ```

use iced::{
    Element, Length,
    widget::{column, container, text},
};
use snora::{AppLayout, render};

// A degenerate message type. The hello app has no interactions.
#[derive(Debug, Clone)]
enum Message {}

#[derive(Default)]
struct App;

impl App {
    fn update(&mut self, _msg: Message) {}

    fn view(&self) -> Element<'_, Message> {
        // Build any iced element as the body. snora does not care what is
        // inside — it only needs something that is `Into<Element>`.
        let body: Element<'_, Message> = container(
            column![
                text("Hello, snora.").size(32),
                text("This is the smallest possible snora application.").size(14),
            ]
            .spacing(12),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();

        render(AppLayout::new(body))
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("snora — hello"))
        .run()
}
