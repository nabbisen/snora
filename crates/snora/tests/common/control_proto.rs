//! **A prototype of `snora::control` and `snora::design::control`, for RFC-122
//! Unit 0. Nothing here ships.**
//!
//! The public type RFC-122 proposes is a thin wrapper over `snora-interaction`'s
//! internal `Control`, with iced `button`'s builder shape and nothing else:
//!
//! - `on_press`, `on_press_maybe` (`None` is disabled and skipped by Tab),
//!   `on_press_with`;
//! - `padding`, `width`, `height`;
//! - `style(Fn(&Theme, button::Status) -> button::Style)`.
//!
//! What it sets itself, with nothing to set: the focus ring (≥ 3:1), the target
//! floor, the text size the floor is measured against. The engine path
//! ([`control`]) has no tokens, so it takes iced's own default style
//! (`button::primary`, which is what `iced::widget::button` draws with) and the
//! unstyled ring and floor. The design path ([`design_control`]) takes the ring and
//! the minimum target from `Tokens` and defaults to `ghost`.
//!
//! **The wrapper stores its configuration and builds the internal control only when
//! it becomes an `Element`**, because `snora_interaction::Control` takes its style
//! and ring in `new` and has no setter for the style.

#![allow(dead_code)]

use std::rc::Rc;

use iced::widget::button;
use iced::{Color, Element, Length, Padding, Theme};
use snora::design::Tokens;
use snora_interaction::Ring;

enum OnPress<'a, M> {
    Direct(M),
    With(Box<dyn Fn() -> M + 'a>),
}

type StyleFn<'a> = Rc<dyn Fn(&Theme, button::Status) -> button::Style + 'a>;

/// How the ring is drawn on this path.
#[derive(Clone, Copy)]
struct RingConfig {
    width: f32,
    offset: f32,
    /// The colour the design path prefers (`Tokens::focus`); `None`: the theme's text.
    preferred: Option<Color>,
    /// What the ring must be readable against: the design path's page background,
    /// or the theme's, which is where a control sits unless the application says
    /// otherwise.
    behind: Option<Color>,
}

pub struct Control<'a, M> {
    content: Element<'a, M>,
    on_press: Option<OnPress<'a, M>>,
    padding: Padding,
    width: Length,
    height: Length,
    style: StyleFn<'a>,
    ring: RingConfig,
    min_target: f32,
    text_size: Option<f32>,
}

/// The engine path: any content, iced `button`'s builder, no tokens.
pub fn control<'a, M: Clone + 'a>(content: impl Into<Element<'a, M>>) -> Control<'a, M> {
    Control {
        content: content.into(),
        on_press: None,
        // What `iced::widget::button` pads its content with.
        padding: Padding::from([5.0, 10.0]),
        width: Length::Shrink,
        height: Length::Shrink,
        // iced's own default, so a move from `button` changes one name only.
        style: Rc::new(button::primary),
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

/// The design path: the ring and the floor from `tokens`, `ghost` by default.
pub fn design_control<'a, M: Clone + 'a>(
    tokens: &Tokens,
    content: impl Into<Element<'a, M>>,
) -> Control<'a, M> {
    let t = tokens.clone();
    Control {
        content: content.into(),
        on_press: None,
        padding: Padding::from([5.0, 10.0]),
        width: Length::Shrink,
        height: Length::Shrink,
        style: Rc::new(move |_theme, status| snora::design::style::button::ghost(&t, status)),
        ring: RingConfig {
            width: tokens.focus.ring_width,
            offset: tokens.focus.ring_offset,
            preferred: Some(snora_style_color(tokens.focus.ring_color)),
            behind: Some(snora_style_color(tokens.palette.background)),
        },
        min_target: tokens.targets.min,
        text_size: Some(tokens.typography.label.size),
    }
}

fn snora_style_color(c: snora::design::Color) -> Color {
    Color::from_rgba(c.r, c.g, c.b, c.a)
}

impl<'a, M: Clone + 'a> Control<'a, M> {
    #[must_use]
    pub fn on_press(mut self, message: M) -> Self {
        self.on_press = Some(OnPress::Direct(message));
        self
    }

    #[must_use]
    pub fn on_press_maybe(mut self, message: Option<M>) -> Self {
        self.on_press = message.map(OnPress::Direct);
        self
    }

    #[must_use]
    pub fn on_press_with(mut self, message: impl Fn() -> M + 'a) -> Self {
        self.on_press = Some(OnPress::With(Box::new(message)));
        self
    }

    #[must_use]
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// iced's own `button::Status` and `button::Style`, so every existing closure,
    /// and `snora::design::style::button::*`, works unchanged.
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, button::Status) -> button::Style + 'a) -> Self {
        self.style = Rc::new(style);
        self
    }

    /// The ring's colour and what it must be readable against, for a theme (not
    /// public API): what `From<Control>` hands the internal control.
    pub fn resolve_ring(&self, theme: &Theme) -> (Color, Color) {
        let behind = self
            .ring
            .behind
            .unwrap_or_else(|| theme.extended_palette().background.base.color);
        let preferred = self
            .ring
            .preferred
            .unwrap_or_else(|| theme.extended_palette().background.base.text);
        (readable(behind, preferred), behind)
    }

    /// The contrast of two colours, for a test (not public API).
    pub fn contrast_of(a: Color, b: Color) -> f32 {
        contrast(a, b)
    }

    /// What the control would draw as, for a test to compare (not public API).
    pub fn resolve_style(&self, theme: &Theme, status: button::Status) -> button::Style {
        (self.style)(theme, status)
    }
}

// ---- the ring's colour: the preferred one where it is readable, else a pole ---------

fn luminance(c: Color) -> f32 {
    fn channel(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b)
}

fn contrast(a: Color, b: Color) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

fn pole(fill: Color) -> Color {
    if contrast(Color::BLACK, fill) >= contrast(Color::WHITE, fill) {
        Color::BLACK
    } else {
        Color::WHITE
    }
}

/// The preferred colour where it clears AA against `behind`, else the stronger pole
/// (at least 4.58:1 against any opaque fill), which is what snora's own controls do.
fn readable(behind: Color, preferred: Color) -> Color {
    if contrast(preferred, behind) >= 4.5 {
        preferred
    } else {
        pole(behind)
    }
}

impl<'a, M: Clone + 'a> From<Control<'a, M>> for Element<'a, M> {
    fn from(c: Control<'a, M>) -> Self {
        let ring = c.ring;
        let style = Rc::clone(&c.style);
        let mut inner = snora_interaction::Control::new(
            c.content,
            Box::new(move |theme, status| style(theme, status)),
            Ring {
                color: Box::new(move |theme: &Theme| {
                    let behind = ring
                        .behind
                        .unwrap_or_else(|| theme.extended_palette().background.base.color);
                    let preferred = ring
                        .preferred
                        .unwrap_or_else(|| theme.extended_palette().background.base.text);
                    readable(behind, preferred)
                }),
                width: ring.width,
                offset: ring.offset,
            },
        )
        .min_target(c.min_target)
        .padding(c.padding)
        .width(c.width)
        .height(c.height);
        if let Some(size) = c.text_size {
            inner = inner.text_size(size);
        }
        inner = match c.on_press {
            Some(OnPress::Direct(m)) => inner.on_press(m),
            Some(OnPress::With(f)) => inner.on_press_with(f),
            None => inner,
        };
        inner.into()
    }
}
