# RFC-119 — The keyboard reaches beneath an open modal

**Status.** Proposed 2026-11-07. The owner accepted the first draft's suggested rulings
"almost", subject to a review against the fundamental philosophy.
**Revised** 2026-11-07 after that review. It found a keyboard trap that R-1 alone would
create, and a state reset in the obvious way to build it. R-4 and R-5 are new, and Q-3
is now firm. **Awaiting the owner's confirmation of the revision.** See "Review against
snora's principles".
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
predict it from what they see. Everything beneath the topmost backdrop is out of the
keyboard's reach, so a menu left open beneath a modal is covered too:
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

**R-4. The keyboard can leave the way the pointer can.** R-1 alone would trap a
keyboard user:
- a pointer user leaves a menu, or a dialog with `on_close_modals`, by clicking
  outside;
- after R-1, a keyboard user cannot reach "outside", and the menu's own trigger is in
  the inert header;
- the only remaining exit is Escape, which today exists only if the application wired
  `dismiss_on_escape`.

That would be a keyboard trap (WCAG 2.1.2), created by this RFC. So **whenever a
backdrop has a close message, Escape sends it.** The backdrop answers Escape and
captures the event.
- **The order is Law 1's,** from the top: an open modal closes before a menu, as
  `dismiss_on_escape` does.
- **No double message.** `iced::keyboard::listen()` delivers only events that no
  widget captured (`iced_futures-0.14.0/src/keyboard.rs`, `Status::Ignored`). An
  application that also calls `dismiss_on_escape` never sees that Escape.
- **Without a close message,** the pointer cannot leave by clicking outside either.
  Escape is then the application's, through `dismiss_on_escape`, as before. The overlay
  checklist's visible close button is what both rely on.
- **A focused `text_input` in a dialog** handles Escape itself first, as iced does.
  The second Escape closes the dialog.

This amends Law 7 for this one case: **an overlay that closes on a click outside
also closes on Escape.** `dismiss_on_escape` is **not** superseded, so it is not
deprecated: it still serves an overlay that closes on Escape but not on a click
outside. RFC-116 R-2's planned `Escape { modal, menu }` is reconsidered in R-2's own
review, since this RFC covers the common case.

**R-5. Opening a modal never resets the page.** `stack` and `column` drop a widget's
state when the tree's shape changes (RFC-116 Unit 3's tab defect). Wrapping the
skeleton only while a modal is open would reshape the tree twice: on open and on close.
Each time, the page would lose scroll offsets, a text input's cursor, and focus.
**`Inert` is always in the tree, around each layer it may cover, and is switched by
a flag.** Alternatively, it is tree-transparent, as `container` is. Either way, a test
shows that a scroll offset and a text input's contents survive opening and closing a
modal.

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
- The work is one wrapper (R-1, R-5, Q-3), Escape on the backdrops
  (R-4), and tests: about the size of one RFC-116 unit.

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
dialog pattern: focus returns to the control that opened it. **Ruling (revised from
"if it comes free"): yes.**
- **When a layer becomes inert,** its focused control loses its focus, so no ring draws
  under the dim. `Inert` remembers which control it was.
- **When the layer becomes reachable again,** that control gets its focus back.

`Inert` is a widget with its own state (R-5), so it can hold that memory without a
focus query.

**If Unit 0 finds this cannot be done without drawing the ring under the dim,** it
comes back to the owner. It does not move quietly to the long-term plan. A
conditional ruling would leave users unable to predict where focus goes.

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

- **Nobody is confused.** One rule, "the keyboard does what the pointer can", covers
  modals, menus and toasts in both directions: what can be **reached** (R-1), and how to
  **leave** (R-4). A reader predicts it by looking at the screen. Focus comes back where
  it was (Q-3), and the page is as it was (R-5).
- **What the review changed, 2026-11-07.** The first draft stated only the reaching
  half. Checked against the principles, it would have:
  - **trapped** a keyboard user in any menu, or any dialog that closes by clicking
    outside, whose application had not wired Escape. R-4 fixes that;
  - **reset the page** whenever a modal opened, built the obvious way: inserting the
    wrapper reshapes the tree. R-5 fixes that;
  - **left Q-3 conditional.** It is now firm, with escalation if it proves impossible.
- **Safe.** A modal gates the keyboard as it gates the pointer. The unsafe case, a
  destructive action pressed beneath a confirmation, is closed by default.
- **Clean.** One wrapper and one engine change, with no new public API for
  applications and no `advanced` feature.
- **Robust.** Both guards are negative assertions, each shown failing by removing the
  wrapper. R-4 and R-5 each get a breakage row: drop the Escape handling, and the trap
  test fails; insert the wrapper only on open, and the state test fails.
- **Deprecation checked.** Nothing is superseded in full: `dismiss_on_escape` keeps a
  case of its own (R-4). Law 8's "focus trapping: no" row and Law 7's "application-owned"
  are rewritten, not left beside the new rule.
- **ABDD.** Nothing about layout or direction changes. The wrapper draws nothing.

## What this is not

- **Not focus on open.** Where focus starts when a modal opens stays the
  application's: `widget::Id` and `operate`, as `overlays.md` says.
- **Not an accessibility tree.** Screen-reader modal semantics still wait on iced.
- **Not Escape in general.** Only an overlay that closes on a click outside answers
  Escape (R-4). Every other Escape stays the application's: `dismiss_on_escape`, and
  RFC-116 R-2's `Shortcuts` router.

## Acceptance criteria

1. **Unit 0:** a failing test that reproduces the finding. With a dialog open, Tab
   reaches a skeleton control, and Enter publishes its message.
2. **Unit 1:** `Inert` and the engine's use of it:
   - R-1's three cases, each with a negative assertion and a breakage row;
   - R-4: no trap. For a menu with a backdrop, and for a dialog with `on_close_modals`,
     Escape publishes the close message, once. An application's `keyboard::listen()`
     does not also receive it;
   - R-5: a scroll offset and a text input's contents survive opening and closing a
     modal;
   - Q-3: focus returns to the opener, with no ring under the dim;
   - Q-4's Tab order asserted.
3. **Law 8 and the book,** by the architect.
4. CHANGELOG under Fixed.
