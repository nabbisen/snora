/// レイアウトの論理的な方向性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutDirection {
    #[default]
    Ltr, // Left-to-Right
    Rtl, // Right-to-Left
}
