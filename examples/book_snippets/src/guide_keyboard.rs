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

// RFC-137: a disclosure keeps focus.
mod disclosure {
    use iced::Element;
    use iced::widget::text;

    #[derive(Clone)]
    enum Message {
        Toggle(String),
    }

    struct Node {
        path: String,
        name: String,
    }

    #[rustfmt::skip]
    fn node_row(node: &Node) -> Element<'_, Message> {
// ANCHOR: keeps_focus
snora::control(text(node.name.as_str()))
    .on_press(Message::Toggle(node.path.clone()))
    // The node's path names the node; a row number would name another one after a folder above opens.
    .keeps_focus(format!("node:{}", node.path))
    .into()
// ANCHOR_END: keeps_focus
}
}
