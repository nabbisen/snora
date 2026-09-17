//! Icon-rail sidebar.
//!
//! Produces a vertical strip of icon buttons with tooltips. The active
//! item (matching [`SideBar::active`]) gets a subtle background highlight.
//!
//! Tooltip side is direction-aware: it appears on the end side of the rail
//! so it never overlaps the main content.

use iced::{
    Alignment, Length, Padding,
    widget::{button, column, container, text, tooltip},
};

use snora_core::{LayoutDirection, SideBar};

use crate::icon::icon_element;
use crate::style::sidebar_active_color;

/// Default rail width in pixels.
const RAIL_WIDTH: f32 = 64.0;
/// Default button size — square, icon-only.
const BUTTON_SIZE: f32 = 48.0;

/// The rail's horizontal padding: whatever width is left once a button is
/// centred in the rail, split evenly (RFC-099, Q-1 (a)).
///
/// **Derived, not chosen.** The rail used to pad all four sides by the same
/// `16`, leaving a 32px content box for a 48px button, which iced resolved
/// by rendering the button 32px wide — in both variants and every preset,
/// since at least 0.10.0. Horizontal padding is not a spacing decision; it
/// is what is left over, so it is computed here from the two dimensions
/// that are decisions.
///
/// Vertical padding and the inter-button gap remain spacing decisions and
/// keep their sources (literals unstyled, `Spacing` tokens styled).
const RAIL_HORIZONTAL_PADDING: f32 = (RAIL_WIDTH - BUTTON_SIZE) / 2.0;

// A button wider than its rail would make the derived padding negative.
// Checked at compile time rather than in a test: this is the one arithmetic
// fact about these constants that can actually fail.
const _: () = assert!(
    BUTTON_SIZE <= RAIL_WIDTH,
    "sidebar BUTTON_SIZE exceeds RAIL_WIDTH"
);

/// Geometry parameters [`build_side_bar`] takes, letting [`app_side_bar`]
/// (unstyled) and the `design`-gated styled variant (RFC-040) share one
/// implementation.
///
/// Horizontal padding is deliberately absent: it is derived from
/// [`RAIL_WIDTH`] and [`BUTTON_SIZE`] ([`RAIL_HORIZONTAL_PADDING`]) and does
/// not vary by variant or token.
#[derive(Debug, PartialEq)]
pub(crate) struct SideBarGeometry {
    /// Gap between icon buttons.
    pub(crate) gap: f32,
    /// Rail's padding above the first button and below the last.
    ///
    /// Renamed from `padding` (RFC-099): that field was applied to all four
    /// sides, and horizontally it did not fit. It now means only the half it
    /// can safely mean.
    pub(crate) vertical_padding: f32,
    /// Icon button corner radius.
    pub(crate) button_radius: f32,
}

impl SideBarGeometry {
    /// The unstyled variant's literals.
    pub(crate) const fn unstyled() -> Self {
        Self {
            gap: 16.0,
            vertical_padding: 16.0,
            button_radius: 6.0,
        }
    }
}

/// Render a [`SideBar`] as an icon rail.
pub fn app_side_bar<'a, Message, ViewId>(
    side_bar: SideBar<Message, ViewId>,
    direction: LayoutDirection,
) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
    ViewId: Clone + PartialEq + 'a,
{
    build_side_bar(side_bar, direction, SideBarGeometry::unstyled())
}

pub(crate) fn build_side_bar<'a, Message, ViewId>(
    side_bar: SideBar<Message, ViewId>,
    direction: LayoutDirection,
    geometry: SideBarGeometry,
) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
    ViewId: Clone + PartialEq + 'a,
{
    let tooltip_position = match direction {
        LayoutDirection::Ltr => tooltip::Position::Right,
        LayoutDirection::Rtl => tooltip::Position::Left,
    };

    let mut col = column![].spacing(geometry.gap).align_x(Alignment::Center);

    for item in side_bar.items {
        let is_active = item.view_id == side_bar.active;
        let icon = icon_element(&item.icon);
        let button_radius = geometry.button_radius;

        // Centred on both axes (RFC-099). Without this, a fixed-size button
        // hands its content fixed limits, the icon's text widget fills the
        // whole content area, and the glyph is drawn at that box's top-left:
        // measured ~10px left of and ~10px above the button's centre. The
        // container passes *loose* limits to the icon, so the glyph box
        // shrinks to its natural size and is then placed at the centre.
        //
        // The button's padding is zeroed. iced's default is 10 horizontal and
        // 5 vertical, which is symmetric, so centring within the padded area
        // would coincidentally land on the same point — but it would leave
        // an icon only 28 x 38 to live in, clipping any icon larger than that,
        // and it would make "centred in the button" depend on the default
        // staying symmetric. With zero padding the container is the full
        // 48 x 48 and centring means exactly what it says.
        let centred_icon = container(icon).center(Length::Fill);

        let btn = button(centred_icon)
            .on_press(item.on_press.clone())
            .padding(0)
            .width(BUTTON_SIZE)
            .height(BUTTON_SIZE)
            .style(move |theme, status| {
                sidebar_button_style(theme, status, is_active, button_radius)
            });

        let with_tip = tooltip(btn, text(item.tooltip), tooltip_position);
        col = col.push(with_tip);
    }

    // `Padding::from([a, b])` is `[vertical, horizontal]` in iced_core 0.14
    // (top/bottom = a, left/right = b).
    container(col)
        .width(RAIL_WIDTH)
        .height(Length::Fill)
        .padding(Padding::from([
            geometry.vertical_padding,
            RAIL_HORIZONTAL_PADDING,
        ]))
        .into()
}

pub(crate) fn sidebar_button_style(
    theme: &iced::Theme,
    status: button::Status,
    is_active: bool,
    radius: f32,
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

    // Corrected (RFC-085 F-14): the active state used
    // `background.base.text` — calibrated against `background.base`,
    // not against the highlight it was actually painted on
    // (`sidebar_active_color`, a `primary` tier). Measured: 2.01:1
    // (design light), 2.13:1 (design dark), 1.59:1 (high_contrast_light),
    // **1.51:1 (high_contrast_dark)** — the low-vision preset scoring
    // worst, on its own a release blocker (RFC-085 Q-4). `primary.strong`
    // is the tier `sidebar_active_color` now uses; `.text` is iced's own
    // guaranteed-readable pairing for it. The inactive states are
    // unaffected — `background.base.text` already clears the floor
    // against both the page background and `background.weak.color`.
    let text_color = if is_active {
        ep.primary.strong.text
    } else {
        ep.background.base.text
    };

    button::Style {
        background: base_bg,
        text_color,
        border: Border {
            radius: radius.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        snap: true,
    }
}
