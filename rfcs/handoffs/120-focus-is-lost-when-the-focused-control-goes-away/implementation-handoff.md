# Implementation handoff — RFC-120: focus after the focused control goes away, and the toast hold

**RFC.** `rfcs/done/120-focus-is-lost-when-the-focused-control-goes-away.md`. Read R-1
(the group-first neighbour rule), R-2, R-3, Q-4 and the Rulings.
**Release target.** 0.56.0.
**Order:**
- **Unit 0 may start now,** in parallel with RFC-121. It measures the keyboard path, which
  RFC-121 does not change.
- **Unit 1 builds on RFC-121,** once that is merged: R-1 relies on a held key pressing
  once.

**Branch.** `rfc120-focus-after-removal`, off `main`. Rebase it onto `main` after
RFC-121 merges.

## Unit 0 — measure today, then find the mechanism

### 1. What 0.55.0 does today

RFC-116's L-1 said "focus goes nowhere". **That was the architect's inference, never a
measurement,** and iced's positional tree state suggests otherwise: the control that
moves into a removed one's place may inherit its focus. Measure each case on both
renderers. For each, give:
- what `find_focused` reports after the rebuild;
- where the ring is, read from pixels;
- what one more Enter does.

| Case | Setup |
|---|---|
| a removable chip's ×, in a row of three, for the first, middle and last chip | a `row` of `design::chip::removable` |
| a notice's dismiss | a notice between two other controls in a `column` |
| a toast's ×, for the first, middle and last of three | the engine's toast stack |
| an ancestor crumb | a three-crumb trail; pressing the first ancestor changes the view |
| an application's own list row | three rows, each with a delete `design::button`, in a `column` |

**Expect some cases to show focus on a control the user never reached.** If so, that is
the finding the RFC's "Safe" bullet anticipated. State it plainly.

### 2. Prototype the mechanism for R-1

The RFC names three candidates. The constraints are known:
- a control's `Id` is new on every `view` (RFC-119 Unit 0);
- position and count cannot say which control went.

| Candidate | What to find out |
|---|---|
| (a) **The removing control hands focus forward on its press.** It knows it is a dismiss, close or remove control | Can it, at the press, mark the neighbour R-1 names, so that the mark survives the application's rebuild (a state flag on that control?) and is resolved at the next `RedrawRequested`, as `Inert`'s restore is? The group-first rule needs the neighbour *within the group*: can the press see its group? |
| (b) **A tracker in `Inert`,** always present around the page | Can it record the Tab order with something that survives a rebuild, and find the removed control's neighbour? |
| (c) **Per-container knowledge** | Works for the toast stack and the breadcrumb bar, which snora owns. Not for chips or notices, which the application places |

**Report which candidate gives R-1 for which cases.** Then the scope answer for Q-2:
- every focus target;
- snora's removal controls only;
- or less.

**If the scope narrows from "every focus target", it returns to the owner,** with your
measurements, as the ruling says. Do not narrow it yourself.

**Watch for the trap RFC-119 found:** a test that asks `find_focused` can only see what
`operate` reports. Where the question is "is a ring drawn there", read pixels.

## Unit 1 — R-1 to R-3. **Ruled 2026-11-08; RFC-121 is on `main`. Start now.**

Read the RFC's last section, "After Unit 0", which the owner confirmed. The rule:

> **When the focused control goes away, focus moves to its nearest neighbour in the
> same group: the next one, or else the previous one. Otherwise, nothing is focused.**

**The mechanism, as Unit 0 found and the review accepted:**
- **A press mark on `Control`:** one `bool`, read through `operation.custom`, as RFC-121's
  `start_here` is. `Inert` acts only after a press, and never on a `text_input`'s Enter.
- **The tracker in `Inert`:**
  - `observe` after a press: survey, remember, and take focus off everything;
  - `resolve` at the next `RedrawRequested`: the application's focus wins; an unchanged
    count means the same index; otherwise R-1, or nothing focused.
- **The declared spans,** at the four snora sites: chip × `(1,0)`, notice dismiss `(1,0)`,
  toast × `(0,0)`, crumb `(0, later crumbs)`. An undeclared removal is `(0,0)`.
- **The group (D-3):** the innermost container that held the control and is still there,
  **never wider than its layer.** The page, a dialog, a sheet and the toast stack are each
  a boundary. With no control left in the group, nothing is focused. **Remove the outward
  search** your prototype had: root crumb, nothing focused; last toast, nothing focused.
- **Toasts by identity (D-2).** The toast stack knows each toast by its `u64` id:
  - focus on toast *k*'s × stays there when another toast expires or is closed;
  - R-1 for a toast's × uses the ids, not a span;
  - add 1b's toast rows as tests: toast 3 expires while toast 2's × is focused, and
    toast 2's × keeps focus.
- **Not covered, a stated limit (L-7):** in an application's own list, a removal
  elsewhere. Add **no** test that claims it is fixed. Unit 0's 1b page rows stay as a
  characterisation, with their comments saying "L-7, a known limit".

**Tests:**
- every row of Unit 0's table, against the ruled targets;
- RTL;
- the application's own focus winning;
- the group boundary: the root crumb and the last toast leave nothing focused, and never
  the footer or the page;
- a removal of more than the pressed control, undeclared: nothing focused;
- **the timing question you raised.** Does a focus task arrive after the first
  `RedrawRequested`? Measure it in a real runtime order. If it does, show that the
  application's focus still wins, and say whether one frame shows R-1's ring first.

**R-3, the hold.** As Q-4 (b) ruled:
- `snora_core::ToastAttention`, `#[non_exhaustive]`, with:
  - `PointerEntered` and `PointerLeft`;
  - `FocusGained` and `FocusLost`, for any control in the toast;
- `AppLayout::on_toast_attention(fn(u64, ToastAttention) -> Message)`;
- `snora::toast::Holds::attend(id, ToastAttention, &mut toasts, now)`. It holds while
  **either** source is present, and restarts the countdown in full **only when both
  have gone**;
- **deprecated, as superseded:** `AppLayout::on_toast_hover` (and its public field),
  `ToastHover`, and `Holds::hover`, `#[deprecated(since = "0.56.0", note = …)]`. Each
  note names its replacement;
- **while both callbacks are set,** the pointer's crossings go to `on_toast_attention`
  only;
- how the toast's control reports focus gained and lost is yours. A control sees its
  focus change at the next update.

Tests:
- hold by pointer alone;
- hold by focus alone;
- both, then one leaves: still held;
- the other leaves: restarted in full;
- both callbacks set: one hold per crossing;
- a deprecated path still works.

## Breakage rows (both units)

Make named `cp` backups, apply the break, run, restore, and finish with `sha256sum -c`.
**Never `git checkout` or `git restore`.** Check that each row *can* fail before relying
on it. Two specified breakages could not, in the last three units.

| Assertion | Breakage |
|---|---|
| focus lands on the next neighbour | take the previous one first |
| group first: a crumb press keeps focus in the bar | search the whole window first |
| the application's focus wins | drop the check |
| hold while either source is present | release on the first source leaving |
| restart in full only when both have gone | restart when one leaves |
| one hold per crossing with both callbacks set | send to both |

## Docs

- **Rustdoc:** the removal controls (chip, notice, toast) state where focus goes, and
  `Holds::attend` states the hold.
- **CHANGELOG `[Unreleased]`:**
  - Fixed: focus after a removal;
  - Added: `ToastAttention`, `on_toast_attention`, `Holds::attend`;
  - Deprecated: the three hover items.
- The book and the migration guide are the architect's.

## Acceptance criteria

1. Unit 0: the measurement table on both renderers, the candidates' results, and the
   scope answer. Reviewed.
2. Unit 1: every test and breakage row above, and `render_semantics.rs` unmodified.
3. All of `ci.yaml`'s Rust quality gate commands, the Docs job's commands, the feature
   matrix, both rendered jobs and parity. Then the branch CI.
