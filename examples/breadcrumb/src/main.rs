//! # Example: breadcrumb
//!
//! Focused demo of [`app_breadcrumb`] — ancestor-level navigation
//! showing the user's current depth in a hierarchy.
//!
//! Demonstrates:
//!
//! * Ancestor crumbs ([`Crumb::ancestor`]) rendered as clickable text.
//! * Leaf crumb ([`Crumb::leaf`]) marking the current page — plain
//!   text, non-clickable.
//! * Direction-aware order *and* separator glyph: `›` under LTR,
//!   `‹` under RTL.
//!
//! For tab (peer-level) navigation see
//! `cargo run -p snora-example-tab`.
//!
//! Run with:
//!
//! ```text
//! cargo run -p snora-example-breadcrumb
//! ```

use iced::{
    Element, Length,
    widget::{button, column, container, row, text},
};
use snora::{
    AppLayout, BreadcrumbAction, Crumb, LayoutDirection, render, widget::app_breadcrumb,
};

/// A small enum modeling a hierarchy: Home → Library → Books → a
/// particular book. Each level can be the user's current page (the
/// leaf) or an ancestor (the user is somewhere deeper).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrumbId {
    Home,
    Library,
    Books,
    TheHobbit,
}

#[derive(Debug, Clone)]
enum Message {
    Breadcrumb(BreadcrumbAction<CrumbId>),
    NavigateTo(CrumbId),
    SetDirection(LayoutDirection),
}

struct App {
    location: CrumbId,
    direction: LayoutDirection,
}

impl Default for App {
    fn default() -> Self {
        Self {
            location: CrumbId::TheHobbit,
            direction: LayoutDirection::default(),
        }
    }
}

impl App {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::Breadcrumb(BreadcrumbAction::Pressed(id))
            | Message::NavigateTo(id) => self.location = id,
            Message::SetDirection(d) => self.direction = d,
        }
    }

    /// Build the trail from the root down to the current location.
    /// Mark the last entry as the leaf.
    fn current_trail(&self) -> Vec<Crumb<CrumbId>> {
        let trail_ids: Vec<CrumbId> = match self.location {
            CrumbId::Home => vec![CrumbId::Home],
            CrumbId::Library => vec![CrumbId::Home, CrumbId::Library],
            CrumbId::Books => vec![CrumbId::Home, CrumbId::Library, CrumbId::Books],
            CrumbId::TheHobbit => vec![
                CrumbId::Home,
                CrumbId::Library,
                CrumbId::Books,
                CrumbId::TheHobbit,
            ],
        };
        let last = trail_ids.len().saturating_sub(1);
        trail_ids
            .into_iter()
            .enumerate()
            .map(|(i, id)| {
                let label = label_for(id);
                if i == last {
                    Crumb::leaf(id, label)
                } else {
                    Crumb::ancestor(id, label)
                }
            })
            .collect()
    }

    fn view(&self) -> Element<'_, Message> {
        let crumbs = self.current_trail();

        let body = container(
            column![
                app_breadcrumb(crumbs, &Message::Breadcrumb, self.direction),
                container(
                    column![
                        text(format!("Current page: {}", label_for(self.location))).size(20),
                        text(
                            "Click any ancestor in the trail above to \
                             navigate up. The last entry (the leaf) is \
                             rendered as plain text and does not respond \
                             to clicks.",
                        )
                        .size(13),
                        text("Jump to a different depth").size(16),
                        row![
                            button(text("Home").size(13))
                                .on_press(Message::NavigateTo(CrumbId::Home)),
                            button(text("Library").size(13))
                                .on_press(Message::NavigateTo(CrumbId::Library)),
                            button(text("Books").size(13))
                                .on_press(Message::NavigateTo(CrumbId::Books)),
                            button(text("The Hobbit").size(13))
                                .on_press(Message::NavigateTo(CrumbId::TheHobbit)),
                        ]
                        .spacing(8),
                        text("Direction").size(16),
                        row![
                            button(text("LTR").size(13))
                                .on_press(Message::SetDirection(LayoutDirection::Ltr)),
                            button(text("RTL").size(13))
                                .on_press(Message::SetDirection(LayoutDirection::Rtl)),
                        ]
                        .spacing(8),
                        text(format!(
                            "(currently {:?} — separator glyph flips)",
                            self.direction
                        ))
                        .size(12),
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

fn label_for(id: CrumbId) -> &'static str {
    match id {
        CrumbId::Home => "Home",
        CrumbId::Library => "Library",
        CrumbId::Books => "Books",
        CrumbId::TheHobbit => "The Hobbit",
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("snora — breadcrumb"))
        .run()
}
