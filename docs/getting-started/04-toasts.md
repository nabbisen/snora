# 4 — Toasts

A toast is a small, auto-stacking notification anchored to one corner
of the window. snora owns the rendering, the stacking, and the lifetime
sweep; the application only stores `Vec<Toast<Message>>` and writes two
one-liners.

## Three pieces

```rust
use std::time::Instant;
use iced::{Subscription, Task};
use snora::{Toast, ToastIntent, ToastLifetime};

struct App {
    toasts: Vec<Toast<Message>>,
    next_id: u64,
}

#[derive(Debug, Clone)]
enum Message {
    ShowSaved,
    Dismiss(u64),
    ToastTick,                        // framework asks us to sweep
}
```

### 1. Push a toast

```rust
fn update(&mut self, msg: Message) -> Task<Message> {
    if let Message::ShowSaved = msg {
        let id = self.next_id;
        self.next_id += 1;
        self.toasts.push(Toast::new(
            id,
            ToastIntent::Success,
            "Saved",
            "Document written to disk.",
            Message::Dismiss(id),
        ));
    }
    if let Message::Dismiss(id) = msg {
        self.toasts.retain(|t| t.id != id);
    }
    if let Message::ToastTick = msg {
        snora::toast::sweep_expired(&mut self.toasts, Instant::now());
    }
    Task::none()
}
```

### 2. Subscribe to TTL ticks

```rust
fn subscription(&self) -> Subscription<Message> {
    snora::toast::subscription(&self.toasts, || Message::ToastTick)
}
```

`subscription` returns `Subscription::none()` when the queue holds only
persistent toasts (or nothing at all), so the runtime does not wake on
an idle screen.

### 3. Pass the queue to the layout

```rust
fn view(&self) -> iced::Element<'_, Message> {
    let body: iced::Element<'_, Message> = /* … */;
    let layout = snora::AppLayout::new(body)
        .toasts(self.toasts.clone());
    snora::render(layout)
}
```

## Lifetime policies

| API call | Behavior |
|---|---|
| `Toast::new(...)` | Default 4-second auto-dismiss |
| `.with_lifetime(ToastLifetime::seconds(10))` | Custom auto-dismiss |
| `.persistent()` | Stays until the user clicks the close button |

Persistent is for messages the user must acknowledge — completed
exports, fatal errors. Transient is for everything else.

## Intent → color

`ToastIntent` is one of `Debug`, `Info`, `Success`, `Warning`, `Error`.
The engine resolves intents to theme-aware colors automatically.

## Position

Toasts anchor at `ToastPosition::TopEnd` by default (top-right under
LTR, top-left under RTL). Override with
`AppLayout::toast_position(ToastPosition::BottomCenter)` etc. The
position can be changed at runtime — re-rendering with a different
position re-anchors the entire stack on the next frame.

## Why not store TTL outside the toast?

`Toast` carries `created_at` and `lifetime` so that any single toast
is self-describing: `Toast::is_expired(now)` is a pure function on the
struct, easy to test without setting up renderer state. See the
[testing guide](../guides/testing.md).

## Next

You now know how to build a working snora app. The [next page](05-when-to-use.md)
helps you decide whether snora is the right fit for what you are
building.
