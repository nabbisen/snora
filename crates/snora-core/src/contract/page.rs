use crate::contract::rtl::LayoutDirection;

/// ページ全体の骨格
pub struct PageLayout<Node> {
    pub body: Node,
    pub header: Option<Node>,
    pub aside: Option<Node>,
    pub footer: Option<Node>,
    pub direction: LayoutDirection,
}
