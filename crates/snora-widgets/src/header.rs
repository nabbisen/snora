//! A minimal desktop-style header bar.
//!
//! Layout (logical, ABDD):
//!
//! ```text
//!  ┌────────────────────────────────────────────────────────────────┐
//!  │ [title] [menu] [menu] [menu] ...           ...   [end_controls]│
//!  └────────────────────────────────────────────────────────────────┘
//!    └────────── start ──────────┘            └────── end ─────────┘
//! ```
//!
//! Under [`LayoutDirection::Rtl`] the two groups swap sides automatically —
//! individual elements inside each group keep their internal order.

use std::fmt::Debug;

use iced::{
    Alignment::Center,
    Element, Length, Padding,
    widget::{container, space, text},
};

use snora_core::{LayoutDirection, Menu, MenuAction};

use crate::direction::row_dir;
use crate::menu::{MenuGeometry, build_menu};
use crate::style::chrome_container_style_with_radius;

/// Geometry parameters [`build_header`] takes, letting [`app_header`]
/// (unstyled) and the `design`-gated styled variant (RFC-040) share one
/// implementation instead of two that could drift.
#[derive(Debug, PartialEq)]
pub(crate) struct HeaderGeometry {
    /// Gap between the title and the first menu, and between the filler
    /// and `end_controls`.
    pub(crate) gap: f32,
    /// Horizontal container padding.
    pub(crate) pad_x: f32,
    /// Vertical container padding.
    pub(crate) pad_y: f32,
    /// Chrome container corner radius.
    pub(crate) radius: f32,
    /// Forwarded to each rendered [`Menu`]'s icon-label gap.
    pub(crate) menu_gap: f32,
}

impl HeaderGeometry {
    /// Today's literals, unmodified.
    pub(crate) const fn unstyled() -> Self {
        Self {
            gap: 12.0,
            pad_x: 16.0,
            pad_y: 8.0,
            radius: 0.0,
            menu_gap: MenuGeometry::unstyled().gap,
        }
    }
}

/// Build an application header.
///
/// * `title` — the app name, rendered bold at the start edge.
/// * `menus` — drop-down menus (File / View / ...). Rendered immediately
///   after the title. Pass `vec![]` for a title-only header.
/// * `on_menu_action` — maps [`MenuAction`] events into your message type.
/// * `active_menu_id` — the currently-open menu, if any. Needed so the
///   menu widget can render its dropdown items. Usually a field on your
///   application state.
/// * `end_controls` — optional element pinned to the end edge
///   (right under LTR, left under RTL). Typically status indicators,
///   theme toggles, etc.
/// * `direction` — application's reading direction.
pub fn app_header<'a, Message, MenuId, MenuItemId, F>(
    title: &'a str,
    menus: Vec<Menu<MenuId, MenuItemId>>,
    on_menu_action: &'a F,
    active_menu_id: Option<&MenuId>,
    end_controls: Option<Element<'a, Message>>,
    direction: LayoutDirection,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    MenuId: Clone + Debug + PartialEq + 'a,
    MenuItemId: Clone + Debug + 'a,
    F: Fn(MenuAction<MenuId, MenuItemId>) -> Message + 'a,
{
    build_header(
        title,
        menus,
        on_menu_action,
        active_menu_id,
        end_controls,
        direction,
        HeaderGeometry::unstyled(),
    )
}

pub(crate) fn build_header<'a, Message, MenuId, MenuItemId, F>(
    title: &'a str,
    menus: Vec<Menu<MenuId, MenuItemId>>,
    on_menu_action: &'a F,
    active_menu_id: Option<&MenuId>,
    end_controls: Option<Element<'a, Message>>,
    direction: LayoutDirection,
    geometry: HeaderGeometry,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    MenuId: Clone + Debug + PartialEq + 'a,
    MenuItemId: Clone + Debug + 'a,
    F: Fn(MenuAction<MenuId, MenuItemId>) -> Message + 'a,
{
    // Start group: [title, gap, menus...].
    let mut start_group = iced::widget::row![
        text(title)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .size(16),
        container(space()).width(Length::Fixed(20.0)),
    ]
    .align_y(Center)
    .spacing(geometry.gap);

    for menu in menus {
        let is_active = active_menu_id == Some(&menu.id);
        start_group = start_group.push(build_menu(
            menu,
            on_menu_action,
            is_active,
            MenuGeometry {
                gap: geometry.menu_gap,
            },
        ));
    }

    // Middle filler — pushes end_controls to the far edge.
    let filler = container(space()).width(Length::Fill);

    // Compose start + filler + end in logical order.
    let end_side: Element<'_, Message> = match end_controls {
        Some(ctrls) => iced::widget::row![filler, ctrls]
            .align_y(Center)
            .spacing(geometry.gap)
            .into(),
        None => filler.into(),
    };

    let header_row = row_dir(direction, start_group, end_side).align_y(Center);

    container(header_row)
        .width(Length::Fill)
        .padding(Padding::from([geometry.pad_y, geometry.pad_x]))
        .style(move |theme| chrome_container_style_with_radius(theme, geometry.radius))
        .into()
}
