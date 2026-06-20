# Buttons

Snora Design provides four semantic button variants via
`snora::design::button`. Each is a thin wrapper around `iced::widget::button`
with Snora Design token styling applied.

## Variants

| Function | Semantic use |
|---|---|
| `button::primary` | Strongest call to action; use once per surface |
| `button::secondary` | Secondary action alongside a primary button |
| `button::ghost` | Tertiary / low-emphasis action |
| `button::danger` | Irreversible actions — delete, revoke, reset |

## Basic usage

```rust,ignore
use snora::design::{Tokens, button};

// In your app state, store tokens:
struct App { tokens: Tokens }

// In view():
let t = &self.tokens;

button::primary(t, "Save", Message::Save)
button::secondary(t, "Cancel", Message::Cancel)
button::ghost(t, "Learn more", Message::LearnMore)
button::danger(t, "Delete", Message::Delete)
```

## Disabled state

Each variant has a `*_maybe` counterpart that accepts `Option<Message>`.
Passing `None` produces a disabled button (iced applies the `Disabled`
status automatically):

```rust,ignore
button::primary_maybe(t, "Save", self.can_save.then_some(Message::Save))
button::danger_maybe(t, "Delete", self.can_delete.then_some(Message::Delete))
```

## Using the raw style bridge

If you need finer control — custom padding, width, child element — use the
style bridge directly:

```rust,ignore
use snora::design::style;
use iced::widget::button;

let t = &self.tokens;
let tok = t.clone();           // clone into the closure

button(my_custom_content)
    .on_press(Message::Save)
    .style(move |_theme, status| style::button::primary(&tok, status))
```

## Accessibility notes

- All four variants use `iced::widget::button`, so they inherit iced's native
  keyboard activation (Enter / Space).
- **Custom focus rings are not rendered in iced 0.14.** `button::Status` has
  no `Focused` variant; the style closure cannot draw a ring. This is a
  documented iced 0.14 limitation, not a regression. When iced exposes focus
  state in a future version, the style bridge will be extended.
- The `danger` variant uses `palette.danger_text` on `palette.danger`; this
  pair is mandatory and tested by `cargo test -p snora-design`.
- Disabled buttons pass the WCAG 1.4.3 exception for low-contrast states.

See [Semantic accessibility](../contributing/semantic-accessibility.md) for
the full policy.
