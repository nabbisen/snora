//! Anchored source for `docs/src/guides/keyboard.md` (RFC-136).

use iced::Task;
use iced::keyboard::Key;

#[derive(Clone)]
enum Message {
    CloseMyOverlay,
}

struct App {
    my_overlay_open: bool,
}

impl App {
    fn update(&mut self, msg: Message) -> Task<Message> {
        Task::none()
    }

#[rustfmt::skip]
    fn on_key(&mut self, key: Key) -> Task<Message> {
// ANCHOR: keyboard_escape
if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) && self.my_overlay_open {
    return self.update(Message::CloseMyOverlay);
}
// ANCHOR_END: keyboard_escape
Task::none()
}
}
