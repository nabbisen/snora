# Implementation handoff — RFC-137: `keeps_focus(id)`, focus that stays on a disclosure

**RFC.** `rfcs/done/137-a-keyboard-press-on-a-disclosure-leaves-nothing-focused.md`. Read the
finding, R-1 to R-3 and the Rulings (every question as suggested).
**Release target.** 0.66.0, as an addition.
**Starts** now (owner, 2026-11-26).
**Where.** On `main`, by the one-`main` route. Probes are scratch, described and not committed.
**The defect's probe:** `.git-exclude/upstream/receive/0.65.0/evidence/zz_probe_logolig.rs`
(`probe_disclosure`), with its output beside it. It was measured on `2be698f`.

## Where the change goes

- **`crates/snora-interaction/src/inert.rs`, `resolve`.** This is the one new branch. Today it
  checks, in order: the application's focus wins, then an unchanged count goes back to the same
  position, then the declared span goes through `nearest_in_group`. The new branch comes after
  "the application's focus wins" and before the count. **If the pressed control declared
  `keeps_focus`, focus goes to the target whose id is its id, and the count is not read.**
- **`Track`** sees each target's id in `focusable(id, …)` and drops it today. Record it in `Seen`.
- **The declaration rides on the press mark,** as `removes` does (`PressMark`). Model it as one
  press effect that is *either* a removal span *or* "keeps focus". The two must not be settable
  together, and the type should say so.
- **An id matches only if the application set it.** A control without one has `Id::unique()`, made
  afresh each build (`control.rs`), so it never matches after a rebuild. Do not rely on this alone,
  though: `keeps_focus` always sets the id (R-1).
- **The public builder,** `crates/snora/src/control.rs` (`snora::Control`, which `snora::control` and
  `snora::design::control` both build), gets `keeps_focus(id: impl Into<widget::Id>)`. It is
  `.id(id)` plus the declaration. If `.id(..)` is called after it, the later id is the control's id
  and the one resolved. Say so in the doc comment.
- **The internal `snora_interaction::Control`** gets the same method.

## Rules to implement exactly

1. **Keyboard press only.** This is `observe`'s existing gate: the pressed control was focused. A
   click focuses nothing, unchanged, and Tab carries on from the clicked control (RFC-121).
2. **The application's focus still wins** (a focus task, or a click into a text input).
3. **Found by id:** focus the one target with that id, wherever it is in Tab order.
4. **No target with that id, or more than one: nothing is focused.** Two targets with one id is a
   misuse snora cannot resolve, and "nothing" is the safe answer. Never fall back to the position.
5. **A covered layer** (the press opened a modal) is unchanged: `resolve`'s existing early return.
6. **A disabled control** registers no target, so the declaration has no effect. That is fine; say
   so in the doc.

## Unit 0 — the shapes, measured (no product code committed)

On a prototype, on **both renderers and both directions**, read focus **three ways**, as
`focus_follows_removal.rs` does: the tree's flags, the ring from pixels, and what one more Enter
publishes. Measure each shape before (0.65.0 behaviour) and after:

| Shape | Expected after |
|---|---|
| a disclosure opening (two controls appear after it) | on the header |
| the same closing (they go) | on the header |
| aaai's file-tree row (`width(Fill)`, an indent, an icon and a name) expanding and collapsing a folder | on the row |
| a disclosure, with a control **inserted before it** in the same press | on the header (by id) |
| a misdeclared control that does go away | nothing |
| two targets given the same id | nothing |
| a click on each of the above | nothing focused; the next Tab goes to the target after the clicked one |
| an undeclared disclosure (no `keeps_focus`) | nothing, as today |

**Report** anything R-1 to R-3 does not fit, and any public API beyond `keeps_focus`.

## Unit 1 (0.66.0)

- R-1 to R-3 on both public paths and the internal control.
- **A test file of its own,** on the removal rig (`common::removal`, `removal_scenes`), with a test
  per Unit 0 shape. Keep the existing removal tests unchanged and passing.
- **Breakage rows,** each shown failing:
  - drop the branch (the disclosure tests fail);
  - resolve by position instead of by id (the inserted-before test fails);
  - fall back to the position when the id is missing (the misdeclared test fails);
  - take the first of two matches (the duplicate-id test fails);
  - let a click take the branch (the click tests fail).
- **The doc comments** on both `keeps_focus` methods, and `inert.rs`'s module text ("After a
  removal", which should now be called *after a press*) for the new branch.
- **CHANGELOG `[Unreleased]`, Added.**

## Acceptance criteria

1. Each test is shown able to fail. Use named `cp` backups and `sha256sum -c`; never `git checkout`
   or `git restore`. Run the breakage script again on the final tree (C-1).
2. `render_semantics.rs` is unmodified.
3. Every `ci.yaml` command, run locally, before pushing to `main`.
4. **The book is the architect's** (RFC-137 R-4): "Where focus goes", the RFC-120 text, the keyboard
   guide's *Your own content* with a compiled snippet, and the route for "Show more".
