# iced style bridge

The style bridge in `snora::design::style` converts Snora Design tokens
into iced widget style types. It is the only place in the design system
where `snora_design::Color` crosses into `iced::Color`.

## Color conversion

```rust,ignore
{{#include ../../../examples/book_snippets/src/iced_style_bridge.rs:bridge_color_conversion}}
```

This is a named function, not a `From` impl. The explicit call keeps the
iced boundary visible in code review.

## Button styles

```rust,ignore
{{#include ../../../examples/book_snippets/src/iced_style_bridge.rs:bridge_button_styles}}
```

Available functions: `primary`, `secondary`, `ghost`, `danger`.

All four map `iced::widget::button::Status` — `Active`, `Hovered`,
`Pressed`, `Disabled`. No `Focused` variant exists in iced 0.14 (see
[Focus limitation](#focus-state-limitation-iced-014) below).

## Container / card styles

```rust,ignore
{{#include ../../../examples/book_snippets/src/iced_style_bridge.rs:bridge_container_styles}}
```

Available functions: `card_surface`, `card_raised`, `card_selected`.

`iced::widget::container` takes `&Theme` only — no status parameter.

## Typography

```rust,ignore
{{#include ../../../examples/book_snippets/src/iced_style_bridge.rs:bridge_typography_sizes}}
```

Available, one pair per role — a size and a line-height helper:

| Role | Size | Line height |
|---|---|---|
| `body` | `body_size` | `body_line_height` |
| `body_small` | `body_small_size` | `body_small_line_height` |
| `label` | `label_size` | `label_line_height` |
| `title` | `title_size` | `title_line_height` |
| `heading` | `heading_size` | `heading_line_height` |
| `display` | `display_size` | `display_line_height` |

The size helpers return `iced::Pixels`; the line-height helpers return
`iced::widget::text::LineHeight::Relative` (RFC-068). **Applying a
line-height helper is not always an improvement over applying none** —
iced already renders at `Relative(1.3)` by default, so `title`'s helper
changes nothing and three roles are deliberately tighter than the
default. See [Typography](typography.md) for each role stated against
that baseline before choosing.

## Focus-state limitation (iced 0.14)

`iced::widget::button::Status` has exactly four variants:

```rust,ignore
Active | Hovered | Pressed | Disabled   // no Focused
```

`iced::widget::container` has **no interaction status at all**.

The style bridge maps every status iced exposes. It cannot render a custom
focus ring through `button::Style` or `container::Style` in iced 0.14.

**The accurate constraint is narrower than "iced 0.14 cannot render a focus
ring":** iced cannot tell a style closure that a widget *iced owns* is
focused. `FocusTokens` (`tokens.focus.*`) therefore has a present-day
audience, not only a future one — any application that already owns focus
as its own state (a focus-zone enum cycled by Tab, say) can read that state
in its own `container` style closure — an arbitrary `Fn(&iced::Theme) ->
Style` — and set border colour *and* width from it, exactly as it would for
any other conditional style. That is not a snora capability; it is what
`Fn` closures already let an application do, and `FocusTokens` supplies the
colour/width/offset vocabulary for it.

What remains genuinely blocked: a focus ring on a **standard button or
card that lets iced own focus** — snora's own prefab widgets do this, and
that is unchanged; snora will wire the ring in when iced exposes focus
state, rather than build an interim mechanism of its own. Native iced
focus handling (keyboard activation) works today regardless; only the
*visual* ring on iced-owned widgets is absent. See
[Semantic accessibility](../contributing/semantic-accessibility.md) for the
full statement.
