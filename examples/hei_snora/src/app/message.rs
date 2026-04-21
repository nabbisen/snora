use iced::Point;
use snora::MenuAction;

use super::{MenuId, MenuItemId, ViewId};

#[derive(Debug, Clone)]
pub enum Message {
    // --- Layout / navigation ---
    ToggleDirection,
    SelectView(ViewId),
    HeaderAction(MenuAction<MenuId, MenuItemId>),

    // --- Overlay control ---
    OpenContextMenu(Point),
    OpenDialog,
    ToggleSheet,

    // --- Close sinks (wired into the framework backdrops) ---
    CloseMenus,
    CloseModals,

    // --- Content interactions ---
    AddLog,

    // --- Toasts ---
    //
    // Three distinct flavors are exposed because toast lifetime is a real
    // product decision, not a styling one:
    //
    // * `ShowToast` uses the app-wide default duration — the normal path.
    // * `ShowCustomToast(flavor, millis)` overrides the duration for this
    //   one toast. Use for quick "saved" blips or long "processing…" hints.
    // * `ShowPersistentToast` has no timer at all. The user must click the
    //   close button to dismiss it. Use sparingly, for errors that should
    //   not quietly disappear.
    ShowToast(ToastFlavor),
    ShowCustomToast(ToastFlavor, u64),
    ShowPersistentToast(ToastFlavor),
    DismissToast(u64),
    /// Periodic tick from the app's time subscription; drives toast expiration.
    Tick,

    // --- Search page ---
    SearchChanged(String),
    SubmitSearch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastFlavor {
    Info,
    Success,
    Warning,
    Error,
}
