# Implementation handoff — RFC-128: a notice's action is a ghost, and drawn text is guarded

**RFC.** `rfcs/accepted/128-a-notices-action-label-is-drawn-in-a-non-text-colour.md`. Read the finding,
R-1 to R-3 and the Rulings.
**Release target.** 0.62.0. **After the audit follow-up's Unit 2 lands; before its Unit 3.** One
branch: Unit 0 is its own review, and if it is small and clean the review may let Unit 1 follow
on the same branch.
**Branch.** `rfc128-notice-action`, off `main`. Land as `main` requires:
1. rebase onto `main`;
2. push the rebased commits to a new branch name;
3. wait for CI green there;
4. fast-forward with `git push origin <sha>:main`.

## The rule

> **A notice's action is a ghost action, and is drawn as every ghost is:** its label in
> `palette.accent`, whatever the notice's tone. The tone shows in the notice's bar and border.

## What exists

- `crates/snora-widgets/src/design/notice.rs`:
  - `notice_accent_color(tokens, tone)` (`:268`), the tone's colour, which serves the bar, the
    border and, today, the action's label;
  - the action is `design_control(t, text(act.label).size(…).color(accent_color), ghost, …)` (`:185`
    onward). **The `.color(accent_color)` override is the defect.**
- The ghost style (`crates/snora-style/src/button.rs:138`) draws its label in `palette.accent`,
  tinted 8% and 14% behind it when hovered and pressed.
- Unit 2's measurement and its sweep, ready to become R-2:
  `.git-exclude/review-request/audit-0-62-unit2/evidence/h5-notice-action-contrast.test.rs.txt`,
  `h5-run.txt`.

## Unit 0 — measure

| Measure | Expected |
|---|---|
| on `main`, every tone × every preset, the action label's drawn colour against `surface`, against the floor of the preset's text class: 4.5, or **7 on `light_aaa` and `dark_aaa`** | fails: the four Neutral cells, and Success, Warning and Danger on `light_aaa` (6.51) and Danger on `dark_aaa` (6.54) |
| with the override removed (R-1): the same, **idle, hovered and pressed** (the ghost's tint is behind the label) | every cell at or above its floor |
| **R-2's scope:** every place a design widget gives text a colour of its own (`.color(…)` on a `text`, or a style's `text_color` other than the palette's text roles) | list them. RFC-128 found only this one. Confirm or correct it |

**Measure contrast on drawn pixels, not only on the palette's values,** for at least one cell per
preset, since the label is anti-aliased onto a tinted background. Say how the two compare.

## Unit 1

- **R-1:** remove the override. Nothing else in `Notice` changes. The bar and the border keep the
  tone colour.
- **R-2: the sweep, in the tree.** Every text a design widget draws in a colour of its own, every
  preset, against the preset's text floor. Start from Unit 2's file. **It must fail on today's
  `main`** (the breakage row: put the override back).
- **A test that the tone still shows:** a notice's bar and border are drawn in the tone's colour,
  for each tone. The fix must not erase the cue it moves to.
- **CHANGELOG `[Unreleased]`, Changed:**
  - a notice's action is drawn as a ghost, in the accent colour, for every tone;
  - why: Neutral's label was 3.1:1, and the AAA presets' were under their 7:1 (RFC-128);
  - screenshot baselines of notices with actions change.
- **The book is the architect's** (R-3).

## Acceptance criteria

1. Unit 0's table, reviewed.
2. Unit 1: every breakage row shown failing. `render_semantics.rs` is unmodified.
3. All of `ci.yaml`'s Rust quality gate commands (17, and Unit 2's new steps), the Docs job's
   commands, the feature matrix, both rendered jobs, and parity. Then the branch CI.
