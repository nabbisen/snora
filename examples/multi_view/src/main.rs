//! # Example: multi_view
//!
//! Sidebar-driven view switching — the canonical "one active page at a
//! time" pattern. Demonstrates:
//!
//! * `SideBar` + `SideBarItem` with an application-defined `ViewId` enum.
//! * How each view is a plain function returning `Element<'_, Message>`.
//!   snora does not impose a "page trait" on you.
//! * Local per-view state (search query, counter) living directly on
//!   the App struct without any dispatcher / reducer boilerplate.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-multi-view
//! ```

use iced::{
    Alignment::Center,
    Element, Length, Padding,
    widget::{button, column, container, row, space, text, text_input},
};
use snora::{
    AppLayout, LayoutDirection, SideBar, SideBarItem,
    render, widget::app_side_bar,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewId {
    Home,
    Search,
    Counter,
}

#[derive(Debug, Clone)]
enum Message {
    Switch(ViewId),

    // Search view
    SearchChanged(String),
    SubmitSearch,

    // Counter view
    Increment,
    Decrement,
}

struct App {
    active: ViewId,

    // Search-view state
    query: String,
    last_query: Option<String>,

    // Counter-view state
    count: i32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            active: ViewId::Home,
            query: String::new(),
            last_query: None,
            count: 0,
        }
    }
}

impl App {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::Switch(v) => self.active = v,
            Message::SearchChanged(s) => self.query = s,
            Message::SubmitSearch => {
                let q = self.query.trim();
                if !q.is_empty() {
                    self.last_query = Some(q.to_string());
                }
            }
            Message::Increment => self.count += 1,
            Message::Decrement => self.count -= 1,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let side_bar = app_side_bar(
            SideBar {
                items: vec![
                    SideBarItem {
                        view_id: ViewId::Home,
                        icon: "🏠".into(),
                        tooltip: "Home".into(),
                        on_press: Message::Switch(ViewId::Home),
                    },
                    SideBarItem {
                        view_id: ViewId::Search,
                        icon: "🔍".into(),
                        tooltip: "Search".into(),
                        on_press: Message::Switch(ViewId::Search),
                    },
                    SideBarItem {
                        view_id: ViewId::Counter,
                        icon: "#".into(),
                        tooltip: "Counter".into(),
                        on_press: Message::Switch(ViewId::Counter),
                    },
                ],
                active: self.active,
            },
            LayoutDirection::Ltr,
        );

        // Dispatch to the active view — each is just a plain function.
        let body = match self.active {
            ViewId::Home => home_view(),
            ViewId::Search => search_view(&self.query, self.last_query.as_deref()),
            ViewId::Counter => counter_view(self.count),
        };

        render(AppLayout::new(body).side_bar(side_bar))
    }
}

fn home_view() -> Element<'static, Message> {
    container(
        column![
            text("Home").size(28),
            text(
                "Pick a view from the left rail. Each view is a separate \
                 function — no page trait, no dispatcher enum. snora \
                 composes them by putting whatever Element you return \
                 into the body slot.",
            )
            .size(14),
        ]
        .spacing(8),
    )
    .padding(Padding::from([24.0, 32.0]))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn search_view<'a>(query: &'a str, last: Option<&'a str>) -> Element<'a, Message> {
    let input = text_input("query…", query)
        .on_input(Message::SearchChanged)
        .on_submit(Message::SubmitSearch)
        .padding(8);

    let submit = button(text("Submit").size(13)).on_press(Message::SubmitSearch);

    let last_line = match last {
        Some(q) => format!("Last submitted: “{q}”"),
        None => "Last submitted: (nothing yet)".into(),
    };

    container(
        column![
            text("Search").size(28),
            text("Type and press Enter, or click Submit.").size(14),
            space().height(Length::Fixed(12.0)),
            row![container(input).width(Length::Fill), submit]
                .spacing(8)
                .align_y(Center),
            text(last_line).size(12),
        ]
        .spacing(8),
    )
    .padding(Padding::from([24.0, 32.0]))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn counter_view(count: i32) -> Element<'static, Message> {
    container(
        column![
            text("Counter").size(28),
            text("A trivial counter — state lives on the App struct.").size(14),
            space().height(Length::Fixed(12.0)),
            row![
                button(text("−").size(20)).on_press(Message::Decrement),
                text(format!("{count}")).size(24),
                button(text("+").size(20)).on_press(Message::Increment),
            ]
            .spacing(12)
            .align_y(Center),
        ]
        .spacing(8),
    )
    .padding(Padding::from([24.0, 32.0]))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("snora — multi view"))
        .run()
}
