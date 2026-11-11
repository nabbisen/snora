//! **PROTOTYPE (RFC-123 Unit 0, second plan).** Arrow keys inside snora's menus, on
//! the anchored dropdown. Not for landing as it stands.
//!
//! # The problem the first plan could not solve
//!
//! A widget over the layer stack that asks its content to `operate` sees the base tree,
//! and **not an overlay**: since RFC-124 the header dropdown is an overlay, and iced's
//! `UserInterface::update` offers every event to the overlay **before** the widget tree.
//! So the overlay sees only its own items, and the tree sees only its own controls, and
//! rule 4 needs both ("focus is on this menu, or nowhere in the window").
//!
//! # One widget, two stages, one decision
//!
//! [`MenuKeys`] wraps the engine's layer stack.
//! - **Overlay stage.** In `overlay()`, before the content's overlay is made, it surveys
//!   the base tree (the only moment both borrows are free), then wraps the content's
//!   overlay. The wrapper is offered every event first. For an arrow key the overlay did
//!   not take, it asks the wrapped overlay to `operate` (a survey of the overlay), adds the
//!   base survey, and decides.
//! - **Root stage.** Whatever the overlay stage did not finish: the base tree is surveyed
//!   again, now fresh, and a menu **in** the base tree (a context menu is a layer of the
//!   stack) is answered here. The overlay stage's one base-side effect (the trigger loses
//!   focus when the focus moves into the dropdown) is applied here too.
//!
//! [`decide`] is the single rule, a pure function of the focus targets in order.

use iced::{Element, Length, Rectangle, Size, Theme, Vector};
use iced_core::layout::{self, Layout};
use iced_core::widget::operation::Focusable;
use iced_core::widget::{Id, Operation, Tree, tree};
use iced_core::{Clipboard, Event, Shell, Widget, keyboard, mouse, overlay, renderer};
use std::any::Any;
use std::cell::Cell;

/// Which part of a menu a marked region is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Part {
    /// The control that opens the menu.
    Trigger,
    /// The items of the open dropdown.
    Items,
}

/// A region entered or left, reported through `operation.custom`.
#[derive(Debug, Clone, Copy)]
struct Mark {
    part: Part,
    enter: bool,
}

/// Runs `inner` with `part` marked around it.
pub(crate) fn marked(
    part: Part,
    bounds: Rectangle,
    operation: &mut dyn Operation,
    inner: impl FnOnce(&mut dyn Operation),
) {
    let mut mark = Mark { part, enter: true };
    operation.custom(None, bounds, &mut mark);
    inner(operation);
    mark.enter = false;
    operation.custom(None, bounds, &mut mark);
}

/// A menu's trigger, or its items, marked as such (for a menu that is not anchored: the
/// context menu is drawn inline).
pub struct MenuPart<'a, Message> {
    content: Element<'a, Message>,
    part: Part,
}

impl<'a, Message> MenuPart<'a, Message> {
    /// The control that opens the menu.
    pub fn trigger(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            part: Part::Trigger,
        }
    }

    /// The items of the open menu.
    pub fn items(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            part: Part::Items,
        }
    }
}

/// One focus target, as a survey sees it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Target {
    part: Option<Part>,
    focused: bool,
}

#[derive(Default)]
struct Survey {
    depth: Vec<Part>,
    targets: Vec<Target>,
}

impl Operation for Survey {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _b: Rectangle, state: &mut dyn Any) {
        if let Some(mark) = state.downcast_mut::<Mark>() {
            if mark.enter {
                self.depth.push(mark.part);
            } else {
                self.depth.pop();
            }
        }
    }

    fn focusable(&mut self, _id: Option<&Id>, _b: Rectangle, state: &mut dyn Focusable) {
        self.targets.push(Target {
            part: self.depth.last().copied(),
            focused: state.is_focused(),
        });
    }
}

/// Focuses the `at`-th target and unfocuses every other.
struct FocusAt {
    at: usize,
    seen: usize,
}

impl Operation for FocusAt {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _b: Rectangle, state: &mut dyn Focusable) {
        if self.seen == self.at {
            state.focus();
        } else {
            state.unfocus();
        }
        self.seen += 1;
    }
}

/// Unfocuses every target.
struct UnfocusAll;

impl Operation for UnfocusAll {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _b: Rectangle, state: &mut dyn Focusable) {
        state.unfocus();
    }
}

/// What a key does, given the focus targets of one side in order and whether focus is
/// anywhere else. Returns the position **among `targets`** to focus; `None`: not ours.
///
/// Rule 4: answered only while focus is on an item or the trigger, or nowhere at all.
fn decide(key: keyboard::key::Named, targets: &[Target], elsewhere: bool) -> Option<usize> {
    use keyboard::key::Named::{ArrowDown, ArrowUp, End, Home};
    let items: Vec<usize> = targets
        .iter()
        .enumerate()
        .filter(|(_, t)| t.part == Some(Part::Items))
        .map(|(i, _)| i)
        .collect();
    let n = items.len();
    if n == 0 || elsewhere {
        return None;
    }
    let focused = targets.iter().position(|t| t.focused);
    let on_item = focused.and_then(|f| items.iter().position(|&i| i == f));
    match (on_item, focused) {
        (Some(p), _) => Some(match key {
            ArrowDown => items[(p + 1) % n],
            ArrowUp => items[(p + n - 1) % n],
            Home => items[0],
            End => items[n - 1],
            _ => return None,
        }),
        // The menu's own trigger, or nothing focused anywhere.
        (None, None) => Some(match key {
            ArrowDown | Home => items[0],
            ArrowUp | End => items[n - 1],
            _ => return None,
        }),
        (None, Some(f)) if targets[f].part == Some(Part::Trigger) => Some(match key {
            ArrowDown | Home => items[0],
            ArrowUp | End => items[n - 1],
            _ => return None,
        }),
        // A text field or a page control: it keeps its arrows.
        _ => None,
    }
}

/// The plain key this event is, if it is one of ours.
fn ours(event: &Event) -> Option<keyboard::key::Named> {
    let Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(named),
        modifiers,
        ..
    }) = event
    else {
        return None;
    };
    matches!(
        named,
        keyboard::key::Named::ArrowDown
            | keyboard::key::Named::ArrowUp
            | keyboard::key::Named::Home
            | keyboard::key::Named::End
    )
    .then_some(())
    .filter(|()| modifiers.is_empty())
    .map(|()| *named)
}

/// State kept in the tree: what the overlay stage leaves for the root stage.
#[derive(Default)]
struct Pending {
    /// Keys the overlay stage moved focus for, in this `update`, not yet confirmed. The
    /// overlay stage never captures: iced offers a batch of events to the overlay before
    /// the tree, so its view of the base tree is as of the start of the batch, and only the
    /// root stage sees the tree as each key reaches it.
    moved: Cell<usize>,
    /// The root stage found the base tree had taken focus after the overlay stage looked
    /// (a click on a field in the same batch): the overlay's items give theirs up, at the
    /// next chance the overlay is alive.
    repair: Cell<bool>,
}

/// `content`, with ↓ ↑ Home End answered for an open menu.
pub struct MenuKeys<'a, Message> {
    content: Element<'a, Message>,
}

impl<'a, Message> MenuKeys<'a, Message> {
    /// `content`.
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl<Message: Clone> Widget<Message, Theme, iced::Renderer> for MenuKeys<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Pending>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Pending::default())
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
        // The overlay stage moved focus for this key: confirm it against the tree as it is
        // now (events of a batch reach the tree one at a time, in order).
        if ours(event).is_some() {
            let pending = tree.state.downcast_ref::<Pending>();
            if pending.moved.get() > 0 {
                pending.moved.set(pending.moved.get() - 1);
                let mut survey = Survey::default();
                self.content.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    &mut survey,
                );
                if survey.targets.iter().any(|t| t.focused && t.part.is_none()) {
                    // Something in the page took focus first: it keeps the key.
                    pending.repair.set(true);
                } else {
                    // The trigger, if it held focus, gives it up to the menu.
                    self.content.as_widget_mut().operate(
                        &mut tree.children[0],
                        layout,
                        renderer,
                        &mut UnfocusAll,
                    );
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
            }
        }

        // Root stage: a menu in the base tree. Decided **before** the tree is offered the
        // key, so that the page never sees a key the menu took (the dropdown's key was
        // never offered to it: the overlay comes first). If the focus is anywhere else
        // (a text field, a page control), `decide` says no and the tree gets the key.
        if let Some(key) = ours(event) {
            let mut survey = Survey::default();
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut survey,
            );
            // One survey sees all focus here, so `decide` needs no word about "elsewhere".
            if let Some(at) = decide(key, &survey.targets, false) {
                self.content.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    &mut FocusAt { at, seen: 0 },
                );
                shell.capture_event();
                shell.request_redraw();
                return;
            }
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
        tree.state.downcast_ref::<Pending>().moved.set(0);
        // The base tree, now: this is the one moment it is free to be asked, because the
        // content's overlay (made next) borrows it until the event is done.
        let mut survey = Survey::default();
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut survey);

        let Tree {
            state, children, ..
        } = tree;
        let inner = self.content.as_widget_mut().overlay(
            &mut children[0],
            layout,
            renderer,
            viewport,
            translation,
        )?;
        Some(overlay::Element::new(Box::new(KeyOverlay {
            inner,
            base: survey.targets,
            pending: state.downcast_ref::<Pending>(),
        })))
    }
}

impl<'a, Message: Clone + 'a> From<MenuKeys<'a, Message>> for Element<'a, Message> {
    fn from(w: MenuKeys<'a, Message>) -> Self {
        Element::new(w)
    }
}

/// The content's overlay, offered every key first.
struct KeyOverlay<'a, Message> {
    inner: overlay::Element<'a, Message, Theme, iced::Renderer>,
    /// The base tree's focus targets, as of this build of the overlay.
    base: Vec<Target>,
    pending: &'a Pending,
}

impl<Message> overlay::Overlay<Message, Theme, iced::Renderer> for KeyOverlay<'_, Message> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        self.inner.as_overlay_mut().layout(renderer, bounds)
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.inner
            .as_overlay()
            .draw(renderer, theme, style, layout, cursor);
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.inner
            .as_overlay_mut()
            .operate(layout, renderer, operation);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        if self.pending.repair.take() {
            self.inner
                .as_overlay_mut()
                .operate(layout, renderer, &mut UnfocusAll);
        }
        self.inner
            .as_overlay_mut()
            .update(event, layout, cursor, renderer, clipboard, shell);
        if shell.is_event_captured() {
            return;
        }
        let Some(key) = ours(event) else {
            return;
        };

        let mut survey = Survey::default();
        self.inner
            .as_overlay_mut()
            .operate(layout, renderer, &mut survey);
        if !survey.targets.iter().any(|t| t.part == Some(Part::Items)) {
            // No dropdown here: a menu in the base tree is the root stage's.
            return;
        }
        let elsewhere = self.base.iter().any(|t| t.focused && t.part.is_none())
            || survey.targets.iter().any(|t| t.focused && t.part.is_none());
        let Some(at) = decide(key, &survey.targets, elsewhere) else {
            return;
        };
        // Through `operate`, so the dropdown brings the item into view as it does for Tab.
        self.inner
            .as_overlay_mut()
            .operate(layout, renderer, &mut FocusAt { at, seen: 0 });
        shell.request_redraw();
        // Not captured here: the root stage confirms it, and captures.
        self.pending.moved.set(self.pending.moved.get() + 1);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.inner
            .as_overlay()
            .mouse_interaction(layout, cursor, renderer)
    }

    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
    ) -> Option<overlay::Element<'a, Message, Theme, iced::Renderer>> {
        self.inner.as_overlay_mut().overlay(layout, renderer)
    }

    fn index(&self) -> f32 {
        self.inner.as_overlay().index()
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for MenuPart<'_, Message> {
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
        marked(self.part, layout.bounds(), operation, |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        });
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

impl<'a, Message: 'a> From<MenuPart<'a, Message>> for Element<'a, Message> {
    fn from(w: MenuPart<'a, Message>) -> Self {
        Element::new(w)
    }
}
