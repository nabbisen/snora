# Recipe: Friendly error recovery notice

**Promotion status:** Recipe — v0.23

---

## 1. Purpose

Surface a recoverable error to the user with a clear explanation and a
primary recovery action, using `notice::Notice` with `Tone::Danger` or
`Tone::Warning`.

## 2. When to use

- A network request, file operation, or background task failed and the
  user can take a concrete action to recover (retry, reconnect, free space).
- A validation problem blocks the user and needs to be visible in context
  (not in a modal).
- A non-fatal warning needs an explicit acknowledgement or action before
  the user can proceed.

## 3. When not to use

- Critical errors that block all interaction — use a `dialog` so the modal
  dim signals nothing else is actionable.
- Informational status messages that do not require user action — use
  `Tone::Info` with no action button, or a toast.
- Errors that clear themselves automatically — a toast with a short lifetime
  is more appropriate.

## 4. Data the app owns

- Whether the error notice is visible (show only when the error exists).
- The specific error message and recovery action label (app-specific).
- The message emitted when the recovery action is pressed.
- The message emitted when the notice is dismissed (if dismissible).

## 5. Snora primitives used

- `snora::design::notice::Notice` — the main surface, with `Tone::Danger`
  or `Tone::Warning`.

## 6. Accessibility notes

- The recovery action and dismiss buttons are snora controls, reachable from
  the keyboard from 0.55 once the application routes Tab with
  `snora::keyboard::traverse` (RFC-116). Dismissing moves focus nowhere: the
  next Tab starts from the top. *(Corrected 2026-11-07: up to 0.54 they were
  iced buttons, which take no keyboard input.)*
- Tone color (red/orange) is supplemented by the notice text — not
  color-only.
- The notice is rendered inline (not in a modal), so keyboard focus is not
  trapped.
- A focus ring from `Tokens::focus` shows which control has focus (0.55).

## 7. Code example

```rust,ignore
use snora::design::{Tokens, Tone, notice::Notice};
use iced::Element;

fn error_recovery_notice<'a, Message: Clone + 'a>(
    tokens: &'a Tokens,
    visible: bool,
    title: &'a str,
    description: &'a str,
    action_label: &'a str,
    on_action: Message,
    on_dismiss: Option<Message>,
) -> Option<Element<'a, Message>> {
    if !visible {
        return None;
    }

    let mut notice = Notice::new(tokens, Tone::Danger, description)
        .title(title)
        .action(action_label, on_action);

    if let Some(msg) = on_dismiss {
        notice = notice.dismiss(msg);
    }

    Some(notice.render())
}
```

Usage in `view()`:

```rust,ignore
if let Some(notice) = error_recovery_notice(
    &self.tokens,
    self.sync_error.is_some(),
    "Sync failed",
    "Could not reach the server. Check your connection.",
    "Retry",
    Message::RetrySyncing,
    Some(Message::DismissSyncError),
) {
    main_col = main_col.push(notice);
}
```

## 8. Customization points

- Use `Tone::Warning` for recoverable problems that are not strictly errors
  (e.g. "Index is stale — rebuild recommended").
- Add a `button::link` ("Learn more") alongside the recovery action for
  complex errors: it takes the reader somewhere, so it is a link, not a
  `ghost`.
- Omit `.dismiss()` if the error must be acknowledged via the action button
  (i.e. it is not optional).

## 9. Promotion status

**Recipe.** `notice::Notice` is already the stable primitive; this recipe
shows how to use it for error recovery specifically. No additional API is
needed. The optional return value pattern (returning `None` when not visible)
is app code, not framework code.
