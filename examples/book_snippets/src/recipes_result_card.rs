//! Anchored source for `docs/src/design/recipes/` (RFC-136): the recipe's function, as the page shows it.

// ANCHOR: recipe_result_card
use iced::widget::{column, row, text};
use iced::{Element, Length};
use snora::design::{Tokens, card, chip, style};

fn result_card<'a, Message: Clone + 'a>(
    tokens: &'a Tokens,
    title: &'a str,
    subtitle: Option<&'a str>,
    tags: &'a [&'a str],
    selected: bool,
    on_select: Message,
) -> Element<'a, Message> {
    let t = tokens;
    let text_primary = style::color::to_iced_color(t.palette.text_primary);
    let text_secondary = style::color::to_iced_color(t.palette.text_secondary);

    let mut content = column![
        text(title)
            .size(style::text::label_size(t))
            .color(text_primary),
    ]
    .spacing(t.spacing.xs);

    if let Some(sub) = subtitle {
        content = content.push(
            text(sub)
                .size(style::text::body_size(t))
                .color(text_secondary),
        );
    }

    if !tags.is_empty() {
        let mut tag_row = row![].spacing(t.spacing.xs);
        for tag in tags {
            // Display-only tags: selected = true, on_toggle = None
            tag_row = tag_row.push(chip::filter(t, *tag, true, Option::<Message>::None));
        }
        content = content.push(tag_row.wrap());
    }

    // Wrap the entire card in a control: one target for the pointer and
    // a tab stop for the keyboard (RFC-122).
    snora::design::control(
        t,
        if selected {
            card::selected(t, content)
        } else {
            card::surface(t, content)
        },
    )
    .on_press(on_select)
    .padding(0)
    .style(|_theme, _status| iced::widget::button::Style {
        background: None,
        border: iced::Border::default(),
        shadow: iced::Shadow::default(),
        text_color: iced::Color::TRANSPARENT,
        snap: false,
    })
    .width(Length::Fill)
    .into()
}
// ANCHOR_END: recipe_result_card
