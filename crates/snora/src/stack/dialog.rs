use iced::{
    Background, Color, Element, Length,
    widget::{center, container, mouse_area, opaque, space, stack},
};
use snora_core::contract::stack::Dialog;

pub fn render_dialog<'a, Message>(
    dialog: Option<Dialog<Element<'a, Message>, Message>>,
) -> Option<Element<'a, Message>>
where
    Message: 'a + Clone,
{
    let dialog = if let Some(x) = dialog {
        x
    } else {
        return None;
    };

    let backdrop = container(space())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
            ..Default::default()
        });

    let backdrop_interactive = if let Some(on_close) = dialog.on_outside_click {
        mouse_area(opaque(backdrop)).on_press(on_close)
    } else {
        mouse_area(opaque(backdrop))
    };

    let dialog_content = center(dialog.content);

    let dialog_stack = stack![backdrop_interactive, dialog_content];

    Some(dialog_stack.into())
}
