//! Size probe binary — engine baseline plus a representative
//! `design::render` use, with NO `widgets` feature (RFC-055).
//!
//! Measures the configuration RFC-054/RFC-055 exist to create: a
//! design-path consumer with zero `snora::widget::*` call sites. Shares
//! its baseline with `size_probe_engine` (same minimal body) and adds
//! exactly one thing on top: a `Dialog`, rendered through
//! `design::render` instead of `render`, so the styled card path
//! (`card_raised`, now from `snora-style`) is actually exercised — not
//! merely compiled in. Per RFC-043, an unexercised feature is stripped
//! by the linker and measures ~0 regardless of real cost.
//!
//! See `docs/src/reference/binary-size-budget.md` and
//! `scripts/measure-binary-size.sh`.

use iced::{
    Element, Length,
    widget::{column, container, text},
};
use snora::{
    AppLayout, Dialog,
    design::{Tokens, render},
};

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
        let dialog: Dialog<Element<'_, Message>, Message> =
            Dialog::new(text("dialog").size(14).into());
        let tokens = Tokens::light();
        render(AppLayout::new(body).dialog(dialog), &tokens)
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("size probe"))
        .run()
}
