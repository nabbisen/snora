//! [`MenuAnchor`]: a trigger with a dropdown that floats over the page, anchored to
//! it (RFC-124).
//!
//! # Why an overlay
//!
//! A dropdown drawn inside the header (a `column` below the trigger) makes the header
//! taller while the menu is open, which pushes the whole page down, and leaves the
//! header in the page's layer, where the menu's own cover has to make an exception for
//! it. iced 0.14 does expose positioning above the page: `iced_core::overlay` is
//! public, and `tooltip` and `pick_list` float their content that way. So the dropdown
//! is an overlay of its trigger, and the **trigger's layout is the whole widget's**:
//! opening a menu moves nothing.
//!
//! # Where it opens
//!
//! Below the trigger, at its start edge (its end edge under RTL), **one ring's reach
//! lower than the trigger's bottom edge**, so the card never covers the trigger's focus
//! ring (WCAG 2.4.11, Focus Not Obscured). It is kept inside the window sideways.
//!
//! # A dropdown taller than the window scrolls
//!
//! Its height is capped to the room below the trigger, and the rest scrolls, so no item
//! is out of reach of the pointer or the keyboard. **When an item takes keyboard focus
//! the dropdown scrolls it into view:** the scroll offset is the dropdown's own
//! `scrollable` state, kept in this widget's tree for as long as the menu is open, and
//! the overlay is offered every operation (a Tab is one), so it can do that itself.
//!
//! # What the engine learns from it
//!
//! While open, the anchor reports one mark ([`OpenMenu`]) to any operation, so a
//! container that covers the page ([`Inert`](crate::Inert)) learns **from the tree**
//! that a menu is open, with no signal from the application.
//!
//! # What the arrows' walk learns from it (RFC-123)
//!
//! The dropdown is an overlay, so the anchor's `operate` does not reach it: Tab and a focus
//! task meet its items through the overlay, once. The arrows are decided at the root, with the
//! page's controls in view too, so the anchor offers every operation passing it a [`Reach`]
//! and **only the arrows' walk answers**. It then operates its dropdown itself, with a layout
//! it **computes at the walk**, by the same function the overlay is laid out with
//! (`place`): from the trigger's bounds as the walk is given them (fresh, even if an earlier
//! event of the same batch moved the trigger), and from the one thing the walk is not given,
//! **the window's size** (and the translation of any scrolling ancestor), which the overlay
//! was last laid out with. That is kept in the anchor's tree state (`AnchorState`) and dropped
//! when the view is rebuilt. The window cannot change size inside an `update`: a resize
//! reaches the interface as a rebuild, between updates.
//!
//! Public because the widgets crate builds it; not for applications.

use iced::{Element, Length, Point, Rectangle, Size, Theme, Vector};
use iced_core::layout::{self, Layout};
use iced_core::widget::operation::Focusable;
use iced_core::widget::operation::scrollable::AbsoluteOffset;
use iced_core::widget::{Id, Operation, Tree, tree};
use iced_core::{Clipboard, Event, Shell, Widget, mouse, overlay, renderer};

use crate::menu_keys::{Part, Reach, Wants, marked};

/// What an anchor reports while its dropdown is open: "a menu is open here".
#[derive(Debug)]
pub(crate) struct OpenMenu;

/// What the overlay was last laid out in: the one input to the dropdown's placement that an
/// `operate` call is not given.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Placed {
    /// The window.
    window: Size,
    /// What the trigger's ancestors have scrolled it by (`overlay()`'s `translation`).
    translation: Vector,
}

/// What the anchor keeps between frames.
///
/// **What is stored:** the window's size and the translation of the trigger's scrolling
/// ancestors, as the overlay was last laid out in them (`Placed`), and nothing else. The arrows'
/// walk places the dropdown with them, by the same `place` the overlay uses, from the trigger's
/// bounds as the walk is given them.
///
/// **Why `operate` cannot get them fresh:** an `operate` call is given the trigger's untranslated
/// bounds and no window, while the overlay is given the window and a translation. The window
/// cannot change inside an `update` (a resize is a rebuild, which drops this state).
///
/// **When the translation can be stale:** only for a trigger inside a scrollable of the
/// application's own, and only if an earlier event of the same batch scrolls that ancestor. In
/// `snora::render` the header has no scrolling ancestor, so the translation is zero there. A stale
/// one affects where the dropdown is placed (the room below it, so the margin of its scroll),
/// never which item is focused; the next operation on the overlay brings the focused item into
/// view with the layout it is then given.
#[derive(Default)]
struct AnchorState {
    /// Where the overlay was last laid out (RFC-123). `None` until it has been laid out since the
    /// view was rebuilt.
    placed: Option<Placed>,
}

/// Reports which of the dropdown's items hold focus, in order.
#[derive(Default)]
struct Collect(Vec<bool>);

impl Operation for Collect {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        self.0.push(state.is_focused());
    }
}

/// Focuses the `at`-th item and no other.
struct FocusItem {
    at: usize,
    seen: usize,
}

impl Operation for FocusItem {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        if self.seen == self.at {
            state.focus();
        } else {
            state.unfocus();
        }
        self.seen += 1;
    }
}

/// `trigger`, with `dropdown` floating below it while there is one.
pub struct MenuAnchor<'a, Message> {
    /// The trigger, then the dropdown if the menu is open.
    elements: Vec<Element<'a, Message>>,
    /// Aligned to the trigger's end edge instead of its start edge (RTL).
    end_aligned: bool,
    /// Between the trigger's bottom edge and the card: the ring's reach.
    gap: f32,
}

impl<'a, Message> MenuAnchor<'a, Message> {
    /// `trigger`, and the open menu's `dropdown` (a **scrollable**, which is what lets
    /// a tall one scroll and keep the focused item in view), if open.
    ///
    /// `gap` is how far outside a control its focus ring reaches (`Ring::offset`): the
    /// card starts that far below the trigger.
    pub fn new(
        trigger: impl Into<Element<'a, Message>>,
        dropdown: Option<Element<'a, Message>>,
        end_aligned: bool,
        gap: f32,
    ) -> Self {
        let mut elements = vec![trigger.into()];
        elements.extend(dropdown);
        Self {
            elements,
            end_aligned,
            gap,
        }
    }
}

/// Reads where the focused item is in the dropdown's content, and how the dropdown is
/// scrolled. Item positions are the content's own (they do not move with the scroll),
/// so the first scrollable's content top is the origin.
#[derive(Default)]
struct Probe {
    viewport: Option<Rectangle>,
    content_top: f32,
    offset: f32,
    /// The focused item: its top and height in the content.
    focused: Option<(f32, f32)>,
}

impl Operation for Probe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn iced_core::widget::operation::Scrollable,
    ) {
        if self.viewport.is_none() {
            self.viewport = Some(bounds);
            self.content_top = content_bounds.y;
            self.offset = translation.y;
        }
    }

    fn focusable(&mut self, _id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() && self.focused.is_none() {
            self.focused = Some((bounds.y - self.content_top, bounds.height));
        }
    }
}

/// Scrolls the dropdown to an absolute offset.
struct ScrollTo(f32);

impl Operation for ScrollTo {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        _translation: Vector,
        state: &mut dyn iced_core::widget::operation::Scrollable,
    ) {
        state.scroll_to(AbsoluteOffset {
            x: None,
            y: Some(self.0),
        });
    }
}

/// The offset that brings an item (`top`, `height`, in the content) into a viewport of
/// `viewport` height scrolled to `offset`, with `margin` to spare on both sides (the
/// ring's reach, so the ring is in view with the item), moving as little as possible.
fn offset_to_show(offset: f32, viewport: f32, top: f32, height: f32, margin: f32) -> f32 {
    let (top, bottom) = ((top - margin).max(0.0), top + height + margin);
    if top < offset {
        top
    } else if bottom > offset + viewport {
        bottom - viewport
    } else {
        offset
    }
}

/// `operation` on the dropdown, and then, whatever the operation was (a Tab is one), an item
/// that holds focus is brought into view. The scroll is run directly on the content, not
/// through this function.
fn operate_dropdown<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    gap: f32,
    layout: Layout<'_>,
    renderer: &iced::Renderer,
    operation: &mut dyn Operation,
) {
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, operation);

    let mut probe = Probe::default();
    content
        .as_widget_mut()
        .operate(tree, layout, renderer, &mut probe);
    if let (Some(viewport), Some((top, height))) = (probe.viewport, probe.focused) {
        let to = offset_to_show(probe.offset, viewport.height, top, height, gap);
        if (to - probe.offset).abs() > f32::EPSILON {
            content
                .as_widget_mut()
                .operate(tree, layout, renderer, &mut ScrollTo(to));
        }
    }
}

struct Dropdown<'a, 'b, Message> {
    content: &'b mut Element<'a, Message>,
    tree: &'b mut Tree,
    anchor: Rectangle,
    end_aligned: bool,
    gap: f32,
    /// The window, as the overlay path was given it.
    viewport: Rectangle,
    translation: Vector,
    /// Where the overlay's placement inputs are kept for the arrows' walk.
    placed: &'b mut Option<Placed>,
}

/// The dropdown's node: below the trigger (`anchor`, in window coordinates) and one ring's reach
/// beyond it, at its start edge (the end edge under RTL), inside the window (`bounds`), and no
/// taller than the room below it. **The one function** the overlay is laid out with and the
/// arrows' walk computes its node with, so the two cannot differ.
fn place<Message>(
    content: &mut Element<'_, Message>,
    tree: &mut Tree,
    renderer: &iced::Renderer,
    anchor: Rectangle,
    bounds: Size,
    end_aligned: bool,
    gap: f32,
) -> layout::Node {
    // Below the trigger and one ring's reach beyond it.
    let top = anchor.y + anchor.height + gap;
    let room = (bounds.height - top).max(0.0);
    let natural =
        content
            .as_widget_mut()
            .layout(tree, renderer, &layout::Limits::new(Size::ZERO, bounds));
    // Taller than the room: laid out again at the room's height, which is what
    // makes its scrollable scroll.
    let node = if natural.bounds().height > room {
        content.as_widget_mut().layout(
            tree,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(bounds.width, room)),
        )
    } else {
        natural
    };
    let drawn = node.bounds();
    // At the start edge (the end edge under RTL), and inside the window.
    let x = if end_aligned {
        anchor.x + anchor.width - drawn.width
    } else {
        anchor.x
    };
    let x = x.min(bounds.width - drawn.width).max(0.0);
    node.move_to(Point::new(x, top))
}

impl<Message> overlay::Overlay<Message, Theme, iced::Renderer> for Dropdown<'_, '_, Message> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        *self.placed = Some(Placed {
            window: bounds,
            translation: self.translation,
        });
        place(
            self.content,
            self.tree,
            renderer,
            self.anchor,
            bounds,
            self.end_aligned,
            self.gap,
        )
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &self.viewport,
        );
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operate_dropdown(
            self.content,
            self.tree,
            self.gap,
            layout,
            renderer,
            operation,
        );
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
        self.content.as_widget_mut().update(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &self.viewport,
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &self.viewport,
            renderer,
        )
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for MenuAnchor<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<AnchorState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(AnchorState::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.elements.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        // What the overlay was laid out in belongs to the build it was made for.
        tree.state.downcast_mut::<AnchorState>().placed = None;
        tree.diff_children(&self.elements);
    }

    fn size(&self) -> Size<Length> {
        Widget::size(self.elements[0].as_widget())
    }

    fn size_hint(&self) -> Size<Length> {
        self.elements[0].as_widget().size_hint()
    }

    /// The trigger's own node: the dropdown takes no room.
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.elements[0]
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
        if self.elements.len() > 1 {
            operation.custom(None, layout.bounds(), &mut OpenMenu);
        }
        let trigger = &mut self.elements[0];
        marked(Part::Trigger, layout.bounds(), operation, |operation| {
            trigger
                .as_widget_mut()
                .operate(&mut tree.children[0], layout, renderer, operation);
        });
        if self.elements.len() > 1 {
            self.reach(tree, layout.bounds(), renderer, operation);
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
        self.elements[0].as_widget_mut().update(
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
        self.elements[0].as_widget().draw(
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
        self.elements[0].as_widget().mouse_interaction(
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
        let (trigger, rest) = self.elements.split_at_mut(1);
        let Tree {
            state, children, ..
        } = tree;
        let (trigger_tree, rest_trees) = children.split_at_mut(1);
        let mut found = Vec::new();
        found.extend(trigger[0].as_widget_mut().overlay(
            &mut trigger_tree[0],
            layout,
            renderer,
            viewport,
            translation,
        ));
        if let Some(content) = rest.first_mut() {
            let b = layout.bounds();
            found.push(overlay::Element::new(Box::new(Dropdown {
                content,
                tree: &mut rest_trees[0],
                anchor: Rectangle {
                    x: b.x + translation.x,
                    y: b.y + translation.y,
                    ..b
                },
                end_aligned: self.end_aligned,
                gap: self.gap,
                viewport: *viewport,
                translation,
                placed: &mut state.downcast_mut::<AnchorState>().placed,
            })));
        }
        match found.len() {
            0 => None,
            1 => found.pop(),
            _ => Some(overlay::Group::with_children(found).overlay()),
        }
    }
}

impl<Message> MenuAnchor<'_, Message> {
    /// Offers the dropdown to the operation passing, which only the arrows' walk takes up.
    fn reach(
        &mut self,
        tree: &mut Tree,
        bounds: Rectangle,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        let mut reach = Reach::default();
        operation.custom(None, bounds, &mut reach);
        if reach.wants == Wants::Nothing {
            return;
        }
        let Tree {
            state, children, ..
        } = tree;
        // Not laid out since the rebuild: the dropdown is not there to be asked.
        let Some(placed) = state.downcast_ref::<AnchorState>().placed else {
            return;
        };
        let content = &mut self.elements[1];
        // Placed as the overlay is, from the trigger as it is now.
        let node = place(
            content,
            &mut children[1],
            renderer,
            Rectangle {
                x: bounds.x + placed.translation.x,
                y: bounds.y + placed.translation.y,
                ..bounds
            },
            placed.window,
            self.end_aligned,
            self.gap,
        );
        let layout = Layout::new(&node);
        match reach.wants {
            Wants::Nothing => {}
            Wants::Survey => {
                let mut collect = Collect::default();
                content
                    .as_widget_mut()
                    .operate(&mut children[1], layout, renderer, &mut collect);
                reach.items = collect.0;
                reach.answered = true;
                operation.custom(None, bounds, &mut reach);
            }
            Wants::Focus(at) => operate_dropdown(
                content,
                &mut children[1],
                self.gap,
                layout,
                renderer,
                &mut FocusItem { at, seen: 0 },
            ),
        }
    }
}

impl<'a, Message: 'a> From<MenuAnchor<'a, Message>> for Element<'a, Message> {
    fn from(anchor: MenuAnchor<'a, Message>) -> Self {
        Element::new(anchor)
    }
}

#[cfg(test)]
mod tests {
    use super::{AnchorState, MenuAnchor, Placed, offset_to_show};
    use iced::widget::space;
    use iced::{Element, Size, Vector};
    use iced_core::widget::Tree;

    /// RFC-123 ruling 2: what the overlay was laid out in belongs to the build it was made for.
    /// (Nothing else makes a dialog over an open menu silent *alone*: the `Hide` boolean has its
    /// own row in `inert`'s tests, and this is the other guard's.)
    #[test]
    fn a_rebuild_drops_what_the_overlay_was_laid_out_in() {
        let element: Element<'_, ()> =
            MenuAnchor::new(space(), Some(space().into()), false, 2.0).into();
        let mut tree = Tree::new(&element);
        tree.state.downcast_mut::<AnchorState>().placed = Some(Placed {
            window: Size::new(100.0, 100.0),
            translation: Vector::ZERO,
        });
        assert!(tree.state.downcast_ref::<AnchorState>().placed.is_some());

        element.as_widget().diff(&mut tree);

        assert!(
            tree.state.downcast_ref::<AnchorState>().placed.is_none(),
            "a rebuild kept the old placement"
        );
    }

    #[test]
    fn an_item_in_view_does_not_move_the_scroll() {
        assert_eq!(offset_to_show(50.0, 100.0, 60.0, 20.0, 2.0), 50.0);
    }

    #[test]
    fn an_item_below_the_view_scrolls_just_enough_to_show_its_ring() {
        // Bottom 140 + 2 of ring, in a view 100 tall.
        assert_eq!(offset_to_show(0.0, 100.0, 120.0, 20.0, 2.0), 42.0);
    }

    #[test]
    fn an_item_above_the_view_scrolls_to_show_its_ring() {
        assert_eq!(offset_to_show(80.0, 100.0, 30.0, 20.0, 2.0), 28.0);
    }

    #[test]
    fn the_first_item_never_scrolls_above_the_start() {
        assert_eq!(offset_to_show(10.0, 100.0, 1.0, 20.0, 2.0), 0.0);
    }
}
