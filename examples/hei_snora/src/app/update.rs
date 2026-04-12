use iced::Task;
use snora::{LayoutDirection, ToastIntent};

use super::{HeiSnora, log::LogEntry, message::Message};

impl HeiSnora {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleDirection => {
                self.direction = match self.direction {
                    LayoutDirection::Ltr => LayoutDirection::Rtl,
                    LayoutDirection::Rtl => LayoutDirection::Ltr,
                };
            }
            Message::SelectView(view_id) => {
                self.active_view_id = view_id.clone();
                self.logs.push(LogEntry {
                    intent: ToastIntent::Info,
                    timestamp: "Now".into(),
                    message: format!("Switched to view: {}", view_id),
                });
            }
            Message::MenuAction(name) => {
                self.logs.push(LogEntry {
                    intent: ToastIntent::Info,
                    timestamp: "Just now".into(),
                    message: format!("Clicked: {}", name),
                });
            }
            Message::ToggleLogSheet => {
                self.is_bottom_sheet_open = !self.is_bottom_sheet_open;
            }
            Message::AddDummyLog => self.logs.push(LogEntry {
                intent: ToastIntent::Debug,
                timestamp: "Now".into(),
                message: "Dummy log entry".into(),
            }),
        }
        Task::none()
    }
}
