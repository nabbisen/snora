# Snora

[![snora](https://img.shields.io/crates/v/snora?label=snora)](https://crates.io/crates/snora)
[![snora Docs](https://docs.rs/snora/badge.svg?version=latest)](https://docs.rs/snora)
[![snora Deps Status](https://deps.rs/crate/snora/latest/status.svg)](https://deps.rs/crate/snora)    
[![snora-core](https://img.shields.io/crates/v/snora-core?label=snora-core)](https://crates.io/crates/snora-core)
[![snora-core Docs](https://docs.rs/snora-core/badge.svg?version=latest)](https://docs.rs/snora-core)
[![snora-core Deps Status](https://deps.rs/crate/snora-core/latest/status.svg)](https://deps.rs/crate/snora-core)    
[![License](https://img.shields.io/github/license/nabbisen/snora)](https://github.com/nabbisen/snora/blob/main/LICENSE)

**An iced GUI framework that gets out of the way of your application.**

## Overview

Snora gives an iced application a small, opinionated **skeleton**
(header, sidebar, body, footer) plus the overlay surfaces it almost
certainly needs (dialog, bottom sheet, context menu, toasts) — and
then steps back. Every slot accepts any `iced::Element`, so your UI
code stays your UI code.

## When to use it

snora is a good fit when you are building:

- a **local-first desktop app** that runs heavy work alongside an
  interactive UI (AI inference, dataset converters, file processors);
- an app that needs **accessibility correct from day one** — RTL
  layout, theme-aware colors, logical edges baked into the API;
- a **standard desktop chrome** (header / sidebar / body / footer)
  with a few overlays.

snora is *not* the right tool for games, real-time visualization,
or web-first applications. See
[docs/getting-started/05-when-to-use.md](docs/getting-started/05-when-to-use.md)
for fuller fit guidance.

## Quick start

```toml
[dependencies]
iced  = { version = "0.14", features = ["tokio"] }
snora = "0.5"
```

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

That's the whole program. Adding a header, sidebar, and footer is
three more chained calls — see
[docs/getting-started](docs/getting-started/).

## Features

- **Skeleton + injected slots.** `AppLayout::new(body).header(h).side_bar(s).footer(f)`.
  Each slot is any `iced::Element`. No trait to implement, no
  dispatcher enum to write.
- **Framework-managed toasts.** `Vec<Toast<Message>>` on your state,
  two one-liners (`subscription` + `sweep_expired`) for lifetime,
  intent → theme color, six anchor positions including RTL-aware ones.
- **One close sink per channel.** `on_close_modals` for dialogs and
  sheets, `on_close_menus` for header / context menus. Wired once.
- **Vocabulary instead of magic numbers.** `SheetHeight::Half`,
  `ToastPosition::TopEnd`, `LayoutDirection::Rtl`, `Edge::Start`
  — explicit choices, not hardcoded constants.
- **Two crates, one direction.** `snora-core` is the iced-free
  vocabulary, `snora` is the engine — an iced upgrade only touches
  one crate.

## Design notes

- *Accessible by Default and by Design.* Layout is described in
  logical edges (`Edge::Start` / `Edge::End`); LTR ↔ RTL is one
  setter on `AppLayout`.
- *No silent drops.* If you populate an overlay but leave its close
  sink `None`, the overlay still renders — snora omits only the
  click-outside backdrop, never the content.
- *Skeleton, not styling.* Snora positions and stacks. The look of
  your dialog card, the typography of your header — those are your
  decisions, not ours.

## Read more

For the full picture, head to **[docs/](docs/)**:

- New to snora? Start with [Getting started](docs/getting-started/).
- Looking up something specific? [Reference](docs/reference/) and
  [Guides](docs/guides/).
- Wanting to contribute? [docs/contributing/](docs/contributing/) is
  for you — and we'd be glad to have you.

