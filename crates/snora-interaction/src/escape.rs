//! [`OnEscape`]: a layer that answers Escape (RFC-119 R-4).
//!
//! A backdrop is a `mouse_area`, which takes no keys. Whenever a backdrop has
//! a close message, Escape must send it, or a keyboard user who can no longer
//! reach "outside" (the layers beneath are inert) is trapped in the overlay
//! (WCAG 2.1.2). This widget publishes its message on Escape and **captures**
//! the event, so a layer beneath it does not also answer, and an application's
//! own `keyboard::listen()` (which sees only events nobody captured) does not
//! receive a second Escape.

use iced::keyboard::{self, key};
use iced::{Element, Length, Rectangle, Size, Theme, Vector};
use iced_core::layout::{self, Layout};
use iced_core::widget::{Operation, Tree};
use iced_core::{Clipboard, Event, Shell, Widget, mouse, overlay, renderer};

/// A layer that publishes `message` when Escape is pressed.
pub struct OnEscape<'a, Message> {
    content: Element<'a, Message>,
    message: Option<Message>,
}

impl<'a, Message> OnEscape<'a, Message> {
    /// `content`, answering Escape with `message`.
    pub fn new(content: impl Into<Element<'a, Message>>, message: Message) -> Self {
        Self::maybe(content, Some(message))
    }

    /// `content`, answering Escape with `message` **if there is one**. The layer
    /// is built either way, so an application that sets or clears its Escape
    /// message at run time does not reshape the tree (and so reset what the
    /// runtime keeps by position) (RFC-116 R-2).
    pub fn maybe(content: impl Into<Element<'a, Message>>, message: Option<Message>) -> Self {
        Self {
            content: content.into(),
            message,
        }
    }
}

impl<Message: Clone> Widget<Message, Theme, iced::Renderer> for OnEscape<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        // Called by path: `scripts/check-literal-sizes.sh` reads a zero-argument
        // call of the accessor's name as a text size.
        Widget::size(self.content.as_widget())
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
        // **The content first** (RFC-116 R-2): a focused `text_input` in a dialog
        // takes Escape for itself (it blurs), and the second press closes the
        // dialog, as iced does and as RFC-119 measured. A backdrop's content takes
        // no key, so for the backdrops this changes nothing.
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

        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key::Named::Escape),
            ..
        }) = event
            && !shell.is_event_captured()
            && let Some(message) = &self.message
        {
            shell.publish(message.clone());
            shell.capture_event();
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

impl<'a, Message: Clone + 'a> From<OnEscape<'a, Message>> for Element<'a, Message> {
    fn from(escape: OnEscape<'a, Message>) -> Self {
        Element::new(escape)
    }
}
