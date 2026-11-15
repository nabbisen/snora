//! [`MirroredRow`]: a row whose children stay in logical order in the tree and are
//! *placed* from the end edge under right-to-left (RFC-132).
//!
//! > **The order is logical, and the positions mirror.**
//!
//! Mirroring a row by reversing its children changes what the tree is. iced matches a
//! widget's state to its position in the tree, so a direction flip lost focus and scroll
//! offsets, and Tab followed the reversed order. This wraps an ordinary
//! [`Row`](iced::widget::Row), laid out exactly as iced lays it out, and then reflects each
//! child's x within the row (`x' = width - x - child width`). Everything else (drawing, events,
//! operations, overlays) is the row's own, over the same tree, so only positions change.
//!
//! Only the row's **direct children** move. A child that is itself a row decides its own
//! placement, which is why each logical row in snora is a mirrored row, not only the outer one.
//!
//! **Not for applications.** `snora-widgets`' `LogicalRow` (what `row_dir` returns) is the
//! public face of this; this crate's public items are the ones `snora` and `snora-widgets` use.

use iced::{Element, Length, Point, Rectangle, Size, Theme, Vector};
use iced_core::layout::{self, Layout};
use iced_core::widget::{Operation, Tree, tree};
use iced_core::{Clipboard, Event, Shell, Widget, mouse, overlay, renderer};

/// `row`, with its children placed from the right edge when `mirrored`.
///
/// `row` must be an iced `Row` (or anything whose layout node's children are its
/// children, left to right). With `mirrored` false this is `row`, unchanged: the same node,
/// the same tree, the same pixels.
pub struct MirroredRow<'a, Message> {
    row: Element<'a, Message>,
    mirrored: bool,
}

impl<'a, Message> MirroredRow<'a, Message> {
    /// `row`, mirrored about its own width if `mirrored`.
    pub fn new(row: impl Into<Element<'a, Message>>, mirrored: bool) -> Self {
        Self {
            row: row.into(),
            mirrored,
        }
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for MirroredRow<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.row)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.row));
    }

    fn size(&self) -> Size<Length> {
        Widget::size(self.row.as_widget())
    }

    fn size_hint(&self) -> Size<Length> {
        self.row.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let node = self
            .row
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        if !self.mirrored {
            return node;
        }
        let own = node.bounds();
        let width = own.width;
        let children = node
            .children()
            .iter()
            .map(|child| {
                let b = child.bounds();
                child
                    .clone()
                    .move_to(Point::new(width - b.x - b.width, b.y))
            })
            .collect();
        layout::Node::with_children(Size::new(own.width, own.height), children)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.row
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
        self.row.as_widget_mut().update(
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
        self.row.as_widget().draw(
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
        self.row.as_widget().mouse_interaction(
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
        self.row.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a> From<MirroredRow<'a, Message>> for Element<'a, Message> {
    fn from(row: MirroredRow<'a, Message>) -> Self {
        Element::new(row)
    }
}
