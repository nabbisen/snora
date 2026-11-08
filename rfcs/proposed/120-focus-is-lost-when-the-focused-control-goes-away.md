# RFC-120 — Focus is lost when the focused control goes away

**Status.** Proposed 2026-11-08. Awaiting the owner's rulings on Q-1 to Q-6.
**Raised.** 2026-11-08, architect. This is RFC-116's long-term plan item L-1, whose
trigger fired when RFC-119 was accepted. It also takes L-3, the toast hold for a
keyboard user. The owner accepted both as the core of 0.56 on 2026-11-08.
**Release target.** 0.56.0.
**Priority.** A gap in the keyboard access that 0.55.0 shipped. Keyboard compatibility
is required (owner, 2026-11-04).

---

## The finding

Several controls snora draws **remove themselves**, or are removed by the
application, when pressed:

| Control | Pressing it | What is left where it was |
|---|---|---|
| a notice's dismiss | the notice goes, and its action control with it | the controls after it |
| a toast's × | the toast goes | the toasts after it, or the page |
| a removable chip's × | the chip goes, its label control with it | the next chip, or whatever follows |
| an ancestor breadcrumb | it becomes the current page (plain text), and the crumbs after it go | the controls after the bar |

An application's own controls can go the same way: a row deleted from a list, say.

**What happens to keyboard focus then is not known, and that is the first finding.**
RFC-116's long-term plan said "focus goes nowhere, and the next Tab starts from the top
of the window". **The architect inferred that from reading the code, and never measured
it.** There is a reason to doubt it:
- iced 0.14 keeps a widget's state **by its position in the tree**;
- when one chip in a row goes, the chip after it moves into its position, and may
  inherit its state, the focus flag included (RFC-116 Unit 3 found the reverse: a
  changed shape dropped state).

So focus may go nowhere, or it may jump to **whichever control happens to inherit the
position**, which depends on how the application arranged its rows. Either way, a
keyboard user who removes a chip, closes a toast or dismisses a notice loses their
place, or lands somewhere they did not choose. **That is the confusion the owner's
rule forbids.**

**L-3, the toast hold.** A toast does not expire while the pointer rests on it
(RFC-113, `Holds`). A keyboard user who Tabs to its × has no equivalent: the toast can
expire while they are reading it. That leads into this finding: the toast vanishes, and
their focus with it.

## Proposal

**R-1. The rule a reader can predict.** It follows WAI-ARIA's guidance for removing an
item from a list:

> **When the focused control goes away, focus moves to the control that came after it
> in Tab order. If it was the last, focus moves to the one before it. If no control
> is left, focus is nowhere, and the next Tab starts from the top.**

What that gives, case by case:

| Case | Focus goes to |
|---|---|
| chip *k*'s × | chip *k* + 1's label; after the last chip, chip *k* − 1's × |
| a notice's dismiss | the first control after the notice |
| toast *k*'s × | toast *k* + 1's ×; after the last toast, the control before it |
| an ancestor crumb | the first control after the breadcrumb bar |
| an application's own control | the same rule |

**"Came after it" means in the Tab order as it was before the removal,** not as it is
afterwards. Only that version is the same to a reader whichever way the application
arranged its widgets.

**R-2. The application's own focus still wins** (RFC-119's rule A). If the application
moves focus itself in the same update, as with a focus task, R-1 does nothing.

**R-3. The toast hold, for the keyboard (L-3).** A toast does not expire while any of
its controls has keyboard focus, just as it does not while the pointer rests on it.
When neither holds it any more, its countdown restarts in full, as it does when the
pointer leaves. **One rule:** *a toast waits while the reader is attending to it, by
pointer or by keyboard.* How the application wires it is Q-4.

**R-4. The mechanism is found by measurement, as RFC-117's underline and RFC-119's focus
memory were.** Unit 0 measures what happens today, and prototypes. The constraints are
known:
- **A control's `Id` is new on every `view`** (RFC-119 Unit 0), so "the control that
  came after it" cannot be found by `Id` across a rebuild;
- **position and count, as RFC-119 used,** cannot say which control went when the count
  changes, because the count changes by definition;
- **the candidates are:**
  - (a) **The removing control hands focus forward when it is pressed**, before the
    application rebuilds. It knows it is a removal control (dismiss, close, remove), so
    at the moment of the press it can mark the target that follows it;
  - (b) **A tracker in the always-present page wrapper** (RFC-119's `Inert`), which
    records the Tab order with something that survives a rebuild;
  - (c) **Per-container knowledge** where snora owns the container: the toast stack and
    the breadcrumb bar. This does not reach chips and notices, which the application
    places.

  If none of these gives R-1 for an application's own controls, Q-2 decides the scope.

**R-5. A held key presses once.** Found while reviewing R-1 against the principles.
- **Today, in 0.55.0, every snora control presses again on each key repeat.**
  `control`'s `update` matches `KeyPressed` on Enter or Space and ignores its `repeat`
  field (`crates/snora-control/src/control.rs:537`; iced 0.14 reports it,
  `iced_core/src/keyboard/event.rs:34`). Holding Enter on a `danger` button presses it
  again and again. A pointer press never repeats.
- **With R-1 it is worse.** Holding Enter on a chip's × would remove that chip, move
  focus on, and press the next control, and so on down the row.
- **Rule:** *a key press activates a control once; holding the key does not repeat
  it.* That is the pointer's behaviour, so it is RFC-119's principle, "the keyboard
  does what the pointer can", applied to time.

## Open questions, with suggested rulings

**Q-1 — is R-1 the rule?** **Suggest yes.** It is the common pattern, WAI-ARIA's for list
items, and a reader predicts it from what they see. The alternative, "focus goes to the
container", has no target in iced, since a container takes no focus. "Back to the top"
is the state today, and the problem itself.

**Q-2 — scope: snora's controls only, or every focus target?** **Suggest every focus
target, if Unit 0 finds a mechanism that reaches them.** Otherwise, snora's own removal
controls, with the rest stated in the book. A rule that holds for snora's chips and not
for an application's list rows would be one more thing to remember. But a rule snora
cannot keep must not be claimed, so the scope follows the mechanism, and the owner rules
again if the scope narrows.

**Q-3 — removing the only control left.** **Suggest "nowhere", stated.** Nothing remains
to receive focus, and inventing a target would be a guess.

**Q-4 — wiring the toast hold (R-3).** `AppLayout::on_toast_hover` takes
`fn(u64, ToastHover) -> Message`, and `ToastHover` is not `#[non_exhaustive]`. A focus
variant added to it would break every application's `match`. Two options:
- **(a) A second, parallel path:** `on_toast_focus(fn(u64, ToastFocus) -> Message)` and
  `Holds::focus(...)`. Nothing breaks, but there are two callbacks and two methods for
  one idea, and an application can wire one and forget the other.
- **(b) One path for both:**
  - `on_toast_attention(fn(u64, ToastAttention) -> Message)`, with `ToastAttention`
    `#[non_exhaustive]` and naming the pointer's two crossings and focus gained and
    lost;
  - `Holds::attend(...)`, which holds while either source is present;
  - `on_toast_hover`, `ToastHover` and `Holds::hover` are **deprecated as superseded**,
    and keep working.

**Suggest (b).** One concept, one callback, and room for a third source without
another break. That is the "finally clean" design. Its cost is a deprecation one
release after `on_toast_hover` arrived (0.54). The migration is mechanical, and the
guide states it. Under (a), the forgotten second callback is exactly the kind of
misunderstanding the owner's rule forbids.

**Q-5 — release.** **Suggest 0.56.0,** the release the owner gave this item. If Unit 0
finds no mechanism for R-1 that holds, R-3 (the hold) can ship alone, and R-1 returns to
the owner with the measurements.

**Q-6 — R-5 now, as a 0.55.1 patch, or with 0.56?** **Suggest 0.56.0, with RFC-120.**
- It needs Enter held past the platform's repeat delay, about half a second, on a
  control whose repeated press does harm. Applications already guard destructive
  actions with a confirmation dialog, and RFC-119 makes the dialog hold.
- No adopter has routed Tab yet: 0.55.0 is one day old.
- It is one line and one test, though, so if the owner prefers it in users' hands
  sooner, a patch costs little.

## Review against snora's principles

- **Nobody is confused.** One sentence for focus (R-1), and one for the hold (R-3).
  Each holds whichever way the application arranged its widgets, because "after" is
  read from the Tab order the reader just moved through.
- **Safe.** Where focus lands after a removal is chosen, and never inherited from a
  tree position. Today, Enter after a removal may press a control the user never
  reached. R-1 replaces that with a control next to the one they pressed, which they
  were moving through.
- **The review found R-5.** Moving focus on after a removal is only safe if a held key
  cannot follow it. For chips, R-1 also lands on the next chip's **label**, not its ×,
  so even a deliberate second Enter toggles rather than removes.
- **Not yet safe: toasts.** A toast's × is its only control, so after closing toast
  *k*, a second deliberate Enter closes toast *k* + 1. With R-5 that takes a second
  press, after the reader has seen the next toast and its ring. An unread error toast
  still needs that deliberate press. This is accepted, and stated, rather than given a
  special case.
- **Clean.** The hold has one path (Q-4 (b)), and what it supersedes is marked
  deprecated, per the owner's rule.
- **Robust.** Unit 0 measures today's behaviour first, so the RFC is not built on an
  inference. RFC-116's L-1 was one, and this RFC says so.
- **ABDD.** "After" is Tab order, which follows the layout direction: under RTL, the
  next control is the one to the left, as Tab already moves.
- **Keyboard compatibility.** This closes L-1 and L-3 of RFC-116's long-term plan.

## What this is not

- **Not where focus starts** when a view opens. That is the application's.
- **Not the shortcuts router** (RFC-116 R-2, 0.57).
- **Not a focus trap or a focus history.** One step: the control that went, and its
  neighbour.

## Acceptance criteria

1. **Unit 0:** today's behaviour measured for each case in the finding table, plus one
   application-owned list removal, on both renderers. Then the three candidates
   prototyped, and the scope answer for Q-2.
2. **Unit 1:**
   - R-1 to R-3 as ruled, each case with a test and a breakage row checked that it can
     fail;
   - RTL Tab order;
   - the application's focus winning;
   - the hold from both sources, and a restart in full only when both have gone;
   - R-5: a repeated `KeyPressed` presses nothing, with the breakage row to honour
     `repeat`.
3. **The book** by the architect: the keyboard pages and `04-toasts.md`. **CHANGELOG**
   under Fixed (focus), and Added and Deprecated (the hold, if Q-4 (b)).
