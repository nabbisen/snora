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
//! * **Keyboard access** (RFC-116): Tab and Shift+Tab move focus between
//!   the tabs, and Enter or Space presses the focused one. iced binds no
//!   Tab of its own, so the application routes the key with
//!   [`snora::keyboard::traverse`] — the `Message::KeyPressed` arm below.
//!   Tab at the last tab leaves the strip rather than wrapping to the
//!   first, which is iced's `focus_next`.
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
    Element, Length, Subscription, Task, keyboard,
    widget::{button, column, container, row, text},
};
use snora::{AppLayout, LayoutDirection, Tab, TabAction, TabBar, render, widget::app_tab_bar};

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
    KeyPressed(keyboard::Key, keyboard::Modifiers),
    /// A key event this example does not use.
    Ignored,
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
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::TabAction(TabAction::Pressed(id)) => self.active_tab = id,
            Message::SetDirection(d) => self.direction = d,
            // iced moves focus only when something runs `focus_next`;
            // `traverse` is that something, for Tab and Shift+Tab.
            Message::KeyPressed(key, modifiers) => {
                if let Some(task) = snora::keyboard::traverse(key, modifiers) {
                    return task;
                }
            }
            Message::Ignored => {}
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. } => {
                Message::KeyPressed(key, modifiers)
            }
            _ => Message::Ignored,
        })
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
                            "Click any tab in the strip above to switch, or press Tab to \
                             move focus along it and Enter or Space to \
                             choose. The active tab gets a colored \
                             underline drawn from the theme's primary \
                             palette.",
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
        .subscription(App::subscription)
        .run()
}
