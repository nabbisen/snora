//! Visible-focus styling tokens ([`FocusTokens`]).

use crate::Color;

/// Tokens describing a focus ring.
///
/// These are design *vocabulary*. Whether a given widget can render a ring
/// depends on the renderer: in iced 0.14, standard `button`/`container`
/// styling does not expose focus state, so these tokens apply only where the
/// widget surface allows it (and on future iced versions). See the
/// `snora-widgets` style bridge for the documented limitations.
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
