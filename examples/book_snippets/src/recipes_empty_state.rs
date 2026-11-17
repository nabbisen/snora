//! Anchored source for `docs/src/design/recipes/` (RFC-136): the recipe's function, as the page shows it.

// ANCHOR: recipe_empty_state
use iced::Element;
use iced::widget::{column, text};
use snora::design::{Tokens, button, card, style};

fn empty_state<'a, Message: Clone + 'a>(
    tokens: &'a Tokens,
    title: &'a str,
    description: &'a str,
    cta_label: Option<&'a str>,
    on_cta: Option<Message>,
) -> Element<'a, Message> {
    let t = tokens;
    let text_col = style::color::to_iced_color(t.palette.text_primary);
    let muted = style::color::to_iced_color(t.palette.text_muted);

    let mut col = column![
        text(title)
            .size(style::text::heading_size(t))
            .color(text_col),
        text(description)
            .size(style::text::body_size(t))
            .color(muted),
    ]
    .spacing(t.spacing.md)
    .align_x(iced::Alignment::Center);

    if let (Some(label), Some(msg)) = (cta_label, on_cta) {
        col = col.push(button::primary(t, label, msg));
    }

    card::surface(t, col)
}
// ANCHOR_END: recipe_empty_state
