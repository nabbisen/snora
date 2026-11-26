# RFC-137 — A keyboard press on a disclosure leaves nothing focused

**Status.** Proposed, 2026-11-26.
**Raised.** 2026-11-26, architect, from logolig's letter of 2026-11-22. They found it on 0.65.0 and
then withdrew the request, because they found a workaround in the application. It was measured true
on `main` at `2be698f`: see the triage,
`.git-exclude/upstream/receive/0.65.0/architect-triage-2026-11-26.md` (D-1), and its probe.
**Release target.** 0.66.0, as an addition (no breaking change).
**Priority.** Keyboard compatibility is required (owner, 2026-11-04). The owner asked for this RFC
as the first item of 0.66 on 2026-11-26.

---

## The finding

A **disclosure** is a control that stays where it is when pressed and shows or hides other controls
after it. Examples are "Advanced settings", a tree node, and a "show details" row. Built on
`snora::control` inside `snora::render`:

| Step | Focus |
|---|---|
| Tab to the header | on the header (target 0 of 2) |
| Enter: the application opens it, and two options appear after it | **nothing** (0 of 4) |
| one more Enter | publishes nothing |
| Enter again from the header: it closes, and the two options go | **nothing** (0 of 2) |

The next Tab starts at the top of the window. This contradicts the book, not a gap in it: the rule
(RFC-120; keyboard guide, "Where focus goes"; accessibility guide) reads *"**When the focused control
goes away**, focus moves to its nearest neighbour …"*, and here the focused control does not go away.
A reader predicts that focus stays on it.

**Why.** After a press of the focused control, `Inert` compares the layer's focus targets before and
after the rebuild (`inert.rs`, `resolve`):
- if the count is unchanged, focus returns to the same position;
- if not, the targets that went must be the span the control declared (`removes`, which is internal:
  RFC-122 R-3), and otherwise nothing is focused.

A disclosure changes the count without the control going, so it lands on "nothing".

**Who it hits.**
- **Every application that builds a disclosure on `snora::control`.** This includes **aaai's file
  tree**, one of RFC-122's three cases: expanding a folder adds rows after it.
- **None of snora's own widgets.** The crumb, the notice and the chip declare `removes`, and the
  select's list is in another layer.

**Why snora cannot infer it.**
- Position and count cannot tell a disclosure (the control stays) from a replacement (the control
  goes and others come).
- iced keeps widget state by position, so a wrong guess hands focus to whichever control moved into
  the place. That is 0.55's defect, which one more Enter turned into a second press.
- RFC-120's answer to "snora cannot tell" is "nothing is focused", and it stays the answer for an
  undeclared change. This RFC adds the declaration that lets the application tell.

**logolig's workaround** is to focus the header by `Id` in the same `update`. It works, and it
matches the documented order: `Inert` settles before an operation that comes first, then the
application's focus wins. But it also runs after a **click**, so the ring appears where a click
should leave none. They asked to expose `on_focus_change` to tell the two apart. Fixing this
removes the need (triage, D-2).

## Proposal

**R-1. A declaration on the public control: `keeps_focus(id)`.** It goes on `snora::Control`, which
`snora::control` and `snora::design::control` both build, and on the internal
`snora_interaction::Control`.

```rust
snora::control(text("Advanced settings"))
    .on_press(Message::ToggleAdvanced)
    .keeps_focus("advanced-settings")
```

The doc comment's sentence: *"Pressing this control never removes it. After a press **from the
keyboard**, focus stays on it, whatever else the press adds or removes. It is found by `id`, so the
id must name what the control is, not where it sits."*
- It sets the control's id: it is `.id(id)` plus the declaration. The same id is what iced's
  `focus(id)` reaches (RFC-127).
- The application cannot declare it without giving the identity it needs.

**R-2. How `Inert` resolves it.** After a press of the focused control that declared
`keeps_focus(id)`:
- **The application's own focus still wins.** If focus is anywhere but the pressed position, the
  application put it there, as today.
- **Otherwise, focus goes to the focus target whose id is `id`,** wherever it now is in Tab order,
  so controls added or removed *before* it do not matter.
- **If no target has that id after the rebuild** (the control did go away, against its
  declaration), **nothing is focused.** Misuse falls to the safe answer, never to a neighbour that
  inherited the position.
- **The count is not consulted.** The declaration replaces RFC-120's removal arithmetic for this
  press.

**R-3. Nothing else changes.**
- An undeclared press that changes the count still focuses nothing.
- A **click** still focuses nothing. Tab carries on from the clicked control (RFC-121), whose start
  point survives the rebuild.
- A control that declares nothing and changes nothing still keeps focus, as today.

**R-4. The book.**
- **"Where focus goes"** gets a row: *"a control that keeps focus (`keeps_focus(id)`: a disclosure,
  a tree node): it keeps focus while others come or go."*
- **The RFC-120 text** in the accessibility guide gets the disclosure case and its one line.
- **The keyboard guide's "Your own content"** gets a compiled snippet (RFC-136) and the id rule:
  name the item (a node's path), not its row number.
- **Release news** for logolig (their 1.28.0 notes list this as a known limitation) and aaai (the
  file tree).

## Also found: a control that replaces itself

**"Show more"**: a press removes the control and adds items in its place. This is not R-1's shape,
because the control goes. Today nothing is focused, by RFC-120's rule. The usual expectation is the
first new item.
- **Suggest:** no new API. snora cannot know which new item is first without a declaration of its
  own.
- **The book gives the route that works today:** focus the first new item by its id in the same
  `update`. The application's focus wins (R-2's first bullet).
- If more cases appear, it becomes an RFC of its own.

## Open questions, with suggested rulings

**Q-1 — the shape: `keeps_focus(id)`, or a bare `stays()` resolved by position?**
**Suggest: `keeps_focus(id)`.**
- A bare `stays()` must find the control by its place in Tab order. That is right only if nothing
  before it came or went.
- Misused on a control that does go away, it would focus the neighbour that moved into the place,
  which is 0.55's defect.
- Identity by id makes both cases safe. The cost is one id, which a disclosure in a list needs
  anyway to be reached by `focus(id)`.

**Q-2 — the name.** **Suggest: `keeps_focus`.** It says what the reader gets.
- `stays` says what is true, but not what it does.
- logolig's `adds(n)` covers opening but not closing, and asks for a count snora does not need.

**Q-3 — public on both paths?** **Suggest: yes.** It goes on `snora::Control`, which both paths
build. It is the first press declaration made public. `removes` stays internal (RFC-122 R-3),
because its span arithmetic is what this RFC shows to be fragile for application content.

**Q-4 — `on_focus_change` on `snora::Control` (logolig's remaining request).** **Suggest: no, not
for this.** R-1 removes the workaround it was needed for. A pointer/keyboard signal built on focus
changes is indirect. If some other use for it appears, weigh it then on its own case.

**Q-5 — when.** **Suggest: 0.66.0,** as an addition. Unit 0 can start on acceptance.

## Review against snora's principles

- **Keyboard.** A keyboard user keeps their place, which is required. A pointer user's experience is
  unchanged (R-3).
- **Nobody is confused.** The book's own rule ("when the focused control goes away") becomes true
  for the case it already implied. The declaration states the fact it relies on, and its argument
  is the identity it needs, so it cannot be half-declared.
- **Safe.** Misuse focuses nothing, not a stranger. An undeclared change keeps RFC-120's safe answer.
- **ABDD.** It resolves by id, not by position, so it is the same under LTR and RTL.
- **Robust.** One mechanism, `Inert`'s resolve step, gains one branch. No new tracker, and no
  second focus path.
- **Cost of correspondence.** None now. One line in the release news that ships it.

## Units

**Unit 0: a probe and a prototype.**
- Probe these shapes on both renderers:
  - a disclosure opening and closing;
  - aaai's file-tree row expanding and collapsing a folder, rows after it;
  - a disclosure with a control inserted before it in the same press;
  - a misdeclared control that does go away;
  - a click on each.
- Read focus three ways, as the removal tests do: the tree's flags, the ring from pixels, and what
  one more Enter publishes.
- Report anything R-1 to R-3 does not fit.

**Unit 1: the implementation.**
- R-1 to R-3, on both public paths and the internal control.
- Tests for each Unit 0 shape, plus the undeclared and click cases unchanged.
- Each test gets a breakage row: drop the branch, and resolve by position instead of id.

## Acceptance criteria

1. **Unit 0:** every shape above measured before and after, and a verdict on Q-1 from the measurements.
2. **Unit 1:** after a keyboard press, a declared disclosure keeps focus; a misdeclared one focuses
   nothing; an undeclared change and a click are unchanged. Each guard is shown able to fail.
3. **The book,** by the architect: R-4.
