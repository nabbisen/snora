//! # Example: responsive
//!
//! Demonstrates `snora::responsive_render` (RFC-046): the available
//! layout width, exposed to the application, with no threshold or
//! adaptive behavior prescribed by snora itself.
//!
//! This example picks its own threshold — **600 logical pixels** — and
//! its own response to crossing it — dropping the sidebar entirely below
//! that width. Both the number and what happens at it are this example's
//! decision, not snora's; a different application would reasonably pick
//! a different threshold, or change something else at it (font size,
//! column count, whatever fits).
//!
//! **Try it:** run this example and resize the window narrower than
//! ~600px — the sidebar disappears and the body message changes. Widen
//! it back past 600px and the sidebar returns.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-responsive
//! ```

use iced::{
    Element, Length,
    widget::{column, container, text},
};
use snora::{
    AppLayout, LayoutDirection, SideBar, SideBarItem, responsive_render,
    widget::{app_header, app_side_bar},
};

/// This example's own choice. snora exposes the width; it prescribes
/// nothing about what a "narrow" layout means.
const SIDEBAR_COLLAPSE_WIDTH: f32 = 600.0;

#[derive(Debug, Clone)]
enum Message {
    #[allow(dead_code)]
    HeaderAction(snora::MenuAction<(), ()>),
    SidebarItemPressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct View;

#[derive(Default)]
struct App;

impl App {
    fn update(&mut self, _msg: Message) {}

    fn view(&self) -> Element<'_, Message> {
        responsive_render(|width| {
            let header = app_header(
                "snora — responsive",
                Vec::<snora::Menu<(), ()>>::new(),
                &Message::HeaderAction,
                None,
                None,
                LayoutDirection::Ltr,
            );

            let is_narrow = width < SIDEBAR_COLLAPSE_WIDTH;

            let body: Element<'_, Message> = container(
                column![
                    text(format!("Available width: {width:.0}px")).size(20),
                    text(if is_narrow {
                        format!(
                            "Narrower than {SIDEBAR_COLLAPSE_WIDTH:.0}px — this example's own \
                             choice — so the sidebar is dropped. A different application might \
                             pick a different threshold, or change something else entirely."
                        )
                    } else {
                        format!(
                            "At or above {SIDEBAR_COLLAPSE_WIDTH:.0}px — sidebar shown. Resize \
                             the window narrower to see it drop."
                        )
                    })
                    .size(14),
                ]
                .spacing(12),
            )
            .padding(32)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

            let mut layout = AppLayout::new(body).header(header);

            if !is_narrow {
                let sidebar = app_side_bar(
                    SideBar {
                        items: vec![SideBarItem {
                            view_id: View,
                            icon: "🏠".into(),
                            tooltip: "Home".into(),
                            on_press: Message::SidebarItemPressed,
                        }],
                        active: View,
                    },
                    LayoutDirection::Ltr,
                );
                layout = layout.side_bar(sidebar);
            }

            layout
        })
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("snora — responsive"))
        .run()
}
