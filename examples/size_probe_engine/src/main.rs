//! Size probe binary — shared baseline, no feature use (RFC-043).
//!
//! This crate exists only to measure binary size. See `scripts/measure-binary-size.sh`
//! and `docs/src/reference/binary-size-budget.md`.
//!
//! This is the baseline: it shares its application code with
//! `size_probe_widgets` and `size_probe_design`, but calls **neither**
//! `snora::widget::*` nor `snora::design::*`. `size_probe_widgets` and
//! `size_probe_design` each add a minimal, representative use of the
//! feature they measure on top of this same baseline, so the size
//! difference is the marginal cost of *using* that feature — not of
//! differing application logic.
//!
//! Do not make this probe identical to the other two by *removing* their
//! feature use to "simplify" — that was the RFC-043 defect. Rust's linker
//! strips code that compiles but is never called, so three probes that
//! never call the feature they're named for all measure ~0 regardless of
//! feature, which is not a marginal cost, just dead-code elimination.

use iced::{
    Element, Length,
    widget::{column, container, text},
};
use snora::{AppLayout, render};

#[derive(Debug, Clone)]
enum Message {}

#[derive(Default)]
struct App;

impl App {
    fn update(&mut self, _msg: Message) {}
    fn view(&self) -> Element<'_, Message> {
        let body: Element<'_, Message> = container(column![text("size probe").size(14),])
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
