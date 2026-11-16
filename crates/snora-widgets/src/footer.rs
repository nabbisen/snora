//! A minimal desktop-style footer / status bar.
//!
//! The footer is a thin container with the given content placed inside.
//! The footer is content-agnostic, but it has a reading order: its content
//! sits at the **start edge** (the left under LTR, the right under RTL), so
//! it takes the direction (RFC-133). Pass a direction-aware `row` (built with
//! [`crate::direction::row_dir`]) as the `content` when you need start / end
//! layout inside the footer.

use iced::{Element, Length, Padding, widget::container};
use snora_core::LayoutDirection;

use crate::direction::start_aligned;
use crate::style::chrome_container_style_with_radius;

/// Geometry parameters [`build_footer`] takes, letting [`app_footer`]
/// (unstyled) and the `design`-gated styled variant (RFC-040) share one
/// implementation.
#[derive(Debug, PartialEq)]
pub(crate) struct FooterGeometry {
    /// Horizontal container padding.
    pub(crate) pad_x: f32,
    /// Vertical container padding.
    pub(crate) pad_y: f32,
    /// Chrome container corner radius.
    pub(crate) radius: f32,
}

impl FooterGeometry {
    /// Today's literals, unmodified.
    pub(crate) const fn unstyled() -> Self {
        Self {
            pad_x: 16.0,
            pad_y: 6.0,
            radius: 0.0,
        }
    }
}

/// Wrap `content` in a chrome-styled footer bar, with the content at the start
/// edge of `direction`.
pub fn app_footer<'a, Message>(
    content: Element<'a, Message>,
    direction: LayoutDirection,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    build_footer(content, FooterGeometry::unstyled(), direction)
}

pub(crate) fn build_footer<'a, Message>(
    content: Element<'a, Message>,
    geometry: FooterGeometry,
    direction: LayoutDirection,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(content)
        .width(Length::Fill)
        .align_x(start_aligned(direction))
        .padding(Padding::from([geometry.pad_y, geometry.pad_x]))
        .style(move |theme| chrome_container_style_with_radius(theme, geometry.radius))
        .into()
}
