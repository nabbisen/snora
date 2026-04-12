use iced::Task;
use snora::{LayoutDirection, MenuAction, ToastIntent};

use super::{App, log::LogEntry, message::Message};

impl App {
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
            Message::HeaderAction(header_action) => match header_action {
                MenuAction::MenuPressed(menu_id) => {
                    // 同じメニューが押されたら閉じる(None)、違うなら開く(Some)
                    if self.active_menu_id.as_ref() == Some(&menu_id) {
                        self.active_menu_id = None;
                    } else {
                        self.active_menu_id = Some(menu_id);
                    }
                }
                MenuAction::MenuItemPressed {
                    menu_id,
                    menu_item_id,
                } => {
                    // 項目が選ばれたら、アクションを実行しつつメニューを閉じる
                    self.active_menu_id = None;

                    self.logs.push(LogEntry {
                        intent: ToastIntent::Info,
                        timestamp: "Just now".into(),
                        message: format!("Clicked: {} - {}", menu_id, menu_item_id),
                    });
                }
            },
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
