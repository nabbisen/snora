//! **PROTOTYPE (RFC-123 Unit 0).** Arrow keys inside snora's menus.
//!
//! Two pieces:
//! - [`MenuPart`] marks a menu's trigger and its items, through `operation.custom`
//!   (the same channel `Group` uses), so a survey of the window's focus targets can say
//!   which belong to a menu;
//! - [`MenuKeys`] wraps the engine's whole layer stack. It hands every event to its
//!   content first, and answers ↓ ↑ Home End only if nothing captured the key, a menu is
//!   open, and focus is **on one of the menu's items, on its trigger, or nowhere at all**.
//!   A focused text field or page control is neither, so it keeps its arrows.

use iced::{Element, Length, Rectangle, Size, Theme, Vector};
use iced_core::layout::{self, Layout};
use iced_core::widget::operation::Focusable;
use iced_core::widget::{Id, Operation, Tree, tree};
use iced_core::{Clipboard, Event, Shell, Widget, keyboard, mouse, overlay, renderer};
use std::any::Any;

/// Which part of a menu a marked region is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
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

/// `content`, marked as a menu's `part`.
pub struct MenuPart<'a, Message> {
    content: Element<'a, Message>,
    part: Part,
}

impl<'a, Message> MenuPart<'a, Message> {
    /// Mark `content` as the menu's `part`.
    pub fn new(content: impl Into<Element<'a, Message>>, part: Part) -> Self {
        Self {
            content: content.into(),
            part,
        }
    }
}

/// `content`, with ↓ ↑ Home End answered for an open menu. "Open" is not a flag: a menu
/// is open when its dropdown's items are in the tree, and only then are there items to
/// move between.
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

#[derive(Default)]
struct Survey {
    depth: Vec<Part>,
    text_next: bool,
    targets: Vec<(Option<Part>, bool, bool)>, // part, focused, is text field
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
    fn text_input(
        &mut self,
        _id: Option<&Id>,
        _b: Rectangle,
        _s: &mut dyn iced_core::widget::operation::TextInput,
    ) {
        self.text_next = true;
    }
    fn focusable(&mut self, _id: Option<&Id>, _b: Rectangle, state: &mut dyn Focusable) {
        self.targets.push((
            self.depth.last().copied(),
            state.is_focused(),
            self.text_next,
        ));
        self.text_next = false;
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

/// What a key does, given the survey. `None`: not ours.
fn decide(key: keyboard::key::Named, targets: &[(Option<Part>, bool, bool)]) -> Option<usize> {
    use keyboard::key::Named::{ArrowDown, ArrowUp, End, Home};
    let items: Vec<usize> = targets
        .iter()
        .enumerate()
        .filter(|(_, t)| t.0 == Some(Part::Items))
        .map(|(i, _)| i)
        .collect();
    if items.is_empty() {
        return None;
    }
    let focused = targets.iter().position(|t| t.1);
    let on_item = focused.and_then(|f| items.iter().position(|&i| i == f));
    let on_trigger = focused.is_some_and(|f| targets[f].0 == Some(Part::Trigger));
    let n = items.len();
    match (on_item, focused, on_trigger) {
        (Some(p), _, _) => Some(match key {
            ArrowDown => items[(p + 1) % n],
            ArrowUp => items[(p + n - 1) % n],
            Home => items[0],
            End => items[n - 1],
            _ => return None,
        }),
        // Nothing focused anywhere, or the menu's own trigger.
        (None, None, _) | (None, Some(_), true) => Some(match key {
            ArrowDown | Home => items[0],
            ArrowUp | End => items[n - 1],
            _ => return None,
        }),
        // A text field or a page control: it keeps its arrows.
        _ => None,
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
        if shell.is_event_captured() {
            return;
        }
        let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(named),
            modifiers,
            ..
        }) = event
        else {
            return;
        };
        if !modifiers.is_empty() {
            return;
        }
        let mut survey = Survey::default();
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut survey);
        let Some(at) = decide(*named, &survey.targets) else {
            return;
        };
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout,
            renderer,
            &mut FocusAt { at, seen: 0 },
        );
        shell.capture_event();
        shell.request_redraw();
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
        let mut mark = Mark {
            part: self.part,
            enter: true,
        };
        operation.custom(None, layout.bounds(), &mut mark);
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
        mark.enter = false;
        operation.custom(None, layout.bounds(), &mut mark);
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
