use iced::{
    Background, Color, Element, Length,
    widget::{column, container, text},
};
use snora_core::contract::stack::Toast;

pub fn render_toast<'a, Message>(toasts: Vec<Toast<Message>>) -> Option<Element<'a, Message>>
where
    Message: 'a + Clone,
{
    if toasts.is_empty() {
        return None;
    }

    let mut toasts_col = column![].spacing(8);

    for toast in toasts {
        let toast_ui = container(column![
            text(toast.title).size(16),
            text(toast.message).size(14)
        ])
        .padding(12)
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        toasts_col = toasts_col.push(toast_ui);
    }

    let toasts_container = container(toasts_col)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(24)
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Bottom);

    Some(toasts_container.into())
}
