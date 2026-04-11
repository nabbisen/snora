use crate::contract::rtl::LayoutDirection;

use super::ui::Icon;

/// ページ全体の骨格
pub struct AppLayout<Node> {
    pub body: Node,
    pub header: Option<Node>,
    pub side_bar: Option<Node>,
    pub footer: Option<Node>,
    pub direction: LayoutDirection,
}

#[derive(Debug, Clone)]
pub struct AppSideBarItem<Message> {
    pub id: String,
    pub icon: Icon,
    pub tooltip: String,
    pub action: Message,
}

#[derive(Debug, Clone)]
pub struct AppSideBar<Message> {
    pub items: Vec<AppSideBarItem<Message>>,
    pub active_id: String,
}

pub struct MenuItem<Message> {
    pub label: String,
    pub icon: Option<Icon>,
    pub action: Option<Message>,
}
