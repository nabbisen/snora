# iced style bridge

The style bridge in `snora::design::style` converts Snora Design tokens
into iced widget style types. It is the only place in the design system
where `snora_design::Color` crosses into `iced::Color`.

## Color conversion

```rust,ignore
use snora::design::style::color::to_iced_color;

let ic = to_iced_color(tokens.palette.accent);
```

This is a named function, not a `From` impl. The explicit call keeps the
iced boundary visible in code review.

## Button styles

```rust,ignore
use snora::design::style::button;
use iced::widget::button as iced_button;

let tok = tokens.clone();
iced_button(my_content)
    .on_press(Message::DoIt)
    .style(move |_theme, status| button::primary(&tok, status))
```

Available functions: `primary`, `secondary`, `ghost`, `danger`.

All four map `iced::widget::button::Status` — `Active`, `Hovered`,
`Pressed`, `Disabled`. No `Focused` variant exists in iced 0.14 (see
[Focus limitation](#focus-state-limitation-iced-014) below).

## Container / card styles

```rust,ignore
use snora::design::style::container;
use iced::widget::container as iced_container;

let tok = tokens.clone();
iced_container(my_content)
    .style(move |_theme| container::card_surface(&tok))
```

Available functions: `card_surface`, `card_raised`, `card_selected`.

`iced::widget::container` takes `&Theme` only — no status parameter.

## Typography sizes

```rust,ignore
use snora::design::style::text;

iced::widget::text("Hello")
    .size(text::body_size(&tokens))
```

Available: `body_size`, `body_small_size`, `label_size`, `title_size`,
`heading_size`, `display_size`.

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
