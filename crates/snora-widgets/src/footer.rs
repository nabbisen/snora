//! A minimal desktop-style footer / status bar.
//!
//! The footer is a thin container with the given content placed inside.
//! Because the footer itself is content-agnostic, direction is not a
//! parameter here — pass a direction-aware `row` (built with
//! [`crate::direction::row_dir`]) as the `content` when you need
//! start / end layout inside the footer.

use iced::{Element, Length, Padding, widget::container};

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

/// Wrap `content` in a chrome-styled footer bar.
pub fn app_footer<'a, Message>(content: Element<'a, Message>) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    build_footer(content, FooterGeometry::unstyled())
}

pub(crate) fn build_footer<'a, Message>(
    content: Element<'a, Message>,
    geometry: FooterGeometry,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(content)
        .width(Length::Fill)
        .padding(Padding::from([geometry.pad_y, geometry.pad_x]))
        .style(move |theme| chrome_container_style_with_radius(theme, geometry.radius))
        .into()
}
