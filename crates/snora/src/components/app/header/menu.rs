use std::fmt::Debug;

use iced::{
    Alignment::Center,
    Element,
    widget::{button, column, container, row, text},
};
use snora_core::contract::app::header::menu::{Menu, MenuAction};

use crate::{components::icon::render_icon, style::menu_button_style};

pub fn render_menu<'a, Message, MenuId, MenuItemId, F>(
    menu: Menu<MenuId, MenuItemId>,
    menus_on_action: &'a F,
    is_active: bool,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    MenuId: Clone + Debug + PartialEq + 'a,
    MenuItemId: Clone + Debug + 'a,
    F: Fn(MenuAction<MenuId, MenuItemId>) -> Message + 'a,
{
    let mut content = column![];

    let header_msg = menus_on_action(MenuAction::MenuPressed(menu.id.clone()));

    let mut header_content = row![].spacing(6).align_y(Center);

    if let Some(ref ic) = menu.icon {
        header_content = header_content.push(render_icon(ic));
    }

    header_content = header_content.push(text(menu.label).size(14));

    content = content.push(
        button(header_content)
            .style(menu_button_style)
            .on_press(header_msg),
    );

    if !is_active {
        return container(content).into();
    }

    for item in menu.items {
        let mut btn_content = row![].spacing(6).align_y(Center);

        if let Some(ref ic) = item.icon {
            btn_content = btn_content.push(render_icon(ic));
        }

        btn_content = btn_content.push(text(item.label).size(14));

        let msg = menus_on_action(MenuAction::MenuItemPressed {
            menu_id: item.menu_id.clone(),
            menu_item_id: item.id.clone(),
        });

        content = content.push(button(btn_content).style(menu_button_style).on_press(msg));
    }

    container(content).into()
}
