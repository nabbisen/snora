use iced::Task;
use snora::LayoutDirection;
use strum_macros::Display;

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

#[derive(Clone, Debug, PartialEq, Display)]
#[strum(serialize_all = "PascalCase")]
pub enum MenuId {
    File,
    Settings,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MenuItemId {
    File(FileMenuItemId),
    Settings(SettingsMenuItemId),
}

impl std::fmt::Display for MenuItemId {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuItemId::File(x) => write!(w, "{}", x),
            MenuItemId::Settings(x) => write!(w, "{}", x),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Display)]
#[strum(serialize_all = "PascalCase")]
pub enum FileMenuItemId {
    New,
}

#[derive(Clone, Debug, PartialEq, Display)]
#[strum(serialize_all = "PascalCase")]
pub enum SettingsMenuItemId {
    About,
}

pub struct App {
    direction: LayoutDirection,
    active_view_id: ViewId, // 現在選択されているAppレベルのビューID
    logs: Vec<LogEntry>,
    active_menu_id: Option<MenuId>,
    is_bottom_sheet_open: bool,
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
