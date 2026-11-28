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
//! | [`Success`](StatusKind::Success) | circle | hollow | a filled centre disc |
//! | [`Debug`](StatusKind::Debug) | a small dot | filled | none |
//!
//! Success has no conventional picture (a check mark needs a diagonal, which rounded
//! rectangles cannot draw). It is arbitrary, but nothing contradicts its meaning, and it
//! sits in the calm half of the rule. **It is not a square**: round 3 of RFC-143 found that
//! a hollow rounded square at card size is an unchecked checkbox, which snora draws itself,
//! and an empty checkbox says "not done", the opposite of success. A centre disc suggests no
//! control to tick. A 6-unit disc survives blur far longer than Info's "i" does, so the two
//! rings still part when blur has taken the inner parts.
//!
//! # Beside its line
//!
//! A mark belongs beside the line it labels, not the middle of a block of lines: a mark centred
//! on a many-line message floats away from the title. [`beside_line`](StatusMark::beside_line)
//! makes the mark's slot as tall as the first line of text and draws the shape centred in it, so
//! a row that aligns its children to the top puts the shape level with that line.
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
//!
//! **Not for applications.** Public because `snora` (the toast) and `snora-widgets` (the notice)
//! both draw it, so one function draws both and the two cannot drift. An application chooses a
//! toast's intent or a notice's tone; the mark follows, and `AppLayout::toast_marks` and
//! `Notice::mark` are the switches.

use iced::{Background, Border, Color, Length, Pixels, Rectangle, Size, Theme};
use iced_core::layout::{self, Layout, Limits};
use iced_core::renderer::{self, Quad};
use iced_core::text::{LineHeight, Renderer as _};
use iced_core::widget::{Operation, Tree};
use iced_core::{Renderer as _, Widget, mouse};

/// Which message a [`StatusMark`] stands for. Not for applications: a toast's intent and a notice's tone
/// choose it.
///
/// Exhaustive on purpose: a sixth kind fails to compile in `parts` until it has a drawing
/// (RFC-063's pattern). This crate is internal, so the enum is not `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    /// Information: a ring with an "i".
    Info,
    /// Success: a ring with a filled disc in the middle.
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

/// A mark's parts, painted in this order, in pixels at 16: nothing here is derived, so the
/// drawing cannot change by accident. (Unit 0b measured the first version of these; round 3
/// replaced Success's.)
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
        // A ring with a filled centre disc. Not a square: a hollow square is an unchecked checkbox.
        StatusKind::Success => &[
            (0.0, 0.0, 16.0, 16.0, 8.0, Ink),
            (2.0, 2.0, 12.0, 12.0, 6.0, Ground),
            (5.0, 5.0, 6.0, 6.0, 3.0, Ink),
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

/// The first line of text a mark sits beside, if it sits beside one.
#[derive(Clone, Copy)]
enum Line {
    /// Not beside a line: the slot is the shape's square.
    None,
    /// The host's default text size, which view code never sees, so it is read when the layout runs.
    Default,
    /// A text of exactly this size.
    Pixels(f32),
}

/// The two colours a mark is drawn in, as `(ink, ground)`.
type Colors<'a> = Box<dyn Fn(&Theme) -> (Color, Color) + 'a>;

/// A status mark: a square slot with the kind's shape in it. See the module documentation. Built by `snora`
/// and `snora-widgets`; not for applications.
///
/// Sized in **ems of the host's text** by default (1 em, resolved when the layout runs, so it
/// follows `default_text_size` as [`Glyph`](crate::Glyph) does), or in pixels
/// ([`pixels`](Self::pixels)) where the caller has the size, as on the design path.
pub struct StatusMark<'a> {
    kind: StatusKind,
    em: f32,
    pixels: Option<f32>,
    line: Line,
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
            line: Line::None,
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

    /// Sits **beside the first line** of a text of `text_size` pixels (`None`: the host's default
    /// size): the slot is as tall as that line (iced's own line height, 1.3 times the size) and the
    /// shape is drawn centred in it, so the shape is level with the line when the row that holds the
    /// mark aligns its children to the top. A mark taller than the line keeps its own height.
    #[must_use]
    pub fn beside_line(mut self, text_size: Option<f32>) -> Self {
        self.line = text_size.map_or(Line::Default, Line::Pixels);
        self
    }
}

/// Where the shape is drawn in `bounds`: a square of the node's width, centred on the node's height and
/// **on a whole logical pixel**, so that every part of the shape falls on the same pixel grid (each part
/// is snapped separately, and a slot at a fractional position would round its parts differently).
fn shape_in(bounds: Rectangle) -> Rectangle {
    let side = bounds.width;
    Rectangle {
        x: bounds.x.round(),
        y: (bounds.y + (bounds.height - side) / 2.0).round(),
        width: side,
        height: side,
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
        let default = renderer.default_size().0;
        let side = self.pixels.unwrap_or(self.em * default);
        let line_of = |size: f32| LineHeight::default().to_absolute(Pixels(size)).0;
        let line = match self.line {
            Line::None => 0.0,
            Line::Default => line_of(default),
            Line::Pixels(size) => line_of(size),
        };
        layout::Node::new(Size::new(side, side.max(line)))
    }

    /// Reports the **shape's** square as a container (not the taller slot a mark beside a line has), as the
    /// slider bar reports its track and thumb: a sweep that reads the drawn pixels (`nontext_sweep.rs`) finds
    /// the mark by it. It is not a focus target and takes no event, so nothing else reads it.
    fn operate(
        &mut self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, shape_in(layout.bounds()));
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
        let bounds = shape_in(layout.bounds());
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
                    // Snapped to the device grid (round 3): a mark is a figure, and an edge that
                    // falls between two pixels is drawn as a soft one, thinner than it is.
                    snap: true,
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
