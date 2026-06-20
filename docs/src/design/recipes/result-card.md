# Recipe: Result card

**Promotion status:** Recipe — v0.23

---

## 1. Purpose

Display a single search result, list item, or selectable record as a card
with a title, optional subtitle, optional metadata chips, and an optional
primary action.

## 2. When to use

- Search results in a list.
- File/document browser rows.
- Inbox items, log entries, or activity feed items.
- Any selectable record where the card itself is the primary interactive
  surface.

## 3. When not to use

- When the item is selected and needs expanded detail — use `card::selected`
  for the selected state and show detail in a separate panel.
- When the card list is the primary navigation surface and the card
  represents a destination — wire the entire card as a button rather than
  placing a button inside it.
- When you have more than five metadata chips — the visual density will be
  unreadable; prefer a two-column layout or a table.

## 4. Data the app owns

- Title and subtitle strings (app-specific).
- Whether the item is selected.
- Metadata chip labels and their on/off state.
- The message emitted when the card (or primary action) is pressed.

## 5. Snora primitives used

- `snora::design::card::surface` — default (unselected) card.
- `snora::design::card::selected` — selected state.
- `snora::design::chip::filter` — metadata chips (read-only tags use
  `selected = true`, no `on_toggle`).
- `snora::design::button::primary` or `ghost` — optional explicit action.
- `snora::design::style::text::{label_size, body_size}`.

## 6. Accessibility notes

- If the entire card is interactive (pressing it selects/opens the item),
  wrap it in `iced::widget::button` rather than placing a button inside the
  card. This makes the full card keyboard reachable as a single tab stop.
- If the card is display-only with one action button, the button is
  `iced::widget::button` — keyboard reachable.
- Metadata chips with `on_toggle = None` are display-only and are not in
  the tab order.
- Selected state is communicated via `card::selected`'s border color — not
  color-only, because the border width also changes.

## 7. Code example

```rust,ignore
use snora::design::{Tokens, card, chip, style};
use iced::widget::{column, row, text};
use iced::{Element, Length};

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
        let mut tag_row = row![].spacing(t.spacing.xs).wrap();
        for tag in tags {
            // Display-only tags: selected = true, on_toggle = None
            tag_row = tag_row.push(
                chip::filter(t, *tag, true, Option::<Message>::None)
            );
        }
        content = content.push(tag_row);
    }

    // Wrap the entire card in a button so it is a single tab stop.
    iced::widget::button(
        if selected {
            card::selected(t, content)
        } else {
            card::surface(t, content)
        }
    )
    .on_press(on_select)
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
```

## 8. Customization points

- Remove the outer `button` wrapper if the card is display-only and
  interaction is driven by a separate action button inside.
- Use `card::raised` instead of `card::surface` for the selected state if
  you want elevation rather than a heavier border.
- Add a secondary `button::ghost` ("Open", "Archive") to the bottom of the
  content column for explicit actions.
- Use `Tone`-colored chips (see `chip::filter` with accent tint) to
  distinguish tag categories.

## 9. Promotion status

**Recipe.** The outer-button-wrapping-a-card pattern is app-specific enough
(different apps want different content shapes) that a stable helper would
be over-parameterized. The recipe documents the key insight (wrap the card
in `iced::widget::button` for keyboard reachability) which is not obvious
from the primitives alone.
