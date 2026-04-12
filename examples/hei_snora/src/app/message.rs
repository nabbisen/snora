use super::ViewId;

#[derive(Debug, Clone)]
pub enum Message {
    ToggleDirection,
    SelectView(ViewId),
    MenuAction(&'static str),
    ToggleLogSheet,
    AddDummyLog,
}
