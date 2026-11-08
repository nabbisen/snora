//! [`Keyed`]: focus that follows an item by its key, not by its position
//! (RFC-120 D-2).
//!
//! iced keeps a widget's state **by its place in the tree**. When one toast
//! of a stack goes, the next moves into its place and inherits its focus, so a
//! toast that expires **elsewhere** moves the reader's focus to a toast they
//! never reached, and one more Enter closes it (Unit 0, case 1b).
//!
//! Where the items have a stable key, this is avoidable. The toast stack is the
//! one place snora has it: every toast has a `u64` id that survives a rebuild.
//! `Keyed` wraps a list whose focus targets are **one per key, in the order of
//! `keys`**, and at each `RedrawRequested`:
//!
//! - if the keys have not changed, any focus change is the reader's, and is
//!   remembered by key;
//! - if they have, **focus is put back on the item it was on**, by key; and if
//!   that item has gone, on its neighbour among the items that are still
//!   there: the one after it in the old order, else the one before; and if
//!   none is left, nowhere (RFC-120 R-1, with the stack as the group).
//!
//! If the content does not have exactly one focus target per key, `Keyed`
//! stands aside and does nothing: it would be guessing.
//!
//! Like [`Inert`](crate::Inert), it is public because the engine builds it, and
//! it is not for applications.

use iced::{Element, Length, Rectangle, Size, Theme, Vector};
use iced_core::layout::{self, Layout};
use iced_core::widget::operation::Focusable;
use iced_core::widget::{Id, Operation, Tree, tree};
use iced_core::{Clipboard, Event, Shell, Widget, mouse, overlay, renderer, window};

use crate::inert::{Refocus, Unfocus};

#[derive(Default)]
struct State {
    /// The keys as they were at the last redraw.
    keys: Vec<u64>,
    /// The key whose item held focus at the last redraw.
    focused: Option<u64>,
}

/// A list whose focus follows its items' keys.
pub struct Keyed<'a, Message> {
    content: Element<'a, Message>,
    keys: Vec<u64>,
}

impl<'a, Message> Keyed<'a, Message> {
    /// `content`, whose focus targets are one per key in `keys`, in Tab order.
    pub fn new(content: impl Into<Element<'a, Message>>, keys: Vec<u64>) -> Self {
        Self {
            content: content.into(),
            keys,
        }
    }
}

/// Which focus targets are focused, in Tab order.
#[derive(Default)]
struct Flags(Vec<bool>);

impl Operation for Flags {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        self.0.push(state.is_focused());
    }
}

/// Where focus belongs after the keys changed: the item it was on, else its
/// neighbour that is still there (next in the old order, else previous), else
/// nowhere.
fn belongs_on(old: &[u64], now: &[u64], focused: u64) -> Option<usize> {
    if let Some(at) = now.iter().position(|k| *k == focused) {
        return Some(at);
    }
    let was = old.iter().position(|k| *k == focused)?;
    let survivor = |k: &&u64| now.contains(*k);
    let neighbour = old[was + 1..]
        .iter()
        .find(survivor)
        .or_else(|| old[..was].iter().rev().find(survivor))?;
    now.iter().position(|k| k == neighbour)
}

impl<Message> Keyed<'_, Message> {
    /// Notes which item holds focus **now**, when the keys are as they were at
    /// the last redraw. A key can move focus (Tab) or press the item that holds
    /// it (Enter), and the redraw that would record the new position may not
    /// have come yet when the item is removed.
    fn remember(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer) {
        let mut flags = Flags::default();
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut flags);
        let state = tree.state.downcast_mut::<State>();
        if state.keys == self.keys && flags.0.len() == self.keys.len() {
            state.focused = flags.0.iter().position(|f| *f).map(|at| self.keys[at]);
        }
    }

    fn follow(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer) {
        let mut flags = Flags::default();
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut flags);
        let state = tree.state.downcast_mut::<State>();

        // One target per key, or this does not know what it is looking at.
        if flags.0.len() != self.keys.len() {
            state.keys = self.keys.clone();
            state.focused = None;
            return;
        }

        let now = flags.0.iter().position(|f| *f);
        let want = if state.keys == self.keys {
            // Nothing changed shape: focus moved only if the reader moved it.
            now
        } else if let Some(focused) = state.focused {
            belongs_on(&state.keys, &self.keys, focused)
        } else {
            now
        };

        if want != now {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut Unfocus,
            );
            if let Some(at) = want {
                self.content.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    &mut Refocus { at, seen: 0 },
                );
            }
        }

        let state = tree.state.downcast_mut::<State>();
        state.keys = self.keys.clone();
        state.focused = want.map(|at| self.keys[at]);
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Keyed<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Widget::size(self.content.as_widget())
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        match event {
            Event::Window(window::Event::RedrawRequested(_)) => {
                self.follow(tree, layout, renderer);
            }
            // The position focus is in when a key is pressed, before the key
            // acts, is what a removal by that key must remember.
            Event::Keyboard(iced_core::keyboard::Event::KeyPressed { .. }) => {
                self.remember(tree, layout, renderer);
            }
            _ => {}
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a> From<Keyed<'a, Message>> for Element<'a, Message> {
    fn from(keyed: Keyed<'a, Message>) -> Self {
        Element::new(keyed)
    }
}

#[cfg(test)]
mod tests {
    use super::belongs_on;

    #[test]
    fn focus_stays_on_its_item_when_another_goes() {
        assert_eq!(belongs_on(&[3, 2, 1], &[2, 1], 2), Some(0));
        assert_eq!(belongs_on(&[3, 2, 1], &[3, 1], 1), Some(1));
    }

    #[test]
    fn a_removed_item_hands_focus_to_the_next_then_the_previous() {
        // Middle: the one after it.
        assert_eq!(belongs_on(&[3, 2, 1], &[3, 1], 2), Some(1));
        // First: the one after it.
        assert_eq!(belongs_on(&[3, 2, 1], &[2, 1], 3), Some(0));
        // Last: the one before it.
        assert_eq!(belongs_on(&[3, 2, 1], &[3, 2], 1), Some(1));
    }

    #[test]
    fn nothing_left_is_nowhere() {
        assert_eq!(belongs_on(&[1], &[], 1), None);
    }

    #[test]
    fn the_nearest_survivor_is_used_when_the_next_one_went_too() {
        // 2 and the one after it (1) both went; 3 is what is left.
        assert_eq!(belongs_on(&[3, 2, 1], &[3], 2), Some(0));
    }
}
