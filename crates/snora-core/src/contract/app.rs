use std::fmt::Debug;

pub mod header;

use super::stack::Dialog;
use super::stack::Toast;
use super::ui::Icon;
use crate::contract::rtl::LayoutDirection;

/// ページ全体の骨格
pub struct AppLayout<Node, Message, MenuId>
where
    MenuId: Clone + Debug + PartialEq,
{
    pub body: Node,
    pub header: Option<Node>,
    pub active_menu_id: Option<MenuId>,
    pub side_bar: Option<Node>,
    pub footer: Option<Node>,
    pub dialog: Option<Dialog<Node, Message>>,
    pub bottom_sheet: Option<BottomSheet<Node, Message>>,
    pub toasts: Vec<Toast<Message>>,
    pub direction: LayoutDirection,
}

#[derive(Debug, Clone)]
pub struct AppSideBarItem<Message, ViewId>
where
    ViewId: PartialEq,
{
    pub view_id: ViewId,
    pub icon: Icon,
    pub tooltip: String,
    pub action: Message,
}

#[derive(Debug, Clone)]
pub struct AppSideBar<Message, ViewId>
where
    ViewId: PartialEq,
{
    pub items: Vec<AppSideBarItem<Message, ViewId>>,
    pub active_view_id: ViewId,
}

pub struct BottomSheet<Node, Message> {
    pub content: Node,
    pub on_close: Option<Message>, // シート外（背景）クリック時などに発火するイベント
}
