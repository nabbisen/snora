//! [`Router`]: the shortcuts router (RFC-116 R-2).
//!
//! One router, placed by the engine around the page's skeleton **inside its
//! [`Inert`](crate::Inert)**, so it is silent under every overlay that blocks the
//! pointer, and answers beneath a menu with no backdrop. It is built from routes
//! the application opted into, each a key **with a modifier**, so WCAG 2.1.4
//! (single-character shortcuts) does not apply to it.
//!
//! # The rules
//!
//! - **A shortcut does exactly what a click on that control does:** the same
//!   message, focus taken away from wherever it was, and the Tab starting point
//!   (RFC-121) on the target. The target is found by its [`Tag`]; a disabled or
//!   absent target does nothing at all (no message, no focus change, the event is
//!   left `Ignored`).
//! - **A shortcut works unless the focused control uses that key.** The router
//!   delivers the key to its content **first** and answers only an event nothing
//!   captured. And **a focused text field is taken to use every arrow key, with
//!   any modifier**: on macOS `Cmd+↑` in a field means "start of text", and the
//!   router must not take a reader who is editing to another page.
//! - **The digit routes match the physical digit row; named keys match the logical
//!   key.** On an AZERTY keyboard the key marked `1` types `&`, so a route on the
//!   logical `1` would never fire there; browsers match the physical row.
//! - **It captures what it answers**, so an application's `keyboard::listen()`
//!   (which receives only what nothing captured) does not see the key again.
//!
//! Public because the engine builds it; not for applications.

use iced::keyboard::{Key, Modifiers};
use iced::{Element, Length, Rectangle, Size, Theme, Vector};
use iced_core::layout::{self, Layout};
use iced_core::widget::operation::Focusable;
use iced_core::widget::{Id, Operation, Tree, tree};
use iced_core::{Clipboard, Event, Shell, Widget, keyboard, mouse, overlay, renderer};

use crate::tag::{Tag, TagReport};

/// Which key a route is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyMatch {
    /// A named key, matched on the **logical** key (`Tab`, the arrows, `PageUp`).
    Named(keyboard::key::Named),
    /// A digit `1…9`, matched on the **physical** digit row.
    Digit(u8),
    /// A key matched on its **physical** position: for punctuation, whose logical
    /// character changes with Shift and with the layout (`Shift+Cmd+]`).
    Physical(keyboard::key::Code),
}

impl KeyMatch {
    fn matches(self, key: &Key, physical: &keyboard::key::Physical) -> bool {
        use keyboard::key::{Code, Physical};
        match self {
            Self::Named(named) => *key == Key::Named(named),
            Self::Physical(code) => *physical == Physical::Code(code),
            Self::Digit(d) => {
                let code = match d {
                    1 => Code::Digit1,
                    2 => Code::Digit2,
                    3 => Code::Digit3,
                    4 => Code::Digit4,
                    5 => Code::Digit5,
                    6 => Code::Digit6,
                    7 => Code::Digit7,
                    8 => Code::Digit8,
                    9 => Code::Digit9,
                    _ => return false,
                };
                *physical == Physical::Code(code)
            }
        }
    }
}

/// One shortcut: a key with a modifier, the message a click on its target
/// publishes, and the target.
#[derive(Debug, Clone)]
pub struct Route<Message> {
    /// The key.
    pub key: KeyMatch,
    /// The modifiers, **exactly** (a route on `Ctrl+Tab` does not answer
    /// `Ctrl+Shift+Tab`).
    pub modifiers: Modifiers,
    /// **Exactly the message a click on the target publishes.**
    pub message: Message,
    /// Which control it is a shortcut to.
    pub target: Tag,
}

/// `content`, with the shortcuts of `routes` answered after it.
pub struct Router<'a, Message> {
    content: Element<'a, Message>,
    routes: Vec<Route<Message>>,
}

impl<'a, Message> Router<'a, Message> {
    /// `content`, with `routes`.
    pub fn new(content: impl Into<Element<'a, Message>>, routes: Vec<Route<Message>>) -> Self {
        Self {
            content: content.into(),
            routes,
        }
    }
}

/// Takes focus from every target.
struct Unfocus;

impl Operation for Unfocus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        state.unfocus();
    }
}

/// Finds the control with a tag and, if asked, makes it Tab's starting point.
struct Find {
    tag: Tag,
    found: bool,
}

impl Operation for Find {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
        if let Some(report) = state.downcast_mut::<TagReport>()
            && report.tag == self.tag
        {
            report.start_here = true;
            self.found = true;
        }
    }
}

/// Whether the focused target is a text field. iced's `text_input` reports
/// `text_input` just **before** its `focusable`, so the flag is carried to the
/// next focus target.
#[derive(Default)]
struct FocusedIsTextField {
    text_input_next: bool,
    result: bool,
}

impl Operation for FocusedIsTextField {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn text_input(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _state: &mut dyn iced_core::widget::operation::TextInput,
    ) {
        self.text_input_next = true;
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() && self.text_input_next {
            self.result = true;
        }
        self.text_input_next = false;
    }
}

fn is_arrow(key: &Key) -> bool {
    use keyboard::key::Named;
    matches!(
        key,
        Key::Named(Named::ArrowUp | Named::ArrowDown | Named::ArrowLeft | Named::ArrowRight)
    )
}

impl<Message: Clone> Router<'_, Message> {
    fn answer(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        let Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            repeat: false,
            ..
        }) = event
        else {
            return;
        };
        let Some(route) = self
            .routes
            .iter()
            .find(|r| r.modifiers == *modifiers && r.key.matches(key, physical_key))
        else {
            return;
        };

        // A focused text field uses every arrow key, with any modifier.
        if is_arrow(key) {
            let mut field = FocusedIsTextField::default();
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut field,
            );
            if field.result {
                return;
            }
        }

        // The target: absent or disabled, and nothing happens.
        let mut find = Find {
            tag: route.target,
            found: false,
        };
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut find);
        if !find.found {
            return;
        }
        // What a click does to focus: it takes it away from wherever it was.
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut Unfocus);

        shell.publish(route.message.clone());
        shell.capture_event();
    }
}

impl<Message: Clone> Widget<Message, Theme, iced::Renderer> for Router<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
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
        // The content first: a focused control that uses the key wins.
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
        // Only a key nothing took.
        if !shell.is_event_captured() {
            self.answer(tree, event, layout, renderer, shell);
        }
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

impl<'a, Message: Clone + 'a> From<Router<'a, Message>> for Element<'a, Message> {
    fn from(router: Router<'a, Message>) -> Self {
        Element::new(router)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyboard::key::{Code, Named, Physical};

    #[test]
    fn a_digit_matches_the_physical_row_and_not_the_logical_key() {
        // An AZERTY-shaped key: the logical key is "&", the physical one Digit1.
        let logical_amp = Key::Character("&".into());
        assert!(KeyMatch::Digit(1).matches(&logical_amp, &Physical::Code(Code::Digit1)));
        // And a logical "1" on another physical key is not the digit row.
        let logical_one = Key::Character("1".into());
        assert!(!KeyMatch::Digit(1).matches(&logical_one, &Physical::Code(Code::KeyQ)));
    }

    #[test]
    fn a_named_key_matches_the_logical_key() {
        assert!(
            KeyMatch::Named(Named::Tab)
                .matches(&Key::Named(Named::Tab), &Physical::Code(Code::KeyQ))
        );
        assert!(
            !KeyMatch::Named(Named::Tab)
                .matches(&Key::Named(Named::Enter), &Physical::Code(Code::Tab))
        );
    }

    #[test]
    fn only_the_arrows_are_arrows() {
        assert!(is_arrow(&Key::Named(Named::ArrowUp)));
        assert!(!is_arrow(&Key::Named(Named::Tab)));
        assert!(!is_arrow(&Key::Character("a".into())));
    }
}
