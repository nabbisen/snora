//! Anchored source for `docs/src/guides/testing.md` (RFC-136): **the application and its tests, which `cargo test` runs.**
//!
//! `Message`, `ViewId`, the report and `issue_id` are the reader's own and are defined here, outside the anchors.

#[derive(Debug, Clone)]
pub enum Message {
    ExportCompleted(Result<Report, String>),
    CancelExport,
    DismissToast(u64),
}

#[derive(Debug, Clone)]
pub struct Report;

fn fake_report() -> Report {
    Report
}

#[derive(Default)]
pub enum ViewId {
    #[default]
    Home,
}

impl App {
    fn issue_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }
}

// ANCHOR: testing_app
// src/app.rs

#[derive(Default)]
pub struct App {
    pub toasts: Vec<snora::Toast<Message>>,
    pub next_id: u64,
    pub active: ViewId,
    pub show_export_dialog: bool,
}

impl App {
    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::ExportCompleted(Ok(_)) => {
                let id = self.issue_id();
                self.toasts.push(
                    snora::Toast::new(
                        id,
                        snora::ToastIntent::Success,
                        "Export complete",
                        "File written to disk.",
                        Message::DismissToast(id),
                    )
                    .persistent(),
                );
            }
            Message::CancelExport => self.show_export_dialog = false,
            // ...
            _ => {}
        }
        iced::Task::none()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        iced::widget::space().into() /* … */
    }
}
// ANCHOR_END: testing_app

// ANCHOR: testing_tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_completion_pushes_persistent_success_toast() {
        let mut app = App::default();

        app.update(Message::ExportCompleted(Ok(fake_report())));

        let last = app.toasts.last().expect("a toast was queued");
        assert_eq!(last.intent, snora::ToastIntent::Success);
        assert!(matches!(last.lifetime, snora::ToastLifetime::Persistent));
    }

    #[test]
    fn cancel_clears_active_dialog_flag() {
        let mut app = App {
            show_export_dialog: true,
            ..Default::default()
        };

        app.update(Message::CancelExport);

        assert!(!app.show_export_dialog);
    }

    #[test]
    fn ttl_sweep_drops_only_expired_transient() {
        use snora::ToastLifetime;
        use std::time::{Duration, Instant};

        let now = Instant::now();
        let mut app = App::default();
        app.toasts.push(
            snora::Toast::new(
                1,
                snora::ToastIntent::Info,
                "old",
                "",
                Message::DismissToast(1),
            )
            .with_lifetime(ToastLifetime::millis(100))
            .with_created_at(now),
        );
        app.toasts.push(
            snora::Toast::new(
                2,
                snora::ToastIntent::Error,
                "keep",
                "",
                Message::DismissToast(2),
            )
            .persistent()
            .with_created_at(now),
        );

        snora::toast::Holds::default().sweep(&mut app.toasts, now + Duration::from_secs(1));

        let ids: Vec<u64> = app.toasts.iter().map(|t| t.id).collect();
        assert_eq!(ids, vec![2]);
    }
}
// ANCHOR_END: testing_tests
