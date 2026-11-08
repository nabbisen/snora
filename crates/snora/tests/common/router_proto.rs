//! **A prototype shortcuts router, for RFC-116 R-2 Unit 0. Nothing here ships.**
//!
//! It is what the amendment proposes, written as small as it can be so that the
//! measurements are about the *placement and the rules*, not the polish:
//!
//! - one widget, placed **inside the page** (so inside `Inert`, which is where
//!   the engine would put it);
//! - **opt-in routes**, each a key with a modifier, and the message a click on
//!   that control publishes;
//! - **B2:** the router sees a key **after** its content has had it
//!   (`Order::AfterChildren`), and answers only an event nothing captured;
//! - **B1:** when it answers, it does what a click does: publishes the message,
//!   takes focus away from wherever it was, and sets the target's Tab starting
//!   point (`start_here`, RFC-121). A target it cannot find does nothing at all.
//!
//! `Order::BeforeChildren` and `B1::Off` exist only so that a test can show the
//! rule it breaks (a breakage row that could not fail would prove nothing).

#![allow(dead_code)]

use iced::keyboard::{Key, Modifiers};
use iced::{Element, Length, Rectangle, Size, Theme, Vector};
use iced_runtime::core::layout::{self, Layout};
use iced_runtime::core::widget::operation;
use iced_runtime::core::widget::{Id, Operation, Tree, tree};
use iced_runtime::core::{Clipboard, Event, Shell, Widget, keyboard, mouse, overlay, renderer};

/// Which control a route is a shortcut *to*: the `index`-th focus target (that
/// is enabled) inside the container named `region`.
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub region: &'static str,
    pub index: usize,
}

#[derive(Clone)]
pub struct Route<M> {
    pub key: Key,
    pub modifiers: Modifiers,
    /// **Exactly the message a click on the target publishes.**
    pub message: M,
    pub target: Target,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Order {
    /// B2 as proposed: the content has the key first.
    AfterChildren,
    /// Broken on purpose, to show what B2 prevents.
    BeforeChildren,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum B1 {
    On,
    /// Broken on purpose: publish the message and nothing else.
    Off,
}

pub struct Router<'a, M> {
    content: Element<'a, M>,
    routes: Vec<Route<M>>,
    order: Order,
    b1: B1,
}

impl<'a, M: Clone + 'a> Router<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>, routes: Vec<Route<M>>) -> Self {
        Self {
            content: content.into(),
            routes,
            order: Order::AfterChildren,
            b1: B1::On,
        }
    }

    pub fn order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }

    pub fn b1(mut self, b1: B1) -> Self {
        self.b1 = b1;
        self
    }
}

/// Does nothing: stands in for `Unfocus` in a breakage row.
struct UnfocusNothing;

impl Operation for UnfocusNothing {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }
}

/// Takes focus from every target.
struct Unfocus;

impl Operation for Unfocus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        state.unfocus();
    }
}

/// Sets Tab's starting point (`start_here`) on the target, if it can find it.
///
/// A `Control` reports `start_here` through `operation.custom` right after its
/// focus target, **only when it is enabled**; so a disabled control is not
/// there to be found, and a target that is not found is a shortcut that does
/// nothing.
struct StartHere {
    region: Id,
    index: usize,
    /// Containers entered, innermost last: `Some(id)` where one has an id.
    stack: Vec<Option<Id>>,
    pending: Option<Id>,
    seen: usize,
    pub found: bool,
}

impl StartHere {
    fn new(target: Target) -> Self {
        Self {
            region: Id::new(target.region),
            index: target.index,
            stack: Vec::new(),
            pending: None,
            seen: 0,
            found: false,
        }
    }
}

impl Operation for StartHere {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        self.stack.push(self.pending.take());
        operate(self);
        self.stack.pop();
    }

    fn container(&mut self, id: Option<&Id>, _bounds: Rectangle) {
        self.pending = id.cloned();
    }

    fn custom(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn std::any::Any) {
        let inside = self
            .stack
            .iter()
            .any(|scope| scope.as_ref() == Some(&self.region));
        if let Some(start_here) = state.downcast_mut::<bool>()
            && inside
        {
            if self.seen == self.index {
                *start_here = true;
                self.found = true;
            }
            self.seen += 1;
        }
    }
}

impl<M: Clone> Router<'_, M> {
    fn answer(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        shell: &mut Shell<'_, M>,
    ) -> bool {
        let Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            repeat: false,
            ..
        }) = event
        else {
            return false;
        };
        let Some(route) = self
            .routes
            .iter()
            .find(|r| r.key == *key && r.modifiers == *modifiers)
        else {
            return false;
        };

        if self.b1 == B1::On {
            // Is the target there? A disabled one is not, and then nothing happens.
            let mut find = StartHere::new(route.target);
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut find,
            );
            if !find.found {
                return false;
            }
            // What a click does to focus: it takes it away from wherever it was.
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut Unfocus,
            );
        }

        shell.publish(route.message.clone());
        shell.capture_event();
        true
    }
}

impl<M: Clone> Widget<M, Theme, iced::Renderer> for Router<'_, M> {
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
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        if self.order == Order::BeforeChildren && self.answer(tree, event, layout, renderer, shell)
        {
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
        // **B2.** Only an event nothing took.
        if self.order == Order::AfterChildren && !shell.is_event_captured() {
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
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, M: Clone + 'a> From<Router<'a, M>> for Element<'a, M> {
    fn from(router: Router<'a, M>) -> Self {
        Element::new(router)
    }
}

/// A widget that takes every key press, as a focused editor might: the router
/// must stay silent when its content captured the key.
pub struct Captures<'a, M> {
    content: Element<'a, M>,
}

impl<'a, M: 'a> Captures<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl<M> Widget<M, Theme, iced::Renderer> for Captures<'_, M> {
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
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
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
        if matches!(event, Event::Keyboard(keyboard::Event::KeyPressed { .. })) {
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
}

impl<'a, M: 'a> From<Captures<'a, M>> for Element<'a, M> {
    fn from(c: Captures<'a, M>) -> Self {
        Element::new(c)
    }
}
