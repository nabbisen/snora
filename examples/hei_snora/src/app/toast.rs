use iced::{
    Alignment::Center,
    Background, Border, Color, Element, Length, Shadow,
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, row, text},
};
use snora_core::contract::stack::Toast;

/// Uniform toast width so right edges line up regardless of content length.
/// Toasts are anchored to the bottom-right; each entry is the same width, so
/// both the right and left edges of stacked toasts align cleanly.
const TOAST_WIDTH: f32 = 340.0;

pub fn render_toast<'a, Message>(toasts: Vec<Toast<Message>>) -> Option<Element<'a, Message>>
where
    Message: 'a + Clone,
{
    if toasts.is_empty() {
        return None;
    }

    let mut toasts_col = column![].spacing(8);

    for toast in toasts {
        let text_col = column![
            text(toast.title).size(16),
            text(toast.message).size(14),
        ]
        .spacing(4);

        // "×" close button. `toast.on_close` is always a valid `Message` —
        // the contract requires it — so we unconditionally wire a click
        // target. This is what makes persistent (no-auto-dismiss) toasts
        // usable: the user can always dismiss them manually.
        let close_btn = button(text("×").size(18))
            .on_press(toast.on_close)
            .padding([0, 8])
            .style(|_theme, status| {
                let text_color = match status {
                    button::Status::Hovered => Color::from_rgb(1.0, 1.0, 1.0),
                    _ => Color::from_rgb(0.75, 0.75, 0.75),
                };
                button::Style {
                    background: None,
                    text_color,
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: true,
                }
            });

        // Body row: title/message block fills; close button pinned to the end.
        let body_row = row![
            container(text_col).width(Length::Fill),
            close_btn,
        ]
        .align_y(Center)
        .spacing(4);

        let toast_ui = container(body_row)
            .width(Length::Fixed(TOAST_WIDTH))
            .padding(12)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                text_color: Some(Color::from_rgb(0.95, 0.95, 0.95)),
                border: Border {
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
        .align_x(Horizontal::Right)
        .align_y(Vertical::Bottom);

    Some(toasts_container.into())
}
