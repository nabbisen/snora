# Migrating from 0.4 to 0.5

Snora 0.5 is a small release whose only breaking change is the toast
anchor default. Most apps need a one-line edit (or no edit at all).

## At a glance

| Change | Severity | Action |
|---|---|---|
| Toast default position is now `TopEnd` (was `BottomEnd`) | Breaking, visual | Add `.toast_position(ToastPosition::BottomEnd)` if you want the old look |
| `AppLayout` gains `toast_position: ToastPosition` field | Compatible | None — `AppLayout::new(...)` still defaults sensibly |
| `lucide-icons` upgraded to `^1` | Compatible at the source level | Rebuild |
| `snora-core = "0.5"` | Same workspace inheritance | None if you only use `snora` (re-exports cover everything) |

## Toast position default

In 0.4, toasts always anchored to the bottom-right (LTR) / bottom-left
(RTL). In 0.5 the default is the top-end corner.

### Why we changed it

Many local-first applications place primary content in the bottom
half of the window — preview panels, editors, lists. Bottom-anchored
toasts compete with that content for visual space. Top-anchored
toasts sit clear of typical primary content and are easier to notice
without obscuring work.

The 0.4 default reflected the OS-level notification convention
(macOS / GNOME / Windows). That convention applies to *system*
notifications outside any application; in-app toasts in modern UI
frameworks (Material Snackbar, Chakra, Mantine) more often default to
the top-end. We followed the in-app convention.

### To keep the 0.4 behavior

```rust
use snora::{AppLayout, ToastPosition};

let layout = AppLayout::new(body)
    .toasts(self.toasts.clone())
    .toast_position(ToastPosition::BottomEnd);   // explicit 0.4 default
```

That single setter on every `view` call restores the previous
positioning exactly.

### To pick something else

`ToastPosition` has six variants:

```rust
pub enum ToastPosition {
    TopEnd,        // default (LTR=top-right, RTL=top-left)
    TopStart,
    TopCenter,
    BottomEnd,
    BottomStart,
    BottomCenter,
}
```

Stack growth direction is derived from the position automatically —
top anchors grow downward, bottom anchors grow upward, so the
*newest* toast is always closest to the screen edge.

## New: opt-in position picker UX

If your application has a settings panel and you want to let users
choose, the position is suitable for runtime configuration. Store the
choice on your state and re-pass it on every render:

```rust
struct App { /* ... */ toast_position: ToastPosition }

fn view(&self) -> iced::Element<'_, Message> {
    AppLayout::new(body)
        .toasts(self.toasts.clone())
        .toast_position(self.toast_position)
        .into()
}
```

Switching position re-anchors the entire stack on the next frame; no
per-toast change is needed.

## `lucide-icons` 1.x

If you depend on `snora` only, no change is needed. If you also
import `lucide_icons::*` directly in your code, double-check that any
constants you reference still exist — the lucide upstream renames
glyphs occasionally. The vast majority of names are stable.

## Crate versions

If your `Cargo.toml` had

```toml
snora = "0.4"
```

just bump it to

```toml
snora = "0.6"
```

If you depend on `snora-core` directly (rare), bump that too:

```toml
snora-core = "0.5"
```

There are no other breaking changes in 0.5. The `AppLayout` builder,
overlay close-sink convention, `Toast` lifetime API, and direction
vocabulary are all source-compatible with 0.4.
