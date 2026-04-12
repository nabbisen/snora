use iced::{Element, Length, Padding, widget::container};

use crate::style::container_box_style;

pub fn app_footer<'a, Message: Clone + 'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    container(content)
        .width(Length::Fill)
        .padding(Padding::from([6.0, 16.0]))
        .style(container_box_style)
        .into()
}
