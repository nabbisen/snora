//! A minimal desktop-style footer / status bar.
//!
//! The footer is a thin container with the given content placed inside.
//! Because the footer itself is content-agnostic, direction is not a
//! parameter here — pass a direction-aware `row` (built with
//! [`crate::direction::row_dir`]) as the `content` when you need
//! start / end layout inside the footer.

use iced::{Element, Length, Padding, widget::container};

use crate::style::chrome_container_style;

/// Wrap `content` in a chrome-styled footer bar.
pub fn app_footer<'a, Message>(content: Element<'a, Message>) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(content)
        .width(Length::Fill)
        .padding(Padding::from([6.0, 16.0]))
        .style(chrome_container_style)
        .into()
}
