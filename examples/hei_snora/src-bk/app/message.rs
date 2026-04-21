use iced::Point;
use snora::MenuAction;

use super::{MenuId, MenuItemId, ViewId};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleDirection,
    SelectView(ViewId),
    HeaderAction(MenuAction<MenuId, MenuItemId>),

    MenuPressed(MenuId),
    OpenContext(Point),
    ToggleDialog,
    ToggleSheet,
    // 閉じ処理 (Frameworkに渡す用)
    CloseMenus,
    CloseModals,
    CloseToast(usize),

    AddDummyLog,
    AddDummyToast,
}
