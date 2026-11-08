//! A wrapper that does what an application's Tab routing does, for tests that
//! drive an `iced_test::Simulator` (RFC-121).
//!
//! The simulator has no runtime, so it cannot run `snora::keyboard::traverse`'s
//! task. Before RFC-121 these tests gave a control focus by clicking it. A click
//! no longer focuses, so they focus by Tab, and this is what turns a Tab key
//! event into the operation `traverse` runs: `snora_control::sequential_focus`,
//! the same one, run over the content.
//!
//! `iced_test::Simulator::tap_key(Named::Tab)` on `tab_router(view)` is then
//! "the user pressed Tab", and a click first makes Tab carry on from the clicked
//! control, exactly as in the window loop.

#![allow(dead_code)]

use iced::{Element, Length, Rectangle, Size, Theme, Vector};
use iced_runtime::core::layout::{self, Layout};
use iced_runtime::core::widget::operation::{self, Outcome};
use iced_runtime::core::widget::{Operation, Tree, tree};
use iced_runtime::core::{Clipboard, Event, Shell, Widget, keyboard, mouse, overlay, renderer};

struct Router<'a, Message> {
    content: Element<'a, Message>,
}

/// `content`, with Tab and Shift+Tab routed to `sequential_focus`.
pub fn tab_router<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    Element::new(Router {
        content: content.into(),
    })
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Router<'_, Message> {
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
        if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event
            && *key == keyboard::Key::Named(keyboard::key::Named::Tab)
        {
            // Run it to completion, as the window loop does.
            let mut current: Box<dyn Operation<()>> =
                Box::new(snora_control::sequential_focus::<()>(!modifiers.shift()));
            loop {
                self.content.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    &mut operation::black_box(current.as_mut()),
                );
                match current.finish() {
                    Outcome::None | Outcome::Some(()) => break,
                    Outcome::Chain(next) => current = next,
                }
            }
            shell.capture_event();
            return;
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
