use iced::{
    Alignment::Center,
    Element, Length, Padding,
    widget::{button, column, container, row, space, text},
};
use snora_core::contract::app::Menu;

use crate::{
    components::icon::render_icon,
    style::{container_box_style, menu_button_style},
};

pub fn app_header<'a, Message: Clone + 'a, MenuId: PartialEq + Clone + std::fmt::Debug + 'a, F>(
    app_title: &'a str,
    menus: Vec<Menu<MenuId>>,
    menu_on_select: &'a F,
    right_controls: Option<Element<'a, Message>>,
) -> Element<'a, Message>
where
    F: Fn(MenuId) -> Message + 'a,
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
        let menu = render_menu(menu, menu_on_select);
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

fn render_menu<'a, MenuId, Message>(
    menu: Menu<MenuId>,
    menu_on_select: impl Fn(MenuId) -> Message + 'a,
) -> Element<'a, Message>
where
    MenuId: PartialEq + Clone + std::fmt::Debug + 'a,
    Message: Clone + 'a,
{
    let mut content = column![text(menu.label).size(14)];

    for item in menu.items {
        let mut btn_content = row![].spacing(6).align_y(Center);
        if let Some(ref ic) = item.icon {
            btn_content = btn_content.push(render_icon(ic));
        }
        btn_content = btn_content.push(text(item.label).size(14));

        let msg = menu_on_select(item.menu_id.clone()); // ここでアプリの Message に変換
        content = content.push(button(btn_content).style(menu_button_style).on_press(msg));
    }

    container(content).into()
}
