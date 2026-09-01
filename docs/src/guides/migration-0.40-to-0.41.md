# Migration 0.40 → 0.41

> **Breaking, for a specific and narrow reason: a fix to a real bug.**
> Overlays did not contain pointer events. Clicking or scrolling on top of
> a dialog, a toast, or a modal dim could reach whatever was rendered
> beneath it — including, for a dialog, dismissing itself on a click on
> its own content. 0.41.0 fixes this. If your application depended on the
> old fall-through behavior (deliberately or by accident), that dependency
> stops working. There is no configuration flag to restore the old
> behavior, because the old behavior was the bug.

## Who is affected

Anyone whose application, before 0.41.0, could observe one of:

- A click on a dialog's own content (its padding, its plain text, any
  part that is not itself an interactive widget) dismissing the dialog.
- A modal shown with no `on_close_modals` handler failing to block clicks
  or scroll-wheel input from reaching content beneath it.
- A click or scroll on a toast's own body (not its `×` button) reaching
  whatever was rendered beneath the toast.

If you never relied on any of these — and nothing in snora's own
documentation ever described them as intended behavior — nothing changes
for you except that a class of latent input bugs in your own application
is now closed rather than open.

**Security note.** A modal that does not block input is a UI-integrity
issue: a confirmation dialog could be bypassed by clicking through it to
the control it was meant to be guarding. If your application shows
confirmation dialogs over destructive actions, this fix closes a real gap
— it is worth re-testing those flows specifically.

## What changed

### Dialogs, toasts, and the no-sink modal dim now capture pointer input over their own bounds

Four surfaces should have contained pointer input and only one of them
(the sheet) did:

| Surface | Before 0.41.0 | As of 0.41.0 |
|---|---|---|
| Dialog content | Fell through to the dim backdrop beneath it — a click on the dialog's own text or padding fired `on_close_modals` | Captured; only the dialog's own interactive content responds |
| Modal dim with **no** close sink | Did not capture clicks or scroll at all | Captures both; produces no message, since there is none to produce |
| Modal dim with a close sink | Captured clicks; did **not** capture scroll | Captures both clicks and scroll, both dispatching the same close message |
| Toast body (outside the `×` button) | Fell through to whatever rendered beneath the toast | Captured; only the `×` button's own click still dispatches its message |
| Sheet | Already correct | Unchanged |

Found by an external architect's audit (four findings, F-01 through
F-04) and fixed the same way in each place: `iced::widget::opaque`
wraps the surface's own content, matching the sheet's existing
(and always-correct) implementation.

### Why the no-sink case does not merely omit the block

Before this fix, `docs/src/reference/overlay-interaction-semantics.md`'s
Law 5 said a missing close sink meant *"outside clicks are not
captured."* That was true, and it was also the bug: a missing dismiss
message is not the same question as whether pointer input should reach
through the modal, and conflating them meant an application that
declined to wire `on_close_modals` — to provide its own explicit close
button instead, exactly as Law 5 recommends — got no pointer blocking as
a side effect nobody asked for. Law 5 is corrected to say what is now
true: a missing sink omits the *message*, never the *containment*.

## What did not change

- No z-order change. The layer stack order (skeleton, menus, modal dim,
  dialog, sheet, toasts) is unchanged; only pointer capture within each
  layer's own bounds was added.
- No new public API and no new overlay vocabulary — `opaque` was already
  used in this codebase (by the sheet) for exactly this purpose.
- The sheet's own rendering is untouched; it was already correct.
- Outside-click dismissal (clicking the dim outside a dialog or sheet to
  close it) is unchanged and still works exactly as before.

## If you are jumping more than one minor

Read the guides for the jumps in between — several carry real changes,
and the [migration index](migrations.md) lists them.
