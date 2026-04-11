#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastIntent {
    Info,
    Success,
    Warning,
    Error,
}

pub struct Toast<Message> {
    pub title: String,
    pub message: String,
    pub intent: ToastIntent,
    pub on_close: Message,
}

pub struct Dialog<Node, Message> {
    pub content: Node,
    pub on_outside_click: Option<Message>,
}
