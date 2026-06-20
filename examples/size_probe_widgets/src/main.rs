//! Size probe binary — identical app logic, different snora feature set.
//!
//! This crate exists only to measure binary size. See `scripts/measure-binary-size.sh`
//! and `docs/src/reference/binary-size-budget.md`.
//!
//! Do not add any logic here — the point is that all three probes
//! (engine / widgets / design) contain the same application code so that
//! `widgets_diff` and `design_diff` are purely the marginal cost of
//! the respective feature, not of different application logic.

use iced::{Element, Length, widget::{column, container, text}};
use snora::{AppLayout, render};

#[derive(Debug, Clone)]
enum Message {}

#[derive(Default)]
struct App;

impl App {
    fn update(&mut self, _msg: Message) {}
    fn view(&self) -> Element<'_, Message> {
        let body: Element<'_, Message> = container(
            column![
                text("size probe").size(14),
            ],
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
        render(AppLayout::new(body))
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("size probe"))
        .run()
}
