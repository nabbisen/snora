# Cards

Snora Design provides three card variants via `snora::design::card`. Each
wraps `iced::widget::container` with Snora Design token styling and accepts
any `Element` as content.

## Variants

| Function | Semantic use |
|---|---|
| `card::surface` | Default content grouping — form sections, summaries |
| `card::raised` | Elevated panel that visually floats above surrounding content |
| `card::selected` | Active card in a peer selection set (accent border) |

## Basic usage

```rust,ignore
use snora::design::{Tokens, card};
use iced::widget::text;

let t = &self.tokens;

card::surface(t, text("Standard card content"))
card::raised(t, text("Floating panel content"))
card::selected(t, text("Currently active item"))
```

## Arbitrary content

Cards accept any `Element`, so you can nest layout, text, and controls:

```rust,ignore
use iced::widget::{column, text};

card::surface(t, column![
    text("Title").size(style::text::title_size(t)),
    text("Body text goes here."),
    button::secondary(t, "Action", Message::DoSomething),
])
```

## Managing selection state

Cards do not own selection state. Call the right variant based on your
application state:

```rust,ignore
let card_fn = if self.selected_id == Some(item.id) {
    card::selected
} else {
    card::surface
};
card_fn(t, item_content)
```

## Card padding

Padding is set from `tokens.spacing.md` automatically. If you need a
different padding (e.g. zero-padding for a full-bleed image), use the raw
style bridge with your own `iced::widget::container`:

```rust,ignore
use snora::design::style;
use iced::widget::container;

let tok = t.clone();
container(my_content)
    .padding(0)  // override padding
    .style(move |_| style::container::card_surface(&tok))
```

## Accessibility notes

- Cards in v0.20 are **non-interactive visual grouping surfaces**. They do
  not emit events and must not appear to be interactive controls.
- If a card must be clickable (e.g. navigating to a detail view), wrap or
  replace the card content with an `iced::widget::button` styled as a card
  surface. See [Semantic accessibility](../contributing/semantic-accessibility.md).
- The `selected` card communicates selection via an accent-colored border
  only; do not rely on color alone if your application must be accessible
  without color vision.
