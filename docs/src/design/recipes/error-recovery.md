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
  `snora::keyboard::traverse` (RFC-116). Dismissing moves focus to the nearest
  control after the notice in its container, or else before it (RFC-120).
  **Retry that removes the notice leaves nothing focused,** RFC-120's safe answer
  when the pressed control and its neighbour go together; the next Tab starts
  from the top. To keep the reader's place, send focus where they should go next
  with a focus task (`iced::widget::operation::focus(id)` on that control's id). *(Corrected 2026-11-07: up to 0.54 they were
  iced buttons, which take no keyboard input.)*
- Tone color (red/orange) is supplemented by the notice text — not
  color-only.
- The notice is rendered inline (not in a modal), so keyboard focus is not
  trapped.
- A focus ring from `Tokens::focus` shows which control has focus (0.55).

## 7. Code example

```rust,ignore
{{#include ../../../../examples/book_snippets/src/recipes_error_recovery.rs:recipe_error_recovery}}
```

Usage in `view()`:

```rust,ignore
{{#include ../../../../examples/book_snippets/src/recipes_error_recovery.rs:recipe_error_recovery_use}}
```

## 8. Customization points

- Use `Tone::Warning` for recoverable problems that are not strictly errors
  (e.g. "Index is stale — rebuild recommended").
- For a second action ("Learn more") on a complex error, put a `button::link`
  beside the notice, not in it: `Notice` has one action. It takes the reader
  somewhere, so it is a link, not a `ghost`.
- Omit `.dismiss()` if the error must be acknowledged via the action button
  (i.e. it is not optional).

## 9. Promotion status

**Recipe.** `notice::Notice` is already the stable primitive; this recipe
shows how to use it for error recovery specifically. No additional API is
needed. The optional return value pattern (returning `None` when not visible)
is app code, not framework code.
