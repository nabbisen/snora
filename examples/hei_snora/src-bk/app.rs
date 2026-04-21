use iced::{Point, Task};
use snora::{LayoutDirection, Toast};

mod log;
mod message;
mod misc;
mod update;
mod view;

use crate::app::log::LogEntry;
use message::Message;
use misc::*;

pub struct App {
    view_id: ViewId,

    menu_id: Option<MenuId>,
    context_menu_pos: Option<Point>,

    show_dialog: bool,
    show_bottom_sheet: bool,

    toasts: Vec<Toast<Message>>,

    direction: LayoutDirection,

    // test data
    logs: Vec<LogEntry>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                view_id: ViewId::Home,
                menu_id: None,
                context_menu_pos: None,
                show_dialog: false,
                show_bottom_sheet: false,
                toasts: vec![],
                direction: LayoutDirection::Ltr,

                logs: vec![],
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "Snora Framework - Modern Layout".into()
    }
}
