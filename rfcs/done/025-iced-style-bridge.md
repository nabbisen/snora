# RFC 025 — iced Style Bridge

**Status.** Implemented (v0.19.0)
**Tracks.** Snora Design System migration; v0.20 foundation.
**Touches.** `snora-widgets::design::style`, iced conversion helpers.

## Summary

This RFC defines the iced-facing style bridge from Snora Design tokens to iced widget styles.

## Goals

- Keep `snora-design` iced-free.
- Implement direct widget-local style functions first.
- Avoid mandatory custom iced theme.
- Preserve non-lossy interaction state information, and avoid implying focus
  state exists where iced does not expose it.
- Allow apps to mix Snora-styled and custom widgets.

## Non-goals

- No global token registry.
- No replacement for iced `Theme`.
- No full coverage of every iced widget in v0.20.
- No public interaction-state abstraction.

## Location

```text
snora-widgets/src/design/style/
  mod.rs
  color.rs
  button.rs
  container.rs
  text.rs
  progress.rs
```

Facade:

```rust
snora::widget::design::style
```

## Data flow

```text
snora_design::Tokens
  -> snora_widgets::design::style functions
  -> iced style types / closures
  -> iced rendering
```

## Color conversion

Use explicit conversion:

```rust
pub fn to_iced_color(color: snora_design::Color) -> iced::Color;
```

## Style function candidates

```rust
pub fn button_primary(tokens: &Tokens, status: button::Status) -> button::Style;
pub fn button_secondary(tokens: &Tokens, status: button::Status) -> button::Style;
pub fn button_ghost(tokens: &Tokens, status: button::Status) -> button::Style;
pub fn button_danger(tokens: &Tokens, status: button::Status) -> button::Style;
```

Card/container:

```rust
pub fn card_surface(tokens: &Tokens) -> container::Style;
pub fn card_raised(tokens: &Tokens) -> container::Style;
pub fn card_selected(tokens: &Tokens) -> container::Style;
```

Exact iced type names follow pinned iced version.

## Interaction state policy

Do not collapse iced status into a lossy enum.

Prefer iced-native status values. If focus comes separately, layer it separately.

Internal flags are allowed:

```rust
pub(crate) struct InteractionFlags {
    pub disabled: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
}
```

Do not expose this publicly in v0.20.

## iced 0.14 focus reality

This is a hard limitation of the pinned iced version, not a design choice:

- `iced::widget::button::Status` is `Active | Hovered | Pressed | Disabled`.
  It has **no focused state**.
- `iced::widget::container` has no interaction `Status` at all.

Therefore the style bridge **cannot** render a keyboard-focus ring on a
standard button or card through `button::Style` / `container::Style`. The
bridge maps the statuses iced does expose (hover/press/disabled); focus
styling is not deliverable here in v0.20.

Consequences:

- `FocusTokens` remain useful vocabulary and apply on any future iced path
  that exposes focus, but v0.20 button/card helpers do not promise a focus
  ring.
- Documentation (`semantic-accessibility.md`) and the visual-QA checklist
  must state this so a missing button focus ring is treated as a known,
  documented limitation, not a regression.
- If focus is ever surfaced through a separate iced mechanism, layer it
  separately rather than forcing it into the status conversion.

## Custom iced theme conversion

Optional and deferred unless trivial.

Direct widget-local style functions are the baseline.

## Data lifecycle

```text
app stores tokens
  -> view constructs widgets
  -> style closure receives iced status
  -> style bridge maps tokens + status
  -> iced renders
```

## Testing

- style functions compile;
- statuses return valid styles;
- high-contrast tokens work;
- widgets-only build excludes design;
- no iced types in `snora-design`.

## Acceptance criteria

- Style bridge module exists.
- Button/card style functions exist.
- No mandatory custom iced theme.
- No lossy public interaction enum.
