//! # Example: tab
//!
//! Focused demo of [`app_tab_bar`] — peer-level horizontal navigation
//! placed below the header.
//!
//! Demonstrates:
//!
//! * Multiple tabs with application-defined `TabId` (a small enum).
//! * Active-tab underline drawn from the theme's primary color.
//! * Direction-aware ordering: toggling LTR/RTL flips the entire tab
//!   row.
//!
//! For breadcrumb (ancestor-level) navigation see
//! `cargo run -p snora-example-breadcrumb`.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-tab
//! ```

use iced::{
    Element, Length,
    widget::{button, column, container, row, text},
};
use snora::{
    AppLayout, LayoutDirection, Tab, TabAction, TabBar, render, widget::app_tab_bar,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspaceTab {
    Library,
    Editor,
    Settings,
    History,
}

#[derive(Debug, Clone)]
enum Message {
    TabAction(TabAction<WorkspaceTab>),
    SetDirection(LayoutDirection),
}

struct App {
    active_tab: WorkspaceTab,
    direction: LayoutDirection,
}

impl Default for App {
    fn default() -> Self {
        Self {
            active_tab: WorkspaceTab::Library,
            direction: LayoutDirection::default(),
        }
    }
}

impl App {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::TabAction(TabAction::Pressed(id)) => self.active_tab = id,
            Message::SetDirection(d) => self.direction = d,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let tabs = TabBar {
            tabs: vec![
                Tab {
                    id: WorkspaceTab::Library,
                    label: "Library".into(),
                    icon: None,
                },
                Tab {
                    id: WorkspaceTab::Editor,
                    label: "Editor".into(),
                    icon: None,
                },
                Tab {
                    id: WorkspaceTab::Settings,
                    label: "Settings".into(),
                    icon: None,
                },
                Tab {
                    id: WorkspaceTab::History,
                    label: "History".into(),
                    icon: None,
                },
            ],
            active: self.active_tab,
        };

        let body = container(
            column![
                app_tab_bar(tabs, &Message::TabAction, self.direction),
                container(
                    column![
                        text(format!("Active tab: {:?}", self.active_tab)).size(20),
                        text(
                            "Click any tab in the strip above to switch. \
                             The active tab gets a colored underline drawn \
                             from the theme's primary palette.",
                        )
                        .size(13),
                        text("Direction").size(16),
                        row![
                            button(text("LTR").size(13))
                                .on_press(Message::SetDirection(LayoutDirection::Ltr)),
                            button(text("RTL").size(13))
                                .on_press(Message::SetDirection(LayoutDirection::Rtl)),
                        ]
                        .spacing(8),
                        text(format!("(currently {:?})", self.direction)).size(12),
                    ]
                    .spacing(12),
                )
                .padding(24),
            ]
            .spacing(0),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        let layout = AppLayout::new(body.into()).direction(self.direction);
        render(layout)
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("snora — tab"))
        .run()
}
