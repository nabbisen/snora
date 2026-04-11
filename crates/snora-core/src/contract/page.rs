use crate::contract::rtl::LayoutDirection;

use super::stack::Dialog;
use super::stack::Toast;

/// ページ全体の骨格
pub struct PageLayout<Node, Message> {
    pub body: Node,
    pub header: Option<Node>,
    pub aside: Option<Node>,
    pub footer: Option<Node>,
    pub dialog: Option<Dialog<Node, Message>>,
    pub toasts: Vec<Toast<Message>>,
    pub direction: LayoutDirection,
}
