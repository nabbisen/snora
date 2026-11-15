# Implementation handoff — RFC-135: non-text at 3:1, on drawn pixels

**RFC.** `rfcs/accepted/135-four-non-text-boundaries-fall-under-three-to-one.md`. Read the finding,
"What is not a gap", R-1 to R-3 and the Rulings.
**Release target.** 0.63.0.
**Unit 0 is done:** the drawn-contrast Unit 0 (`.git-exclude/reviewed/drawn-contrast-unit0/review-result.md`,
rulings M-1 to M-5). This is Unit 1.
**Order.** After RFC-131's Unit 1, which lands the sweep's machinery: the frame inventory, the
supersampled reading, and the tripwires. This unit adds the non-text half to it. If the two fit one
test file better, say so and do it that way.
**Where.** On `main`, by the one-`main` route.

## The rule

> Every non-text element a reader needs in order to see a control or its state holds **3:1** against
> what is beside it, on the drawn pixels. That covers a glyph that is not a word, a boundary, a
> track, a mark and a scrollbar. **A glyph** holds 3:1 if the pixel furthest in contrast from the
> background beside it, drawn at twice the pixel density, is at least 3:1. **A boundary** is the
> component's fill or its border, whichever is stronger, against what is beside it.

## Unit 1

- **The four fixes, each by the smallest change that reaches 3:1 and moves nothing that passes:**
  1. **the progress track:** measure a border and a darker track, take the one that changes the
     filled bar less, and report both;
  2. **a `Debug` toast:** a border at 3:1, with its fill unchanged;
  3. **snora's own scrollables** (the menu dropdown): a scrollbar style whose thumb holds 3:1 against
     its track and the page, on both paths;
  4. **`card::raised`:** its border clears 3:1 against the shadowed page, or the shadow stops under
     the border. Take whichever moves less, and report both.
- **The sweep's non-text half:**
  - every non-text subject of Unit 0's inventory, at 3:1;
  - both renderers (the rendered jobs), both directions;
  - glyphs by M-2;
  - **"What is not a gap" written into the test as listed exceptions, each with its reason,** so
    nobody "fixes" a dialog border or a hover tint.
- **The tripwires:** the non-text one (a `fill_quad` with no finder) joins RFC-131's three, with its
  plant.
- **Nothing that passes moves:** a table of today's colours for every passing non-text subject.
- **CHANGELOG `[Unreleased]`, Fixed:** the four, with "retake the screenshot baselines of" each.

## Acceptance criteria

1. Each fix and each tripwire is shown able to fail. Use named `cp` backups and `sha256sum -c`;
   never `git checkout` or `git restore`.
2. `render_semantics.rs` is unmodified.
3. Every `ci.yaml` command, run locally, before pushing to `main`. State the sweep's added CI time.
4. **The book is the architect's:** the accessibility guide's non-text statement, and the
   high-contrast page's progress-track sentence.
