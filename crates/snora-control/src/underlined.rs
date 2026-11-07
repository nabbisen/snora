//! A label with an underline under **each line** it wraps to (RFC-117,
//! Unit 0b prototype).
//!
//! `rich_text`'s `underline(true)` underlines every line but draws a fixed
//! one-pixel hairline whatever the text size, touching the descenders at 16
//! px (RFC-117 Unit 0, measured on both renderers). This widget lays out its
//! own paragraph, reads one rectangle per line through
//! [`Paragraph::span_bounds`](iced_core::text::Paragraph::span_bounds) — the
//! same on both renderers, because it is implemented in `iced_graphics` — and
//! draws the underline itself, so its thickness and its distance from the
//! text can follow the text size.
//!
//! **Prototype:** the rule's three numbers are parameters so a probe can sweep
//! them; the ruled values become constants in Unit 1.

use iced::{Color, Element, Font, Length, Pixels, Rectangle, Size, Theme};
use iced_core::layout::{self, Layout, Limits};
use iced_core::renderer::{self, Quad};
use iced_core::text::{self, Paragraph as _, Renderer as _, Span};
use iced_core::widget::{Tree, tree};
use iced_core::{Renderer as _, Widget, mouse};

type Para = <iced::Renderer as text::Renderer>::Paragraph;

/// The underline's geometry, in ems of the text size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnderlineRule {
    /// Thickness, in ems of the text size.
    pub thickness_em: f32,
    /// Thickness never goes below this, in logical pixels.
    pub min_thickness: f32,
    /// How far **above the bottom of the line's box** the underline's top
    /// edge sits, in ems of the text size. Larger is closer to the glyphs;
    /// negative puts it below the box.
    pub rise_em: f32,
    /// Round the underline's top edge and its thickness to whole logical
    /// pixels, so it is crisp at scale 1 and 2 without the renderer's help.
    pub snap: bool,
}

impl UnderlineRule {
    fn thickness(&self, size: f32) -> f32 {
        let t = (self.thickness_em * size).max(self.min_thickness);
        if self.snap { t.round().max(1.0) } else { t }
    }

    /// How far the underline reaches below the bottom of its line's box.
    fn overhang(&self, size: f32) -> f32 {
        (self.thickness(size) - self.rise_em * size).max(0.0)
    }
}

struct State {
    paragraph: Para,
    content: String,
}

/// A wrapping label, underlined line by line, in its inherited text colour
/// unless one is given.
pub struct Underlined {
    content: String,
    size: Option<Pixels>,
    color: Option<Color>,
    width: Length,
    font: Option<Font>,
    rule: UnderlineRule,
}

impl Underlined {
    /// `content` underlined by `rule`.
    pub fn new(content: impl Into<String>, rule: UnderlineRule) -> Self {
        Self {
            content: content.into(),
            size: None,
            color: None,
            width: Length::Shrink,
            font: None,
            rule,
        }
    }

    /// A size of its own; unset, the host's text size.
    #[must_use]
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// A colour of its own, used for the text **and** the underline.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// A font of its own; unset, the host's.
    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    /// The width the label wraps at.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    fn text_size(&self, renderer: &iced::Renderer) -> Pixels {
        self.size.unwrap_or_else(|| renderer.default_size())
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Underlined {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            paragraph: Para::default(),
            content: String::new(),
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        let size = self.text_size(renderer);
        let content = self.content.clone();
        let width = self.width;
        let rule = self.rule;
        let font = self.font.unwrap_or_else(|| renderer.default_font());

        layout::sized(limits, width, Length::Shrink, |limits| {
            let bounds = limits.max();
            let make = |paragraph: &mut Para| {
                let spans = [Span::<(), Font>::new(content.as_str())];
                *paragraph = Para::with_spans(text::Text {
                    content: spans.as_slice(),
                    bounds,
                    size,
                    line_height: text::LineHeight::default(),
                    font,
                    align_x: text::Alignment::Default,
                    align_y: iced::alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::WordOrGlyph,
                });
            };
            if state.content != content {
                make(&mut state.paragraph);
                state.content = content.clone();
            } else {
                match state.paragraph.compare(text::Text {
                    content: (),
                    bounds,
                    size,
                    line_height: text::LineHeight::default(),
                    font,
                    align_x: text::Alignment::Default,
                    align_y: iced::alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::WordOrGlyph,
                }) {
                    text::Difference::None => {}
                    text::Difference::Bounds => state.paragraph.resize(bounds),
                    text::Difference::Shape => make(&mut state.paragraph),
                }
            }
            let bounds = state.paragraph.min_bounds();
            // The last line's underline may hang below its box; the label's
            // own height includes it, so nothing around it clips or crowds it.
            Size::new(bounds.width, bounds.height + rule.overhang(size.0).ceil())
        })
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
        if !layout.bounds().intersects(viewport) {
            return;
        }
        let state = tree.state.downcast_ref::<State>();
        let color = self.color.unwrap_or(style.text_color);
        let size = self.text_size(renderer).0;
        let origin = layout.position();
        let thickness = self.rule.thickness(size);

        for line in state.paragraph.span_bounds(0) {
            let bottom = origin.y + line.y + line.height;
            let top = bottom - self.rule.rise_em * size;
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle {
                        x: origin.x + line.x,
                        y: if self.rule.snap { top.round() } else { top },
                        width: line.width,
                        height: thickness,
                    },
                    snap: true,
                    ..Quad::default()
                },
                color,
            );
        }

        iced_core::widget::text::draw(
            renderer,
            style,
            layout.bounds(),
            &state.paragraph,
            iced_core::widget::text::Style { color: Some(color) },
            viewport,
        );
    }
}

impl<'a, Message: 'a> From<Underlined> for Element<'a, Message> {
    fn from(underlined: Underlined) -> Self {
        Element::new(underlined)
    }
}
