use std::time::{Duration, Instant};

use iced::{Point, Subscription, Task};
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
pub use misc::{FileMenuItemId, HelpMenuItemId, MenuId, MenuItemId, ViewId, ViewMenuItemId};

/// A toast queued on the App; converted to `snora::Toast<Message>` at render time.
///
/// `expires_at` drives auto-dismissal:
/// * `Some(instant)` — the toast disappears once the wall-clock passes `instant`.
///   The app's tick subscription sweeps expired toasts on each interval.
/// * `None` — the toast is *persistent*: it has no timer and can only be
///   dismissed by the user clicking the close button. This is the channel we
///   use for important errors that must stay visible until acknowledged.
pub struct ToastData {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub intent: ToastIntent,
    pub expires_at: Option<Instant>,
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
    /// App-wide default lifetime for auto-dismiss toasts. Individual toasts
    /// can override this by supplying their own duration (or opt out of
    /// auto-dismiss entirely by passing `None`, for persistent errors).
    pub default_toast_duration: Duration,

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
                // 4-second default — long enough to read a short message,
                // short enough not to stack up if the user is busy.
                default_toast_duration: Duration::from_secs(4),
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

    /// Periodic subscription for toast expiration.
    ///
    /// Only active while at least one toast has a finite lifetime. When all
    /// queued toasts are persistent (or the queue is empty), we return
    /// `Subscription::none()` so the runtime doesn't wake us up for nothing —
    /// subscriptions are declarative in iced, so the runtime will stop the
    /// underlying timer stream as soon as we stop returning it.
    pub fn subscription(&self) -> Subscription<Message> {
        let has_transient = self.toasts.iter().any(|t| t.expires_at.is_some());
        if has_transient {
            // 500 ms resolution is plenty — users won't notice a sub-second
            // lag on toast removal, and it keeps CPU wakeups low.
            iced::time::every(Duration::from_millis(500)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }
}
