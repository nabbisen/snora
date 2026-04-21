use std::fmt::Debug;

pub mod bottom_sheet;
pub mod header;
pub mod side_bar;

use crate::contract::{
    page::PageContract,
    rtl::LayoutDirection,
    stack::{Dialog, Toast},
};
use bottom_sheet::BottomSheet;

/// アプリ全体の骨格
/// 型パラメータ Node は iced::Element になる想定
pub struct AppLayout<P, Message, MenuId>
where
    P: PageContract<Message = Message>,
    MenuId: Clone + Debug + PartialEq,
{
    // --- Layout slots (これらはすべて PageContract を実装したオブジェクト) ---
    pub header: Option<P>,
    pub body: P,
    pub side_bar: Option<P>,
    pub footer: Option<P>,

    // --- Overlay layers (UI実体) ---
    pub header_menu: Option<P::Node>,
    pub context_menu: Option<P::Node>,

    // --- Notification/Modal (Page から吸い出すことも、App層から直接流すことも可能にする) ---
    pub toasts: Vec<Toast<Message>>,
    pub dialog: Option<Dialog<P::Node, Message>>,
    pub bottom_sheet: Option<BottomSheet<P::Node, Message>>,

    // --- Config ---
    pub direction: LayoutDirection,
    pub menu_id: Option<MenuId>,
}
