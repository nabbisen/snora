#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastIntent {
    Debug,
    Info,
    Success,
    Warning,
    Error,
}

impl std::fmt::Display for ToastIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ToastIntent::Debug => write!(f, "Debug"),
            ToastIntent::Info => write!(f, "Info"),
            ToastIntent::Success => write!(f, "Success"),
            ToastIntent::Warning => write!(f, "Warning"),
            ToastIntent::Error => write!(f, "Error"),
        }
    }
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
