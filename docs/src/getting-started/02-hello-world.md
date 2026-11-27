# 2 — Hello, snora

The smallest possible snora application — a single body element,
nothing else.

```rust,ignore
{{#include ../../../examples/book_snippets/src/getting_started_hello.rs:hello_world}}
```

That is the entire program. There is no trait to implement, no wrapper
enum to write, no overlay scaffolding to plumb. `AppLayout::new(body)`
returns a layout with `body` filled in and every other slot empty;
`render` consumes it and produces an `iced::Element`.

## What snora did for you

Even at this size, snora has already:

- Wrapped your body in a full-window container.
- Established the z-stack layers (skeleton at the bottom, toasts at the
  top) so future overlays slot in without code changes.
- Picked a default `LayoutDirection` (LTR) and `ToastPosition` (TopEnd).

When you want a header, a sidebar, or a footer, you set them as fields
on `AppLayout` — see the [next page](03-add-a-header.md).

## Try it locally

The `examples/hello/` directory in the snora workspace is a slightly
fuller version of the program above: a centred title and a line of text,
in a window titled "snora — hello". Run it with:

```text
cargo run -p snora-example-hello
```
