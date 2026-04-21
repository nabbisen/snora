use iced::{Point, Task};
use snora::{LayoutDirection, ToastIntent};

mod chrome;
mod log;
mod message;
mod misc;
mod overlay;
mod pages;
mod section;
mod update;
mod view;

use crate::app::log::LogEntry;

pub use message::{Message, ToastFlavor};
pub use misc::{
    FileMenuItemId, HelpMenuItemId, MenuId, MenuItemId, ViewId, ViewMenuItemId,
};

/// A toast queued on the App; converted to `snora::Toast<Message>` at render time.
pub struct ToastData {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub intent: ToastIntent,
}

pub struct App {
    // --- Navigation state ---
    pub active_view_id: ViewId,

    // --- Menu / overlay state ---
    pub active_menu_id: Option<MenuId>,
    pub context_menu_pos: Option<Point>,
    pub show_dialog: bool,
    pub show_bottom_sheet: bool,

    // --- Toast queue ---
    pub toasts: Vec<ToastData>,
    pub next_toast_id: u64,

    // --- Framework-level direction ---
    pub direction: LayoutDirection,

    // --- Content state ---
    pub search_query: String,
    pub logs: Vec<LogEntry>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                active_view_id: ViewId::Home,
                active_menu_id: None,
                context_menu_pos: None,
                show_dialog: false,
                show_bottom_sheet: false,
                toasts: vec![],
                next_toast_id: 1,
                direction: LayoutDirection::Ltr,
                search_query: String::new(),
                logs: vec![LogEntry {
                    intent: ToastIntent::Info,
                    timestamp: "boot".into(),
                    message: "Snora showcase started.".into(),
                }],
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "Snora Framework — Showcase".into()
    }
}
