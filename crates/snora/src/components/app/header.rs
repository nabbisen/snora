use std::fmt::Debug;

use iced::{
    Alignment::Center,
    Element, Length, Padding,
    widget::{container, row, space, text},
};
use snora_core::contract::app::header::menu::{Menu, MenuAction};

mod menu;

use crate::style::container_box_style;
use menu::render_menu;

pub fn app_header<'a, Message, MenuId, MenuItemId, F>(
    app_title: &'a str,
    menus: Vec<Menu<MenuId, MenuItemId>>,
    menus_on_action: &'a F,
    active_menu_id: Option<&MenuId>,
    right_controls: Option<Element<'a, Message>>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    MenuId: Clone + Debug + PartialEq + 'a,
    MenuItemId: Clone + Debug + 'a,
    F: Fn(MenuAction<MenuId, MenuItemId>) -> Message + 'a,
{
    let mut left_row = row![
        text(app_title)
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
        let is_active = Some(&menu.id) == active_menu_id;
        let menu = render_menu(menu, menus_on_action, is_active);
        left_row = left_row.push(menu);
    }

    let mut header_row = row![left_row, container(space()).width(Length::Fill)].align_y(Center);

    if let Some(controls) = right_controls {
        header_row = header_row.push(controls);
    }

    container(header_row)
        .width(Length::Fill)
        .padding(Padding::from([8.0, 16.0]))
        .style(container_box_style)
        .into()
}
