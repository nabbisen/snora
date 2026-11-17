//! Anchored source for `docs/src/design/recipes/` (RFC-136): the recipe's function, as the page shows it.

// ANCHOR: recipe_background_task
use iced::Element;
use iced::widget::{column, row, text};
use snora::design::{Tokens, Tone, button, card, progress};

fn background_task_card<'a, Message: Clone + 'a>(
    tokens: &'a Tokens,
    task_name: &'a str,
    value: Option<f32>,
    tone: Tone,
    on_cancel: Option<Message>,
    direction: snora::LayoutDirection,
) -> Element<'a, Message> {
    let t = tokens;

    let mut controls = row![].spacing(t.spacing.sm);
    if let Some(msg) = on_cancel {
        controls = controls.push(button::ghost(t, "Cancel", msg));
    }

    let inner = column![
        text(task_name)
            .size(snora::design::style::text::label_size(t))
            .color(snora::design::style::color::to_iced_color(
                t.palette.text_primary
            )),
        progress::row(t, task_name, value, tone, direction),
        controls,
    ]
    .spacing(t.spacing.sm);

    card::surface(t, inner)
}
// ANCHOR_END: recipe_background_task
