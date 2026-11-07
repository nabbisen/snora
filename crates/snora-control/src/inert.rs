//! [`Inert`]: a layer the keyboard cannot reach (RFC-119, Unit 0 prototype).
//!
//! # What it is for
//!
//! A modal stops the pointer (its dim captures the press) and not the
//! keyboard: `focus_next` walks every focusable in the window's tree, and a
//! key press reaches every layer of a `stack` that nobody captured. So with a
//! dialog open, Tab lands on a control the dim is covering and Enter presses
//! it. `Inert` wraps a layer the pointer cannot reach and, while it is inert:
//!
//! - reports **no focus target** from its content (`operate` hides
//!   `focusable`; `container`, `text`, `scrollable` and the rest still pass, so
//!   selectors and tests still find the content);
//! - delivers **no keyboard or input-method event** to its content;
//! - leaves drawing, layout and the pointer alone: pointer blocking is the
//!   backdrops' job, and an animation beneath must keep running.
//!
//! # Always in the tree
//!
//! The runtime keeps a widget's state by its place in the tree, so wrapping a
//! layer **only while a modal is open** would reshape the tree twice (on open
//! and on close) and reset the page each time: scroll offsets, a text input's
//! cursor, focus. `Inert` is therefore built around the layer **always**, and
//! switched by a flag.
//!
//! # The opener's focus
//!
//! When it becomes inert, the control that held focus in its content loses it
//! (so no ring is drawn under the dim) and `Inert` remembers which it was; when
//! it becomes reachable again, that control gets its focus back.
//!
//! **How the control is identified: by its position among the content's focus
//! targets, with their count.** Not by `widget::Id`: a `Control`'s id is made
//! when it is built, and every `view` builds again. The position survives a
//! rebuild because `Inert`'s own state does (it is always in the tree), and it
//! is checked against the count: if the layer's focus targets changed while
//! covered, nothing is restored, and focus is simply nowhere, which is what a
//! user can predict.
//!
//! **The application's own choice wins.** If anything in the layer has focus
//! when it becomes reachable again (the application closed the modal and moved
//! focus itself, say to a search field, with a focus task), nothing is
//! restored: two targets would otherwise hold focus, and the explicit choice
//! would be overridden.
//!
//! > **When a modal closes, focus returns to the control that had it, unless
//! > the page now has a different number of controls, or the application has
//! > focused something itself. Then focus is wherever the application put it,
//! > or nowhere.**
//!
//! # Overlays
//!
//! iced draws overlays above every layer and delivers events to them first. A
//! tooltip or a `pick_list`'s dropdown that was open on the page when a modal
//! opened would therefore stay **above the dim** and keep taking the pointer and
//! the keys. While inert, `Inert` reports **no overlay**; the content's state is
//! untouched, so the overlay returns if it is still open when the layer is
//! uncovered.
//!
//! **When it runs: on the `RedrawRequested` event,** which the runtime sends
//! before every draw (`iced_winit` loops `update(RedrawRequested)` and then
//! `draw`; `Simulator::snapshot` does the same). `diff` has no layout and no
//! renderer, and `update` has both, so it can run the focus operation on its
//! own content directly. No frame is drawn with the ring under the dim.

use iced::{Element, Length, Rectangle, Size, Theme, Vector};
use iced_core::layout::{self, Layout};
use iced_core::widget::operation::Focusable;
use iced_core::widget::{Id, Operation, Tree, tree};
use iced_core::{Clipboard, Event, Shell, Widget, mouse, overlay, renderer, window};

/// What `Inert` remembers while its layer is covered.
#[derive(Default)]
struct State {
    /// Has the layer been made inert (and its focus parked) already?
    engaged: bool,
    /// The position of the focus target that held focus, among `count`.
    parked: Option<(usize, usize)>,
}

/// A layer that the keyboard cannot reach while `inert` is set.
pub struct Inert<'a, Message> {
    content: Element<'a, Message>,
    inert: bool,
}

impl<'a, Message> Inert<'a, Message> {
    /// `content`, unreachable by the keyboard while `inert` is true.
    pub fn new(content: impl Into<Element<'a, Message>>, inert: bool) -> Self {
        Self {
            content: content.into(),
            inert,
        }
    }
}

/// An operation that does not see focus targets.
struct Hide<'a>(&'a mut dyn Operation);

impl Operation for Hide<'_> {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        self.0.traverse(&mut |inner| {
            let mut hidden = Hide(inner);
            operate(&mut hidden);
        });
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        self.0.container(id, bounds);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        state: &mut dyn iced_core::widget::operation::Scrollable,
    ) {
        self.0
            .scrollable(id, bounds, content_bounds, translation, state);
    }

    // `focusable` is the one thing not forwarded.

    fn text_input(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        state: &mut dyn iced_core::widget::operation::TextInput,
    ) {
        self.0.text_input(id, bounds, state);
    }

    fn text(&mut self, id: Option<&Id>, bounds: Rectangle, text: &str) {
        self.0.text(id, bounds, text);
    }

    fn custom(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn std::any::Any) {
        self.0.custom(id, bounds, state);
    }
}

/// Counts the focus targets, and finds the focused one.
#[derive(Default)]
struct Survey {
    seen: usize,
    focused: Option<usize>,
}

impl Operation for Survey {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() && self.focused.is_none() {
            self.focused = Some(self.seen);
        }
        self.seen += 1;
    }
}

/// Takes focus from every target (so none is left focused under the dim).
struct Unfocus;

impl Operation for Unfocus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        state.unfocus();
    }
}

/// Gives focus to the target at one position.
struct Refocus {
    at: usize,
    seen: usize,
}

impl Operation for Refocus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if self.seen == self.at {
            state.focus();
        }
        self.seen += 1;
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Inert<'_, Message> {
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
        // Called by path: `scripts/check-literal-sizes.sh` reads a zero-argument
        // call of the accessor's name as a text size.
        Widget::size(self.content.as_widget())
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    /// The content's own node: `Inert` has no box of its own.
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
        if self.inert {
            let mut hidden = Hide(operation);
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut hidden,
            );
        } else {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        }
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
        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            self.sync(tree, layout, renderer);
        }

        // The keyboard does not reach a covered layer, and neither does text
        // input. Everything else does: window events, redraws, the pointer.
        if self.inert && matches!(event, Event::Keyboard(_) | Event::InputMethod(_)) {
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
        tree: &tree::Tree,
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
        // A covered layer has no overlay: it would float above the dim.
        if self.inert {
            return None;
        }

        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<Message> Inert<'_, Message> {
    /// Brings the content's focus in line with the flag: park it on the way
    /// in, give it back on the way out.
    fn sync(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer) {
        let state = tree.state.downcast_mut::<State>();

        if self.inert && !state.engaged {
            let mut survey = Survey::default();
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut survey,
            );
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut Unfocus,
            );
            let state = tree.state.downcast_mut::<State>();
            state.parked = survey.focused.map(|at| (at, survey.seen));
            state.engaged = true;
        } else if !self.inert && state.engaged {
            let parked = state.parked.take();
            state.engaged = false;
            if let Some((at, count)) = parked {
                let mut survey = Survey::default();
                self.content.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    &mut survey,
                );
                // Only if the layer still has the same focus targets (a
                // position means nothing in a different layer), and only if
                // the application has not focused something itself.
                if survey.seen == count && survey.focused.is_none() {
                    self.content.as_widget_mut().operate(
                        &mut tree.children[0],
                        layout,
                        renderer,
                        &mut Refocus { at, seen: 0 },
                    );
                }
            }
        }
    }
}

impl<'a, Message: 'a> From<Inert<'a, Message>> for Element<'a, Message> {
    fn from(inert: Inert<'a, Message>) -> Self {
        Element::new(inert)
    }
}

#[cfg(test)]
mod tests;
