# Implementation handoff — RFC-133: every snora widget with a reading order takes the direction

**RFC.** `rfcs/accepted/133-five-snora-widgets-take-no-direction.md`. Read the finding, R-1 to R-3
and the Rulings.
**Release target.** 0.64.0, the release that also removes the deprecated F6 types.
**Starts after** RFC-132 has landed (its `LogicalRow` is what these widgets lay out with) **and**
0.63.0 has shipped, so that no breaking change enters 0.63.
**Where.** On `main`, by the one-`main` route.

## The rule

> **Every snora widget whose layout has a reading order takes a required `direction:
> LayoutDirection`,** and lays itself out with RFC-132's row: logical order, mirrored positions.
> The progress bar fills from the start edge.

## Unit 0 — measure (no product code committed)

1. **The list of every public widget function with a row,** made mechanically from the source and
   checked against RFC-132 Unit 0's table of 18 sites. Say how the list is built, because R-3's
   test will build it the same way.
2. **The five,** measured both ways after RFC-132, so that what is still LTR is known exactly.
3. **Q-2, the fill:** prototype the progress fill from the start edge, either through
   `LogicalRow` or with a fill snora draws, and report the size of each and **any public API**.
   If it needs some beyond the new parameter, stop and report.

## Unit 1

- **The parameter** on `removable`, `removable_with_tooltip`, `progress::row`, `progress::card`,
  both `app_footer`s, `render_menu` and `Notice::new`. Each is laid out with `LogicalRow`.
- **Tests both ways,** from bounds, with the fill read from pixels.
- **R-3's guard:** a test that builds the list as Unit 0 did and fails if a public widget function
  with a row takes no direction. Plant a sixth and show it fails.
- **Every LTR frame unchanged** (one renderer, before against after).
- **CHANGELOG `[Unreleased]`, Changed:** each signature, with its one-line migration.

## Acceptance criteria

1. Each test is shown able to fail. Use named `cp` backups and `sha256sum -c`; never `git checkout`
   or `git restore`.
2. `render_semantics.rs` is unmodified.
3. Every `ci.yaml` command, run locally, before pushing to `main`.
4. **The book is the architect's:** `direction.md`'s table, and the 0.63 → 0.64 guide.
