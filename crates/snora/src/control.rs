//! The public control: [`Control`], and the two functions that make one. The
//! documentation is on the type, because this module is private.

use iced::widget::button;
use iced::{Color, Element, Length, Padding, Theme};
use snora_interaction::contrast::readable_text;
use snora_interaction::{Ring, StyleFn};

/// What a press publishes: the message itself, or a way to make it.
enum OnPress<'a, Message> {
    Direct(Message),
    With(Box<dyn Fn() -> Message + 'a>),
}

/// How the ring is drawn on one path.
#[derive(Clone, Copy)]
struct RingConfig {
    width: f32,
    offset: f32,
    /// The colour the design path prefers (`Tokens::focus`); `None` prefers the
    /// theme's own text colour.
    preferred: Option<Color>,
    /// What the ring must be readable against: the design path's page background,
    /// or the theme's, which is what a control sits on unless the application says
    /// otherwise.
    behind: Option<Color>,
}

impl RingConfig {
    /// The ring's colour on `theme`: the preferred one where it reads against what
    /// is behind the control (4.5:1, the shared rule), and the stronger of black and
    /// white where it does not. At least 4.58:1 against an opaque `behind`, which is
    /// how it clears 3:1 against that surface on a theme snora has never seen.
    fn color(self, theme: &Theme) -> Color {
        let palette = theme.extended_palette();
        let behind = self.behind.unwrap_or(palette.background.base.color);
        let preferred = self.preferred.unwrap_or(palette.background.base.text);
        readable_text(behind, preferred)
    }
}

/// **Your own content, as a control the keyboard can reach** (RFC-122).
///
/// iced 0.14's `button` handles the mouse and touch and registers no focus target,
/// so a row, a card or an icon-and-label that an application makes pressable with
/// `button` cannot be reached with Tab, and Enter and Space press nothing. snora's
/// own controls (the sidebar, the chips, the toast's close mark, …) are built on a
/// control that does all of it; [`control()`](fn@control) is that control, around whatever content
/// an application draws.
///
/// # One name changes
///
/// ```
/// # use iced::widget::{row, text};
/// # #[derive(Clone)] enum Message { Open }
/// # fn view<'a>() -> iced::Element<'a, Message> {
/// // before: iced::widget::button(row![text("📂"), text("Open")]).on_press(Message::Open)
/// snora::control(row![text("📂"), text("Open")])
///     .on_press(Message::Open)
///     .padding([6, 12])
///     .into()
/// # }
/// ```
///
/// The builder is `button`'s: [`on_press`](Control::on_press),
/// [`on_press_maybe`](Control::on_press_maybe) (where `None` is disabled and
/// Tab skips it), [`on_press_with`](Control::on_press_with),
/// [`padding`](Control::padding), [`width`](Control::width),
/// [`height`](Control::height), [`clip`](Control::clip) and
/// [`style`](Control::style), which takes iced's own
/// `Fn(&Theme, button::Status) -> button::Style`, so an existing style closure, and
/// `snora::design::style::button::*`, work unchanged. **Nothing else is public.**
///
/// # What is built in, with nothing to set
///
/// - a focus target: Tab and Shift+Tab reach it, in the order the content is laid out;
/// - Enter and Space press it, once per key press;
/// - a click presses it and does **not** focus it, so Tab carries on from where it was;
/// - a focus ring while it is focused, and none under the pointer;
/// - a target no smaller than `max(24, 1.5 × the label's text size)` (WCAG 2.5.8);
/// - out of reach while a dialog or sheet is open over it;
/// - when it goes away after being pressed, focus follows snora's rule for a
///   removal (RFC-120).
///
/// ## The ring
///
/// The ring is at least 3:1 against the page and every neutral surface snora's tokens
/// define; on the engine path, against the theme's background tiers except
/// SolarizedDark's `strongest` (2.91:1). On a coloured fill of your own, check it.
///
/// On the engine path a dialog or sheet has no card of its own: put its content on a
/// container with the theme's background. A control or text placed directly on the dim may
/// not be legible, and its focus ring may fall under 3:1.
///
/// ## The target
///
/// The target follows the *label's* size, not the size of whatever you put inside.
/// Content with larger text of its own should set its own `padding`.
///
/// # Built on `snora-interaction`
///
/// This is a thin layer over `snora-interaction`'s `Control`, which snora's own
/// widgets use and which applications do not depend on: depend on `snora`. The
/// internal control's other settings (its tag, the span of a removal, the minimum
/// target and text size, the focus-change messages) are not offered, on purpose.
///
/// # Two paths
///
/// [`control()`](fn@control) is the **engine path**: no tokens, iced's own default style
/// (`button::primary`, what `iced::widget::button` draws with), a 2 px ring offset by
/// 2 px in the theme's text colour, and no target token. With the `design` feature,
/// `snora::design::control` takes the ring and the minimum target from your
/// `Tokens` and draws as `ghost` until you say otherwise.
///
/// Made by [`control()`](fn@control) (engine path) or `snora::design::control` (design
/// path). It is not a widget on its own: convert it with `.into()` or use it where an
/// `Element` is expected.
#[must_use = "a control does nothing until it is added to a view"]
pub struct Control<'a, Message> {
    content: Element<'a, Message>,
    on_press: Option<OnPress<'a, Message>>,
    padding: Padding,
    width: Length,
    height: Length,
    clip: bool,
    style: StyleFn<'a>,
    ring: RingConfig,
    min_target: f32,
    text_size: Option<f32>,
}

/// Make `content` a control the keyboard can reach. The **engine path**: no tokens.
///
/// The one-line move from iced's `button`:
///
/// ```
/// # use iced::widget::text;
/// # #[derive(Clone)] enum Message { Open }
/// # fn view<'a>() -> iced::Element<'a, Message> {
/// // iced::widget::button(text("Open")).on_press(Message::Open)
/// snora::control(text("Open")).on_press(Message::Open).into()
/// # }
/// ```
///
/// Without [`on_press`](Control::on_press) (or with `on_press_maybe(None)`) the
/// control is disabled: it draws as `button::Status::Disabled`, Tab skips it and
/// nothing presses it. See [`Control`] for everything that is built in, and for the ring's
/// guarantee.
pub fn control<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Control<'a, Message> {
    Control {
        content: content.into(),
        on_press: None,
        // What `iced::widget::button` pads its content with.
        padding: Padding::from([5.0, 10.0]),
        width: Length::Shrink,
        height: Length::Shrink,
        clip: false,
        // iced's own default, so a move from `button` changes one name only.
        style: Box::new(button::primary),
        ring: RingConfig {
            width: 2.0,
            offset: 2.0,
            preferred: None,
            behind: None,
        },
        min_target: 0.0,
        text_size: None,
    }
}

/// The design path (`design` feature): the ring and the minimum target from `tokens`,
/// `ghost` by default. Reached as `snora::design::control`.
#[cfg(feature = "design")]
pub(crate) fn designed<'a, Message: Clone + 'a>(
    tokens: &snora_design::Tokens,
    content: impl Into<Element<'a, Message>>,
) -> Control<'a, Message> {
    let for_style = tokens.clone();
    Control {
        content: content.into(),
        on_press: None,
        padding: Padding::from([5.0, 10.0]),
        width: Length::Shrink,
        height: Length::Shrink,
        clip: false,
        style: Box::new(move |_theme, status| snora_style::button::ghost(&for_style, status)),
        ring: RingConfig {
            width: tokens.focus.ring_width,
            offset: tokens.focus.ring_offset,
            preferred: Some(snora_style::color::to_iced_color(tokens.focus.ring_color)),
            behind: Some(snora_style::color::to_iced_color(tokens.palette.background)),
        },
        min_target: tokens.targets.min,
        // The target is measured against the text the control carries, which on
        // this path is drawn at the label size.
        text_size: Some(tokens.typography.label.size),
    }
}

impl<'a, Message: Clone + 'a> Control<'a, Message> {
    /// The message published when the control is pressed: by a click, or by Enter or
    /// Space while it is focused.
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(OnPress::Direct(message));
        self
    }

    /// Like [`on_press`](Self::on_press), and `None` leaves the control disabled:
    /// Tab skips it and nothing presses it.
    pub fn on_press_maybe(mut self, message: Option<Message>) -> Self {
        self.on_press = message.map(OnPress::Direct);
        self
    }

    /// Like [`on_press`](Self::on_press), with the message made when the press
    /// happens, for a message that is costly to build.
    pub fn on_press_with(mut self, message: impl Fn() -> Message + 'a) -> Self {
        self.on_press = Some(OnPress::With(Box::new(message)));
        self
    }

    /// Padding between the control's bounds and its content. Defaults to
    /// `[5, 10]`, what `iced::widget::button` pads with.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// The control's width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// The control's height.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Cut the content off at the control's own edge rather than letting it run on
    /// past it. As `button`'s.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// How the control looks in each state: iced's own `button::Status` in and
    /// `button::Style` out, so every existing `button` style closure, and
    /// `snora::design::style::button::*`, works unchanged. Replaces the default
    /// (`button::primary` on the engine path, `ghost` on the design path).
    ///
    /// The ring is not part of the style: it is drawn by the control, at the
    /// guarantee in [`Control`]'s documentation, whatever the style says.
    pub fn style(mut self, style: impl Fn(&Theme, button::Status) -> button::Style + 'a) -> Self {
        self.style = Box::new(style);
        self
    }
}

impl<'a, Message: Clone + 'a> From<Control<'a, Message>> for Element<'a, Message> {
    fn from(control: Control<'a, Message>) -> Self {
        let ring = control.ring;
        let mut inner = snora_interaction::Control::new(
            control.content,
            control.style,
            Ring {
                color: Box::new(move |theme: &Theme| ring.color(theme)),
                width: ring.width,
                offset: ring.offset,
            },
        )
        .min_target(control.min_target)
        .padding(control.padding)
        .width(control.width)
        .height(control.height)
        .clip(control.clip);
        if let Some(size) = control.text_size {
            inner = inner.text_size(size);
        }
        inner = match control.on_press {
            Some(OnPress::Direct(message)) => inner.on_press(message),
            Some(OnPress::With(make)) => inner.on_press_with(make),
            None => inner,
        };
        inner.into()
    }
}

#[cfg(test)]
mod tests {
    //! What the wrapper decides on its own: the default style of each path, that
    //! `style` replaces it, and the ring's colour, against the surfaces the rustdoc
    //! promises it for. What the control *does* (focus, keys, clicks, the floor) is
    //! `tests/every_control.rs`'s and `tests/public_control.rs`'s, through a real
    //! window loop.
    use super::*;
    use iced::widget::text;
    use snora_interaction::contrast::contrast_ratio;

    #[derive(Clone)]
    struct Msg;

    const STATUSES: [button::Status; 4] = [
        button::Status::Active,
        button::Status::Hovered,
        button::Status::Pressed,
        button::Status::Disabled,
    ];

    /// The engine path draws as iced's own `button::primary`, so a move from `button`
    /// changes one name; `style` replaces it.
    #[test]
    fn the_engine_default_is_iced_primary_and_style_replaces_it() {
        for theme in Theme::ALL {
            let default = control::<Msg>(text("x"));
            let replaced = control::<Msg>(text("x")).style(button::danger);
            for status in STATUSES {
                assert_eq!(
                    (default.style)(theme, status),
                    button::primary(theme, status),
                    "{theme:?} / {status:?}: the engine default is not iced's primary"
                );
                assert_eq!(
                    (replaced.style)(theme, status),
                    button::danger(theme, status),
                    "{theme:?} / {status:?}: `style` did not replace the default"
                );
            }
            assert_ne!(
                (default.style)(theme, button::Status::Active),
                (replaced.style)(theme, button::Status::Active),
                "{theme:?}: primary and danger look the same, so the check cannot tell"
            );
        }
    }

    /// The design path draws as `ghost`, on every preset and in every status;
    /// `style` replaces it.
    #[cfg(feature = "design")]
    #[test]
    fn the_design_default_is_ghost_and_style_replaces_it() {
        for (preset, tokens) in snora_design::Tokens::presets() {
            let theme = snora_style::theme::theme(&tokens);
            let default = designed::<Msg>(&tokens, text("x"));
            let for_style = tokens.clone();
            let replaced = designed::<Msg>(&tokens, text("x"))
                .style(move |_theme, status| snora_style::button::primary(&for_style, status));
            for status in STATUSES {
                assert_eq!(
                    (default.style)(&theme, status),
                    snora_style::button::ghost(&tokens, status),
                    "{preset} / {status:?}: the design default is not ghost"
                );
                assert_eq!(
                    (replaced.style)(&theme, status),
                    snora_style::button::primary(&tokens, status),
                    "{preset} / {status:?}: `style` did not replace the default"
                );
            }
            assert_ne!(
                (default.style)(&theme, button::Status::Active),
                (replaced.style)(&theme, button::Status::Active),
                "{preset}: ghost and primary look the same at rest, so the check cannot tell"
            );
        }
    }

    /// **The rustdoc's ring guarantee, as a test**, so the sentence cannot go stale:
    ///
    /// - engine path, in the 22 stock themes: at least 3:1 against each of the
    ///   theme's eight background tiers, **except SolarizedDark's `strongest`**, which
    ///   is the one measured miss (2.91:1) and is asserted to still be one, so that
    ///   the day it clears, the sentence is shortened;
    /// - design path, in the 6 presets: at least 3:1 against `background`, `surface`
    ///   and `surface_raised`.
    ///
    /// Not asserted, and not promised: a coloured fill (an accent, a status colour,
    /// a primary button's base). The sentence says "check it"; they measure under 3:1 and
    /// are not placements snora makes (RFC-122 Unit 0).
    #[test]
    fn the_ring_guarantee_in_the_rustdoc_holds() {
        let mut misses = Vec::new();
        for theme in Theme::ALL {
            let ring = control::<Msg>(text("x")).ring.color(theme);
            let b = &theme.extended_palette().background;
            for (tier, colour) in [
                ("base", b.base.color),
                ("weakest", b.weakest.color),
                ("weaker", b.weaker.color),
                ("weak", b.weak.color),
                ("neutral", b.neutral.color),
                ("strong", b.strong.color),
                ("stronger", b.stronger.color),
                ("strongest", b.strongest.color),
            ] {
                let ratio = contrast_ratio(ring, colour);
                if ratio < 3.0 {
                    misses.push((format!("{theme:?}"), tier, ratio));
                }
            }
        }
        assert_eq!(
            misses.len(),
            1,
            "the rustdoc names exactly one engine-path exception, SolarizedDark's \
             `strongest`; found {misses:?}. Change the sentence in the module \
             documentation with this assertion"
        );
        let (theme, tier, ratio) = &misses[0];
        assert_eq!((theme.as_str(), *tier), ("SolarizedDark", "strongest"));
        assert!(
            (2.9..3.0).contains(ratio),
            "the rustdoc says 2.91:1; it is {ratio:.2}:1"
        );
    }

    /// **The fallback itself.** No stock theme or preset ever needs it (their own
    /// text colour and `Tokens::focus` already read against the page), so the table
    /// above cannot tell the rule from "always the preferred colour". This can: a
    /// preferred colour that does not read against what is behind it is replaced by
    /// a pole that does, and one that does is kept.
    #[test]
    fn a_preferred_colour_that_does_not_read_is_replaced_by_a_pole() {
        let behind = Color::from_rgb(0.5, 0.5, 0.5);
        let theme = Theme::Light;
        let weak = RingConfig {
            width: 2.0,
            offset: 2.0,
            preferred: Some(Color::from_rgb(0.55, 0.55, 0.55)),
            behind: Some(behind),
        };
        let chosen = weak.color(&theme);
        assert!(
            contrast_ratio(chosen, behind) >= 4.5,
            "a ring of {chosen:?} on {behind:?} was kept at {:.2}:1",
            contrast_ratio(chosen, behind)
        );
        assert!(chosen == Color::BLACK || chosen == Color::WHITE);

        let strong = RingConfig {
            preferred: Some(Color::from_rgb(1.0, 0.0, 0.0)),
            behind: Some(Color::WHITE),
            ..weak
        };
        // Red on white is 4.0:1, under the floor: replaced. Dark red is 9.6:1: kept.
        assert!(strong.color(&theme) == Color::BLACK);
        let kept = RingConfig {
            preferred: Some(Color::from_rgb(0.5, 0.0, 0.0)),
            ..strong
        };
        assert_eq!(kept.color(&theme), Color::from_rgb(0.5, 0.0, 0.0));
    }

    #[cfg(feature = "design")]
    #[test]
    fn the_design_ring_clears_the_page_and_every_neutral_surface() {
        let to_iced = snora_style::color::to_iced_color;
        for (preset, tokens) in snora_design::Tokens::presets() {
            let theme = snora_style::theme::theme(&tokens);
            let ring = designed::<Msg>(&tokens, text("x")).ring.color(&theme);
            let p = &tokens.palette;
            for (name, colour) in [
                ("background", p.background),
                ("surface", p.surface),
                ("surface_raised", p.surface_raised),
            ] {
                let ratio = contrast_ratio(ring, to_iced(colour));
                assert!(
                    ratio >= 3.0,
                    "{preset}: the ring is {ratio:.2}:1 against {name}; the rustdoc promises 3:1"
                );
            }
        }
    }
}
