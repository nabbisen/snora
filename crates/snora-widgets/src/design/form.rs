//! The five form controls on the design path (RFC-134): the same
//! [`Checkbox`](crate::Checkbox), [`Toggler`](crate::Toggler), [`RadioGroup`](crate::RadioGroup), [`Slider`](crate::Slider) and [`Select`](crate::Select) as the engine path's, drawn from
//! [`Tokens`](snora_design::Tokens) instead of the host's theme.
//!
//! The marks take the palette's `border` (the outline, held at 3:1 against the page), `accent` and
//! `accent_text` (the checked box, the switch, the dot and the filled bar, and what is drawn on
//! them), and `text_primary`; the label is `Typography::label`; the ring and the minimum target are
//! the tokens'. A hovered or pressed row is `snora_style::form::row`: the label is held at the
//! palette's text level on every neutral surface (RFC-129).

use snora_core::LayoutDirection;
use snora_design::Tokens;

use crate::form::{Checkbox, RadioGroup, Select, Skin, Slider, Toggler};

/// A checkbox drawn from `tokens`. See [`Checkbox`].
pub fn checkbox<'a, Message>(
    tokens: &Tokens,
    is_checked: bool,
    direction: LayoutDirection,
) -> Checkbox<'a, Message> {
    Checkbox::with_skin(Skin::designed(tokens), is_checked, direction)
}

/// A toggle drawn from `tokens`. See [`Toggler`].
pub fn toggler<'a, Message>(
    tokens: &Tokens,
    is_toggled: bool,
    direction: LayoutDirection,
) -> Toggler<'a, Message> {
    Toggler::with_skin(Skin::designed(tokens), is_toggled, direction)
}

/// A radio group drawn from `tokens`. See [`RadioGroup`].
pub fn radio_group<'a, T, Message>(
    tokens: &Tokens,
    options: impl Into<Vec<T>>,
    selected: Option<&T>,
    on_select: impl Fn(T) -> Message + 'a,
    direction: LayoutDirection,
) -> RadioGroup<'a, T, Message>
where
    T: ToString + PartialEq + Clone,
{
    RadioGroup::with_skin(
        Skin::designed(tokens),
        options,
        selected,
        on_select,
        direction,
    )
}

/// A select drawn from `tokens`. See [`Select`].
pub fn select<'a, T, Message>(
    tokens: &Tokens,
    options: impl Into<Vec<T>>,
    selected: Option<&T>,
    on_select: impl Fn(T) -> Message + 'a,
    direction: LayoutDirection,
) -> Select<'a, T, Message>
where
    T: ToString + PartialEq + Clone,
{
    Select::with_skin(
        Skin::designed(tokens),
        options,
        selected,
        on_select,
        direction,
    )
}

/// A slider drawn from `tokens`. See [`Slider`].
pub fn slider<'a, Message>(
    tokens: &Tokens,
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    on_change: impl Fn(f32) -> Message + 'a,
    direction: LayoutDirection,
) -> Slider<'a, Message> {
    Slider::with_skin(Skin::designed(tokens), range, value, on_change, direction)
}
