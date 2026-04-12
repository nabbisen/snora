use std::fmt::Debug;

pub mod bottom_sheet;
pub mod header;
pub mod side_bar;

use crate::contract::{
    rtl::LayoutDirection,
    stack::{Dialog, Toast},
};
use bottom_sheet::BottomSheet;

/// アプリ全体の骨格
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
