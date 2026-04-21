//! Section: a uniform PageContract wrapper for the four layout slots.
//!
//! `render_app` binds a single `Node` type parameter to header / body / sidebar /
//! footer, so every slot must resolve to the same type. We satisfy that by giving
//! each slot the same `Section<'a>` wrapper and letting the `role` tag dispatch to
//! a slot-specific renderer.
//!
//! Overlays (dialog / toasts / context menu) are associated with the Body via
//! `PageContract`, matching the framework's convention of letting the page declare
//! its own feedback surface.

use iced::Element;
use snora::{BottomSheet, Dialog, PageContract, Toast};

use super::{App, Message, chrome, overlay, pages};

#[derive(Clone, Copy)]
pub enum SectionRole {
    Header,
    SideBar,
    Body,
    Footer,
}

pub struct Section<'a> {
    pub app: &'a App,
    pub role: SectionRole,
}

impl<'a> Section<'a> {
    pub fn header(app: &'a App) -> Self {
        Section { app, role: SectionRole::Header }
    }
    pub fn sidebar(app: &'a App) -> Self {
        Section { app, role: SectionRole::SideBar }
    }
    pub fn body(app: &'a App) -> Self {
        Section { app, role: SectionRole::Body }
    }
    pub fn footer(app: &'a App) -> Self {
        Section { app, role: SectionRole::Footer }
    }
}

impl<'a> PageContract for Section<'a> {
    type Node = Element<'a, Message>;
    type Message = Message;

    fn view(&self) -> Self::Node {
        match self.role {
            SectionRole::Header => chrome::header(self.app),
            SectionRole::SideBar => chrome::sidebar(self.app),
            SectionRole::Body => pages::body(self.app),
            SectionRole::Footer => chrome::footer(self.app),
        }
    }

    fn context_menu(&self) -> Option<Self::Node> {
        match self.role {
            SectionRole::Body => overlay::context_menu(self.app),
            _ => None,
        }
    }

    fn dialog(&self) -> Option<Dialog<Self::Node, Self::Message>> {
        match self.role {
            SectionRole::Body => overlay::dialog(self.app),
            _ => None,
        }
    }

    fn toasts(&self) -> Vec<Toast<Self::Message>> {
        match self.role {
            SectionRole::Body => overlay::toasts(self.app),
            _ => vec![],
        }
    }

    fn on_close_menus(&self) -> Option<Self::Message> {
        Some(Message::CloseMenus)
    }

    fn on_close_modals(&self) -> Option<Self::Message> {
        Some(Message::CloseModals)
    }
}

// Bottom sheet isn't part of `PageContract` directly (it's an AppLayout field), so we
// expose it here for the view layer to attach. Keeping the helper alongside the Section
// keeps all AppLayout wiring colocated.
pub fn bottom_sheet(app: &App) -> Option<BottomSheet<Element<'_, Message>, Message>> {
    overlay::bottom_sheet(app)
}
