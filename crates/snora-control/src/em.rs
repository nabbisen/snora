//! Marks sized in **ems of the text they sit with**, read at layout time
//! (RFC-115's amendment, RFC-116 R-1 Unit 3).
//!
//! # Why a widget and not a number
//!
//! RFC-115 ruled that a pictogram is 1 em of the text it sits with, and
//! that the × close mark is 1.125 em. On the design path that is a
//! number view code can compute, because it holds the tokens. On the
//! unstyled path it is not: the text size is the host's
//! `default_text_size`, which view code never sees. **Only a widget's
//! layout can read it** (`text::Renderer::default_size`), so the
//! relative size has to be resolved there. These two widgets do that,
//! and `Control` does the same for its target.
//!
//! Neither draws anything `text` or `svg` does not. They differ in
//! where the size comes from.

use iced::{Color, Element, Font, Length, Pixels, Rectangle, Size, Theme};
use iced_core::layout::{self, Layout, Limits};
use iced_core::renderer;
use iced_core::text::{self, Renderer as _, paragraph};
use iced_core::widget::{Operation, Tree, tree};
use iced_core::{Widget, mouse};

type Paragraph = paragraph::Plain<<iced::Renderer as text::Renderer>::Paragraph>;

/// A string drawn at `em` times the host's text size, resolved when the
/// layout runs.
///
/// For a pictogram (`em` = 1.0) or a close mark (1.125) that has to
/// follow the text it sits beside on a path that has no tokens. It draws
/// exactly as `text` does: the same shaping, the same inherited colour,
/// and it announces its content to operations the way `text` does, so a
/// selector that finds text finds it.
pub struct Glyph {
    content: String,
    em: f32,
    font: Option<Font>,
    color: Option<Color>,
}

impl Glyph {
    /// `content`, drawn at `em` times the host's text size.
    pub fn new(content: impl Into<String>, em: f32) -> Self {
        Self {
            content: content.into(),
            em,
            font: None,
            color: None,
        }
    }

    /// The font to draw in (an icon font, say). Unset, the host's.
    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    /// A colour of its own. Unset, the glyph takes the colour it
    /// inherits, which is what a mark inside a control wants: the
    /// control's own `text_color`.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Glyph {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Paragraph>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Paragraph::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &Limits,
    ) -> layout::Node {
        iced_core::widget::text::layout(
            tree.state.downcast_mut::<Paragraph>(),
            renderer,
            limits,
            &self.content,
            iced_core::widget::text::Format {
                size: Some(Pixels(self.em * renderer.default_size().0)),
                font: self.font,
                ..Default::default()
            },
        )
    }

    fn operate(
        &mut self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.text(None, layout.bounds(), &self.content);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        iced_core::widget::text::draw(
            renderer,
            style,
            layout.bounds(),
            tree.state.downcast_ref::<Paragraph>().raw(),
            iced_core::widget::text::Style { color: self.color },
            viewport,
        );
    }
}

impl<'a, Message: 'a> From<Glyph> for Element<'a, Message> {
    fn from(glyph: Glyph) -> Self {
        Element::new(glyph)
    }
}

/// A square of `em` times the host's text size, with `content` filling
/// it.
///
/// For a mark that cannot be sized by a text size: an SVG icon needs a
/// number of pixels, and the number is `em` times a size only layout
/// can read. Give the content `Length::Fill` on both sides and it takes
/// the whole square.
///
/// Not interactive: events and the mouse pass through to nothing. It
/// is for icons, not for controls.
pub struct EmBox<'a, Message> {
    content: Element<'a, Message>,
    em: f32,
}

impl<'a, Message> EmBox<'a, Message> {
    /// A square of `em` times the host's text size around `content`.
    pub fn new(content: impl Into<Element<'a, Message>>, em: f32) -> Self {
        Self {
            content: content.into(),
            em,
        }
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for EmBox<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        _limits: &Limits,
    ) -> layout::Node {
        let side = self.em * renderer.default_size().0;
        let exact = Limits::new(Size::new(side, side), Size::new(side, side));
        let content = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &exact);
        layout::Node::with_children(Size::new(side, side), vec![content])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().expect("the box has content"),
                renderer,
                operation,
            );
        });
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
            layout.children().next().expect("the box has content"),
            cursor,
            viewport,
        );
    }
}

impl<'a, Message: 'a> From<EmBox<'a, Message>> for Element<'a, Message> {
    fn from(em_box: EmBox<'a, Message>) -> Self {
        Element::new(em_box)
    }
}

#[cfg(test)]
mod tests;
