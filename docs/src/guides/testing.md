# Testing UI logic without a renderer

snora does not ship a separate test-helper crate. Instead, snora's
public types expose enough fields directly that you can verify
state-driven UI logic with plain `assert!` against your `App` state.

## What you can test today

- "Did the right toast get pushed?" — assert against `state.toasts`.
- "Is the toast persistent?" — match on `toast.lifetime`.
- "Is a dialog open?" — check `state.show_dialog` or whatever flag
  drives `AppLayout::dialog`.
- "Did the active view switch?" — assert `state.active == ViewId::X`.

What you *cannot* test with this approach is the rendered pixel
output — that is iced's responsibility and would need a windowing
backend. snora deliberately stops at the data shape.

## Pattern: split state from view

Keep your `update` function pure (mutates state, returns `Task`) and
have `view` be the only function that touches iced widgets. Tests
exercise `update`; the renderer is never invoked.

```rust
// src/app.rs

#[derive(Default)]
pub struct App {
    pub toasts: Vec<snora::Toast<Message>>,
    pub next_id: u64,
    pub active: ViewId,
}

impl App {
    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::ExportCompleted(Ok(_)) => {
                let id = self.issue_id();
                self.toasts.push(
                    snora::Toast::new(
                        id,
                        snora::ToastIntent::Success,
                        "Export complete",
                        "File written to disk.",
                        Message::DismissToast(id),
                    )
                    .persistent(),
                );
            }
            // ...
        }
        iced::Task::none()
    }

    pub fn view(&self) -> iced::Element<'_, Message> { /* … */ }
}
```

## Pattern: assert against the queue

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_completion_pushes_persistent_success_toast() {
        let mut app = App::default();

        app.update(Message::ExportCompleted(Ok(fake_report())));

        let last = app.toasts.last().expect("a toast was queued");
        assert_eq!(last.intent, snora::ToastIntent::Success);
        assert!(matches!(last.lifetime, snora::ToastLifetime::Persistent));
    }

    #[test]
    fn cancel_clears_active_dialog_flag() {
        let mut app = App {
            show_export_dialog: true,
            ..Default::default()
        };

        app.update(Message::CancelExport);

        assert!(!app.show_export_dialog);
    }

    #[test]
    fn ttl_sweep_drops_only_expired_transient() {
        use std::time::{Duration, Instant};
        use snora::ToastLifetime;

        let now = Instant::now();
        let mut app = App::default();
        app.toasts.push(
            snora::Toast::new(1, snora::ToastIntent::Info, "old", "", Message::DismissToast(1))
                .with_lifetime(ToastLifetime::millis(100))
                .with_created_at(now),
        );
        app.toasts.push(
            snora::Toast::new(2, snora::ToastIntent::Error, "keep", "", Message::DismissToast(2))
                .persistent()
                .with_created_at(now),
        );

        snora::toast::sweep_expired(&mut app.toasts, now + Duration::from_secs(1));

        let ids: Vec<u64> = app.toasts.iter().map(|t| t.id).collect();
        assert_eq!(ids, vec![2]);
    }
}
```

Three things to notice:

1. `Toast`'s fields are `pub`, so the assertion reads naturally.
2. `Toast::with_created_at` is a public builder method intended for
   tests — it lets you control the timestamp without freezing the
   real clock.
3. `snora::toast::sweep_expired` is a public function. Calling it from
   a test is identical to how production code calls it — the same
   logic gets exercised.

## What is not currently testable this way

- **Click coordinates** for context menus. snora does not surface mouse
  events; you would need to test through iced's own `mouse_area` /
  subscription primitives.
- **Layout measurements.** Whether two columns fit, whether a sheet
  reaches the top of the screen, etc. These are renderer-side concerns.

## What Snora tests internally

Snora uses [`iced_test`](https://crates.io/crates/iced_test) — a
CPU-only headless renderer — to verify the engine's own behavioral
contract. These tests live in `crates/snora/tests/render_semantics.rs`
and cover:

- skeleton body is reachable (layer 0 renders and handles clicks);
- outside-click on a modal emits `on_close_modals` (layer 4 backdrop);
- dialog interactive content is reachable above the dim (layer 5);
- missing `on_close_modals` sink omits the backdrop but still renders
  content (Law 5);
- toast dismiss button fires above a modal (layer 7 above layers 4–6);
- sheet content is reachable via its `opaque` wrapper (layer 6);
- toast ordering policy (unit tests in `crates/snora/src/toast.rs`).

`iced_test` is a `[dev-dependencies]` entry only — it does not affect
the public API, feature flags, or binary size.

**Applications should not depend on these internals.** The contract you
can rely on is the public API: `AppLayout`, `render`, `Dialog`, `Sheet`,
and `Toast` behave as documented. Snora does not ship a public
`snora-test` crate; the current "pub fields + pure update" approach
covers the common application-testing cases, as shown in this guide.

## A note on `snora-test`

We considered shipping a dedicated test-helper crate. The conclusion
was that doing so would freeze internal data shapes into the public API
and create a second surface to maintain. The current approach — `pub`
fields on vocabulary types, pure `update` functions, and internal
render-semantics tests using `iced_test` — covers both application and
framework needs without that cost.
