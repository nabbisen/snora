use iced::Task;
use snora::LayoutDirection;

mod log;
mod message;
mod misc;
mod update;
mod view;

use crate::app::log::LogEntry;
use message::Message;
use misc::*;

pub struct App {
    direction: LayoutDirection,
    active_view_id: ViewId, // 現在選択されているAppレベルのビューID
    logs: Vec<LogEntry>,
    active_menu_id: Option<MenuId>,
    is_bottom_sheet_open: bool,
    // context_menu_state: Option<(Point, String)>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                direction: LayoutDirection::Ltr,
                active_view_id: ViewId::Home,
                logs: vec![],
                active_menu_id: None,
                is_bottom_sheet_open: false,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "Snora Framework - Modern Layout".into()
    }
}
