//! A label with an underline under **each line** it wraps to (RFC-117).
//!
//! # Why it is a widget
//!
//! `rich_text`'s `underline(true)` underlines every line, but draws a fixed
//! hairline whatever the text size, touching the descenders at 16 px
//! (RFC-117 Unit 0, measured on both renderers). This widget lays out its own
//! paragraph, reads one rectangle per line through
//! [`Paragraph::span_bounds`](iced_core::text::Paragraph::span_bounds) — the
//! same on both renderers, because it is implemented in `iced_graphics` — and
//! draws the underline itself, so its thickness and its distance from the text
//! follow the text size.
//!
//! # The rule (RFC-117, ruled 2026-11-07)
//!
//! > An underline is 1/16 em thick, and its top edge is 1/16 em below its
//! > line's box. Each of the two is rounded to the nearest whole pixel, halves
//! > up, and is never under 1 px. Every wrapped line has its own underline.
//! > Snapping to the pixel grid never moves an underline up toward its text.
//!
//! | text | thickness | offset below the box |
//! |---|---|---|
//! | 12 px | 1 | 1 |
//! | 16 px | 1 | 1 |
//! | 24 px | 2 | 2 |
//! | 32 px | 2 | 2 |
//! | 40 px | 3 | 3 |
//!
//! The two numbers are always equal, which is what makes the rule one
//! sentence. "Whole pixel" is a **logical** pixel: at scale 1 and 2 that is
//! the physical grid, and at 1.5 it is not, where the renderer's own snapping
//! decides the edge.
//!
//! # What the underline is measured from
//!
//! The bottom of each line's **box**, which is all `span_bounds` gives. The
//! descenders of the default font end 0.01 to 0.09 em above that bottom, and
//! the 1/16 em offset is the margin that leaves. A font whose descenders reach
//! past its own line box would come closer: iced 0.14's `Paragraph` exposes no
//! baseline or descent to measure it from.
//!
//! # Lines are spaced further apart
//!
//! An underline sits inside the *next* line's box, and an accented capital or
//! a tall ascender there comes within a row of it (RFC-117 Unit 1, measured at
//! scale 1 and 2, three fonts, 12 to 32 px). So **every line of a link is
//! spaced further from the next than plain text's is**, all of them and not
//! only the one that collides, so the rule stays predictable: by **four
//! units** — thickness, offset, and two more of clearance. Two units (thickness
//! and offset alone) leave `É` and `Å` touching the underline above them at 12
//! and 16 px, and three leave 12 px touching at scale 1, because snapping the
//! underline down can cost up to a pixel of what spacing there is. The line box is 1.3 em
//! *plus* [`Underlined::reach`], with the text centred in it, so the underline
//! is still `offset` below the text's own 1.3 em box, as ruled.
//!
//! # Where the underline is drawn
//!
//! **Inside the label's own bounds**: the text is centred in a line box four
//! units taller than its own, so the underline — `offset` below the text's box
//! — ends exactly at the line box's bottom. The label needs no room beyond its
//! own, and its owner can take the extra height back from its padding.

use iced::{Color, Element, Font, Length, Pixels, Rectangle, Size, Theme};
use iced_core::layout::{self, Layout, Limits};
use iced_core::renderer::{self, Quad};
use iced_core::text::{self, Paragraph as _, Renderer as _, Span};
use iced_core::widget::{Tree, tree};
use iced_core::{Renderer as _, Widget, mouse};

type Para = <iced::Renderer as text::Renderer>::Paragraph;

/// 1/16 em, in logical pixels, rounded half up and never under 1: both the
/// underline's thickness and its offset below the line's box.
fn unit(size: f32) -> f32 {
    (size / 16.0).round().max(1.0)
}

/// The one text description both the first layout and every re-layout use,
/// so the paragraph's comparison cannot disagree with how it was built.
fn shaped<C>(content: C, bounds: Size, size: Pixels, font: Font) -> text::Text<C> {
    // iced's default line height, 1.3 em, plus the underline's reach.
    let plain = text::LineHeight::default().to_absolute(size);
    text::Text {
        content,
        bounds,
        size,
        line_height: text::LineHeight::Absolute(Pixels(plain.0 + Underlined::reach(size.0))),
        font,
        align_x: text::Alignment::Default,
        align_y: iced::alignment::Vertical::Top,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::WordOrGlyph,
    }
}

struct State {
    paragraph: Para,
    content: String,
}

/// A wrapping label, underlined line by line, in the colour it inherits
/// unless one is given (the underline is always the text's colour).
pub struct Underlined {
    content: String,
    size: Option<Pixels>,
    color: Option<Color>,
    width: Length,
}

impl Underlined {
    /// `content`, underlined under each line.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            size: None,
            color: None,
            width: Length::Shrink,
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

    /// The width the label wraps at.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// How much **taller** than plain text each line of the label is at this
    /// text size, in logical pixels: four units, split equally above and
    /// below the text. A label of `n` lines is `n` times this taller than a
    /// plain one, and the text is centred in each line. The underline is
    /// inside that height.
    #[must_use]
    pub fn reach(size: f32) -> f32 {
        4.0 * unit(size)
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
        let font = renderer.default_font();
        let width = self.width;

        layout::sized(limits, width, Length::Shrink, |limits| {
            let bounds = limits.max();
            let make = |paragraph: &mut Para| {
                let spans = [Span::<(), Font>::new(content.as_str())];
                *paragraph = Para::with_spans(shaped(spans.as_slice(), bounds, size, font));
            };
            if state.content != content {
                make(&mut state.paragraph);
                state.content = content.clone();
            } else {
                match state.paragraph.compare(shaped((), bounds, size, font)) {
                    text::Difference::None => {}
                    text::Difference::Bounds => state.paragraph.resize(bounds),
                    text::Difference::Shape => make(&mut state.paragraph),
                }
            }
            state.paragraph.min_bounds()
        })
    }

    /// Announces its text to operations, as `text` does, so a selector that
    /// finds text finds this, and a test can read where the label's own box is
    /// (which excludes the underline).
    fn operate(
        &mut self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn iced_core::widget::Operation,
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
        if !layout.bounds().intersects(viewport) {
            return;
        }
        let state = tree.state.downcast_ref::<State>();
        let color = self.color.unwrap_or(style.text_color);
        let size = self.text_size(renderer).0;
        let thickness = unit(size);
        let origin = layout.position();

        for line in state.paragraph.span_bounds(0) {
            // The text's own 1.3 em box ends half the extra spacing above the
            // line box's bottom, and the underline starts `offset` below it.
            // `ceil`: snapping to the logical pixel grid only ever moves the
            // underline **down**, away from its text.
            let plain_bottom = origin.y + line.y + line.height - Self::reach(size) / 2.0;
            let top = (plain_bottom + thickness).ceil();
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle {
                        x: origin.x + line.x,
                        y: top,
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
