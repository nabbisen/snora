//! Size probe binary — baseline plus a representative `widgets` use (RFC-043).
//!
//! This crate exists only to measure binary size. See `scripts/measure-binary-size.sh`
//! and `docs/src/reference/binary-size-budget.md`.
//!
//! Shares its baseline application code with `size_probe_engine`, and adds
//! exactly one thing on top: a representative prefab-widget call
//! (`app_header` + `app_side_bar`, wired into the `AppLayout`) so the
//! `widgets` feature is actually *used*, not merely compiled in. The size
//! difference against `size_probe_engine` is the marginal cost of a
//! typical `widgets` adopter, which is what `widgets_diff_bytes` is
//! meant to track.
//!
//! Do not remove the `app_header`/`app_side_bar` calls to make this probe
//! match `size_probe_engine` again — that produces a `widgets_diff` of
//! ~0 regardless of the feature's real cost, because Rust's linker strips
//! code that compiles but is never called. See RFC-043 for the full
//! finding.

use iced::{
    Element, Length,
    widget::{column, container, text},
};
use snora::{
    AppLayout, LayoutDirection, SideBar, SideBarItem, render,
    widget::{app_header, app_side_bar},
};

#[derive(Debug, Clone)]
enum Message {
    #[allow(dead_code)]
    HeaderAction(snora::MenuAction<(), ()>),
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
            Message::HeaderAction(_) => {}
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
        let body: Element<'_, Message> = container(column![text("size probe").size(14),])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        render(AppLayout::new(body).header(header).side_bar(sidebar))
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("size probe"))
        .run()
}
