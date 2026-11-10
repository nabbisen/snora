//! The cover an open menu puts over the layer it is in (RFC-124).
//!
//! An [`Inert`](super::Inert) built with
//! [`beneath_menus`](super::Inert::beneath_menus) is covered **while a menu is open in
//! its content**, with no signal from the application: the menu's anchor reports a
//! mark ([`MenuAnchor`](crate::MenuAnchor)) and the layer surveys for it. Covered by a
//! menu means, for the layer's content:
//!
//! - **the keyboard does not reach it** and its focus targets are hidden, exactly as
//!   under a modal (RFC-119), and the focus it held is set aside and comes back when
//!   the menu closes;
//! - **the pointer does not reach it either**: a press that the menu did not take
//!   closes the menu and goes no further, and the page gets no cursor at all, so no
//!   hover, no pointer cursor, and a tooltip already showing goes away;
//! - **Escape closes the menu** (Law 7: the application's escape message if it set
//!   one, otherwise the close message);
//! - **the menu's own overlay is let through** where any other overlay of a covered
//!   layer is hidden. `Inert` is handed an overlay it cannot look into, so it cannot
//!   pick the menu's out of a group; it relies on the page having no cursor instead.
//!
//! **A modal switches this off** (the engine passes `None`): nothing of a covered layer
//! floats above the dim.
//!
//! # A known limit
//!
//! Because `Inert` cannot tell the menu's overlay from another overlay of the page, an
//! overlay that does **not** depend on the pointer and is open when a menu opens would
//! stay above the menu. A `pick_list`'s open list is one: an application would have to
//! open the menu from its own code while the list is open, since a press anywhere else
//! closes the list first. Overlays that need the pointer (a tooltip) go on their own,
//! because the page is given none. This is a literal exception to "nothing from a covered
//! layer floats above what covers it" for a menu (never for a modal), recorded here by
//! the architect's ruling.
//!
//! # When it looks
//!
//! A menu opens or closes only when the application rebuilds its view, so the survey
//! runs **once per rebuild** (`diff` marks it stale), at the first call that has a
//! layout and a renderer. That can be `overlay()`, which the runtime asks for before
//! it updates anything: a survey that waited for the first event would draw the
//! dropdown a frame late.

use iced::keyboard::{self, key};
use iced_core::widget::{Id, Operation, Tree};
use iced_core::{Event, Shell, mouse, touch};
use iced_core::{Rectangle, layout::Layout};

use super::{Inert, State};

/// What a covered-by-menu layer publishes.
pub(super) struct Cover<Message> {
    /// A press the menu did not take.
    pub(super) close: Message,
    /// Escape: the application's escape message, else `close`.
    pub(super) escape: Message,
}

/// What the last survey found.
#[derive(Default)]
pub(super) struct Seen {
    /// A menu was open in the content.
    pub(super) open: bool,
    /// The survey is of this build of the tree.
    pub(super) current: bool,
}

/// Is a menu open in the content?
#[derive(Default)]
struct FindOpenMenu(bool);

impl Operation for FindOpenMenu {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
        if state.is::<crate::menu_anchor::OpenMenu>() {
            self.0 = true;
        }
    }
}

impl<'a, Message: Clone> Inert<'a, Message> {
    /// While a menu is open in the content, this layer is covered by it (see the cover's
    /// module docs, `inert/menu_cover.rs`). `close` is what a press outside the menu publishes; `escape`
    /// is what Escape publishes if the application set one, else `close` is.
    ///
    /// `None` for `close` switches the cover off, which is what a modal above does,
    /// and what an application with no close message gets: the page stays live.
    #[must_use]
    pub fn beneath_menus(mut self, close: Option<Message>, escape: Option<Message>) -> Self {
        self.menu = close.map(|close| Cover {
            escape: escape.unwrap_or_else(|| close.clone()),
            close,
        });
        self
    }
}

impl<Message: Clone> Inert<'_, Message> {
    /// Covered: by the flag, or by a menu open in the content.
    pub(super) fn covered(&self, tree: &Tree) -> bool {
        self.inert || self.covered_by_menu(tree)
    }

    pub(super) fn covered_by_menu(&self, tree: &Tree) -> bool {
        self.menu.is_some() && tree.state.downcast_ref::<State>().menu.open
    }

    /// Looks for an open menu in the content, if the tree was rebuilt since the last look.
    pub(super) fn refresh_menu(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
    ) {
        if self.menu.is_none() || tree.state.downcast_ref::<State>().menu.current {
            return;
        }
        let mut find = FindOpenMenu::default();
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut find);
        let seen = &mut tree.state.downcast_mut::<State>().menu;
        seen.open = find.0;
        seen.current = true;
    }

    /// A press the menu did not take, or Escape, while covered by a menu: publishes
    /// and **captures**, and the content never sees it. `true` if it did.
    pub(super) fn answer_menu(
        &self,
        tree: &Tree,
        event: &Event,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        let Some(cover) = self.menu.as_ref().filter(|_| self.covered_by_menu(tree)) else {
            return false;
        };
        let message = match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                repeat: false,
                ..
            }) => &cover.escape,
            Event::Mouse(mouse::Event::ButtonPressed(_))
            | Event::Touch(touch::Event::FingerPressed { .. }) => &cover.close,
            _ => return false,
        };
        shell.publish(message.clone());
        shell.capture_event();
        true
    }

    /// The pointer the content is given: none, while a menu covers it.
    pub(super) fn cursor_for_content(&self, tree: &Tree, cursor: mouse::Cursor) -> mouse::Cursor {
        if self.covered_by_menu(tree) {
            mouse::Cursor::Unavailable
        } else {
            cursor
        }
    }
}
