# RFC-127 — An application cannot put focus on a snora control

**Status.** Done — shipped v0.61.0. Accepted 2026-11-12. Every open question ruled as suggested (see Rulings).
**Handoff.** [`handoffs/127-…`](../handoffs/127-a-snora-control-cannot-be-given-focus/implementation-handoff.md).
**Raised.** 2026-11-12, architect, scoping 0.61, found while reading RFC-126's finding 1.
**Release target.** 0.61.0, with RFC-126.
**Priority.** The book gives advice that cannot be followed with snora's own controls.
Keyboard compatibility is required (owner, 2026-11-04).

---

## The finding

**The book tells an application to choose where focus starts, and the application cannot
do it with a snora control.**
- `overlays.md`'s dialog checklist: *"If initial keyboard focus inside the dialog matters,
  set it with iced's `widget::Id` and a focus operation."*
- `traverse`'s rustdoc: *"Where focus starts when an overlay opens is still yours
  (`widget::Id` and `operate`)."*

**But a snora control's `widget::Id` is made inside it and cannot be set**
(`crates/snora-interaction/src/control.rs`: `id: Id::unique()`, with no builder method).
That covers `snora::control`, `snora::design::control` and every `design::button` variant,
which return an `Element`. iced's `focus(id)` therefore cannot reach any of them. The advice
works only for iced widgets that take an id, such as `text_input`.

**Where it matters:**
- a confirmation dialog whose safe choice ("Cancel") should hold focus when it opens;
- a form that opens on its first field;
- returning focus to a particular control after the application's own action.

Since 0.55, opening a dialog sets the page's focus aside and focuses nothing in the dialog.
The first Tab reaches the dialog's first control. That is safe, and it is what a reader gets
today. But an application cannot do better, although the book says it can.

## Proposal

**R-1. `snora::control` and `snora::design::control` take an id:**
`.id(impl Into<widget::Id>)`, as iced's `text_input` does. With one set, iced's own
`iced::widget::operation::focus(id)` focuses the control, and `is_focused(id)` reports it. No
snora-specific focus API is added: an application uses the iced pattern it already knows.

**R-2. A focused control behaves as one reached by Tab.**
- It shows its ring, because focus was moved by the program for the keyboard.
- Enter and Space press it.
- Tab carries on from it.

**R-3. Focusing a control that is covered focuses nothing beneath** (RFC-119): `Inert` hides
covered controls from every operation. Like any `focus(id)` that reaches no control, it
**leaves nothing focused**, and `is_focused(id)` gives no answer. *(Corrected 2026-11-12 from
Unit 0's measurement: it read "does nothing", but a control above that held focus loses it.
That is iced's one rule for `focus(id)`, and it is kept.)*

**R-4. The book:** `overlays.md`'s checklist and `traverse`'s rustdoc show the working
pattern, with a snora control:

```rust,ignore
const CANCEL: &str = "confirm-cancel";

// view
snora::design::control(&t, text("Cancel")).id(CANCEL).on_press(Message::Cancel)
    .style(move |_, s| style::button::secondary(&tok, s))

// update, when the dialog opens
self.confirm = true;
return iced::widget::operation::focus(CANCEL);
```

## Open questions, with suggested rulings

**Q-1 — does the focus land when the task is returned by the same `update` that opens the
dialog?** The dialog appears in the next `view`, and `Inert` sets the page's focus aside at
that rebuild. **Suggest:** Unit 0 measures the order on both renderers. If the task runs
before the dialog exists, R-4's pattern must say so and show what works. A pattern in the book
that does not work is the defect this RFC exists to fix.

**Q-2 — the `design::button` variants (`primary`, `secondary`, `ghost`, `link`, `danger`) return
an `Element`, so they cannot take `.id()`.** **Suggest: leave them unchanged now.** A control
that needs an id is built with `snora::design::control` and the same style function, as
R-4 shows, and it looks identical.
- Changing five return types would break every caller that names the type.
- It would also open the variants to restyling (`.style()`), which weakens what "danger"
  promises.
- **Recorded as future wanted:** an id on the variants themselves, if an adopter finds the
  `control` route confusing.

**Q-3 — the prefab controls** (tabs, sidebar items, the header's trigger, a notice's
buttons). **Suggest: not in this RFC.** RFC-126's F6 moves between zones without them, and
none of their callers has asked. Recorded as future wanted.

## Review against snora's principles

- **Nobody is confused.** The book's advice becomes true, and it uses iced's own pattern rather
  than a second one of snora's.
- **Safe.** `focus(id)` cannot reach beneath a cover (R-3). A confirmation dialog can open with
  its safe choice focused.
- **Clean.** One builder method on one type. No new focus API.
- **Predictable.** A program-moved focus behaves as a Tab-moved one (R-2).
- **The initial cost is not the deciding factor.** Q-2 is ruled on compatibility and on what a
  variant promises, not on effort.
- **Keyboard.** Closes the gap between "where focus starts is yours" and what snora's controls
  allow.

## What this is not

- Not a declarative `Dialog::initial_focus`. iced's task is the one way, as for `text_input`.
- Not ids on prefab controls (Q-3), or on the `design::button` variants (Q-2).

## Acceptance criteria

1. **Unit 0:** R-1 prototyped. It measures Q-1 and R-3 on both renderers: a dialog opened with
   a focus task in the same `update`; a covered control given focus; the ring after
   `focus(id)`.
2. **Unit 1:** R-1 to R-3, with a test and a breakage row each. `every_control.rs`'s subjects
   gain an id case.
3. **The book,** by the architect (R-4).

## Rulings, 2026-11-12 (owner)

The RFC is accepted as written:
- **Q-1:** Unit 0 measures whether a focus task returned with the dialog's opening lands, and
  R-4's pattern says what works.
- **Q-2:** the `design::button` variants are unchanged. A control that needs an id is built
  with `snora::design::control` and the same style. An id on the variants is future wanted.
- **Q-3:** no ids on prefab controls. Future wanted.
