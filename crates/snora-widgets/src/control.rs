//! A focusable, key-activatable control (RFC-116 Unit 0, prototype).
//!
//! # Why this exists
//!
//! iced 0.14's `button` takes no keyboard input: its `update` handles
//! mouse and touch events only, and its `operate` registers no focus
//! target. Only `text_input` and `text_editor` implement
//! [`Focusable`]. So no snora control built on `button` can be focused,
//! and no key presses one — which the probe in
//! `.git-exclude/reviewed/iced-buttons-take-no-keyboard-input/`
//! established by experiment rather than by reading.
//!
//! `control` is the smallest widget that fixes that: it is `button`'s
//! behaviour plus a focus target, plus Enter and Space, plus a ring to
//! show where focus is.
//!
//! # What it depends on, and why not `iced`'s `advanced` feature
//!
//! [`iced_core::Widget`] and
//! [`iced_core::widget::operation::Focusable`] are public with no
//! feature gate; iced's `advanced` feature only re-exports them. So
//! this depends on `iced_core` directly, at the same `0.14`
//! requirement as `iced`, and Cargo unifies the two. Turning on
//! `advanced` would have widened snora's own feature surface for
//! nothing.
//!
//! # Prototype, not the shipped design
//!
//! Unit 0 asks what one focusable control costs. This one is
//! crate-private and used only by the sidebar item. The list of what
//! `button` does that it does not is in RFC-116's review request.

use iced::border::Border;
use iced::keyboard::{self, key};
use iced::mouse;
use iced::touch;
use iced::window;
use iced::{
    Background, Color, Element, Event, Length, Padding, Pixels, Rectangle, Shadow, Size, Theme,
    Vector,
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
/// One of the copies `scripts/check-wcag-floors.sh` pins. This one is
/// runtime rather than test-only: RFC-115's rule has to hold at layout
/// time, not only in a suite.
pub(crate) const MIN_TARGET: f32 = 24.0;

/// RFC-115's multiplier, at the position its Unit 0 measured as free:
/// at the default text size of 16px, `1.5 x 16 = 24`, which is exactly
/// what every target already measured, so the rule changes nothing for
/// an application that changes nothing.
///
/// **Prototype only.** RFC-115's amendment makes this a token
/// (`minimum-target`); Unit 0 is here to show the rule can live in
/// `control`'s layout at all.
pub(crate) const TARGET_PER_TEXT: f32 = 1.5;

/// How the control's fill is styled.
///
/// Deliberately `button::Status`, not a status of its own: it makes
/// `control` drop-in for `button` on the styling side, so the
/// sidebar's existing `sidebar_button_style` is reused unchanged. A
/// `Focused` variant would have been a new enum for every caller to
/// match on, and the ring is drawn separately anyway.
pub(crate) type StyleFn<'a> = Box<dyn Fn(&Theme, button::Status) -> button::Style + 'a>;

/// The focus ring's own appearance.
pub(crate) struct Ring<'a> {
    /// Colour, given the theme. The caller supplies it because only the
    /// caller knows what the ring will sit on — the rail, or the active
    /// item's own highlight.
    pub(crate) color: Box<dyn Fn(&Theme) -> Color + 'a>,
    pub(crate) width: f32,
    /// How far outside the control's bounds the ring is drawn.
    pub(crate) offset: f32,
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

pub(crate) struct Control<'a, Message> {
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
    /// Whether RFC-115's minimum applies. On for the prototype's
    /// sidebar item; the figures are in the review request.
    enforce_minimum: bool,
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
    pub(crate) fn new(
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
            enforce_minimum: true,
        }
    }

    /// Publish `message` when pressed.
    pub(crate) fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(OnPress::Direct(message));
        self
    }

    /// Publish what `message` returns, built **only when pressed**.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "R-1 Unit 1 adds it ahead of the rollout (Units 2-3) that calls it; \
                      the tests exercise it until then"
        )
    )]
    pub(crate) fn on_press_with(mut self, message: impl Fn() -> Message + 'a) -> Self {
        self.on_press = Some(OnPress::Closure(Box::new(message)));
        self
    }

    /// Publish `message` when pressed, or — for `None` — **disable the
    /// control**: it ignores the pointer and the keyboard, and is not a
    /// focus target. Reaching a control that does nothing is the
    /// confusion the owner's rule forbids.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "R-1 Unit 1 adds it ahead of the rollout (Units 2-3) that calls it; \
                      the tests exercise it until then"
        )
    )]
    pub(crate) fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message.map(OnPress::Direct);
        self
    }

    /// Draw the content within this control's own bounds, so content
    /// wider than the control is cut off at its edge rather than running
    /// on past it. As `button`'s own.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "R-1 Unit 1 adds it ahead of the rollout (Units 2-3) that calls it; \
                      the tests exercise it until then"
        )
    )]
    pub(crate) fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    pub(crate) fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub(crate) fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub(crate) fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
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

    /// `button`'s padded layout, with RFC-115's floor applied through
    /// the limits.
    ///
    /// The text size comes from the renderer
    /// (`text::Renderer::default_size`), which is the host's
    /// `default_text_size` — the one setting RFC-104 made the chrome
    /// follow. So the floor follows the text with no new API, which is
    /// the feasibility question Unit 0 was asked to answer.
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = if self.enforce_minimum {
            let text: Pixels = renderer.default_size();
            let floor = MIN_TARGET.max(TARGET_PER_TEXT * text.0);
            limits.min_width(floor).min_height(floor)
        } else {
            *limits
        };

        layout::padded(&limits, self.width, self.height, self.padding, |limits| {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        })
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
        if self.on_press.is_some() && state.is_focused && self.ring.width > 0.0 {
            let o = self.ring.offset;
            renderer.fill_quad(
                Quad {
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
                },
                Background::Color(Color::TRANSPARENT),
            );
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
