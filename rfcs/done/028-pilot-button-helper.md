# RFC 028 — v0.20 Pilot Button Helper

**Status.** Implemented (v0.19.0)
**Tracks.** Snora Design System migration; v0.20 foundation.
**Touches.** `snora-widgets::design::button`, button styles, examples.

## Summary

This RFC defines the v0.20 pilot button helper/wrapper.

It is intentionally small and validates ergonomics alongside the style bridge.

## Motivation

If v0.20 ships only raw style functions, early adopters may write boilerplate that v0.21 primitives replace. A pilot button helper avoids that churn.

## Goals

- Provide basic ergonomic button helper.
- Use native iced button semantics.
- Support primary/secondary/ghost/danger variants.
- Support tokens and high contrast.
- Keep app behavior app-owned.

## Non-goals

- No loading button state.
- No icon-only button.
- No split/menu button.
- No command framework.
- No form submit semantics.

## Public API candidates

Simple function style:

```rust
pub fn primary<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    on_press: Message,
) -> Element<'a, Message>;
```

Token lifetime strategy (v0.20): the returned `Element<'a, _>` carries the
iced `.style(...)` closure, which must live for `'a`. The `&Tokens` argument
is **not** bound to `'a`; instead the helper clones tokens into the closure:

```rust
pub fn primary<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    on_press: Message,
) -> Element<'a, Message> {
    let tokens = tokens.clone();
    iced::widget::button(text(label.into()))
        .on_press(on_press)
        .style(move |_theme, status| style::button::primary(&tokens, status))
        .into()
}
```

`Tokens` is `Clone` and small; this clones once per `view()`, which is
acceptable. If it ever becomes hot, switch to `Arc<Tokens>` or a borrowed
`*_with(&'a Tokens, ...)` variant. Do not bind `&Tokens` to `'a` in the
simple helper — the clone keeps callers free of lifetime juggling.

Similar:

```rust
secondary
ghost
danger
```

Builder style can be added later if needed.

## Internal data model

```rust
pub enum ButtonKind {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

pub struct ButtonSpec<'a, Message> {
    pub label: String,
    pub kind: ButtonKind,
    pub size: Size,
    pub on_press: Option<Message>,
    pub tokens: &'a Tokens,
}
```

This may remain internal.

## Data lifecycle

```text
app creates tokens
  -> app calls helper in view
  -> helper creates iced button
  -> style bridge maps status to style
  -> iced emits app message
```

## Events

No Snora event is introduced.

Button emits the app-provided `Message`.

Disabled state may be represented by `Option<Message>` or iced equivalent.

## Internal design

Helper must be built on iced button.

```text
button helper
  -> iced button
  -> .style(|theme, status| style::button_primary(tokens, status))
  -> Element
```

Exact closure follows pinned iced version.

## Accessibility

- Uses iced button.
- Focus visible where possible. In iced 0.14, `button::Status` has no focused
  state (RFC-025), so the helper does not render a custom focus ring; this is
  a documented limitation, not a regression.
- Disabled style readable.
- No icon-only API in v0.20.
- The `danger` variant uses `palette.danger` with `palette.danger_text` as
  its foreground; `danger_text on danger` must pass the RFC-023 contrast
  test for every preset before the danger button ships.

## Visual fit

Workbench must inspect:

- vertical centering;
- line-height clipping;
- focus affordance / documented absence of custom focus ring (iced 0.14);
- disabled state readability;
- high-contrast border clarity.

## Acceptance criteria

- Primary button helper exists.
- Other style variants exist or are documented if deferred.
- Helper uses iced button.
- Minimal path unaffected.
- Workbench includes button states.
