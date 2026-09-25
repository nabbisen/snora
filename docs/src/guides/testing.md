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

```rust,ignore
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

```rust,ignore
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

## Pattern: hash captures to check a snora upgrade changed nothing

Contributed by the **arama** team, who used it to produce the only
pixel-level confirmation snora's no-visual-change guarantee has.

Everything above tests *logic*. This tests *rendered output*, and it is the
one technique that closes the gap `render_semantics` leaves — that suite
asserts composition, not pixels, and nothing in snora's CI compares images.

The method is ordinary and the discipline is the point:

1. **Split the upgrade into two commits** — the version bump alone, then any
   adoption of new entry points such as `snora::design::render`. This is what
   makes the first step verifiable in isolation; a single combined commit
   cannot distinguish "snora changed nothing" from "our own change offset it".
2. Capture the same screen, preset and content at both commits. Window-scoped,
   fixed size, a scratch profile with deterministic fixtures rather than real
   user data.
3. **Compare hashes, not eyes.**

```text
md5  daae7534fc2a219d58e145339a9ea236   before-01-high_contrast_dark.png
md5  daae7534fc2a219d58e145339a9ea236   commit1-01-high_contrast_dark.png
```

Byte-identical output across four minor versions, with the render call
unchanged, on a real application.

**Why hashing beats looking.** A visual comparison yields "we could not see a
difference", which is a judgement. A hash yields a fact, and it catches
sub-perceptual drift a reviewer would pass. It also costs nothing to re-run.

**What it does not establish.** One application, one preset, one dialog is not
a general proof, and a hash tells you *that* something changed, never *what*.
When it differs you still need the images. Determinism also depends on your
own fixtures: fonts, scaling, animation, timestamps and any real content will
each break it before snora does.

If you adopt it, snora would like to hear the result either way — a hash that
*differs* across a snora upgrade with your own code unchanged is a bug report
we would want.

## What Snora tests internally

Snora uses [`iced_test`](https://crates.io/crates/iced_test) — a
headless renderer — to verify the engine's own behavioral contract. The
engine's tests live in `crates/snora/tests/render_semantics.rs`, and
the sidebar's rendered geometry is measured in
`crates/snora/tests/side_bar_fit.rs` (RFC-099). `render_semantics.rs`
covers:

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

### `iced_test` is not CPU-only by default — and parallel tests can crash

This page used to describe `iced_test` as *"a CPU-only headless
renderer."* **That was wrong.** Each simulator builds a renderer that
tries **wgpu first**, and wgpu's constructor calls into the system Vulkan
loader. With several simulators starting at once — which is what a
parallel `cargo test` does — that was observed to crash the whole test
binary with **SIGSEGV** inside the loader (RFC-099). Cargo reports that
crash with the same exit code as a failing assertion, so read the output
for `signal: 11` rather than trusting the exit code.

**snora's own suite renders with tiny-skia**, set in the workspace's
`.cargo/config.toml`:

```toml
[env]
ICED_TEST_BACKEND = "tiny-skia"
```

`iced_test` reads that variable and passes it on as a backend hint, and
wgpu steps aside before it ever touches the Vulkan loader. Only
`iced_test` reads it, so `cargo run` of an application is unaffected.

**If you write `iced_test` tests of your own, you likely want the same
line.** Two caveats worth knowing before you rely on it:

- **A value already set in your shell wins.** Cargo's `[env]` does not
  override an exported variable unless the key uses `force = true`. If
  you export `ICED_TEST_BACKEND` in a shell profile, you have silently
  replaced this protection. Leaving it unforced is deliberate here: it is
  how to test under wgpu on purpose, e.g. `ICED_TEST_BACKEND=wgpu cargo
  test`.
- **It depends on `iced_test` 0.14.** If a future iced renames the
  variable or changes the renderer fallback, the line stops working
  without an error, and the crash can return. snora re-checks it at its
  next iced major.

**Applications should not depend on these internals.** The contract you
can rely on is the public API: `AppLayout`, `render`, `Dialog`, `Sheet`,
and `Toast` behave as documented. Snora does not ship a public
`snora-test` crate; the current "pub fields + pure update" approach
covers the common application-testing cases, as shown in this guide.


### What rendered evidence from this harness does and does not show

Because snora's suite pins `ICED_TEST_BACKEND = "tiny-skia"`, **every pixel a
snora test reads comes from tiny-skia**, iced's software renderer, and not from
wgpu, which is what an application with a working GPU uses. **The two renderers
are proven to draw some primitives differently.** A shadow behind a transparent
quad is the known case: wgpu draws it only *outside* the quad's shape, and
tiny-skia fills its whole shape. At 0.50.0 that made the active tab a filled
block under tiny-skia and a thin underline under wgpu, from the same code
(RFC-102).

So:

- **Layout agrees across renderers.** Bounds, positions and hit-testing come
  from iced's layout pass, and text is shaped by cosmic-text under both. The
  rendered-bounds and click-probe tests (`side_bar_fit.rs`,
  `dismiss_remove_controls.rs`, `tab_bar_edges.rs`) measure properties that do
  not depend on the renderer.
- **Pixels need not agree.** Frame-hash and pixel-sample evidence describes
  tiny-skia. Treat it as evidence for the software-fallback path, and check a
  pixel-level conclusion against a real wgpu render before generalising it.
- **The safer fix is to stop depending on a primitive the renderers disagree
  on**, rather than asserting what one of them draws. snora no longer uses a
  shadow for anything a user must see.
## Widget identifiers for external observation (RFC-047)

snora attaches a stable `iced::widget::Id` to every surface it renders
itself — backdrops, the dialog card, the sheet panel, the toast stack,
the skeleton regions. See the [rendered surface identifiers
reference](../reference/rendered-surface-identifiers.md) for the full
list and naming convention.

**This is not a test harness.** It provides labels on snora-rendered
output — nothing more. It does not give you a state-query API, a
scripted-interaction driver, or accessibility semantics; an `Id` is not
a role. If you are building scripted GUI verification against a snora
application and need to locate a snora-rendered surface (the modal dim,
the dialog card, a specific toast) rather than application content you
already control, these identifiers are what to assert against instead
of window titles or coordinate guessing.

## A note on `snora-test`

We considered shipping a dedicated test-helper crate. The conclusion
was that doing so would freeze internal data shapes into the public API
and create a second surface to maintain. The current approach — `pub`
fields on vocabulary types, pure `update` functions, and internal
render-semantics tests using `iced_test` — covers both application and
framework needs without that cost.
