# Migration 0.57 → 0.58

> **Your own content can be a keyboard-reachable control.** `snora::control` takes iced
> `button`'s builder, so a `button` moves over by changing one name. Nothing is deprecated
> or removed, and nothing changes until you use it.

## Who is affected

| You use… | What changes | Do |
|---|---|---|
| iced's `button` around content of your own (a glyph and a label, a list row, a card) | You can make it keyboard-reachable | Rename it (§1). Optional |
| `snora-control` in your `Cargo.toml` | No longer released; 0.57.0 was its last version, a notice | Depend on `snora` instead |
| Anything else | Nothing | Nothing |

**For code:** nothing breaks, and nothing warns.

## 1. `snora::control` (RFC-122)

```rust,ignore
// Before: iced's button, pointer and touch only
iced::widget::button(row![glyph, text("Open")])
    .on_press(Message::Open)
    .padding([6, 12])
    .style(move |theme, status| my_style(theme, status))

// After: keyboard-reachable, the rest unchanged
snora::control(row![glyph, text("Open")])
    .on_press(Message::Open)
    .padding([6, 12])
    .style(move |theme, status| my_style(theme, status))
```

**On the design path,** use `snora::design::control(&tokens, content)`:
- the focus ring comes from `Tokens::focus`;
- the minimum size comes from `Tokens::targets.min`;
- the default look is `ghost`, and `.style(…)` replaces it.

**The builder** is iced `button`'s:
- `on_press`, `on_press_maybe` (where `None` is disabled and skipped by Tab) and
  `on_press_with`;
- `padding` (default `[5, 10]`, as `button`'s), `width`, `height` and `clip`;
- `style`, with iced's own `button::Status` and `button::Style`, so your closures, and
  `snora::design::style::button::*`, work unchanged.

**Built in, with nothing to set:** everything the [Keyboard map](keyboard.md) says of a
snora control.
- Tab reaches it, through `traverse`; Enter and Space press it, once.
- A click presses it without focusing it.
- It has a focus ring and the minimum target size.
- It is silent beneath an open overlay, and focus moves to a neighbour if it is removed.

**Where the ring is guaranteed:**

> The ring is at least 3:1 against the page and every neutral surface snora's tokens
> define; on the engine path, against the theme's background tiers except
> SolarizedDark's `strongest` (2.91:1). On a coloured fill of your own, check it.

**On the engine path, a dialog or sheet has no card of its own:** put its content on a
container with the theme's background. A control or text placed directly on the dim may
not be legible, and its ring may fall under 3:1.

On the design path, the minimum size follows the label size. Content with larger text of
its own should set its own padding.

## What did not change

- Every API, appearance, and keyboard rule.
- `design::button::*` is still the shorter way for a plain label.

## If you are jumping more than one minor

Read [0.56 → 0.57](migration-0.56-to-0.57.md) first: it has 0.57's one breaking change.
