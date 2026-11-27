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
{{#include ../../../examples/book_snippets/src/design_pages.rs:cards_variants}}
```

## Arbitrary content

Cards accept any `Element`, so you can nest layout, text, and controls:

```rust,ignore
{{#include ../../../examples/book_snippets/src/design_pages.rs:cards_composition}}
```

## Managing selection state

Cards do not own selection state. Call the right variant based on your
application state:

```rust,ignore
{{#include ../../../examples/book_snippets/src/design_pages.rs:cards_selected}}
```

## Card padding

Padding is set from `tokens.spacing.md` automatically. If you need a
different padding (e.g. zero-padding for a full-bleed image), use the raw
style bridge with your own `iced::widget::container`:

```rust,ignore
{{#include ../../../examples/book_snippets/src/design_pages.rs:cards_override}}
```

## Accessibility notes

- Cards in v0.20 are **non-interactive visual grouping surfaces**. They do
  not emit events and must not appear to be interactive controls.
- If a card must be clickable (e.g. navigating to a detail view), wrap it in
  `snora::design::control`, as the [result card recipe](recipes/result-card.md)
  does: the whole card is then one target for the pointer and one Tab stop.
  **Not `iced::widget::button`:** it takes no keyboard input in iced 0.14. See
  [Semantic accessibility](../contributing/semantic-accessibility.md).
  *(Corrected 2026-11-28, from the 0.67 audit.)*
- The `selected` card communicates selection via an accent-colored border
  only; do not rely on color alone if your application must be accessible
  without color vision.
