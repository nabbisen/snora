use crate::contract::{
    rtl::LayoutDirection,
    stack::{Dialog, Toast},
};

/// ページ全体の骨格
pub struct PageLayout<Node> {
    pub body: Node,
    pub header: Option<Node>,
    pub aside: Option<Node>,
    pub footer: Option<Node>,
    pub direction: LayoutDirection,
}

pub trait PageContract {
    type Node;
    type Message;

    /// このページ（またはコンポーネント）のメインUIを生成する
    /// これが定義されていないと、render_app 内で Node から Element を取り出せません
    fn view(&self) -> Self::Node;

    fn context_menu(&self) -> Option<Self::Node> {
        None
    }

    fn dialog(&self) -> Option<Dialog<Self::Node, Self::Message>> {
        None
    }

    // Element ではなく Toast データの Vec を返す
    // デフォルトを空の Vec にすることで、不要なページでの実装負荷をゼロにする
    fn toasts(&self) -> Vec<Toast<Self::Message>> {
        vec![]
    }

    /// バックドロップ（透明背景）がクリックされた時に発行すべきメッセージ
    fn on_close_menus(&self) -> Option<Self::Message> {
        None
    }

    /// モーダル背景（半透明背景）がクリックされた時に発行すべきメッセージ
    fn on_close_modals(&self) -> Option<Self::Message> {
        None
    }

    #[allow(unused_variables)]
    fn on_close_toast(&self, id: usize) -> Option<Self::Message> {
        None
    }
}
