//! [`StatusMark`]: the mark that says *what kind* of message a toast or a notice is, by
//! **shape**, so that the colour is not the only thing saying it (RFC-143, WCAG 1.4.1).
//!
//! # Drawn, not typed
//!
//! A mark is not a glyph. A glyph the system does not cover draws **nothing** (measured in
//! RFC-143 Unit 0: on both renderers an uncovered codepoint leaves a gap and no box), which
//! would put the toast back to colour alone with no sign that it had. This widget draws
//! rounded rectangles only, which every renderer draws, in the same pixels everywhere.
//!
//! # The set (RFC-143 round 2, ruled 2026-11-28)
//!
//! Every mark has its own **outline**, so that the five survive blur, and the **fill**
//! carries a rule a reader can learn: filled means *attention* (Error, Warning), hollow
//! means *calm* (Info, Success). The inner part adds the familiar meaning when it can be
//! seen; when blur takes it, the outline still separates all five.
//!
//! | Kind | Outline | Fill | Inner part |
//! |---|---|---|---|
//! | [`Error`](StatusKind::Error) | circle | filled | a bar cut out (the "no entry" sign) |
//! | [`Warning`](StatusKind::Warning) | rounded square | filled | a "!" cut out |
//! | [`Info`](StatusKind::Info) | circle | hollow | an "i" |
//! | [`Success`](StatusKind::Success) | rounded square | hollow | none |
//! | [`Debug`](StatusKind::Debug) | a small dot | filled | none |
//!
//! Success has no conventional picture (a check mark needs a diagonal, which rounded
//! rectangles cannot draw). It is arbitrary, but nothing contradicts its meaning, and it
//! sits in the calm half of the rule.
//!
//! # Not interactive
//!
//! It is information: no Tab stop, no press, nothing for the pointer. It takes no event.
//!
//! # Colours
//!
//! The mark is drawn in two colours, **ink** (the shape) and **ground** (the surface it sits
//! on, which the cut-out parts and the hollow centres are painted in). They come from a
//! function of the theme, read when the mark is drawn, so a toast's mark follows the same
//! colour pair the toast's text does and cannot be given a pairing of its own.

use iced::{Background, Border, Color, Length, Rectangle, Size, Theme};
use iced_core::layout::{self, Layout, Limits};
use iced_core::renderer::{self, Quad};
use iced_core::text::Renderer as _;
use iced_core::widget::{Operation, Tree};
use iced_core::{Renderer as _, Widget, mouse};

/// Which message a [`StatusMark`] stands for.
///
/// Exhaustive on purpose: a sixth kind fails to compile in `parts` until it has a drawing
/// (RFC-063's pattern). This crate is internal, so the enum is not `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    /// Information: a ring with an "i".
    Info,
    /// Success: a rounded-square ring.
    Success,
    /// Warning: a filled rounded square with a "!" cut out.
    Warning,
    /// Error: a filled disc with a bar cut out.
    Error,
    /// Debug: a small filled dot.
    Debug,
}

/// Which colour a part is painted in.
#[derive(Clone, Copy)]
enum Paint {
    /// The shape's own colour.
    Ink,
    /// The surface the mark sits on: a cut-out, or a hollow centre.
    Ground,
}

/// One rounded rectangle of a mark, in a 16 x 16 slot: `(x, y, width, height, radius, paint)`.
type Part = (f32, f32, f32, f32, f32, Paint);

/// The side of the design slot every [`Part`] is expressed in.
const SLOT: f32 = 16.0;

/// A mark's parts, painted in this order. Exactly what RFC-143's Unit 0b measured, in pixels
/// at 16: nothing here is derived, so the drawing cannot change by accident.
fn parts(kind: StatusKind) -> &'static [Part] {
    use Paint::{Ground, Ink};
    match kind {
        // A ring (a disc with a smaller disc in the surface colour over it), with an "i": a dot over a bar.
        StatusKind::Info => &[
            (0.0, 0.0, 16.0, 16.0, 8.0, Ink),
            (2.0, 2.0, 12.0, 12.0, 6.0, Ground),
            (7.0, 4.0, 2.0, 2.0, 1.0, Ink),
            (7.0, 7.0, 2.0, 5.0, 1.0, Ink),
        ],
        // A rounded-square ring: the same two-layer construction, square.
        StatusKind::Success => &[
            (0.0, 0.0, 16.0, 16.0, 3.0, Ink),
            (2.0, 2.0, 12.0, 12.0, 1.0, Ground),
        ],
        // A filled rounded square with a "!" cut out: a bar over a dot.
        StatusKind::Warning => &[
            (0.0, 0.0, 16.0, 16.0, 3.0, Ink),
            (7.0, 3.0, 2.0, 6.0, 1.0, Ground),
            (7.0, 11.0, 2.0, 2.0, 1.0, Ground),
        ],
        // A filled disc with a horizontal bar cut out.
        StatusKind::Error => &[
            (0.0, 0.0, 16.0, 16.0, 8.0, Ink),
            (3.0, 7.0, 10.0, 2.0, 1.0, Ground),
        ],
        // A small dot.
        StatusKind::Debug => &[(5.0, 5.0, 6.0, 6.0, 3.0, Ink)],
    }
}

/// The two colours a mark is drawn in, as `(ink, ground)`.
type Colors<'a> = Box<dyn Fn(&Theme) -> (Color, Color) + 'a>;

/// A status mark: a square slot with the kind's shape in it. See the module documentation.
///
/// Sized in **ems of the host's text** by default (1 em, resolved when the layout runs, so it
/// follows `default_text_size` as [`Glyph`](crate::Glyph) does), or in pixels
/// ([`pixels`](Self::pixels)) where the caller has the size, as on the design path.
pub struct StatusMark<'a> {
    kind: StatusKind,
    em: f32,
    pixels: Option<f32>,
    colors: Colors<'a>,
}

impl<'a> StatusMark<'a> {
    /// A mark of `kind`, 1 em square, drawn in the `(ink, ground)` pair `colors` returns for
    /// the theme it is drawn under.
    pub fn new(kind: StatusKind, colors: impl Fn(&Theme) -> (Color, Color) + 'a) -> Self {
        Self {
            kind,
            em: 1.0,
            pixels: None,
            colors: Box::new(colors),
        }
    }

    /// `em` times the host's text size, instead of 1.
    #[must_use]
    pub fn em(mut self, em: f32) -> Self {
        self.em = em;
        self
    }

    /// Exactly `pixels` square, ignoring the host's text size: for a caller that already
    /// holds the size (the design path's `Typography`).
    #[must_use]
    pub fn pixels(mut self, pixels: f32) -> Self {
        self.pixels = Some(pixels);
        self
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for StatusMark<'_> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        renderer: &iced::Renderer,
        _limits: &Limits,
    ) -> layout::Node {
        let side = self
            .pixels
            .unwrap_or_else(|| self.em * renderer.default_size().0);
        layout::Node::new(Size::new(side, side))
    }

    /// Reports the slot as a container, as the slider bar reports its track and thumb: a sweep that
    /// reads the drawn pixels (`nontext_sweep.rs`) finds the mark by it. It is not a focus target and
    /// takes no event, so nothing else reads it.
    fn operate(
        &mut self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let (ink, ground) = (self.colors)(theme);
        let bounds = layout.bounds();
        let k = bounds.width / SLOT;
        for &(x, y, w, h, radius, paint) in parts(self.kind) {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle {
                        x: bounds.x + x * k,
                        y: bounds.y + y * k,
                        width: w * k,
                        height: h * k,
                    },
                    border: Border {
                        radius: (radius * k).into(),
                        ..Border::default()
                    },
                    // iced's own default, which is what a container's quad takes: the
                    // mark is measured (RFC-143 Unit 0b) in the pixels a container draws.
                    ..Quad::default()
                },
                Background::Color(match paint {
                    Paint::Ink => ink,
                    Paint::Ground => ground,
                }),
            );
        }
    }
}

impl<'a, Message: 'a> From<StatusMark<'a>> for iced::Element<'a, Message> {
    fn from(mark: StatusMark<'a>) -> Self {
        iced::Element::new(mark)
    }
}
