use std::time::{Duration, Instant};

use iced::Task;
use snora::{LayoutDirection, MenuAction, ToastIntent};

use super::{
    App, FileMenuItemId, HelpMenuItemId, MenuId, MenuItemId, Message, ToastData, ToastFlavor,
    ViewMenuItemId,
    log::LogEntry,
};

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleDirection => {
                self.direction = match self.direction {
                    LayoutDirection::Ltr => LayoutDirection::Rtl,
                    LayoutDirection::Rtl => LayoutDirection::Ltr,
                };
                self.log_info(format!("Direction → {:?}", self.direction));
            }
            Message::SelectView(view_id) => {
                self.log_info(format!("Switched view: {}", view_id));
                self.active_view_id = view_id;
            }
            Message::HeaderAction(action) => match action {
                MenuAction::MenuPressed(menu_id) => {
                    // Same menu pressed → close; different menu → switch.
                    self.active_menu_id = if self.active_menu_id.as_ref() == Some(&menu_id) {
                        None
                    } else {
                        Some(menu_id)
                    };
                }
                MenuAction::MenuItemPressed {
                    menu_id,
                    menu_item_id,
                } => {
                    self.active_menu_id = None;
                    self.handle_menu_item(menu_id, menu_item_id);
                }
            },

            Message::OpenContextMenu(pos) => {
                self.context_menu_pos = Some(pos);
            }
            Message::OpenDialog => {
                self.show_dialog = true;
            }
            Message::ToggleSheet => {
                self.show_bottom_sheet = !self.show_bottom_sheet;
            }

            // Close sinks — the framework's transparent / dimmed backdrops call these.
            Message::CloseMenus => {
                self.active_menu_id = None;
                self.context_menu_pos = None;
            }
            Message::CloseModals => {
                self.show_dialog = false;
                self.show_bottom_sheet = false;
            }

            Message::AddLog => {
                let ts = format!("t+{}", self.logs.len());
                self.logs.push(LogEntry {
                    intent: ToastIntent::Debug,
                    timestamp: ts,
                    message: "Manual log entry".into(),
                });
            }

            // --- Toast entry points ---
            Message::ShowToast(flavor) => {
                // Default duration pulled from app settings.
                self.push_toast_default(flavor);
            }
            Message::ShowCustomToast(flavor, millis) => {
                let (title, body) = Self::default_copy(flavor);
                self.push_toast_with(
                    flavor,
                    title,
                    format!("{body} (auto-dismiss in {millis} ms)"),
                    Some(Duration::from_millis(millis)),
                );
            }
            Message::ShowPersistentToast(flavor) => {
                let (title, body) = Self::default_copy(flavor);
                self.push_toast_with(
                    flavor,
                    title,
                    format!("{body} (manual dismiss only)"),
                    None,
                );
            }
            Message::DismissToast(id) => {
                self.toasts.retain(|t| t.id != id);
            }
            Message::Tick => {
                // Sweep expired toasts. Persistent toasts (`expires_at: None`)
                // are always retained and must be closed by the user.
                let now = Instant::now();
                self.toasts.retain(|t| match t.expires_at {
                    None => true,
                    Some(e) => now < e,
                });
            }

            Message::SearchChanged(q) => {
                self.search_query = q;
            }
            Message::SubmitSearch => {
                let q = self.search_query.trim().to_string();
                if q.is_empty() {
                    self.push_toast_with(
                        ToastFlavor::Warning,
                        "Empty query",
                        "Type something before submitting.",
                        Some(self.default_toast_duration),
                    );
                } else {
                    self.push_toast_with(
                        ToastFlavor::Success,
                        "Search dispatched",
                        format!("Searched for: “{}”", q),
                        Some(self.default_toast_duration),
                    );
                    self.log_info(format!("Search submitted: '{}'", q));
                }
            }
        }
        Task::none()
    }

    fn handle_menu_item(&mut self, _menu_id: MenuId, item: MenuItemId) {
        match item {
            MenuItemId::File(FileMenuItemId::New) => {
                self.log_info("File → New (placeholder)".into());
                self.push_toast_default_with(
                    ToastFlavor::Info,
                    "File",
                    "New document created (demo)",
                );
            }
            MenuItemId::File(FileMenuItemId::Open) => {
                self.log_info("File → Open (placeholder)".into());
                self.push_toast_default_with(
                    ToastFlavor::Info,
                    "File",
                    "Open dialog would appear here",
                );
            }
            MenuItemId::File(FileMenuItemId::Quit) => {
                self.log_info("File → Quit (placeholder)".into());
                self.push_toast_default_with(
                    ToastFlavor::Warning,
                    "Quit",
                    "Quit is a no-op in this showcase.",
                );
            }
            MenuItemId::View(ViewMenuItemId::ToggleLogs) => {
                self.show_bottom_sheet = !self.show_bottom_sheet;
            }
            MenuItemId::View(ViewMenuItemId::FlipDirection) => {
                self.direction = match self.direction {
                    LayoutDirection::Ltr => LayoutDirection::Rtl,
                    LayoutDirection::Rtl => LayoutDirection::Ltr,
                };
                self.log_info(format!("Direction → {:?} (from menu)", self.direction));
            }
            MenuItemId::Help(HelpMenuItemId::Documentation) => {
                self.push_toast_default_with(
                    ToastFlavor::Info,
                    "Docs",
                    "See README.md in the workspace root.",
                );
            }
            MenuItemId::Help(HelpMenuItemId::About) => {
                self.show_dialog = true;
            }
        }
    }

    fn log_info(&mut self, msg: String) {
        self.logs.push(LogEntry {
            intent: ToastIntent::Info,
            timestamp: format!("t+{}", self.logs.len()),
            message: msg,
        });
    }

    /// Canonical title/body copy used when the caller didn't supply their own.
    fn default_copy(flavor: ToastFlavor) -> (&'static str, &'static str) {
        match flavor {
            ToastFlavor::Info => ("Info", "A neutral notification."),
            ToastFlavor::Success => ("Success", "Operation completed."),
            ToastFlavor::Warning => ("Warning", "Something deserves attention."),
            ToastFlavor::Error => ("Error", "Something went wrong (demo only)."),
        }
    }

    fn push_toast_default(&mut self, flavor: ToastFlavor) {
        let (title, body) = Self::default_copy(flavor);
        self.push_toast_with(flavor, title, body, Some(self.default_toast_duration));
    }

    fn push_toast_default_with(
        &mut self,
        flavor: ToastFlavor,
        title: impl Into<String>,
        body: impl Into<String>,
    ) {
        let default = self.default_toast_duration;
        self.push_toast_with(flavor, title, body, Some(default));
    }

    /// Enqueue a toast with explicit duration.
    /// * `Some(d)` — auto-dismiss after `d`
    /// * `None` — persistent; only the close button removes it
    fn push_toast_with(
        &mut self,
        flavor: ToastFlavor,
        title: impl Into<String>,
        body: impl Into<String>,
        duration: Option<Duration>,
    ) {
        let intent = match flavor {
            ToastFlavor::Info => ToastIntent::Info,
            ToastFlavor::Success => ToastIntent::Success,
            ToastFlavor::Warning => ToastIntent::Warning,
            ToastFlavor::Error => ToastIntent::Error,
        };
        let id = self.next_toast_id;
        self.next_toast_id += 1;
        // `Instant::now()` here is an intentional impurity — time travel
        // replay would regenerate different `expires_at` values, but that's
        // acceptable for a showcase; correctness doesn't depend on replay
        // determinism.
        let expires_at = duration.map(|d| Instant::now() + d);
        self.toasts.push(ToastData {
            id,
            title: title.into(),
            message: body.into(),
            intent,
            expires_at,
        });
    }
}
