use snora::MenuAction;

use super::{MenuId, MenuItemId, ViewId};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleDirection,
    SelectView(ViewId),
    HeaderAction(MenuAction<MenuId, MenuItemId>),
    ToggleLogSheet,
    AddDummyLog,
}
