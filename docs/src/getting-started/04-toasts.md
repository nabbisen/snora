# 4 — Toasts

A toast is a small, auto-stacking notification anchored to one corner
of the window. snora owns the rendering, the stacking, and the lifetime
sweep. The application stores `Vec<Toast<Message>>` and one
`snora::toast::Holds`, and writes three one-liners.

## Three pieces

```rust,ignore
use std::time::Instant;
use iced::{Subscription, Task};
use snora::{Toast, ToastAttention, ToastIntent, ToastLifetime};
use snora::toast::Holds;

struct App {
    toasts: Vec<Toast<Message>>,
    holds: Holds,                     // which toasts the reader is attending to
    next_id: u64,
}

#[derive(Debug, Clone)]
enum Message {
    ShowSaved,
    Dismiss(u64),
    ToastAttention(u64, ToastAttention), // pointer or keyboard came or went
    ToastTick,                        // framework asks us to sweep
}
```

### 1. Push a toast

```rust,ignore
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
    if let Message::ToastAttention(id, attention) = msg {
        // Held while the pointer rests on it or the keyboard is on its ×;
        // when both have gone, the countdown restarts in full. `Holds`
        // does the restart itself.
        self.holds.attend(id, attention, &mut self.toasts, Instant::now());
    }
    if let Message::ToastTick = msg {
        self.holds.sweep(&mut self.toasts, Instant::now());
    }
    Task::none()
}
```

### 2. Subscribe to TTL ticks

```rust,ignore
fn subscription(&self) -> Subscription<Message> {
    snora::toast::subscription(&self.toasts, || Message::ToastTick)
}
```

`subscription` returns `Subscription::none()` when the queue holds only
persistent toasts (or nothing at all), so the runtime does not wake on
an idle screen.

### 3. Pass the queue to the layout

```rust,ignore
fn view(&self) -> iced::Element<'_, Message> {
    let body: iced::Element<'_, Message> = /* … */;
    let layout = snora::AppLayout::new(body)
        .toasts(self.toasts.clone())
        .on_toast_attention(Message::ToastAttention);
    snora::render(layout)
}
```

## Lifetime policies

| API call | Behavior |
|---|---|
| `Toast::new(...)`, intent `Error` | **Stays until the user dismisses it** |
| `Toast::new(...)`, any other intent | Stays as long as it takes to read (below) |
| `.with_lifetime(ToastLifetime::seconds(10))` | Custom auto-dismiss |
| `.persistent()` | Stays until the user clicks the close button |

**How long "as long as it takes to read" is** (RFC-113), a rule you can
apply by hand: *a toast stays 2 seconds, plus 1 second for every 6
characters of its title and message, rounded up to a whole second, and
never less than 6 seconds.* "Saved" / "Your changes are stored." is 29
characters, so 7 s. It counts characters, not words, so Japanese and
Chinese text, which has no spaces, gets the same margin.
`ToastLifetime::reading_time(title, message)` is the rule as a function.

**A toast waits while the reader is attending to it, by pointer or by
keyboard** (0.56, RFC-120), once you wire `on_toast_attention` and `Holds` as
above. It does not expire while the pointer rests on it, or while the keyboard
is on its ×. When both have gone, its countdown starts again in full. Still,
use `.persistent()` for anything a reader must not miss.

**Keyboard focus and toasts** (RFC-120). Closing a toast from the keyboard
moves focus to the next toast's ×, or else the previous one's. With no toast
left, nothing is focused. A toast that expires or is closed elsewhere leaves
your focus where it was, because snora follows toasts by their id.

*Moving from 0.54 or 0.55:* `on_toast_hover`, `ToastHover` and `Holds::hover`
are deprecated since 0.56.0 and still work. Replace them with
`on_toast_attention`, `ToastAttention` and `Holds::attend`, as above.

Persistent is for messages the user must acknowledge — completed
exports, failures. Errors are persistent by default.

## Intent → color

`ToastIntent` is one of `Debug`, `Info`, `Success`, `Warning`, `Error`.
The engine resolves intents to theme-aware colors automatically.

## Position

Toasts anchor at `ToastPosition::TopEnd` by default (top-right under
LTR, top-left under RTL). Override with
`AppLayout::toast_position(ToastPosition::BottomCenter)` etc. The
position can be changed at runtime — re-rendering with a different
position re-anchors the entire stack on the next frame.

### Stack ordering

The **newest toast is always closest to the anchor edge**. The engine
handles this automatically; your application only needs to push new
toasts to the back of the queue in chronological order.

| Anchor | Column order (top → bottom) | Newest sits at |
|---|---|---|
| `TopStart` / `TopCenter` / `TopEnd` | newest → oldest | top edge |
| `BottomStart` / `BottomCenter` / `BottomEnd` | oldest → newest | bottom edge |

## Why not store TTL outside the toast?

`Toast` carries `created_at` and `lifetime` so that any single toast
is self-describing: `Toast::is_expired(now)` is a pure function on the
struct, easy to test without setting up renderer state. See the
[testing guide](../guides/testing.md).

## Next

You now know how to build a working snora app. The [next page](05-when-to-use.md)
helps you decide whether snora is the right fit for what you are
building.
