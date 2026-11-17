//! [`SliderBar`]: the track, the fill and the thumb of a slider, and the pointer's way of
//! moving it (RFC-134).
//!
//! iced 0.14's `slider` registers no focus target and takes ← → only while the pointer is over
//! it, so snora draws its own: this is **what is drawn and what the pointer does**, and nothing
//! else. The keys, Tab and the ring are [`Control`](crate::Control)'s, which the slider is built
//! around (a `focus_only` control with [`Control::on_key`](crate::Control::on_key)).
//!
//! # What a click means
//!
//! The thumb is a circle that stays inside the bar's bounds, so its centre runs from one thumb
//! **radius** in from the start edge to one radius in from the end. **A click at `x` means the
//! value whose thumb centre is at `x`**, on that run, and a click on the first or last radius
//! means the end. (The Unit 0 probe measured a click at 25% reading 0.267: it mapped `x` over the
//! whole width, thumb and padding included.)
//!
//! # Direction
//!
//! The fill runs from the **start edge**: from the left under LTR and from the right under RTL, and
//! a click on the right half means the lower half of the range under RTL.
//!
//! # A drag
//!
//! A press on the bar starts one and a release ends it. The pointer may leave the bar and the
//! window while it is down: the value follows it, clamped to the run, and **a release anywhere
//! ends it, as does the window losing focus** (a release the window never saw must not leave the
//! next move dragging).
//!
//! # Sizes
//!
//! In ems of the text it sits with: the bar is 1.5 em high, the thumb 1.25 em across and the
//! track 0.375 em thick. On the unstyled path an em is the host's `default_text_size`, which only
//! layout can read; the design path hands the label size in with [`SliderBar::unit`].

use iced::{Border, Color, Element, Event, Length, Pixels, Rectangle, Shadow, Size, Theme, touch};
use iced_core::layout::{self, Layout};
use iced_core::renderer::{self, Quad};
use iced_core::text::Renderer as _;
use iced_core::widget::{Operation, Tree, tree};
use iced_core::{Clipboard, Renderer as _, Shell, Widget, mouse, window};

/// The bar's height, as a multiple of the text it sits with.
const HEIGHT_EM: f32 = 1.5;
/// The thumb's diameter.
const THUMB_EM: f32 = 1.25;
/// The track's thickness.
const TRACK_EM: f32 = 0.375;

/// The colours a [`SliderBar`] draws in, for the theme it is drawn on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarColors {
    /// The track's outline.
    pub outline: Color,
    /// The track's own fill, under the filled part.
    pub track: Color,
    /// The filled part, from the start edge to the thumb.
    pub fill: Color,
    /// The thumb.
    pub thumb: Color,
    /// The thumb's outline.
    pub thumb_outline: Color,
}

/// A slider's bar. See the module documentation for what a click means, the direction and a drag.
pub struct SliderBar<'a, Message> {
    fraction: f32,
    rtl: bool,
    enabled: bool,
    unit: Option<f32>,
    width: Length,
    colors: Box<dyn Fn(&Theme) -> BarColors + 'a>,
    on_move: Box<dyn Fn(f32) -> Option<Message> + 'a>,
}

impl<'a, Message> SliderBar<'a, Message> {
    /// A bar showing `fraction` (0 to 1, where 0 is the start edge), drawn in `colors`.
    ///
    /// `on_move` is told the fraction the pointer means, from 0 to 1, whenever the pointer
    /// presses or drags; it returns the message to publish, or `None` when the value would not
    /// change (a drag publishes on every pixel, and a value that is the same is not news).
    pub fn new(
        fraction: f32,
        rtl: bool,
        colors: impl Fn(&Theme) -> BarColors + 'a,
        on_move: impl Fn(f32) -> Option<Message> + 'a,
    ) -> Self {
        Self {
            fraction: fraction.clamp(0.0, 1.0),
            rtl,
            enabled: true,
            unit: None,
            width: Length::Fill,
            colors: Box::new(colors),
            on_move: Box::new(on_move),
        }
    }

    /// Whether the pointer moves it. A disabled bar draws and answers nothing.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// The size of one em, in logical pixels, when it is not the host's text size (the design
    /// path's label).
    #[must_use]
    pub fn unit(mut self, px: f32) -> Self {
        self.unit = Some(px);
        self
    }

    /// The bar's width. `Fill` unless said.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

#[derive(Debug, Default)]
struct State {
    dragging: bool,
    /// One em in pixels, resolved when the layout last ran.
    unit: f32,
}

/// The geometry of a bar in `bounds` at `unit` pixels to the em: the thumb's radius, the run its
/// centre moves on (`x0` to `x1`), and the track's rectangle.
struct Geometry {
    radius: f32,
    x0: f32,
    x1: f32,
    centre_y: f32,
    thickness: f32,
}

impl Geometry {
    fn of(bounds: Rectangle, unit: f32) -> Self {
        let radius = THUMB_EM * unit / 2.0;
        // A bar narrower than its thumb has a run of no length, and every click means the start.
        let (x0, x1) = {
            let a = bounds.x + radius;
            let b = (bounds.x + bounds.width - radius).max(a);
            (a, b)
        };
        Self {
            radius,
            x0,
            x1,
            centre_y: bounds.y + bounds.height / 2.0,
            thickness: TRACK_EM * unit,
        }
    }

    fn track(&self) -> Rectangle {
        Rectangle {
            x: self.x0,
            y: self.centre_y - self.thickness / 2.0,
            width: self.x1 - self.x0,
            height: self.thickness,
        }
    }

    /// The thumb's centre for a fraction.
    fn centre_x(&self, fraction: f32, rtl: bool) -> f32 {
        let t = if rtl { 1.0 - fraction } else { fraction };
        self.x0 + t * (self.x1 - self.x0)
    }

    /// The filled part: from the start edge to the thumb's centre, or `None` when it has no length.
    fn fill(&self, fraction: f32, rtl: bool) -> Option<Rectangle> {
        let cx = self.centre_x(fraction, rtl);
        let (from, to) = if rtl { (cx, self.x1) } else { (self.x0, cx) };
        (to - from > 0.0).then(|| Rectangle {
            x: from,
            width: to - from,
            ..self.track()
        })
    }

    fn thumb(&self, fraction: f32, rtl: bool) -> Rectangle {
        let cx = self.centre_x(fraction, rtl);
        Rectangle {
            x: cx - self.radius,
            y: self.centre_y - self.radius,
            width: self.radius * 2.0,
            height: self.radius * 2.0,
        }
    }

    /// The fraction a pointer at `x` means: the one whose thumb centre is at `x`.
    fn fraction_at(&self, x: f32, rtl: bool) -> f32 {
        let run = self.x1 - self.x0;
        if run <= 0.0 {
            return 0.0;
        }
        let t = ((x - self.x0) / run).clamp(0.0, 1.0);
        if rtl { 1.0 - t } else { t }
    }
}

impl<Message> SliderBar<'_, Message> {
    fn publish_at(&self, state: &State, bounds: Rectangle, x: f32, shell: &mut Shell<'_, Message>) {
        let geometry = Geometry::of(bounds, state.unit);
        if let Some(message) = (self.on_move)(geometry.fraction_at(x, self.rtl)) {
            shell.publish(message);
        }
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for SliderBar<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let unit = self.unit.unwrap_or_else(|| {
            let host: Pixels = renderer.default_size();
            host.0
        });
        tree.state.downcast_mut::<State>().unit = unit;
        layout::atomic(limits, self.width, Length::Fixed(HEIGHT_EM * unit))
    }

    /// The track, the filled part and the thumb are reported as containers, as `button` reports its
    /// own box: that is how a selector or a test finds what a bar draws.
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let geometry = Geometry::of(layout.bounds(), state.unit);
        operation.container(None, geometry.track());
        if let Some(fill) = geometry.fill(self.fraction, self.rtl) {
            operation.container(None, fill);
        }
        operation.container(None, geometry.thumb(self.fraction, self.rtl));
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();
        if !self.enabled {
            // A bar that was disabled mid-drag lets go.
            state.dragging = false;
            return;
        }
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(at) = cursor.position_over(bounds) {
                    state.dragging = true;
                    self.publish_at(state, bounds, at.x, shell);
                    shell.capture_event();
                } else {
                    // A press elsewhere is not a drag, whatever the last one left behind.
                    state.dragging = false;
                }
            }
            Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                if bounds.contains(*position) {
                    state.dragging = true;
                    self.publish_at(state, bounds, position.x, shell);
                    shell.capture_event();
                } else {
                    state.dragging = false;
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => {
                // The cursor, as the runtime tracks it: `CursorMoved`'s own position is the
                // window's, and a bar inside a scrollable is not where the window says.
                if let Some(at) = cursor.position() {
                    self.publish_at(state, bounds, at.x, shell);
                }
                shell.capture_event();
            }
            Event::Touch(touch::Event::FingerMoved { position, .. }) if state.dragging => {
                self.publish_at(state, bounds, position.x, shell);
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. })
                if state.dragging =>
            {
                state.dragging = false;
                shell.capture_event();
            }
            Event::Window(window::Event::Unfocused) => state.dragging = false,
            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let geometry = Geometry::of(layout.bounds(), state.unit);
        let colors = (self.colors)(theme);
        let track = geometry.track();
        let round = geometry.thickness / 2.0;

        renderer.fill_quad(
            Quad {
                bounds: track,
                border: Border {
                    color: colors.outline,
                    width: 1.0,
                    radius: round.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            colors.track,
        );

        if let Some(fill) = geometry.fill(self.fraction, self.rtl) {
            renderer.fill_quad(
                Quad {
                    bounds: fill,
                    // No border: a GPU draws a quad's border as a rim over its fill.
                    border: Border {
                        radius: round.into(),
                        ..Border::default()
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                colors.fill,
            );
        }

        renderer.fill_quad(
            Quad {
                bounds: geometry.thumb(self.fraction, self.rtl),
                border: Border {
                    color: colors.thumb_outline,
                    width: 2.0,
                    radius: geometry.radius.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            colors.thumb,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        if !self.enabled {
            mouse::Interaction::None
        } else if state.dragging {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::None
        }
    }
}

impl<'a, Message: 'a> From<SliderBar<'a, Message>> for Element<'a, Message> {
    fn from(bar: SliderBar<'a, Message>) -> Self {
        Element::new(bar)
    }
}
