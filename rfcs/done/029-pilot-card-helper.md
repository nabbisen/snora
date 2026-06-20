# RFC 029 — v0.20 Pilot Card Helper

**Status.** Implemented (v0.19.0)
**Tracks.** Snora Design System migration; v0.20 foundation.
**Touches.** `snora-widgets::design::card`, card styles, examples.

## Summary

This RFC defines the v0.20 pilot basic card helper/wrapper.

The card is a shallow visual surface. It does not own app behavior.

## Motivation

Cards are foundational surfaces for local-first productivity apps. They support setup panels, result recipes, recent items, notices, and status summaries. A pilot card validates spacing/radius/border tokens early.

## Goals

- Provide basic card helper.
- Support surface/raised/selected styling.
- Host arbitrary app content.
- Keep behavior app-owned.
- Support high contrast.

## Non-goals

- No result card component.
- No recent search component.
- No setup wizard component.
- No selection manager.
- No navigation semantics.

## Public API candidates

```rust
pub fn surface<'a, Message>(
    tokens: &Tokens,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message>;
```

Variants:

```rust
raised
selected
```

Builder can be deferred.

## Internal data model

```rust
pub enum CardKind {
    Surface,
    Raised,
    Selected,
    Interactive,
}

pub struct CardSpec<'a, Message> {
    pub kind: CardKind,
    pub tokens: &'a Tokens,
    pub content: Element<'a, Message>,
}
```

May remain internal.

## Data lifecycle

```text
app builds content
  -> app wraps content in card helper
  -> helper applies padding/radius/border/background
  -> card returns Element
  -> app owns any interaction inside or around content
```

## Events

Non-interactive cards emit no events.

Interactive cards are not required in v0.20.

## Internal design

Use iced container for non-interactive cards.

```text
card helper
  -> iced container
  -> style::card_surface(tokens)
  -> Element
```

## Accessibility

Non-interactive cards are visual grouping.

They must not look like controls unless they are actually interactive.

Interactive card support must wait for semantic review or be clearly limited.

## Visual fit

Workbench must inspect:

- title/body readability;
- high-contrast border clarity;
- padding;
- selected state visibility;
- line-height fit.

## Acceptance criteria

- basic card helper exists;
- card uses tokens;
- app supplies arbitrary content;
- no domain behavior;
- workbench shows card variants.
