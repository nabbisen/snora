//! PROTOTYPE (RFC-124 Unit 0): a trigger with a dropdown that floats as an overlay
//! anchored to it. Not for landing as written.
//!
//! The dropdown is **not** in the page's layout: the trigger's layout is the whole
//! widget, so opening a menu moves nothing. Its items are the overlay's own content,
//! reached by `operate`, `update` and `draw` through iced's overlay path.

use iced::{Element, Length, Point, Rectangle, Size, Theme, Vector};
use iced_core::layout::{self, Layout};
use iced_core::widget::{Operation, Tree, tree};
use iced_core::{Clipboard, Event, Shell, Widget, mouse, overlay, renderer};

/// What an anchor reports while its dropdown is open: "a menu is open here". The
/// engine learns it from the tree (RFC-124 R-3) instead of from a slot.
#[derive(Debug)]
pub(crate) struct OpenMenu;

/// `trigger`, with `dropdown` floating below it while there is one.
pub struct MenuAnchor<'a, Message> {
    /// The trigger, then the dropdown if the menu is open.
    elements: Vec<Element<'a, Message>>,
    /// Aligned to the trigger's end edge instead of its start edge (RTL).
    end_aligned: bool,
}

impl<'a, Message> MenuAnchor<'a, Message> {
    /// `trigger`, and the open menu's `dropdown`, if open.
    pub fn new(
        trigger: impl Into<Element<'a, Message>>,
        dropdown: Option<Element<'a, Message>>,
        end_aligned: bool,
    ) -> Self {
        let mut elements = vec![trigger.into()];
        elements.extend(dropdown);
        Self {
            elements,
            end_aligned,
        }
    }
}

struct Dropdown<'a, 'b, Message> {
    content: &'b mut Element<'a, Message>,
    tree: &'b mut Tree,
    anchor: Rectangle,
    end_aligned: bool,
    /// The window, as the overlay path was given it.
    viewport: Rectangle,
}

impl<Message> overlay::Overlay<Message, Theme, iced::Renderer> for Dropdown<'_, '_, Message> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);
        let node = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);
        let drawn = node.bounds();
        let size = Size::new(drawn.width, drawn.height);
        // Below the trigger, at its start edge (its end edge under RTL), and inside
        // the window.
        let x = if self.end_aligned {
            self.anchor.x + self.anchor.width - size.width
        } else {
            self.anchor.x
        };
        let x = x.min(bounds.width - size.width).max(0.0);
        let below = self.anchor.y + self.anchor.height;
        let y = if below + size.height <= bounds.height {
            below
        } else {
            (self.anchor.y - size.height).max(0.0)
        };
        node.move_to(Point::new(x, y))
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
        self.content
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
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
        tree::Tag::stateless()
    }

    fn children(&self) -> Vec<Tree> {
        self.elements.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
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
        self.elements[0].as_widget_mut().operate(
            &mut tree.children[0],
            layout,
            renderer,
            operation,
        );
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
        let (trigger_tree, rest_trees) = tree.children.split_at_mut(1);
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
                viewport: *viewport,
            })));
        }
        match found.len() {
            0 => None,
            1 => found.pop(),
            _ => Some(overlay::Group::with_children(found).overlay()),
        }
    }
}

impl<'a, Message: 'a> From<MenuAnchor<'a, Message>> for Element<'a, Message> {
    fn from(anchor: MenuAnchor<'a, Message>) -> Self {
        Element::new(anchor)
    }
}
