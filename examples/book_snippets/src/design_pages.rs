//! Anchored source for the `docs/src/design/` pages' fragments (RFC-136).
//!
//! Each fragment is the body of a method of the application (`scaffold::App`), outside the anchor the
//! method's signature and the names the fragment takes for granted (`self`, `Message`, `direction`).

use crate::scaffold::*;
use iced::Element;
use snora::LayoutDirection;

impl App {
    #[rustfmt::skip]
    fn buttons_variants(&self, direction: LayoutDirection) {
// ANCHOR: buttons_variants
use snora::design::{Tokens, button};

// In your app state, store tokens:
struct App { tokens: Tokens }

// In view():
let t = &self.tokens;

button::primary(t, "Save", Message::Save);
button::secondary(t, "Cancel", Message::Cancel);
button::ghost(t, "Skip", Message::Skip);
button::link(t, "Choose another folder", Message::ChooseFolder);
button::danger(t, "Delete", Message::Delete);
// ANCHOR_END: buttons_variants
}
}

impl App {
    #[rustfmt::skip]
    fn buttons_maybe(&self) {
let t = &self.tokens;
use snora::design::button;
// ANCHOR: buttons_maybe
button::primary_maybe(t, "Save", self.can_save.then_some(Message::Save));
button::danger_maybe(t, "Delete", self.can_delete.then_some(Message::Delete));
// ANCHOR_END: buttons_maybe
}
}

impl App {
    #[rustfmt::skip]
    fn buttons_iced_button(&self, my_custom_content: Element<'static, Message>) -> impl Into<Element<'_, Message>> {
// ANCHOR: buttons_iced_button
use snora::design::style;
use iced::widget::button;

let t = &self.tokens;
let tok = t.clone();           // clone into the closure

button(my_custom_content)
    .on_press(Message::Save)
    .style(move |_theme, status| style::button::primary(&tok, status))
// ANCHOR_END: buttons_iced_button
}
}

impl App {
    #[rustfmt::skip]
    fn buttons_control(&self, my_custom_content: Element<'static, Message>) -> impl Into<Element<'_, Message>> {
use snora::design::style;
let tok = self.tokens.clone();
// ANCHOR: buttons_control
snora::design::control(&self.tokens, my_custom_content)
    .on_press(Message::Save)
    .style(move |_theme, status| style::button::primary(&tok, status))
// ANCHOR_END: buttons_control
}
}

impl App {
    #[rustfmt::skip]
    fn cards_variants(&self) -> impl Into<Element<'_, Message>> {
// ANCHOR: cards_variants
use snora::design::{Tokens, card};
use iced::widget::{column, text};

let t = &self.tokens;

column![
    card::surface(t, text("Standard card content")),
    card::raised(t, text("Floating panel content")),
    card::selected(t, text("Currently active item")),
]
// ANCHOR_END: cards_variants
}
}

impl App {
    #[rustfmt::skip]
    fn cards_composition(&self) -> impl Into<Element<'_, Message>> {
use snora::design::{card, style, button};
let t = &self.tokens;
// ANCHOR: cards_composition
use iced::widget::{column, text};

card::surface(t, column![
    text("Title").size(style::text::title_size(t)),
    text("Body text goes here."),
    button::secondary(t, "Action", Message::DoSomething),
])
// ANCHOR_END: cards_composition
}
}

impl App {
    #[rustfmt::skip]
    fn cards_selected(&self, item: &Item, item_content: Element<'static, Message>) -> impl Into<Element<'_, Message>> {
use snora::design::card;
let t = &self.tokens;
// ANCHOR: cards_selected
let card_fn = if self.selected_id == Some(item.id) {
    card::selected
} else {
    card::surface
};
card_fn(t, item_content)
// ANCHOR_END: cards_selected
}
}

impl App {
    #[rustfmt::skip]
    fn cards_override(&self, my_content: Element<'static, Message>) -> impl Into<Element<'_, Message>> {
let t = &self.tokens;
// ANCHOR: cards_override
use snora::design::style;
use iced::widget::container;

let tok = t.clone();
container(my_content)
    .padding(0)  // override padding
    .style(move |_| style::container::card_surface(&tok))
// ANCHOR_END: cards_override
}
}

impl App {
    #[rustfmt::skip]
    fn chips_filter(&self) {
let tokens = self.tokens.clone();
// ANCHOR: chips_filter
use snora::design::chip;

chip::filter(&tokens, "Draft",   self.show_drafts, Message::ToggleDrafts);
chip::filter(&tokens, "Active",  self.show_active, Message::ToggleActive);
// ANCHOR_END: chips_filter
}
}

impl App {
    #[rustfmt::skip]
    fn chips_removable(&self, direction: LayoutDirection) -> impl Into<Element<'_, Message>> {
use snora::design::chip;
let tokens = self.tokens.clone();
// ANCHOR: chips_removable
chip::removable(
    &tokens,
    "Tag: Rust",
    self.rust_selected,
    Message::ToggleRust,
    Message::RemoveRust,
    direction,          // 0.64: the label then the × in reading order
)
// ANCHOR_END: chips_removable
}
}

impl App {
    #[rustfmt::skip]
    fn chips_loop(&self) {
use snora::design::chip;
let tokens = self.tokens.clone();
// ANCHOR: chips_loop
for tag in &self.active_tags {
    chip::filter(&tokens, tag.as_str(), true, Message::RemoveTag(tag.clone()));
}
// ANCHOR_END: chips_loop
}
}

#[rustfmt::skip]
fn chrome_header() {
let on_menu_action = |_: snora::MenuAction<(), ()>| Message::Save;
// ANCHOR: chrome_header
use snora::{AppLayout, LayoutDirection, widget::app_header};
use snora::design::{Tokens, widget};

let tokens = Tokens::light();
let styled_header = widget::app_header(
    &tokens, "My App", vec![], &on_menu_action, None, None, LayoutDirection::Ltr,
);
// ANCHOR_END: chrome_header
}

#[rustfmt::skip]
fn engine_render(layout: snora::AppLayout<Element<'static, Message>, Message>) {
// ANCHOR: engine_render
use snora::design::{Tokens, render};

let tokens = Tokens::light();
let element = render(layout, &tokens);
// ANCHOR_END: engine_render
}

impl App {
    #[rustfmt::skip]
    fn notices_builder(&self, direction: LayoutDirection) -> impl Into<Element<'_, Message>> {
let tokens = &self.tokens;
// ANCHOR: notices_builder
use snora::design::{Tokens, Tone, notice::Notice};

Notice::new(&tokens, Tone::Info, "Background sync is running.", direction)
    .title("Syncing")                               // optional
    .action("View details", Message::OpenSync)      // optional
    .dismiss(Message::DismissNotice)                // optional
    .render()
// ANCHOR_END: notices_builder
}
}

impl App {
    #[rustfmt::skip]
    fn notices_conditional(&self, direction: LayoutDirection) -> Element<'_, Message> {
use snora::design::{Tone, notice::Notice};
let tokens = &self.tokens;
// ANCHOR: notices_conditional
if self.show_notice {
    Notice::new(&tokens, Tone::Warning, "Index is stale.", direction)
        .dismiss(Message::DismissNotice)
        .render()
} else {
    iced::widget::space().into()
}
// ANCHOR_END: notices_conditional
}
}

impl App {
    #[rustfmt::skip]
    fn progress_variants(&self, direction: LayoutDirection) -> impl Into<Element<'_, Message>> {
let tokens = &self.tokens;
// ANCHOR: progress_variants
use snora::design::{Tone, progress};
use iced::widget::column;

column![
    // Compact inline (no outer card)
    progress::row(&tokens, "Indexing files", Some(0.6), Tone::Accent, direction),

    // Wrapped in a card::surface for visual prominence
    progress::card(&tokens, "Uploading", Some(1.0), Tone::Success, direction),

    // Indeterminate — pass None
    progress::row(&tokens, "Connecting…", None, Tone::Info, direction),
]
// ANCHOR_END: progress_variants
}
}

impl App {
    #[rustfmt::skip]
    fn v021_notice(&self, direction: LayoutDirection) -> impl Into<Element<'_, Message>> {
let tokens = &self.tokens;
// ANCHOR: v021_notice
use snora::design::{Tokens, Tone, notice::Notice};

Notice::new(&tokens, Tone::Warning, "Disk space low.", direction)
    .title("Storage warning")
    .action("Free space", Message::FreeSpace)
    .dismiss(Message::DismissNotice)
    .render()
// ANCHOR_END: v021_notice
}
}

impl App {
    #[rustfmt::skip]
    fn v021_chips(&self, direction: LayoutDirection) {
let tokens = self.tokens.clone();
// ANCHOR: v021_chips
use snora::design::chip;

chip::filter(&tokens, "Draft", self.show_drafts, Message::ToggleDrafts);
chip::removable(&tokens, "Tag: Rust", true, Message::Toggle, Message::Remove, direction);
// ANCHOR_END: v021_chips
}
}

impl App {
    #[rustfmt::skip]
    fn v021_progress(&self, direction: LayoutDirection) -> impl Into<Element<'_, Message>> {
let tokens = &self.tokens;
// ANCHOR: v021_progress
use snora::design::{Tone, progress};
use iced::widget::column;

column![
    progress::row(&tokens, "Indexing files", Some(0.6), Tone::Accent, direction),
    progress::card(&tokens, "Syncing", None, Tone::Info, direction),
]
// ANCHOR_END: v021_progress
}
}
