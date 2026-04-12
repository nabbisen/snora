use super::{MenuItemId, ViewId};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleDirection,
    SelectView(ViewId),
    MenuAction(MenuItemId),
    ToggleLogSheet,
    AddDummyLog,
}
