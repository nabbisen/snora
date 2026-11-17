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

- If the entire card is interactive (pressing it selects or opens the item),
  wrap it in **`snora::design::control`** rather than placing a button inside
  the card. The full card is then one target for the pointer **and** a tab
  stop: Enter and Space press it, and its ring sits outside the card, on the
  page (0.58, RFC-122). *(Until 0.58 this recipe used iced's `button`, which
  takes no keyboard input in iced 0.14.)*
- If the card is display-only with one action button, use
  `button::primary`, `ghost` or `link`: snora controls, reachable from the
  keyboard from 0.55 once the application routes Tab with
  `snora::keyboard::traverse`.
- Metadata chips with `on_toggle = None` are display-only and are not in
  the tab order.
- Selected state is communicated via `card::selected`'s border color — not
  color-only, because the border width also changes.

## 7. Code example

```rust,ignore
{{#include ../../../../examples/book_snippets/src/recipes_result_card.rs:recipe_result_card}}
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
in `snora::design::control`, so the whole card is one target for the pointer and the keyboard) which is not obvious
from the primitives alone.
