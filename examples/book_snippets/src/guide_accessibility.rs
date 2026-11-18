//! Anchored source for `docs/src/guides/accessibility.md` (RFC-136): the two keyboard fragments, in an `update`.

use iced::Task;
use iced::keyboard::{Key, Modifiers};

enum Message {
    KeyPressed(Key, Modifiers),
    Other,
}

struct App;

impl App {
    #[expect(clippy::single_match, reason = "the page shows one arm of the reader's own, larger `match`")]
    #[rustfmt::skip]
    fn update(&mut self, msg: Message) -> Task<Message> {
match msg {
// ANCHOR: accessibility_tab
  Message::KeyPressed(key, modifiers) => {
      if let Some(task) = snora::keyboard::traverse(key, modifiers) {
          return task;
      }
  }
// ANCHOR_END: accessibility_tab
_ => {}
}
Task::none()
}
}

impl App {
    #[rustfmt::skip]
    fn on_key(&mut self, key: Key, modifiers: Modifiers) -> Task<Message> {
// ANCHOR: accessibility_f6
  if let Some(cycle) = snora::keyboard::cycle_zones(key, modifiers) {
      return snora::keyboard::focus_zone(cycle);
  }
// ANCHOR_END: accessibility_f6
Task::none()
}
}
