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
    on_press: Option<Message>,
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

impl<'a, Message> Control<'a, Message> {
    pub(crate) fn new(
        content: impl Into<Element<'a, Message>>,
        style: StyleFn<'a>,
        ring: Ring<'a>,
    ) -> Self {
        Self {
            content: content.into(),
            on_press: None,
            id: Id::unique(),
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::ZERO,
            style,
            ring,
            enforce_minimum: true,
        }
    }

    pub(crate) fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
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

    /// Registers as a focus target.
    ///
    /// This one call is what `button` does not do, and the whole reason
    /// Tab cannot reach a snora control today.
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();
        operation.focusable(Some(&self.id), bounds, state);

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

        if shell.is_event_captured() {
            return;
        }

        let bounds = layout.bounds();

        match event {
            // Pointer and touch, as `button`.
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if self.on_press.is_some() && cursor.is_over(bounds) {
                    let state = tree.state.downcast_mut::<State>();
                    state.is_pressed = true;
                    // A click also focuses, so that a pointer user who
                    // then reaches for the keyboard carries on from
                    // where they were rather than from the start.
                    state.is_focused = true;
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
                            shell.publish(on_press.clone());
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
                    shell.publish(on_press.clone());
                    shell.capture_event();
                }
            }
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
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();

        let status = if self.on_press.is_none() {
            button::Status::Disabled
        } else if cursor.is_over(bounds) {
            if state.is_pressed {
                button::Status::Pressed
            } else {
                button::Status::Hovered
            }
        } else {
            button::Status::Active
        };

        let style = (self.style)(theme, status);

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

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color,
            },
            layout.children().next().expect("the control has content"),
            cursor,
            viewport,
        );

        // The ring last, so it is never painted over by the content.
        if state.is_focused && self.ring.width > 0.0 {
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
mod sizing_tests {
    //! RFC-115's rule, living in `control`'s layout (RFC-116 Unit 0).
    //!
    //! The floor is `max(24, 1.5 x text)`, and the text size comes from
    //! the renderer — `iced::Settings::default_text_size`, the one
    //! setting RFC-104 made the chrome follow. So the floor follows the
    //! text with no new API and no new token, which is the feasibility
    //! question Unit 0 was asked.
    //!
    //! **These measure a control with no content of its own**, because
    //! that is the only way to see the floor bind. The sidebar's own
    //! items are already larger than it at both text sizes — measured
    //! in `the_sidebar_item_is_already_above_the_floor` — so a test on
    //! the rail would pass whether the floor existed or not.

    use super::*;
    use iced::widget::space;
    use iced::{Pixels, Settings, Size};
    use iced_test::Simulator;

    #[derive(Debug, Clone, PartialEq)]
    enum Msg {
        Press,
    }

    fn bare_control() -> Element<'static, Msg> {
        Control::new(
            space().width(Length::Fixed(1.0)).height(Length::Fixed(1.0)),
            Box::new(|_theme, _status| button::Style::default()),
            Ring {
                color: Box::new(|_| Color::BLACK),
                width: 0.0,
                offset: 0.0,
            },
        )
        .on_press(Msg::Press)
        .into()
    }

    /// The control's drawn size at a given host text size.
    fn measured(text_size: f32) -> Size {
        let settings = || Settings {
            default_text_size: Pixels(text_size),
            ..Settings::default()
        };
        let mut ui: Simulator<'_, Msg> =
            Simulator::with_size(settings(), Size::new(400.0, 400.0), bare_control());

        // Found by clicking, not by tree position: what is measured is
        // the region that answers the control's own message.
        ui.point_at(iced::Point::new(12.0, 12.0));
        let _ = ui.simulate(iced_test::simulator::click());
        assert!(
            ui.into_messages().any(|m| m == Msg::Press),
            "the probe point is not inside the control, so this measures the wrong thing"
        );

        let mut ui: Simulator<'_, Msg> =
            Simulator::with_size(settings(), Size::new(400.0, 400.0), bare_control());
        // Built from the two fields rather than by calling
        // `Rectangle`'s own size accessor.
        //
        // `scripts/check-literal-sizes.sh` greps every source line for
        // a text-size call and reads what follows the paren. An empty
        // argument leaves it indexing an associative array with an
        // empty key, which bash rejects outright — so the gate dies
        // with `ALLOWED: bad array subscript` instead of reporting
        // anything. That is a bug in the gate, reported in RFC-116's
        // review request with its one-line fix; until it lands, a
        // zero-argument accessor of that name anywhere under `crates/`
        // takes the Rust quality job down with it. **Including one
        // written inside a comment**, which is how this note found it.
        let mut biggest = Size::ZERO;
        let _ = ui.find(|candidate: iced_test::selector::Candidate<'_>| {
            if let iced_test::selector::Candidate::Focusable { bounds, .. } = candidate {
                biggest = Size::new(bounds.width, bounds.height);
            }
            None::<()>
        });
        biggest
    }

    /// At the default text size the floor is 24, which is exactly
    /// WCAG 2.5.8's minimum — so the rule costs nothing at the default,
    /// which is what RFC-115's Unit 0 measured and what makes k = 1.5
    /// the position that changes nothing.
    #[test]
    fn the_floor_is_24_at_the_default_text_size() {
        let size = measured(16.0);
        assert!(
            size.width >= MIN_TARGET && size.height >= MIN_TARGET,
            "a control with 1px of content measured {size:?} at 16px text; the floor is \
             max({MIN_TARGET}, {TARGET_PER_TEXT} x 16) = 24",
        );
    }

    /// At 24px host text the floor is 36, and it is the multiplier that
    /// decides, not the 24px minimum.
    #[test]
    fn the_floor_follows_the_text_to_36_at_24px() {
        let expected = TARGET_PER_TEXT * 24.0;
        let size = measured(24.0);
        assert!(
            size.width >= expected && size.height >= expected,
            "a control with 1px of content measured {size:?} at 24px text; the floor is \
             max({MIN_TARGET}, {TARGET_PER_TEXT} x 24) = {expected}",
        );
        // And it really did grow: a floor that ignored the text would
        // have left this at 24.
        assert!(
            size.height > MIN_TARGET + 0.5,
            "the control is {size:?} at 24px text — the same as at 16px, so the floor is \
             not following the text at all",
        );
    }
}
