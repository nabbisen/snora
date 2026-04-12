#[derive(Debug, Clone)]
pub enum Message {
    ToggleDirection,
    SelectView(String),
    MenuAction(&'static str),
    ToggleLogSheet,
    AddDummyLog,
}
