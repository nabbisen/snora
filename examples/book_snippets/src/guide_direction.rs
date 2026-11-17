//! Anchored source for `docs/src/guides/direction.md` (RFC-136).

use iced::Element;

#[derive(Clone)]
enum Message {
    Save,
}

struct State {
    direction: snora::LayoutDirection,
}

#[rustfmt::skip]
fn layout(body: Element<'static, Message>) -> Element<'static, Message> {
use snora::AppLayout;
// ANCHOR: direction_layout
use snora::{LayoutDirection, ToastPosition};

let layout = AppLayout::new(body)
    .direction(LayoutDirection::Rtl)            // body row mirrors
    .toast_position(ToastPosition::TopEnd);      // toast anchor mirrors
// ANCHOR_END: direction_layout
snora::render(layout)
}

#[rustfmt::skip]
fn bar(state: &State) -> Element<'_, Message> {
// ANCHOR: direction_row_dir
use snora::direction::row_dir;

let bar = row_dir(
    state.direction,
    iced::widget::text("File: untitled"),       // start
    iced::widget::button("Save")                // end
        .on_press(Message::Save),
);
// ANCHOR_END: direction_row_dir
bar.into()
}
