# RFC-121 — A click gives a control keyboard focus, and a held key presses it again and again

**Status.** Done — shipped v0.56.0. Accepted 2026-11-08. Q-1, Q-2 and Q-4 ruled as suggested; Q-3 ruled otherwise:
no patch, the fix ships in 0.56.0 (see Rulings).
**Handoff.** [`handoffs/121-…`](../handoffs/121-a-click-gives-keyboard-focus-and-a-held-key-repeats/implementation-handoff.md).
**Raised.** 2026-11-08, architect, while re-reviewing RFC-120 against the owner's
fundamental philosophy, at the owner's request.
**Release target.** 0.56.0 (Q-3, owner).
**Merged to `main` 2026-11-08** (`b74ab3e`), with R-2 (Tab carries on after a click) built within its stop
condition.
**Priority.** **0.55.0 changed what every pointer user sees and what a key does, in
every application, including those that never route Tab. Its release notes say it did
not.**

---

## The finding

**F-1. A click gives the control keyboard focus.**
- `control`'s `update`, on a pointer or touch press, sets the control focused
  (`crates/snora-control/src/control.rs:511`). Its comment gives the reason: "so that a
  pointer user who then reaches for the keyboard carries on from where they were".
- `draw` draws the ring whenever the control is focused (`:631`), whatever focused it.
- So in **every** 0.55.0 application:
  - **a focus ring appears on every snora button a mouse or touch user presses,** and
    stays until they press somewhere else. Before 0.55.0 there was none;
  - **Enter or Space then presses that control again.** iced delivers key presses to
    every widget, and a focused `control` answers them whether or not the application
    routes Tab. A user who clicks "Send", and then presses Space for any other reason,
    sends again.

**F-2. A held key presses again and again.** `update` answers every `KeyPressed` for
Enter or Space and ignores its `repeat` field (`:537`; iced reports it,
`iced_core/src/keyboard/event.rs:34`). Holding Enter on a focused `danger` button
presses it repeatedly. A pointer press never repeats. Through F-1, this also reaches
applications that never route Tab.

**F-3. The release notes say otherwise.**
- `CHANGELOG.md`, 0.55.0, Added: *"Nothing changes for an application that does not
  route Tab: its controls still work by pointer and touch as before."*
- The lead: *"a focus ring appears on the control the keyboard has reached"*.
- The 0.54 → 0.55 guide's table: *"Route Tab (§1). Without it, nothing changes."*

All three are false under F-1. **Nothing measured it:** every keyboard test reached
controls by Tab or by an operation, and no test clicked and then looked for a ring or
pressed a key. The architect wrote the claims and did not check them against the
pointer path.

**Who is affected.** Every application on 0.55.0. It is one day old, and no adopter is
known to have upgraded yet.

## Proposal

**R-1. A pointer press does not give keyboard focus.**

> **Clicking presses a control. It does not focus it. Focus moves only by the keyboard
> (Tab, through `traverse`) or by the application.**

- This is 0.54.0's behaviour for a pointer user, exactly, so F-3's sentences become true.
- It is also the behaviour a reader can predict from what they did: no ring appears
  from a click, and no key answers for a control they only clicked.
- A press elsewhere still takes focus away from a focused control (`:490`), as now.

**R-2. Tab after a click carries on from the clicked control, where it can.** The
comment at `:511` had a real goal: a pointer user who reaches for the keyboard should not
start again from the top. Browsers meet it without focusing: a click sets a **starting
point for Tab**, and the next Tab moves to the control after it (HTML's "sequential
focus navigation starting point"). **If Unit 0 finds that `traverse` can honour such a
starting point without iced's `advanced` feature,** a click sets one. Otherwise, Tab
after a click starts from the top, and the book says so (Q-2).

**R-3. A held key presses once.**

> **A key press activates a control once. Holding the key does not repeat it.**

A `KeyPressed` with `repeat: true` presses nothing. This is RFC-119's principle, "the
keyboard does what the pointer can", applied to time.

**R-4. The record is corrected where readers will look** (RFC-067's obligation).
- The 0.54 → 0.55 guide gains a correction stating what 0.55.0 actually did. **What to
  re-check:** screenshot baselines taken on 0.55.0 after a click, which show a ring that
  0.55.1 removes.
- The 0.55.1 CHANGELOG entry says the same.
- The 0.55.0 entry is not rewritten; released entries are history. The guide is
  canonical (RFC-080).

## Open questions, with suggested rulings

**Q-1 — R-1, or "focus-visible"?** Browsers' `:focus-visible` focuses on click, but
shows the ring only once the keyboard is used. **Suggest R-1.**
- Focus-visible is a heuristic. Whether a ring appears depends on what the reader did
  last and how, which they cannot predict from the screen.
- It also leaves the stray Space or Enter pressing a control the reader only clicked,
  with no ring to warn them.
- R-1 is one sentence, matches 0.54.0 for every pointer user, and its one loss is met
  by R-2.

**Q-2 — R-2's starting point.** **Suggest yes, if Unit 0 shows it is feasible without
`advanced`.** If it is not, ship R-1 and R-3 without it, and record the starting point
in RFC-116's long-term plan, with the trigger "an iced release that exposes a focus
starting point, or an adopter who asks".

**Q-3 — a 0.55.1 patch?** **Suggest yes.**
- **Every** application's pointer users see the ring.
- The repeating press reaches destructive buttons.
- A published claim is false.
- The fix is small, and confined to `control`: R-1 and R-3 are a few lines each, plus
  tests. R-2 is included only if Unit 0 finds it small; otherwise it moves to 0.56.

**Q-4 — the two letters waiting to be sent (aaai, orbok).** **Suggest holding them
until 0.55.1 is out, and naming 0.55.1 in them** as the version to take. Neither letter
repeats F-3's claim, but both point at 0.55.0. No other letter: a day-old release that
no adopter is known to have taken does not meet the correspondence bar. The guide's
correction reaches everyone.

## Review against snora's principles

- **Nobody is confused.** A click does what it did in 0.54. A ring appears only where
  the keyboard put focus, so it always means the same thing.
- **Safe.** No key answers for a control the reader only clicked, and a held key presses
  once. Both stray-press paths are closed.
- **Clean.** It removes behaviour rather than adding modes. There is no new API, and
  nothing to deprecate.
- **Robust.** Unit 1 adds the tests that would have caught this: click, then look for a
  ring; click, then press Space; hold a key. Each has a breakage row checked that it can
  fail.
- **Honest record.** The false claims are named, with what to re-check (R-4), not
  quietly reworded.

## What this is not

- **Not RFC-120** (focus after the focused control goes away). That stays in 0.56. RFC-120
  no longer carries the held-key rule, which moved here.
- **Not a change to Tab, `traverse`, or RFC-119's overlays,** apart from R-2's starting
  point.

## Acceptance criteria

1. **Unit 0 (short):** reproduce F-1 and F-2 as failing tests, and answer R-2's
   feasibility.
2. **Unit 1:** R-1 and R-3, and R-2 if feasible, with breakage rows. Every existing
   keyboard test still passes, unmodified. A test that relied on click-focus is reported
   to the architect, not edited.
3. **The architect:** the guide's correction, the 0.55.1 CHANGELOG lead, and the cut.

## Rulings, 2026-11-08 (owner)

- **Q-1:** R-1. A click presses a control and does not focus it.
- **Q-2:** R-2's starting point, if Unit 0 shows it is feasible without `advanced`.
  One lead to check: `iced_runtime::task::widget(operation)` is public in
  `iced_runtime` (`task.rs:423`), which `snora` already uses as a dev-dependency.
  Making it a normal dependency would add an edge, not a package.
- **Q-3: no 0.55.1. The fix ships in 0.56.0,** which starts now and is to be reached as
  soon as possible. The reason is cost balance. 0.55.0's behaviour stands until then.
  **The guide's correction (R-4) is not held back:** the book is live, and a false
  claim on it is corrected now.
- **Q-4:** the letters to aaai and orbok are held, and they name **0.56.0** as the
  version to take.
