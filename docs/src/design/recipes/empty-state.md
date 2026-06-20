# Recipe: Empty state

**Promotion status:** Recipe — v0.23

---

## 1. Purpose

Display a placeholder when a list, search result, or collection is empty,
with an optional call-to-action button.

## 2. When to use

- A list or table has no rows to show (first launch, filtered to zero).
- A search returned no results.
- A folder, project, or inbox is empty and the user should be guided toward
  creating content.

## 3. When not to use

- When the emptiness is a loading state — use the
  [Background task card](background-task.md) instead.
- When the emptiness represents an error — use the
  [Friendly error recovery notice](error-recovery.md) instead.
- When the content is hidden by a filter the user can remove — show the
  filter state so the user understands why the list is empty.

## 4. Data the app owns

- The empty-state label and description text (app-specific).
- Whether to show the call-to-action button.
- The message emitted when the button is pressed.
- Any icon or illustration (outside Snora Design's scope).

## 5. Snora primitives used

- `snora::design::card::surface` — outer container.
- `snora::design::button::primary` (optional CTA).
- `snora::design::style::text::{heading_size, body_size}` — text sizing.

## 6. Accessibility notes

- The CTA button is `iced::widget::button` — keyboard reachable.
- Descriptive text is plain `iced::widget::text` — readable by screen
  readers (to the extent iced 0.14 supports it).
- No interactive controls besides the optional CTA — no focus-trapping
  concern.

## 7. Code example

```rust,ignore
use snora::design::{Tokens, card, button, style};
use iced::widget::{column, text};
use iced::Element;

fn empty_state<'a, Message: Clone + 'a>(
    tokens: &'a Tokens,
    title: &'a str,
    description: &'a str,
    cta_label: Option<&'a str>,
    on_cta: Option<Message>,
) -> Element<'a, Message> {
    let t = tokens;
    let text_col = style::color::to_iced_color(t.palette.text_primary);
    let muted    = style::color::to_iced_color(t.palette.text_muted);

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
        col = col.push(button::primary(t, label, Some(msg)));
    }

    card::surface(t, col)
}
```

## 8. Customization points

- Replace `card::surface` with `card::raised` for more visual prominence.
- Adjust `iced::Alignment::Center` to `Start` if the empty state sits
  inside a list panel with left-aligned content.
- Add an SVG illustration above the title using `iced::widget::svg` — Snora
  Design does not provide illustrations.

## 9. Promotion status

**Recipe.** The pattern is simple enough to remain a recipe indefinitely.
Promotion would only be warranted if a `Tone`-aware variant (e.g. a
warning-toned empty state for quota limits) emerged from dogfood evidence.
