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
    ShowToast(ToastFlavor),
    DismissToast(u64),
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
