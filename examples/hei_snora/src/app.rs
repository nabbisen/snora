use iced::Task;
use snora::LayoutDirection;

mod log;
mod message;
mod update;
mod view;

use crate::app::log::LogEntry;
use message::Message;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewId {
    Home,
    Search,
    Settings,
}

impl std::fmt::Display for ViewId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ViewId::Home => write!(f, "Home"),
            ViewId::Search => write!(f, "Search"),
            ViewId::Settings => write!(f, "Settings"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuItemId {
    FileDummy,
    SettingsDummy,
}

impl std::fmt::Display for MenuItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MenuItemId::FileDummy => write!(f, "FileDummy"),
            MenuItemId::SettingsDummy => write!(f, "SettingsDummy"),
        }
    }
}

pub struct App {
    direction: LayoutDirection,
    active_view_id: ViewId, // 現在選択されているAppレベルのビューID
    logs: Vec<LogEntry>,
    is_bottom_sheet_open: bool,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                direction: LayoutDirection::Ltr,
                active_view_id: ViewId::Home,
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
