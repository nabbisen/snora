# 2 — Hello, snora

The smallest possible snora application — a single body element,
nothing else.

```rust
use iced::{Element, widget::text};
use snora::{AppLayout, render};

#[derive(Debug, Clone)]
enum Message {}

#[derive(Default)]
struct App;

impl App {
    fn update(&mut self, _msg: Message) {}

    fn view(&self) -> Element<'_, Message> {
        let body: Element<'_, Message> = text("Hello, snora!").into();
        render(AppLayout::new(body))
    }
}

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title(|_: &App| String::from("Hello"))
        .run()
}
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

The `examples/hello/` directory in the snora workspace is exactly the
program above. Run it with:

```text
cargo run -p snora-example-hello
```
