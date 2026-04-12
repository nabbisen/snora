pub struct BottomSheet<Node, Message> {
    pub content: Node,
    pub on_close: Option<Message>, // シート外（背景）クリック時などに発火するイベント
}
