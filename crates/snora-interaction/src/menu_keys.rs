//! **PROTOTYPE (RFC-123 Unit 0b).** Arrow keys inside snora's menus: one stage, one survey,
//! taken when the key reaches the root. Not for landing as it stands.
//!
//! # The problem
//!
//! The header's dropdown is an overlay (RFC-124), and iced's `UserInterface::update` offers
//! every event to the overlay **before** the widget tree. Rule 4 ("answered only while focus is
//! on this menu, or nowhere in the window") needs the dropdown's items *and* the page's
//! controls in one view. The first plan (Unit 0) got there with two stages, a counted move and
//! a repair; this one has no stage in the overlay at all.
//!
//! # One stage
//!
//! [`MenuKeys`] wraps the engine's layer stack. For a plain ↓ ↑ Home End it runs **one walk**
//! over the tree **as it is when the key reaches the root** (events of a batch reach the root
//! one at a time, each after the one before has been handled), decides, and, if the key is
//! ours, runs a second walk that moves focus. A key that is not ours costs nothing.
//!
//! The overlay is dropped before the root is updated, so the root's `operate` is free to reach
//! the dropdown: **through [`MenuAnchor`](crate::MenuAnchor)**, which keeps the layout its
//! overlay was last given and, when the walk asks ([`Reach`]), operates its dropdown with it.
//! Any other operation (a Tab, a focus task) never asks, so it sees each dropdown item exactly
//! once, through the overlay, as before.
//!
//! [`decide`] is the single rule, a pure function of the focus targets in order.

use iced::{Element, Length, Rectangle, Size, Theme, Vector};
use iced_core::layout::{self, Layout};
use iced_core::widget::operation::Focusable;
use iced_core::widget::{Id, Operation, Tree, tree};
use iced_core::{Clipboard, Event, Shell, Widget, keyboard, mouse, overlay, renderer};
use std::any::Any;

/// Which part of a menu a marked region is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Part {
    /// The control that opens the menu.
    Trigger,
    /// The items of the open menu.
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

/// What an operation wants of an anchored dropdown.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Wants {
    /// Nothing: every operation but the arrows'.
    #[default]
    Nothing,
    /// Report which items hold focus.
    Survey,
    /// Focus this item (and no other), and bring it into view.
    Focus(usize),
}

/// Offered by an open [`MenuAnchor`](crate::MenuAnchor) to **every** operation passing it, and
/// answered by the arrows' walk alone: the dropdown is an overlay, so the anchor's `operate`
/// does not reach it. Plain data, so it passes `Inert`'s `Hide` (which forwards `custom`);
/// `Inert` drops it when a modal covers the layer, which is what keeps the menu silent under a
/// dialog.
#[derive(Debug, Default)]
pub(crate) struct Reach {
    /// Set by the operation that wants the dropdown.
    pub(crate) wants: Wants,
    /// The dropdown's items, in order, and whether each holds focus (filled for `Survey`).
    pub(crate) items: Vec<bool>,
    /// The anchor has answered: the operation reads `items`.
    pub(crate) answered: bool,
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

/// Where a focus target is, for the walk that moves focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Home {
    /// The `n`-th focus target the walk meets in the tree.
    Tree(usize),
    /// The `n`-th item of an anchored dropdown.
    Dropdown(usize),
}

/// One focus target, as the survey sees it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Target {
    part: Option<Part>,
    focused: bool,
    home: Home,
}

/// The first walk: every focus target, in order, and which are a menu's.
#[derive(Default)]
struct Survey {
    depth: Vec<Part>,
    targets: Vec<Target>,
    in_tree: usize,
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
        } else if let Some(reach) = state.downcast_mut::<Reach>() {
            if reach.answered {
                for (n, focused) in reach.items.iter().enumerate() {
                    self.targets.push(Target {
                        part: Some(Part::Items),
                        focused: *focused,
                        home: Home::Dropdown(n),
                    });
                }
            } else {
                reach.wants = Wants::Survey;
            }
        }
    }

    fn focusable(&mut self, _id: Option<&Id>, _b: Rectangle, state: &mut dyn Focusable) {
        self.targets.push(Target {
            part: self.depth.last().copied(),
            focused: state.is_focused(),
            home: Home::Tree(self.in_tree),
        });
        self.in_tree += 1;
    }
}

/// The second walk: focus goes to `to`, and to nothing else.
struct Act {
    to: Home,
    seen: usize,
}

impl Operation for Act {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn custom(&mut self, _id: Option<&Id>, _b: Rectangle, state: &mut dyn Any) {
        if let (Home::Dropdown(n), Some(reach)) = (self.to, state.downcast_mut::<Reach>()) {
            reach.wants = Wants::Focus(n);
        }
    }

    fn focusable(&mut self, _id: Option<&Id>, _b: Rectangle, state: &mut dyn Focusable) {
        if self.to == Home::Tree(self.seen) {
            state.focus();
        } else {
            state.unfocus();
        }
        self.seen += 1;
    }
}

/// What a key does, given the focus targets in order. Returns the position **among
/// `targets`** to focus; `None`: not ours.
///
/// Rule 4: answered only while focus is on an item or the trigger, or nowhere at all.
fn decide(key: keyboard::key::Named, targets: &[Target]) -> Option<usize> {
    use keyboard::key::Named::{ArrowDown, ArrowUp, End, Home};
    let items: Vec<usize> = targets
        .iter()
        .enumerate()
        .filter(|(_, t)| t.part == Some(Part::Items))
        .map(|(i, _)| i)
        .collect();
    let n = items.len();
    if n == 0 {
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
        // One stage, decided **before** the tree is offered the key: the page never sees a key
        // the menu took. If focus is anywhere else (a text field, a page control) `decide` says
        // no and the tree gets the key as if no menu were open.
        if let Some(key) = ours(event) {
            let mut survey = Survey::default();
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                &mut survey,
            );
            if let Some(at) = decide(key, &survey.targets) {
                self.content.as_widget_mut().operate(
                    &mut tree.children[0],
                    layout,
                    renderer,
                    &mut Act {
                        to: survey.targets[at].home,
                        seen: 0,
                    },
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
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: Clone + 'a> From<MenuKeys<'a, Message>> for Element<'a, Message> {
    fn from(w: MenuKeys<'a, Message>) -> Self {
        Element::new(w)
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
