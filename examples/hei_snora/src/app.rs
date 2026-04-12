use iced::Task;
use snora::LayoutDirection;

mod log;
mod message;
mod update;
mod view;

use crate::app::log::LogEntry;
use message::Message;

pub struct HeiSnora {
    direction: LayoutDirection,
    active_view: String, // 現在選択されているAppレベルのビューID
    logs: Vec<LogEntry>,
    is_bottom_sheet_open: bool,
}

impl HeiSnora {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                direction: LayoutDirection::Ltr,
                active_view: "home".into(),
                logs: vec![],
                is_bottom_sheet_open: false,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "Snora Framework - Modern Layout".into()
    }
}
