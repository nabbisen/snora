/// レイアウトの論理的な方向性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutDirection {
    #[default]
    Ltr, // Left-to-Right
    Rtl, // Right-to-Left
}

/// アイコンの抽象表現
#[derive(Debug, Clone)]
pub enum Icon {
    Text(String),
    #[cfg(feature = "lucide-icons")]
    Lucide(lucide_icons::Icon),
    #[cfg(feature = "svg-icons")]
    Svg(std::path::PathBuf),
}

// 文字列からIcon::Textへの変換を容易にする
impl From<&str> for Icon {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

pub struct MenuItem<Message> {
    pub label: String,
    pub icon: Option<Icon>,
    pub action: Option<Message>,
}

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

/// ページ全体の骨格
pub struct PageLayout<Node, Message> {
    pub direction: LayoutDirection,
    pub header: Option<Node>,
    pub body: Node,
    pub aside: Option<Node>,
    pub footer: Option<Node>,
    pub dialog: Option<Dialog<Node, Message>>,
    pub toasts: Vec<Toast<Message>>,
}
