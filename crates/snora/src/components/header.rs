use iced::{
    widget::{button, container, row, space, text},
    Alignment::Center,
    Element, Length, Padding,
};
use snora_core::contract::MenuItem;

use crate::{
    components::icon::render_icon,
    style::{container_box_style, menu_button_style},
};

pub fn standard_header<'a, Message: Clone + 'a>(
    app_title: &'a str,
    items: Vec<MenuItem<Message>>,
    right_controls: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
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

    for item in items {
        let mut btn_content = row![].spacing(6).align_y(Center);
        if let Some(ref ic) = item.icon {
            btn_content = btn_content.push(render_icon(ic));
        }
        btn_content = btn_content.push(text(item.label).size(14));

        let mut btn = button(btn_content)
            .padding(Padding::new(6.0))
            .style(menu_button_style);

        if let Some(action) = item.action {
            btn = btn.on_press(action);
        }

        left_row = left_row.push(btn);
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
