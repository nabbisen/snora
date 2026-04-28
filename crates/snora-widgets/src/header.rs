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
use crate::style::chrome_container_style;
use crate::menu::render_menu;

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
    .spacing(12);

    for menu in menus {
        let is_active = active_menu_id == Some(&menu.id);
        start_group = start_group.push(render_menu(menu, on_menu_action, is_active));
    }

    // Middle filler — pushes end_controls to the far edge.
    let filler = container(space()).width(Length::Fill);

    // Compose start + filler + end in logical order.
    let end_side: Element<'_, Message> = match end_controls {
        Some(ctrls) => iced::widget::row![filler, ctrls]
            .align_y(Center)
            .spacing(12)
            .into(),
        None => filler.into(),
    };

    let header_row = row_dir(direction, start_group, end_side).align_y(Center);

    container(header_row)
        .width(Length::Fill)
        .padding(Padding::from([8.0, 16.0]))
        .style(chrome_container_style)
        .into()
}
