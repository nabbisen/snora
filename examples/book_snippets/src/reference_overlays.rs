//! Anchored source for `docs/src/reference/overlay-interaction-semantics.md` (RFC-136).

use iced::Element;

#[derive(Clone)]
enum Message {
    OpenSettingsDialog,
    CloseModals,
    CloseMenus,
    EscapePressed,
    CancelDelete,
}

enum DialogState {
    Settings,
}

struct App {
    header_menu: Option<u8>,
    context_menu: Option<u8>,
    dialog: Option<DialogState>,
    sheet: Option<u8>,
}

impl App {
    #[rustfmt::skip]
    fn update(&mut self, msg: Message) {
// ANCHOR: overlays_laws_update
match msg {
    Message::OpenSettingsDialog => {
        // Law 2: clear menus before opening a modal.
        self.header_menu = None;
        self.context_menu = None;
        self.dialog = Some(DialogState::Settings);
    }
    Message::CloseModals => {
        self.dialog = None;
        self.sheet = None;
    }
    Message::CloseMenus => {
        self.header_menu = None;
        self.context_menu = None;
    }
    _ => {}
}
// ANCHOR_END: overlays_laws_update
}
}

impl App {
    #[rustfmt::skip]
    fn update_escape(&mut self, msg: Message) {
// ANCHOR: overlays_escape_update
match msg {
    // Prioritize modal over menu when both are present.
    Message::EscapePressed if self.dialog.is_some() || self.sheet.is_some() => {
        self.dialog = None;
        self.sheet = None;
    }
    Message::EscapePressed => {
        self.header_menu = None;
        self.context_menu = None;
    }
    _ => {}
}
// ANCHOR_END: overlays_escape_update
}
}

#[rustfmt::skip]
fn layout(body: Element<'static, Message>) -> Element<'static, Message> {
// ANCHOR: overlays_layout
let layout = snora::AppLayout::new(body)
    .on_close_modals(Message::CloseModals)   // a click outside, and Escape
    .on_close_menus(Message::CloseMenus)
    // Optional: Escape alone, for a modal that a stray click must not close.
    .escape(snora::keyboard::Escape { modal: Some(Message::CancelDelete), menu: None });
// ANCHOR_END: overlays_layout
snora::render(layout)
}
