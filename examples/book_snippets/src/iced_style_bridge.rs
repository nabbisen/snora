//! Anchored source for `docs/src/design/iced-style-bridge.md`.
//!
//! The focus-status-variants fence (`Active | Hovered | Pressed |
//! Disabled // no Focused`) is deliberately **not** anchored here
//! (review R-3, RFC-069 round 2): the original is a list, not Rust, and
//! reads better as one — converting it to an exhaustive `match` changed
//! what the documentation says, not merely how it is verified.

use snora::design::Tokens;

#[derive(Debug, Clone)]
enum Message {
    DoIt,
}

// ANCHOR: bridge_color_conversion
fn color_conversion(tokens: &Tokens) {
    use snora::design::style::color::to_iced_color;

    let ic = to_iced_color(tokens.palette.accent);
}
// ANCHOR_END: bridge_color_conversion

// ANCHOR: bridge_button_styles
fn button_styles<'a>(
    tokens: &Tokens,
    my_content: impl Into<iced::Element<'a, Message>>,
) -> iced::widget::Button<'a, Message> {
    use iced::widget::button as iced_button;
    use snora::design::style::button;

    let tok = tokens.clone();
    iced_button(my_content)
        .on_press(Message::DoIt)
        .style(move |_theme, status| button::primary(&tok, status))
}
// ANCHOR_END: bridge_button_styles

#[rustfmt::skip]
// ANCHOR: bridge_container_styles
fn container_styles<'a>(
    tokens: &Tokens,
    my_content: impl Into<iced::Element<'a, Message>>,
) -> iced::widget::Container<'a, Message> {
    use iced::widget::container as iced_container;
    use snora::design::style::container;

    let tok = tokens.clone();
    iced_container(my_content)
        .style(move |_theme| container::card_surface(&tok))
}
// ANCHOR_END: bridge_container_styles

#[rustfmt::skip]
// ANCHOR: bridge_typography_sizes
fn typography_sizes(tokens: Tokens) -> iced::widget::Text<'static> {
    use snora::design::style::text;

    iced::widget::text("Hello")
        .size(text::body_size(&tokens))
        .line_height(text::body_line_height(&tokens))
}
// ANCHOR_END: bridge_typography_sizes
