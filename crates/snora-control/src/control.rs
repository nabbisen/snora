//! [`Control`]: `button`'s behaviour, plus focus.
//!
//! What it adds to iced's `button`, and nothing else:
//!
//! - **`operate` registers it as a focus target** (`Focusable`), so
//!   `focus_next` / `focus_previous` can land on it;
//! - **Enter and Space press it** while focused, publishing **the same
//!   message a click publishes** — a control with two ways in and two
//!   messages out would let an application handle one and not the
//!   other, with nothing to say so;
//! - **a click focuses it**, and a press anywhere else takes focus
//!   away, so at most one control holds focus;
//! - **a focus ring** is drawn while it is focused;
//! - **it tells the runtime when its look changes** — `button`'s own
//!   mechanism, recorded status compared on each event — so a hover
//!   colour appears when the pointer arrives and not when something
//!   else next redraws the window;
//! - **a control with no handler is disabled, and is not a focus
//!   target**: reaching a control that does nothing is the confusion
//!   the owner's rule forbids;
//! - **it is at least `max(24, 1.5 x its text, the minimum-target
//!   token)`** in both directions, with its content centred in whatever
//!   the floor adds (RFC-115's amendment): see [`Control::min_target`] and
//!   [`Control::text_size`].
//!
//! Everything else — padding, size, the style closure, `clip`,
//! `on_press_with` and `on_press_maybe` — is as on `button`.

use iced::border::Border;
use iced::keyboard::{self, key};
use iced::mouse;
use iced::touch;
use iced::window;
use iced::{
    Background, Color, Element, Event, Length, Padding, Pixels, Point, Rectangle, Shadow, Size,
    Theme, Vector,
};
use iced_core::layout::{self, Layout};
// `Renderer` is the trait that owns `fill_quad`; `text::Renderer` the
// one that owns `default_size`. Both are imported anonymously — they
// are needed in scope, not by name.
use iced_core::Renderer as _;
use iced_core::renderer::{self, Quad};
use iced_core::text::Renderer as _;
use iced_core::widget::Operation;
use iced_core::widget::operation::Focusable;
use iced_core::widget::{Tree, tree};
use iced_core::{Clipboard, Shell, Widget, overlay, widget::Id};

use iced::widget::button;

/// WCAG 2.5.8's minimum pointer target, in logical pixels.
///
/// Runtime rather than test-only: RFC-115's rule has to hold at layout
/// time, not only in a suite.
///
/// **Not pinned by `scripts/check-wcag-floors.sh`** — an earlier version
/// of this comment said it was, and it was not. That script pins
/// `AA_TEXT`, `NON_TEXT_MIN`, `FOCUS_MIN` and `TEXT_SIZE_MIN` only, and
/// this number has copies it does not see (the toast's
/// `CLOSE_TARGET_MIN`, and the test mirrors).
pub(crate) const MIN_TARGET: f32 = 24.0;

/// RFC-115's multiplier, at the position its Unit 0 measured as free:
/// at the default text size of 16px, `1.5 x 16 = 24`, which is exactly
/// what every target already measured, so the rule changes nothing for
/// an application that changes nothing.
///
/// A control's target is **the largest of three things**: 24 px, this
/// factor times the text it carries, and the minimum-target token the
/// design-path wrappers pass in through [`Control::min_target`]
/// (`Tokens::targets.min`). The unstyled path has no token, and passes
/// none.
pub(crate) const TARGET_PER_TEXT: f32 = 1.5;

/// How the control's fill is styled.
///
/// Deliberately `button::Status`, not a status of its own: it makes
/// `control` drop-in for `button` on the styling side, so the
/// sidebar's existing `sidebar_button_style` is reused unchanged. A
/// `Focused` variant would have been a new enum for every caller to
/// match on, and the ring is drawn separately anyway.
pub type StyleFn<'a> = Box<dyn Fn(&Theme, button::Status) -> button::Style + 'a>;

/// The focus ring's own appearance.
pub struct Ring<'a> {
    /// Colour, given the theme. The caller supplies it because only the
    /// caller knows what the ring will sit on — the rail, or the active
    /// item's own highlight.
    pub color: Box<dyn Fn(&Theme) -> Color + 'a>,
    /// The ring's stroke width, in logical pixels. `0` draws no ring.
    pub width: f32,
    /// How far outside the control's bounds the ring is drawn.
    pub offset: f32,
}

/// What a press publishes — the message itself, or a way to make it.
///
/// As `button`'s own: `on_press_with` builds the message **only when
/// pressed**, which matters when making it is not free (an allocation, a
/// clone of something large) and the control is drawn and hovered far
/// more often than it is pressed.
enum OnPress<'a, Message> {
    Direct(Message),
    Closure(Box<dyn Fn() -> Message + 'a>),
}

impl<Message: Clone> OnPress<'_, Message> {
    fn get(&self) -> Message {
        match self {
            OnPress::Direct(message) => message.clone(),
            OnPress::Closure(f) => f(),
        }
    }
}

#[derive(Debug, Default)]
struct State {
    is_pressed: bool,
    is_focused: bool,
}

impl Focusable for State {
    fn is_focused(&self) -> bool {
        self.is_focused
    }

    fn focus(&mut self) {
        self.is_focused = true;
    }

    fn unfocus(&mut self) {
        self.is_focused = false;
    }
}

/// A focusable, key-activatable control: `button` plus focus.
///
/// See the module documentation for exactly what it adds.
pub struct Control<'a, Message> {
    content: Element<'a, Message>,
    /// `None` is a **disabled** control: it ignores the pointer and the
    /// keyboard, draws as `Status::Disabled`, and is not a focus target.
    on_press: Option<OnPress<'a, Message>>,
    /// What the control last drew as, recorded when a frame is about to
    /// be drawn and compared against on every other event. This is
    /// `button`'s own mechanism: it is how a widget knows its look has
    /// changed and the runtime must be told.
    status: Option<button::Status>,
    /// Whether content is drawn within this control's own bounds.
    clip: bool,
    id: Id,
    width: Length,
    height: Length,
    padding: Padding,
    style: StyleFn<'a>,
    ring: Ring<'a>,
    /// The minimum-target token (`Tokens::targets.min`), when the control
    /// is on the design path. `0` is "none": the unstyled path has no
    /// token, and 24 px and 1.5 x text apply regardless.
    min_target: f32,
    /// The size of the text the control carries, when it knows it (a
    /// design-path control is told its label size). `None` reads the
    /// renderer's default text size, which is what an unstyled `text`
    /// with no size of its own draws at.
    text_size: Option<Pixels>,
}

impl<Message> Control<'_, Message> {
    /// What the control looks like right now, from its state and the
    /// cursor. The same four statuses `button` has.
    fn status_now(
        &self,
        state: &State,
        cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> button::Status {
        if self.on_press.is_none() {
            button::Status::Disabled
        } else if cursor.is_over(bounds) {
            if state.is_pressed {
                button::Status::Pressed
            } else {
                button::Status::Hovered
            }
        } else {
            button::Status::Active
        }
    }
}

impl<'a, Message> Control<'a, Message> {
    /// A control around `content`, styled by `style`, with `ring` shown
    /// while it is focused. It has no handler yet, so it starts
    /// **disabled**; give it one with [`Control::on_press`] and friends.
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        style: StyleFn<'a>,
        ring: Ring<'a>,
    ) -> Self {
        Self {
            content: content.into(),
            on_press: None,
            status: None,
            clip: false,
            id: Id::unique(),
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::ZERO,
            style,
            ring,
            min_target: 0.0,
            text_size: None,
        }
    }

    /// Publish `message` when pressed — by a click, a tap, or Enter or
    /// Space while focused.
    #[must_use]
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(OnPress::Direct(message));
        self
    }

    /// Publish what `message` returns, built **only when pressed**.
    ///
    /// For a message that cannot be cloned up front, or that is not free
    /// to make: the control is drawn and hovered far more often than it
    /// is pressed.
    #[must_use]
    pub fn on_press_with(mut self, message: impl Fn() -> Message + 'a) -> Self {
        self.on_press = Some(OnPress::Closure(Box::new(message)));
        self
    }

    /// Publish `message` when pressed, or — for `None` — **disable the
    /// control**: it ignores the pointer and the keyboard, and is not a
    /// focus target. Reaching a control that does nothing is the
    /// confusion the owner's rule forbids.
    #[must_use]
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message.map(OnPress::Direct);
        self
    }

    /// Draw the content within this control's own bounds, so content
    /// wider than the control is cut off at its edge rather than running
    /// on past it. As `button`'s own.
    ///
    /// **Crate-private on purpose.** This crate's public items are only
    /// what `snora` and `snora-widgets` use, and neither uses `clip`:
    /// `button`'s own `clip` has no caller in snora either. It exists for
    /// parity and is tested. The `expect` below is a tripwire: the day a
    /// dependent needs it, the expectation goes unfulfilled, the build
    /// says so, and making it `pub` is a decision rather than a drift.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "no dependent uses `clip`; kept for parity with `button`, and tested"
        )
    )]
    pub(crate) fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Padding between the control's bounds and its content.
    #[must_use]
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// The control's width.
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// The control's height.
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// The minimum-target token (`Tokens::targets.min`): the control's
    /// target, in both dimensions, is never smaller than this.
    ///
    /// It **adds to** the other two inputs rather than replacing them:
    /// the target is `max(24, 1.5 x text, this)`. A value under 24
    /// therefore changes nothing, which is deliberate: WCAG 2.5.8's own
    /// 24 px is not something a token can take a target under.
    #[must_use]
    pub fn min_target(mut self, min: f32) -> Self {
        self.min_target = min;
        self
    }

    /// The size of the text this control carries, which RFC-115's
    /// `1.5 x text` is measured against.
    ///
    /// Left unset, the control reads the renderer's default text size.
    /// That is right for content with no size of its own, and wrong for
    /// a label drawn at a token's size: a design-path wrapper that sets
    /// the label's size says so here, so the target follows the text
    /// that is actually on it.
    #[must_use]
    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = Some(size.into());
        self
    }
}

impl<Message> Widget<Message, Theme, iced::Renderer> for Control<'_, Message>
where
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    /// `button`'s padded layout, grown to RFC-115's floor, with the content
    /// **centred in the growth**.
    ///
    /// The target is `max(24, 1.5 x text, the token)`. The text size is
    /// the control's own (`text_size`) when it was told one, and
    /// otherwise the renderer's (`text::Renderer::default_size`), which
    /// is the host's `default_text_size` — the one setting RFC-104 made
    /// the chrome follow.
    ///
    /// **The content is laid out at its natural size, and the box grown
    /// around it.** Handing the floor to the content as a minimum (the
    /// obvious way to apply it) stretches the content's own bounds, and
    /// a mark or a label then sits in the corner of its stretched
    /// bounds: a bigger target around a mark that has drifted to one
    /// edge is a defect of its own. Only the growth the *floor* caused
    /// is split, so a control the application made wider with `width`,
    /// whose content is meant to start at the edge (a menu item), keeps
    /// it there.
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let node = layout::padded(limits, self.width, self.height, self.padding, |limits| {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        });

        let text: Pixels = self.text_size.unwrap_or_else(|| renderer.default_size());
        let floor = MIN_TARGET
            .max(TARGET_PER_TEXT * text.0)
            .max(self.min_target);
        // Built from the bounds: `scripts/check-literal-sizes.sh` reads a
        // zero-argument accessor of the other name as a text size.
        let bounds = node.bounds();
        let natural = Size::new(bounds.width, bounds.height);
        let size =
            limits
                .min_width(floor)
                .min_height(floor)
                .resolve(self.width, self.height, natural);

        let grown = Vector::new(
            (size.width - natural.width).max(0.0) / 2.0,
            (size.height - natural.height).max(0.0) / 2.0,
        );
        if grown == Vector::ZERO {
            return node;
        }

        let child = node.children()[0].clone();
        let at = Point::new(child.bounds().x + grown.x, child.bounds().y + grown.y);
        layout::Node::with_children(size, vec![child.move_to(at)])
    }

    /// Registers as a focus target — **if it can be pressed.**
    ///
    /// The registration is what `button` does not do, and the whole
    /// reason Tab cannot reach a snora control today. A control with no
    /// handler does not register: Tab would land on it and show a ring
    /// around something that does nothing, which is the confusion the
    /// owner's rule forbids.
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        let bounds = layout.bounds();
        // As `button` does: the control's whole box is reported as a
        // container, which is how a selector or a test finds the region
        // that answers a click. Without it the only container a control's
        // region has is whatever its content happens to be.
        operation.container(None, bounds);
        let state = tree.state.downcast_mut::<State>();
        if self.on_press.is_some() {
            operation.focusable(Some(&self.id), bounds, state);
        }

        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().expect("the control has content"),
                renderer,
                operation,
            );
        });
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
            layout.children().next().expect("the control has content"),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let bounds = layout.bounds();
        let enabled = self.on_press.is_some();
        let is_press = matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. })
        );

        {
            let state = tree.state.downcast_mut::<State>();

            // A control that cannot be pressed cannot be held down or
            // focused. This is where a control that was focused and was
            // then rebuilt with no handler lets go: its state survives
            // the rebuild in the tree, and nothing else would clear it.
            if !enabled && (state.is_focused || state.is_pressed) {
                state.is_focused = false;
                state.is_pressed = false;
                shell.request_redraw();
            }

            // **A press anywhere else takes focus away from this
            // control.** Unit 0 lacked this: a press focused the control
            // it hit and nothing unfocused the one it had left, so
            // clicking A then B left both focused. `Focusable` assumes
            // one at a time, and the focus operations enforce it — but a
            // click is not an operation. `text_input` does the same.
            //
            // **Before the captured-event return below, on purpose.**
            // Every child of a row gets every event, in order: when the
            // press lands on B, B captures it, and A — earlier or later
            // in the row — must still let go.
            if is_press && state.is_focused && !cursor.is_over(bounds) {
                state.is_focused = false;
                shell.request_redraw();
            }
        }

        if shell.is_event_captured() {
            return;
        }

        match event {
            // Pointer and touch, as `button`.
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if enabled && cursor.is_over(bounds) {
                    let state = tree.state.downcast_mut::<State>();
                    state.is_pressed = true;
                    // A click also focuses, so that a pointer user who
                    // then reaches for the keyboard carries on from
                    // where they were rather than from the start.
                    if !state.is_focused {
                        state.is_focused = true;
                        shell.request_redraw();
                    }
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(on_press) = &self.on_press {
                    let state = tree.state.downcast_mut::<State>();
                    if state.is_pressed {
                        state.is_pressed = false;
                        if cursor.is_over(bounds) {
                            shell.publish(on_press.get());
                        }
                        shell.capture_event();
                    }
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {
                tree.state.downcast_mut::<State>().is_pressed = false;
            }
            // Enter or Space while focused emits **the same message a
            // click emits**. Not a second message: a control with two
            // ways in and two messages out would let an application
            // handle one and not the other, and nothing would say so.
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                let state = tree.state.downcast_mut::<State>();
                let activates = matches!(
                    key.as_ref(),
                    keyboard::Key::Named(key::Named::Enter | key::Named::Space)
                );
                if let Some(on_press) = &self.on_press
                    && state.is_focused
                    && activates
                {
                    shell.publish(on_press.get());
                    shell.capture_event();
                }
            }
            _ => {}
        }

        // **Tell the runtime when the look has changed.** `button`'s own
        // mechanism: a frame about to be drawn records what the control
        // looks like, and any other event compares against that. Without
        // it a hover colour does not appear until something else
        // happens to redraw the window — and a headless snapshot cannot
        // see the difference, because it draws on demand.
        let current = self.status_now(tree.state.downcast_ref::<State>(), cursor, bounds);
        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            self.status = Some(current);
        } else if self.status.is_some_and(|drawn| drawn != current) {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();

        // The recorded status, as `button`: every draw is preceded by a
        // `RedrawRequested` that records it. Before the first one the
        // control draws as disabled, which no frame ever shows.
        let style = (self.style)(theme, self.status.unwrap_or(button::Status::Disabled));

        if style.background.is_some() || style.border.width > 0.0 || style.shadow.color.a > 0.0 {
            renderer.fill_quad(
                Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                    snap: style.snap,
                },
                style
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );
        }

        // `clip`: the content may draw only inside this control. As
        // `button`'s own — text, for one, clips to the area it is given.
        let content_viewport = if self.clip {
            bounds.intersection(viewport).unwrap_or(*viewport)
        } else {
            *viewport
        };

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color,
            },
            layout.children().next().expect("the control has content"),
            cursor,
            &content_viewport,
        );

        // The ring last, so it is never painted over by the content. A
        // disabled control draws none even if its state says focused.
        //
        // **In a layer of its own, so a neighbour cannot paint over it.**
        // The ring is drawn outside the control, and a control whose
        // neighbour sits closer than `offset + width` (a chip's label and
        // its remove button, with nothing between) has the neighbour drawn
        // *after* it in tree order, filling the very pixels the ring is on:
        // measured, the ring lost its whole right side. Layers are
        // composited in the order they are made, whatever order the
        // widgets draw in, so the ring lands above every sibling's fill.
        if self.on_press.is_some() && state.is_focused && self.ring.width > 0.0 {
            let o = self.ring.offset;
            let ring = Quad {
                bounds: Rectangle {
                    x: bounds.x - o,
                    y: bounds.y - o,
                    width: bounds.width + 2.0 * o,
                    height: bounds.height + 2.0 * o,
                },
                border: Border {
                    color: (self.ring.color)(theme),
                    width: self.ring.width,
                    radius: style.border.radius,
                },
                shadow: Shadow::default(),
                snap: true,
            };
            renderer.with_layer(*viewport, |renderer| {
                renderer.fill_quad(ring, Background::Color(Color::TRANSPARENT));
            });
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) && self.on_press.is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
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
            layout.children().next().expect("the control has content"),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<Control<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(control: Control<'a, Message>) -> Self {
        Element::new(control)
    }
}

#[cfg(test)]
mod behaviour_tests;
#[cfg(test)]
mod sizing_tests;
#[cfg(test)]
mod target_tests;
