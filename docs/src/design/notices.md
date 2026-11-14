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

**The tone colours the notice's bar and border.** Its text is drawn in text colours: the title and
body in `text_primary`, and the action as a ghost action, in `palette.accent`, whatever the tone
(0.62, RFC-128). An action does something where the reader is, so it is drawn as every action is.

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

- Action and dismiss controls are snora controls (RFC-116): from 0.55, Tab
  reaches them once the application routes it with `snora::keyboard::traverse`,
  Enter and Space press them, and a ring from `Tokens::focus` shows focus.
  Dismissing moves focus to the nearest control after the notice in its
  container, or else before it (RFC-120).
  *(Corrected 2026-11-07: up to 0.54 they were iced buttons, which take no
  keyboard input; this page had said "keyboard-reachable".)*
- **Every text a notice draws clears its preset's text floor:** 4.5:1, or 7:1 on `light_aaa` and
  `dark_aaa`. The title and body are declared pairs, and the palette's suite tests them. The action's
  label is measured as drawn, in every tone of every preset (`text_floor.rs`, 0.62). The bar and border,
  in the tone colour, are non-text, at 3:1. *(Until 0.62 the action's label took the tone colour:
  Neutral's read 3.1:1, and on the AAA presets Success, Warning and Danger fell under 7:1. RFC-128.)*
