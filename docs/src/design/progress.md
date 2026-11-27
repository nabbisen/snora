# Progress

Progress indicators are display-only: they emit no events, and the
application owns the task, the progress value, and any cancellation
semantics.

## API

```rust,ignore
{{#include ../../../examples/book_snippets/src/design_pages.rs:progress_variants}}
```

## Value

The value is `Option<f32>`:

- `Some(v)` — determinate progress. `v` is clamped to `0.0..=1.0`.
- `None` — indeterminate. Renders as a 0% bar with a "…" suffix.

**iced 0.14 limitation:** there is no native indeterminate animation. The
"…" suffix is the only visual signal. If a pulsing bar is required,
consider using a custom canvas widget.

## Tone

The `Tone` parameter colors the filled bar portion:

| Tone | Typical use |
|---|---|
| `Tone::Accent` | General-purpose progress |
| `Tone::Success` | Completed or healthy |
| `Tone::Warning` | Slow or at risk |
| `Tone::Danger` | Error or critical |
| `Tone::Info` | Background or informational |
| `Tone::Neutral` | Subdued |

> **Note:** The RFC-032 API sketch omitted the `Tone` parameter. The
> implementation adds it so progress bars can be toned consistently with
> notices and chips. Pass `Tone::Accent` to match the sketch's intent.

## Layout variants

`progress::row` — a label + bar with no outer container. Use inside an
existing card or panel.

`progress::card` — the same content wrapped in a `card::surface`. Use when
the progress item is a primary focus of a region.

## Accessibility

Drawn by snora itself since 0.64 (RFC-133): iced's `progress_bar` has no direction and always fills from the left. Only its `progress_bar::Style` type is still used. Display-only; no keyboard interaction.
No ARIA progressbar role is exposed in iced 0.14. If assistive-technology
support is required, supplement the progress indicator with a visible
percentage label (the helper already renders the percentage as text).
