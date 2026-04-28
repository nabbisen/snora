//! Icon-rail sidebar.
//!
//! Produces a vertical strip of icon buttons with tooltips. The active
//! item (matching [`SideBar::active`]) gets a subtle background highlight.
//!
//! Tooltip side is direction-aware: it appears on the end side of the rail
//! so it never overlaps the main content.

use iced::{
    Alignment, Length,
    widget::{button, column, container, text, tooltip},
};

use snora_core::{LayoutDirection, SideBar};

use crate::style::sidebar_active_color;
use crate::icon::icon_element;

/// Default rail width in pixels.
const RAIL_WIDTH: f32 = 64.0;
/// Default button size — square, icon-only.
const BUTTON_SIZE: f32 = 48.0;

/// Render a [`SideBar`] as an icon rail.
pub fn app_side_bar<'a, Message, ViewId>(
    side_bar: SideBar<Message, ViewId>,
    direction: LayoutDirection,
) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
    ViewId: Clone + PartialEq + 'a,
{
    let tooltip_position = match direction {
        LayoutDirection::Ltr => tooltip::Position::Right,
        LayoutDirection::Rtl => tooltip::Position::Left,
    };

    let mut col = column![].spacing(16).align_x(Alignment::Center);

    for item in side_bar.items {
        let is_active = item.view_id == side_bar.active;
        let icon = icon_element(&item.icon);

        let btn = button(icon)
            .on_press(item.on_press.clone())
            .width(BUTTON_SIZE)
            .height(BUTTON_SIZE)
            .style(move |theme, status| sidebar_button_style(theme, status, is_active));

        let with_tip = tooltip(btn, text(item.tooltip), tooltip_position);
        col = col.push(with_tip);
    }

    container(col)
        .width(RAIL_WIDTH)
        .height(Length::Fill)
        .padding(16.0)
        .into()
}

fn sidebar_button_style(
    theme: &iced::Theme,
    status: button::Status,
    is_active: bool,
) -> button::Style {
    use iced::{Background, Border, Color, Shadow};

    let ep = theme.extended_palette();
    let base_bg = if is_active {
        Some(Background::Color(sidebar_active_color(theme)))
    } else {
        match status {
            button::Status::Hovered => Some(Background::Color(ep.background.weak.color)),
            _ => None,
        }
    };

    button::Style {
        background: base_bg,
        text_color: ep.background.base.text,
        border: Border {
            radius: 6.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        snap: true,
    }
}
