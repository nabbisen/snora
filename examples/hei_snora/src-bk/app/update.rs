use iced::Task;
use snora::{LayoutDirection, MenuAction, Toast, ToastIntent};

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
                self.view_id = view_id.clone();
                self.logs.push(LogEntry {
                    intent: ToastIntent::Info,
                    timestamp: "Now".into(),
                    message: format!("Switched to view: {}", view_id),
                });
            }
            Message::HeaderAction(header_action) => match header_action {
                MenuAction::MenuPressed(menu_id) => {
                    // 同じメニューが押されたら閉じる(None)、違うなら開く(Some)
                    if self.menu_id.as_ref() == Some(&menu_id) {
                        self.menu_id = None;
                    } else {
                        self.menu_id = Some(menu_id);
                    }
                }
                MenuAction::MenuItemPressed {
                    menu_id,
                    menu_item_id,
                } => {
                    // 項目が選ばれたら、アクションを実行しつつメニューを閉じる
                    self.menu_id = None;

                    self.logs.push(LogEntry {
                        intent: ToastIntent::Info,
                        timestamp: "Just now".into(),
                        message: format!("Clicked: {} - {}", menu_id, menu_item_id),
                    });
                }
            },

            Message::MenuPressed(menu_id) => self.menu_id = Some(menu_id),
            Message::OpenContext(pos) => self.context_menu_pos = Some(pos),
            Message::ToggleDialog => self.show_dialog = !self.show_dialog,

            Message::ToggleSheet => self.show_bottom_sheet = !self.show_bottom_sheet,

            // 閉じ処理 (Frameworkに渡す用)
            Message::CloseMenus => self.menu_id = None,
            Message::CloseModals => self.show_dialog = false,
            Message::CloseToast(id) => {
                self.toasts = self.toasts.into_iter().filter(|x| x.id != id).collect()
            }

            Message::AddDummyLog => self.logs.push(LogEntry {
                intent: ToastIntent::Debug,
                timestamp: "Now".into(),
                message: "Dummy log entry".into(),
            }),
            Message::AddDummyToast => self.toasts.push(Toast {
                id: 0,
                title: "".into(),
                message: "Toast".into(),
                intent: ToastIntent::Info,
                on_close: Message::CloseToast(0),
            }),
        }
        Task::none()
    }
}
