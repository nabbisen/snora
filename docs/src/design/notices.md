# Notices

A notice is a toned, behavior-light banner that communicates status to the
user. It may carry an optional title, an optional action button, and an
optional dismiss button. The application owns visibility state entirely —
whether a notice is shown, dismissed, or acted upon is always app-controlled.

## API

```rust,ignore
use snora::design::{Tokens, Tone, notice::Notice};

Notice::new(&tokens, Tone::Info, "Background sync is running.")
    .title("Syncing")                               // optional
    .action("View details", Message::OpenSync)      // optional
    .dismiss(Message::DismissNotice)                // optional
    .render()
```

`Notice::new(tokens, tone, body)` returns a builder. Call `.render()` to
produce the `Element`.

## Tones

| Tone | Semantic use |
|---|---|
| `Tone::Info` | Informational; neutral process state |
| `Tone::Success` | Completed action |
| `Tone::Warning` | Requires attention; non-blocking |
| `Tone::Danger` | Error or destructive action consequence |
| `Tone::Accent` | Primary call-out; default emphasis |
| `Tone::Neutral` | Low-emphasis status |

## Visibility

The notice has no internal visible/hidden state. Conditionally include it in
the view based on application state:

```rust,ignore
if self.show_notice {
    Notice::new(&tokens, Tone::Warning, "Index is stale.")
        .dismiss(Message::DismissNotice)
        .render()
} else {
    iced::widget::space().into()
}
```

## Accessibility

- Action and dismiss controls are `iced::widget::button` — keyboard-reachable.
- Tone colors all pass WCAG AA (verified by automated contrast tests).
- Custom focus ring not rendered in iced 0.14 — documented limitation, not a
  regression (see [Semantic accessibility](../contributing/semantic-accessibility.md)).
