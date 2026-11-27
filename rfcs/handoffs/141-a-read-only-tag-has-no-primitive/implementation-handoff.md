# Implementation handoff — RFC-141: `chip::tag`, a read-only tag

**RFC.** `rfcs/accepted/141-a-read-only-tag-has-no-primitive.md`. Read the finding, R-1 to R-3 and the Rulings (every
question as suggested).
**Release target.** 0.67.0, as an addition.
**Starts** after RFC-140's Unit 1 has been reviewed.
**Where.** On `main`, by the one-`main` route, **after the review approves it**. Probes are scratch, described and not
committed.
**Order of units.** Send Unit 0 for review first.
**The finding's probe:** `.git-exclude/review-request/audit-0-67-b/evidence/zz_probe_b3_readonly_chip.rs`, with its
output beside it.

## Where the change goes

- **`crates/snora-widgets/src/design/chip.rs`:** `pub fn tag<'a, Message: 'a>(tokens: &Tokens, label: impl
  Into<String>) -> Element<'a, Message>`.
  - **Not built on `design_control`.** A tag is not a control: it registers no focus target, takes no press, and has
    no hover or pressed state. A `container` with the label at `label_size`.
  - **The same padding as `filter`** (`[spacing.xs, spacing.sm]`), so a row of tags and filter chips lines up.
- **`crates/snora-style/src/chip.rs`:** the tag's style, as a container style. **Its colours are an unselected chip's
  resting colours** (`unselected(tokens, Status::Active)`'s fill, border and text). Derive them from that function, so
  the tag and the resting chip cannot drift apart, and so that the tag never looks selected.
- **The module doc** of `design/chip.rs` gains the third variant. Its "every control on a chip is focusable" sentence
  says that a tag is not a control.
- **`crates/snora/tests/direction_guard.rs`:** `tag` takes no direction, because a single label has nothing to order.
  Add it to the guard's list of functions that say why, with that reason.
- **Do not touch `filter`.** `None` still disables it. The rustdoc line pointing to `tag` is the architect's.

## Unit 0 — measured (nothing committed)

A prototype, in all six presets, on both renderers, read from pixels:
- the label's contrast against the tag's fill, next to the preset's `text_contrast` level;
- the boundary's contrast against `background`, `surface` and `surface_raised`;
- the tag's height beside a filter chip with the same label;
- that a click on it publishes nothing, and that Tab passes over it (`every_control.rs`'s probe).

**Report** anything R-1 does not fit, in particular any preset where the resting chip's colours do not hold the
label's level as a tag. A tag has no hover state to fall back on.

## Unit 1

- R-1, with a doc-test.
- **The sweeps (R-3):** a `"tag"` scene in `common/sweep.rs`, picked up by `text_sweep.rs` (the label) and
  `nontext_sweep.rs` (the boundary). Both renderers.
- **Tests, each with a breakage row shown failing:**
  - the label at the preset's level, from pixels (breakage: the disabled alpha applied);
  - not a Tab stop (breakage: built on `design_control`);
  - no message on a click (breakage: a press handler);
  - the same height as a filter chip (breakage: other padding);
  - not drawn as selected (breakage: the selected fill).
- **CHANGELOG `[Unreleased]`, Added.**

## Acceptance criteria

1. Each test is shown able to fail. Use named `cp` backups and `sha256sum -c`; never `git checkout` or `git restore`.
   Run the breakage script again on the final tree.
2. Every `ci.yaml` command, run locally, before pushing to `main`, and only once the review has approved it.
3. **The book is the architect's** (R-2): the recipe, `filter`'s rustdoc line, and the design pages that list the
   chips.
