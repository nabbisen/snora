# RFC-134 — No checkbox, toggle, radio, pick list or slider can be reached by the keyboard

**Status.** Done — shipped v0.65.0. Accepted 2026-11-15 (owner), every open question ruled as suggested (see Rulings). The
direction was ruled first: "Should snora provide focusable form controls? — Yes. Preferred if
available and reasonable."
**Handoff.** [`handoffs/134-…`](../handoffs/134-no-form-control-can-be-reached-by-the-keyboard/implementation-handoff.md).
**Raised.** 2026-11-15, architect, from logolig's report ("snora has no focusable checkbox, slider
or pick list in any version"), verified true by measurement.
**Release target.** After 0.64.0 (Q-4). Unit 0 can run once 0.63 ships.
**Priority.** Keyboard compatibility is required (owner, 2026-11-04). Today a form built on snora
has no keyboard route, and the book recommended the controls that lack one (corrected
2026-11-15).

---

## The finding

Measured on iced 0.14 (`iced_widget` 0.14.2), by the focus operation Tab uses:

| Control | A Tab stop? | Keys |
|---|---|---|
| `text_input` | yes | typing, arrows |
| `checkbox`, `toggler`, `radio` | **no** | none |
| `pick_list` | **no** | none |
| `combo_box` | **no** (it does not expose its text input to focus) | none by Tab |
| `slider` | **no** | ↑ ↓ only while the pointer is over it |

- **snora has never had any of these.** Form widgets were a stated non-goal ("use iced
  primitives").
- **The book advised iced's,** until the correction of 2026-11-15. Its keyboard guide now gives the
  workaround that exists: a checkbox, toggle or radio built on `snora::control`. A pick list and a
  slider have no keyboard route.

## Proposal

**R-1. The non-goal narrows.** It was "form widgets / validation: out of scope". It becomes:
- **in scope:** keyboard-reachable versions of the controls iced already provides, namely a
  checkbox, a toggle, a radio group, a select (pick list) and a slider;
- **still out:** validation, form layout, and new kinds of control.

snora is not becoming a widget library. It closes the keyboard gap iced leaves.

**R-2. Each control's keyboard, by platform convention, stated before it is built:**

| Control | Tab | Keys |
|---|---|---|
| checkbox, toggle | one stop | **Enter or Space** toggles (snora's press rule, RFC-130's chords excluded) |
| radio group | **one stop for the group,** on the selected option | ↑ ↓ ← → move **and select**, wrapping (← → by the screen, RFC-132 R-5) |
| select | one stop | Enter, Space or ↓ opens; inside, snora's menu keys (RFC-123): ↓ ↑ Home End, Enter chooses, Escape closes and returns focus |
| slider | one stop | ← → ↑ ↓ one step, Page Up / Page Down a larger one, Home End to the ends (← → by the screen) |

- **A disabled control is no stop.**
- **Every control meets snora's floors:** the focus ring (RFC-116), the 24 px target (RFC-115),
  its text at the palette's level (RFC-131), and its non-text parts at 3:1.

**R-3. One message per action,** as `snora::control` does: a click and a key publish the same
message.

## Open questions, with suggested rulings

**Q-1 — build on iced's controls, or snora's own?** **Suggest:** snora's own, on
`snora-interaction`'s `Control`, which already holds focus, the ring, the press rule and the
target floor.
- A wrapper over iced's `checkbox` gives two messages for one action, which the 2026-11-15 probe
  showed.
- iced's `pick_list` and `slider` take no focus to wrap.
- Unit 0 measures the size of each.

**Q-2 — which paths?** **Suggest: both,** as `snora::control` and `snora::design::control`:
- `snora::checkbox` (and the others) styled by the host's iced theme;
- `snora::design::checkbox` styled from `Tokens`.

**Q-3 — "if available and reasonable".** **Suggest: Unit 0 rules control by control.**
- A control that cannot be built in iced 0.14 without forking iced, or not at a reasonable size,
  is reported with its reason and left to a trigger (an iced release that adds focus to it).
- The keyboard guide's workaround stays the documented route for that control.

**Q-4 — when.** **Suggest:**
- Unit 0 after 0.63 ships;
- the controls Unit 0 finds reasonable from **0.65.0**, so that 0.64 stays the one breaking
  release (RFC-133, the F6 removal) and these arrive as additions.

## Rulings, 2026-11-15 (owner)

Accepted as proposed:
- **Q-1:** snora's own controls, on `snora-interaction`'s `Control`. No wrapper over iced's.
- **Q-2:** both paths, `snora::…` and `snora::design::…`.
- **Q-3:** Unit 0 decides control by control. One that is not reasonable is left to a trigger, with
  the keyboard guide's workaround.
- **Q-4:** Unit 0 after 0.63 ships; the controls from 0.65.0, as additions.

## Review against snora's principles

- **Keyboard.** Required, and this is where snora's applications lack it most.
- **Nobody is confused.** One message per action. The keys are the platform's, written down
  first.
- **ABDD.** The radio group and the slider follow RFC-132 R-5. Their order is logical.
- **Robust.** Each control is built on the one `Control` mechanism, not a second.
- **Scope.** A narrowing of a non-goal, stated, not an expansion into a widget library.

## Acceptance criteria

1. **Unit 0:** each control's feasibility and size, and Q-3's verdict per control.
2. **Unit 1:** each reasonable control with R-2's keys, R-3, and every floor, each with a breakage
   row; both paths.
3. **The book,** by the architect: the keyboard guide, the scope page, and the guides.
