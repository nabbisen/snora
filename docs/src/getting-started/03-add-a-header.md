# 3 — Add a header, sidebar, footer

`AppLayout` has four skeleton slots: `header`, `side_bar`, `body`,
`footer`. Each accepts any `iced::Element` — you can hand-roll your own
or use the prefab helpers in `snora::widget`.

## Adding slots one at a time

```rust,ignore
{{#include ../../../examples/book_snippets/src/getting_started_header.rs:header_view}}
```

## A note on direction

Every prefab widget that has a left/right asymmetry takes a
`LayoutDirection` argument. Passing the same direction everywhere is
the typical pattern; in apps that support live LTR/RTL flipping you
keep the active direction on your state and re-pass it on each `view`.

`AppLayout::direction(...)` separately controls the *body row* mirroring
(sidebar side flips). See [Direction and ABDD](../guides/direction.md)
for the full picture.

## Custom slots

You are not required to use the prefab widgets. Anything that yields an
`Element<'_, Message>` slots in:

```rust,ignore
{{#include ../../../examples/book_snippets/src/getting_started_header.rs:header_custom}}
```

snora draws the skeleton; what fills each slot is your decision.

## Next

Toasts have framework-managed lifetime. See
[Toasts](04-toasts.md).
