//! Size probe binary — widgets baseline plus a representative `design` use (RFC-043).
//!
//! This crate exists only to measure binary size. See `scripts/measure-binary-size.sh`
//! and `docs/src/reference/binary-size-budget.md`.
//!
//! Shares its baseline with `size_probe_widgets` (same `app_header` /
//! `app_side_bar` use) and adds exactly one thing on top: a representative
//! `snora::design::*` use — one `design::button::primary` and one
//! `design::style::container::card_surface`, against `Tokens::light()` —
//! so the `design` feature is actually *used*, not merely compiled in. The
//! size difference against `size_probe_widgets` is the marginal cost of a
//! typical `design` adopter, which is what `design_diff_bytes` is meant to
//! track.
//!
//! Do not remove the design calls to make this probe match
//! `size_probe_widgets` again — that produces a `design_diff` of ~0
//! regardless of the feature's real cost, because Rust's linker strips
//! code that compiles but is never called. See RFC-043 for the full
//! finding.

use iced::{Element, Length, widget::{column, container, text}};
use snora::{
    AppLayout, LayoutDirection, SideBar, SideBarItem, render,
    widget::{app_header, app_side_bar},
    design::{Tokens, button, style::container as design_container},
};

#[derive(Debug, Clone)]
enum Message {
    #[allow(dead_code)]
    HeaderAction(snora::MenuAction<(), ()>),
    Noop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewId {
    Home,
}

#[derive(Default)]
struct App;

impl App {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::HeaderAction(_) | Message::Noop => {}
        }
    }
    fn view(&self) -> Element<'_, Message> {
        let header = app_header(
            "size probe",
            Vec::<snora::Menu<(), ()>>::new(),
            &Message::HeaderAction,
            None,
            None,
            LayoutDirection::Ltr,
        );
        let sidebar = app_side_bar(
            SideBar {
                items: vec![SideBarItem {
                    view_id: ViewId::Home,
                    icon: "🏠".into(),
                    tooltip: "Home".into(),
                    on_press: Message::HeaderAction(snora::MenuAction::MenuPressed(())),
                }],
                active: ViewId::Home,
            },
            LayoutDirection::Ltr,
        );

        let tokens = Tokens::light();
        let card: Element<'_, Message> = container(
            column![
                text("size probe").size(14),
                button::primary(&tokens, "Go", Message::Noop),
            ],
        )
        .style(move |_theme| design_container::card_surface(&tokens))
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        render(AppLayout::new(card).header(header).side_bar(sidebar))
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("size probe"))
        .run()
}
