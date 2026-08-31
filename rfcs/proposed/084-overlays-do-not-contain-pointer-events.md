# RFC 084 — Overlays do not contain pointer events, and the dialog dismisses itself

**Status.** Proposed
**Tracks.** Overlay semantics / correctness. **Severity: Critical.**
**Found by** an external architect's audit, 2026-09-01 (F-01, F-02, F-03, F-04).
**F-01 reproduced and its remedy validated by the architect before opening.**
**Touches.** `crates/snora/src/overlay/dialog.rs`, `crates/snora/src/render.rs`,
`crates/snora/src/toast.rs`, `crates/snora/tests/render_semantics.rs`,
`docs/src/reference/overlay-interaction-semantics.md`.
**Release target.** 0.41.0 — **minor.** Behaviour changes on the default path.

## The one that has to be read first

**Clicking inside a dialog dismisses it.** Not a corner case: a click on the
dialog's own text, padding or card chrome falls through to the backdrop and
fires `on_close_modals`.

Reproduced with a throwaway `render_semantics` test — dialog content is plain
padded text, click at the window centre:

```
F-01 REPRO: click inside dialog -> [CloseModals]
```

`render_dialog` wraps content in `center(…)`, which captures nothing.
**`sheet.rs:48` does it correctly** — `let body = opaque(body_surface);` — three
files away. The dialog is the only modal surface that does not.

**The remedy is validated, not proposed.** Wrapping the *inner* node (not the
`center`) in `opaque` makes all twelve tests pass, including the reproduction
**and** the existing `outside_click_on_modal_emits_close_modals`. Outside-click
dismissal survives; only the fall-through stops. Wrapping the `center` itself
would capture the whole window and break dismissal — do not.

## The same defect, three more times

**F-02 — a modal without `on_close_modals` blocks nothing.**
`dim_without_capture` is `container(space())`: it neither captures events nor
reports a `mouse_interaction`, so every widget beneath a "modal" stays live.
This contradicts Law 8's unconditional *"Pointer blocking — yes"*.

**F-03 — the modal dim does not block scrolling.** Same cause, different input.

**F-04 — clicking a toast presses the widget underneath it.** The toast stack is
`container(body)` with no capture; only the `×` button captures, because it is a
button.

**One class, one fix shape.** Four surfaces that should contain pointer input;
one of them (`sheet`) does. This is not four bugs, it is one omission repeated —
which is why they are one RFC.

## Why nothing caught it

Every containment test we have is **positive**: *the button inside the dialog
works*, *the corner click dismisses*. **None asks whether the dialog dismisses
when it must not.** F-30 names this and it is the reason to fix the suite, not
just the code: a negative assertion would have failed on the day `render_dialog`
was written.

**Gate 5 is marked satisfied.** It should not have been, and correcting that is
part of this RFC.

## Non-goals

- **No change to outside-click dismissal.** It works, it is tested, and the
  validated fix preserves it. If a change here breaks
  `outside_click_on_modal_emits_close_modals`, the fix is wrong.
- **No new overlay vocabulary.** `opaque` is already in use in this codebase for
  exactly this purpose.
- **No z-order change.** The stack order is correct; only capture is missing.

## Open questions

**Q-1 — does the no-sink dim capture silently, or is the "modal without a sink"
shape itself wrong?** Wrapping it in `opaque` with no message makes it block
without dismissing, which matches Law 8. **Suggest that** — but note it changes
behaviour for anyone who has been relying on a no-sink modal being
non-blocking, which the docs never permitted and which is arguably the bug.

**Q-2 — does F-03 (scroll) need a separate mechanism from F-02 (click)?**
`opaque` may or may not stop wheel events reaching widgets beneath. **Measure
it; do not assume the click fix covers scroll**, and if it does not, say so
rather than silently shipping half.

**Q-3 — what is the negative-assertion set?** At minimum: click inside dialog
does **not** dismiss; click on toast body does **not** press the body; click
with a modal open and no sink does **not** reach the body. **Suggest deriving
the list from Law 8's own text** rather than inventing one, so the suite and the
law cannot drift.

## Acceptance criteria

1. A click inside the dialog — on padding, chrome, or plain text — produces no
   `on_close_modals`. **Demonstrated by a test that fails before the fix.**
2. Outside-click dismissal still works; the existing test is unmodified.
3. F-02, F-03, F-04 each fixed **or** explicitly deferred with a measurement
   showing why.
4. Q-3's negative assertions exist and are derived from Law 8.
5. `docs/src/reference/overlay-interaction-semantics.md` matches the behaviour
   after the fix — Laws 5 and 8 in particular.
6. **Gate 5's status is corrected** in `api-freeze-review.md`, with the reason.

## Compatibility and security

**Compatibility.** Behaviour change on the default path: clicks that previously
fell through stop doing so. That is the defect being fixed, but an application
that *relied* on the fall-through — deliberately or not — sees a change.
**Minor**, with a migration guide saying exactly that.

**Security.** A modal that does not block input is a UI-integrity issue: a
confirmation dialog can be bypassed by clicking the control it was guarding.
