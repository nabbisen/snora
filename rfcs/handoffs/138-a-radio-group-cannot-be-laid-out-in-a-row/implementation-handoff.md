# Implementation handoff — RFC-138: `horizontal(bool)`, a radio group laid out in a row

**RFC.** `rfcs/accepted/138-a-radio-group-cannot-be-laid-out-in-a-row.md`. Read the finding, R-1 to R-4 and the Rulings
(every question as suggested).
**Release target.** 0.66.0, as an addition.
**Starts** now (owner, 2026-11-26). **It comes before the 0.67 audit's request A**, since it ships in 0.66. H-1
(`rfcs/handoffs/housekeeping-0-66/implementation-handoff.md`) may go in the same review request or beside it.
**Where.** On `main`, by the one-`main` route, **after the review approves it**. Probes are scratch, described and not
committed.
**Order.** Send Unit 0 for review first. Unit 1 starts after that review rules on Q-2 (the space between options).

## Where the change goes

- **One place serves both paths.** `crates/snora-widgets/src/form.rs`: `snora::radio_group` and
  `snora::design::radio_group` both build `RadioGroup`, and differ only in their `Skin`. A `horizontal: bool` field and
  a `horizontal(bool)` builder next to `disabled(bool)` cover both. `snora` re-exports the type, so nothing changes
  there.
- **The layout** is in `From<RadioGroup> for Element`: today `column(rows)`, aligned to the start edge. With
  `horizontal(true)`, the same `rows` go into a row that wraps. Each option is already a `LogicalRow`, so the mark and
  its label mirror inside it under RTL, as today.
- **The keys are not touched.** The `on_key` closure reads only the option index and `direction`, so it serves both
  layouts as it is. A change there is a finding to report, not a step.
- **The doc comment** on `RadioGroup` and on `horizontal`: the column is the default; the row always wraps; the keys are
  the same in either layout, and in a wrapped row ↓ is "next", not "the option below".

## What to find out, not assume

1. **Line-by-line mirroring under RTL.** `snora_interaction`'s `MirroredRow` (`mirror.rs`) reflects each direct
   child's x within the row's width. Over a wrapping row whose width is its widest line, that reflection *should* give
   R-2 exactly: the first option at the right end of the first line, and each shorter line starting from the right
   edge. Whether `MirroredRow` accepts iced's `Wrapping` (`Row::wrap()`), and whether its children are the options,
   is for Unit 0 to measure. If it does not fit, report it with what you measured. Do not build a second mirroring
   mechanism without a review.
2. **Never reverse the options to mirror them.** The tree must stay in logical order (RFC-132): iced keeps state by
   tree position, and a reversed row also breaks the lines in the wrong place (the last options would be on the
   first line).
3. **The width.** The group is `width(Length::Shrink)` today, and its ring hugs the options. Measure that a horizontal
   group wraps against the room its parent gives it (a narrow column, a narrow window), and that the ring hugs the
   drawn lines, not the parent's full width.
4. **No new public API** beyond `horizontal(bool)`. `LogicalRow` deliberately has no `wrap`, so do not add one. Report
   anything else the work seems to need.

## Rules to implement exactly

1. **`horizontal(false)` is today's group, pixel for pixel.** The existing form-control tests pass unchanged.
2. **The options keep their logical order, line by line,** and sit at the start edge of each line. No option is ever
   clipped or squeezed (RFC-105).
3. **The space between lines is the column's.** The column has no spacing between rows; each option carries its own
   6 px vertical padding. Lines stack at the column's pitch.
4. **The space between options** is the value Unit 0 proposes and the review rules (Q-2): it is a token on the design
   path and one constant on the engine path, and `scripts/check-literal-sizes.sh` passes. It must be wider than the
   space between a mark and its own label (`Skin::gap`), so that a label never reads as belonging to the next option.
5. **The keys, a click, and Space and Enter behave as in the column:** one Tab stop; ↓ and → move to the next option, ↑
   and ← to the previous one, wrapping at the ends, with ← → following the screen; a click on the selected option, and
   Space or Enter, publish nothing (RFC-134 ruling 3).
6. **Every floor holds** (R-4): the ring at 3:1, each option's target at least 24 px, the labels at the palette's level,
   and the mark at 3:1 against the page and the row's tints.

## Unit 0 — the layout, measured (no product code committed)

On a prototype, on **both renderers, both directions and both paths**, with two, three and six options, at a wide
window and a narrow one, measure:

- where each option is drawn, line by line (from pixels, not from the tree);
- each option's pointer target;
- the ring around the group, and what it encloses;
- the keys, with `every_control.rs`'s probe;
- what is clipped, which must be nothing;
- **Q-2's candidates,** drawn: at least two token choices on the design path and two constants on the engine path,
  each with the measured space between options next to the space between a mark and its label. Propose one for each
  path, and say why.

**Report** finding 1 above (does `MirroredRow` give the line-by-line mirror?), anything R-1 to R-4 does not fit, and
any public API beyond `horizontal`.

## Unit 1 (0.66.0)

- R-1 to R-4, with Q-2's ruled value.
- **The drawn sweeps** (`text_sweep.rs`, `nontext_sweep.rs`) take a horizontal group in, on both paths.
- **Tests**, in the form-control tests or a file of their own:
  - **horizontal:** the options are drawn on one line when there is room;
  - **narrow:** they wrap, nothing is clipped, and the lines are at the column's pitch;
  - **mirrored:** under RTL, each line is the mirror image of its LTR line, and the lines break at the same options;
  - **gap:** the space between options is wider than the space between a mark and its label, measured from pixels;
  - **keys:** the same messages in either layout, for every arrow, with and without a selection;
  - **the column unchanged:** `horizontal(false)` draws as the group does without the call.
- **Breakage rows,** each shown failing:
  - ignore the setting (the horizontal test fails);
  - a row that does not wrap (the narrow test fails);
  - no mirroring (the mirrored test fails);
  - mirror by reversing the options (the mirrored test fails on where the lines break);
  - the space between options equal to `Skin::gap` (the gap test fails);
  - a spacing between lines (the pitch check fails);
  - ↓ ignored in a row, as though it meant "down" (the keys test fails).
- **CHANGELOG `[Unreleased]`, Added.**

## Acceptance criteria

1. Each test is shown able to fail. Use named `cp` backups and `sha256sum -c`; never `git checkout` or `git restore`.
   Run the breakage script again on the final tree.
2. The existing form-control tests (`form_controls.rs`, `form_unstyled_drawn.rs`) pass unchanged.
3. Every `ci.yaml` command, run locally, before pushing to `main`, and only once the review has approved it.
4. **The book is the architect's** (the RFC's criterion 3): the keyboard guide's form-control table and the radio
   group's line on the keys.
