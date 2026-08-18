//! Visible-focus styling tokens ([`FocusTokens`]).

use crate::Color;

/// Tokens describing a focus ring.
///
/// These are design *vocabulary*, with a **present-day audience**: any
/// application that already owns focus as its own state (a focus-zone enum
/// cycled by Tab, say) can read that state inside its own `container` style
/// closure — an arbitrary `Fn(&iced::Theme) -> Style` — and set border
/// colour and width from these tokens. That is not blocked by iced 0.14.
///
/// What iced 0.14 does not support is **standard** `button`/`container`
/// styling telling its *own* style closure that the widget is focused —
/// iced does not report that state, so a closure that does not already
/// know it cannot draw a ring from it. This affects only widgets that let
/// iced own focus (snora's own prefab widgets do); it does not affect
/// applications with their own focus state. See the `snora-style` style
/// bridge for the documented limitation.
///
/// ```
/// use snora_design::{Color, FocusTokens};
/// let f = FocusTokens::new(2.0, 2.0, Color::rgb(0.11, 0.31, 0.85));
/// assert_eq!(f.ring_width, 2.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusTokens {
    /// Focus-ring thickness (logical pixels).
    pub ring_width: f32,
    /// Gap between the control edge and the ring (logical pixels).
    pub ring_offset: f32,
    /// Focus-ring color.
    pub ring_color: Color,
}

impl FocusTokens {
    /// Constructs focus tokens from a width, offset, and ring color.
    #[must_use]
    pub const fn new(ring_width: f32, ring_offset: f32, ring_color: Color) -> Self {
        Self {
            ring_width,
            ring_offset,
            ring_color,
        }
    }
}

#[cfg(test)]
mod tests;
