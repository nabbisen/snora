# RFC-119 — The keyboard reaches beneath an open modal

**Status.** Proposed 2026-11-07. Awaiting the owner's rulings on Q-1 to Q-5.
**Raised.** 2026-11-07, architect, while rewriting the book's keyboard pages after
RFC-116 R-1 merged (`952e1a2`).
**Release target.** 0.55.0, if Q-1 rules that the release waits for it.
**Priority.** A defect introduced by 0.55's own keyboard work, before any release
carries it.

---

## The finding

From RFC-116 R-1, every control snora draws is a focus target. An application that
routes Tab with `snora::keyboard::traverse` can now Tab to **controls beneath an open
dialog or sheet**, and Enter or Space presses them.

**Why, from iced 0.14.2's source.** I read the source; there is no probe yet, and
Unit 0 reproduces it first.
- `stack` hands every event to its top layer, and then to each layer beneath it,
  unless the event is captured (`iced_widget-0.14.2/src/stack.rs`, `update`).
- The modal's dim and the dialog are `mouse_area` and `opaque`. `opaque` captures only
  a mouse press over its bounds (`helpers.rs`, `opaque`'s `update`). Key presses are
  never captured, so they reach the skeleton.
- `focus_next` walks every focusable in the window's tree, the skeleton included.

**So the modal stops the pointer and not the keyboard.** Law 8 promises pointer
blocking unconditionally, "so nothing beneath a modal … is reachable through it".
The keyboard now breaks that promise.

**Before 0.55 there was nothing beneath to reach,** except an application's own
`text_input` or `text_editor`. The gap arrives with the release that announces
keyboard access. **No release carries it yet.**

**What it costs a user.** A confirmation dialog ("Discard changes?") is up, and a
keyboard user Tabs on, past its buttons. Focus lands on a control the dim is
covering, say "Delete" in the page beneath, and Enter presses it. The dialog was meant
to gate exactly that action, and a pointer user could not have done it. The owner's
rule forbids this kind of misunderstanding, and here it is also unsafe.

## Proposal

**R-1. The keyboard reaches exactly what the pointer can press.** A reader can
predict it from what they see:
- **Dialog or sheet open:** everything under the dim is out of the keyboard's reach.
  Tab moves between the dialog or sheet and the toasts, which sit above the dim and
  take the pointer (Law 6). Neither focus nor key presses reach the skeleton.
- **Menu open, with its outside-click backdrop** (`on_close_menus` set): the skeleton
  is out of reach in the same way, because a click there closes the menu and presses
  nothing. **Without a backdrop,** the pointer can press the page, and so can the
  keyboard.
- **Nothing open:** unchanged.

**How:** an `Inert` wrapper in `snora-control`, which the engine puts around each
layer the pointer cannot reach while an overlay is open. It:
- **reports no focus target** from its content: it does not forward `focusable`
  operations, and `container` and `text` operations still pass, so selectors and tests
  find content;
- **delivers no keyboard event** to its content;
- **leaves drawing alone.**

It needs no focus query, and so no `advanced` feature
(`design-decisions.md`, "Changed 2026-11-07").

**R-2. A control that was focused when it became covered draws no ring under the
dim, and Enter does not press it.** R-1 already stops Enter. Whether its focus is
**kept**, so it returns when the modal closes, or **dropped**, is Q-3.

**R-3. The rule goes into Law 8.** "Keyboard blocking" becomes a row beside "Pointer
blocking", with the same unconditional "yes", and both guards:
- `render_semantics.rs` gains a negative assertion for each blocked layer: Tab never
  finds a skeleton control while a modal is open, and Enter never publishes its
  message;
- `every_control.rs` gains a subject under a modal.

## Open questions, with suggested rulings

**Q-1 — does 0.55.0 wait for this?** **Suggest yes.**
- 0.55 is the release that tells applications to route Tab.
- Shipping it with modals that block only the pointer would put the gap in every
  adopter's hands on the day they adopt the fix.
- The work is one wrapper, the engine's use of it, and tests. It is smaller than any
  RFC-116 unit.

**Q-2 — default behaviour, or an optional field?** The design decision on focus
trapping says trapping "must arrive as a new optional `Dialog`/`Sheet` field …, not a
change to an existing field or a new default behaviour." **Suggest default, with no
opt-out.**
- That constraint was written for *trapping*: a new behaviour that changes where
  focus moves.
- This RFC makes the keyboard keep a promise the pointer already keeps
  unconditionally (Law 8).
- **It changes nothing snora's own controls did in 0.54,** because nothing beneath a
  modal was reachable then.
- It does change one thing: an application's own `text_input` beneath an open modal
  stops taking keys. The pointer could not reach it either, so that is the fix, not
  a break. The migration guide says so.
- An opt-in would leave every application that does not read the guide unsafe.

**Q-3 — when a modal closes, does focus return to where it was?** That is WAI-ARIA's
dialog pattern: focus returns to the control that opened it. **Suggest yes, if Unit 0
shows it comes free.**
- If `Inert` keeps the covered control's state and only hides it from focus
  operations, the opener's focus survives the modal and comes back with it.
- The ring must still not draw under the dim (R-2).
- If keeping focus means drawing the ring under the dim, drop focus instead, and
  record the return in RFC-116's long-term plan.

**Q-4 — toasts during a modal.** **Suggest reachable**, because the pointer reaches
them (Law 6). Tab order is the dialog, then the sheet, then the toasts.

**Q-5 — does this close RFC-060 Q-1, focus trapping?** **Suggest: it answers the user
need, and the question is restated, not closed.**
- With the skeleton inert, Tab moves through the modal and the toasts only. Past the
  last, nothing is focused for one press, and then it starts again from the modal's
  first control.
- That bounds a keyboard user to what the modal allows, which is what trapping was
  for.
- The one-press gap is iced's `focus_next`, and the same as RFC-116's no-wrap rule.
  A strict wrap is not proposed.

## Review against snora's principles

- **Nobody is confused.** One rule, "the keyboard reaches what the pointer can press",
  covers modals, menus and toasts. A reader predicts it by looking at the screen.
- **Safe.** A modal gates the keyboard as it gates the pointer. The unsafe case, a
  destructive action pressed beneath a confirmation, is closed by default.
- **Clean.** One wrapper and one engine change, with no new public API for
  applications and no `advanced` feature.
- **Robust.** Both guards are negative assertions, each shown failing by removing the
  wrapper.
- **ABDD.** Nothing about layout or direction changes. The wrapper draws nothing.

## What this is not

- **Not focus on open.** Where focus starts when a modal opens stays the
  application's: `widget::Id` and `operate`, as `overlays.md` says.
- **Not an accessibility tree.** Screen-reader modal semantics still wait on iced.
- **Not Escape.** That stays `dismiss_on_escape`, and RFC-116 R-2's `Shortcuts` router.

## Acceptance criteria

1. **Unit 0:** a failing test that reproduces the finding. With a dialog open, Tab
   reaches a skeleton control, and Enter publishes its message.
2. **Unit 1:** `Inert` and the engine's use of it:
   - R-1's three cases, each with a negative assertion and a breakage row;
   - Q-3 answered from the implementation;
   - Q-4's Tab order asserted.
3. **Law 8 and the book,** by the architect.
4. CHANGELOG under Fixed.
