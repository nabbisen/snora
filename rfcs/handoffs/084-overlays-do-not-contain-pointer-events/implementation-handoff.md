# Developer Handoff — RFC-084 overlay pointer containment

**Governing RFC.** [RFC-084](../../accepted/084-overlays-do-not-contain-pointer-events.md)
**Status.** Accepted (owner, 2026-09-01). **Critical.**
**Release target.** 0.41.0 — **minor.** Behaviour changes on the default path.
**Implementation units.** Four, plus the suite. **Unit 1 first, alone if
anything slips** — it is the shipped Critical.

---

## 1. Reproduce before you fix

**Do not start from the fix.** Write the failing test first and watch it fail —
this project's rule, and here it is also the only proof the bug is what we think.

The architect's reproduction, which you should be able to re-create in
`crates/snora/tests/render_semantics.rs`:

```rust
let content: Element<Msg> = container(text("Are you sure…")).padding(80).into();
let dialog = Dialog::new(content);
let layout = AppLayout::new(btn("body", Msg::BodyPressed))
    .dialog(dialog).on_close_modals(Msg::CloseModals);
let mut ui = simulator(render(layout));
ui.point_at(Point::new(512.0, 384.0));      // dead centre — inside the dialog
let _ = ui.simulate(iced_test::simulator::click());
```

Observed today: `[CloseModals]`. **If you do not see that, stop and report** —
the premise has moved.

## 2. Unit 1 — the dialog (the Critical)

`crates/snora/src/overlay/dialog.rs`. Wrap the **inner** node in
`iced::widget::opaque`, both arms:

- `None => center(opaque(dialog.content))…`
- `Some(card) => center(opaque(container(dialog.content)…))…`

**Do not wrap the `center`.** It would capture the whole window and break
outside-click dismissal — which the existing
`outside_click_on_modal_emits_close_modals` will catch, and which is why that
test must not be modified.

**Validated already:** the architect applied exactly this and all twelve tests
passed, including the new reproduction. Your job is to confirm it, not discover
it — and to say so if your result differs.

**`sheet.rs:48` is the reference implementation.** Read it first.

## 3. Units 2–4 — the same omission, three more places

- **F-02** `dim_without_capture` (`render.rs:261`) is `container(space())` and
  captures nothing, so a modal with no close sink blocks nothing. **Q-1 ruled:
  give it the same capture without a message** — `opaque` with no sink. That
  makes Law 8's unconditional "pointer blocking — yes" true.
- **F-03** the dim does not block scrolling. **Q-2 ruled: measure, do not
  assume.** `opaque` may not stop wheel events. If it does not, say so and
  scope the remedy separately rather than shipping half a fix quietly.
- **F-04** clicking a toast presses the widget underneath (`toast.rs:184`).
  Same shape; the `×` works only because buttons capture.

**Each gets its own failing test first**, same discipline as Unit 1.

## 4. The suite is half the deliverable — Q-3 ruled

**Every containment test we have is positive.** That is why a dialog that
dismisses itself passed all of them.

**Derive the negative assertions from Law 8's own text** in
`docs/src/reference/overlay-interaction-semantics.md`, rather than inventing a
list — so the suite and the law cannot drift. At minimum:

- click inside the dialog does **not** dismiss
- click on a toast body does **not** reach the widget beneath
- with a modal open and **no** sink, a click does **not** reach the body

**State how you derived them**, and if Law 8 implies an assertion you did not
write, say which and why.

## 5. Gate 5

`docs/src/contributing/api-freeze-review.md` marks gate 5 satisfied. It was not.
**Correct it, cite RFC-084, and do not silently re-tick it** once the tests
exist — that is a separate judgement and it is the owner's.

## 6. Explicit non-change scope

- **No z-order change.** The stack order is correct; only capture is missing.
- **No new overlay vocabulary.** `opaque` is already used here for this purpose.
- **Do not modify `outside_click_on_modal_emits_close_modals`.** If your change
  breaks it, your change is wrong.
- **No change to the sheet.** It is the thing that was right.

## 7. Required evidence

- Each of the four failures reproduced **before** its fix, captured
- All tests after, including the untouched outside-click test
- The Law-8 derivation for the negative assertions
- Q-2's scroll measurement, whichever way it comes out
- `git diff` on `sheet.rs` — **expected empty**

## 8. Acceptance criteria

1. Four failing tests, then four fixes, then green — in that order, captured.
2. Outside-click dismissal unchanged and its test unmodified.
3. Negative assertions derived from Law 8, derivation stated.
4. Q-2 answered with a measurement.
5. Gate 5 corrected, not re-ticked.
6. `overlay-interaction-semantics.md` matches behaviour; migration guide notes
   the fall-through change.

## 9. Required review-request format

`.git-exclude/review-request/084-overlays-do-not-contain-pointer-events/`,
`README.md` entry point, evidence under `evidence/`, relative paths, single
entry-point path in the summary.

**Requested review focus: the four before-fix failures.** The fixes are one line
each and one is already validated. That each bug was real, and that the tests
catch it, is the whole deliverable.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
