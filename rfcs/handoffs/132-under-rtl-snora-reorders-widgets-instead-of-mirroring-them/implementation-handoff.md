# Implementation handoff — RFC-132: mirroring is a layout, not a reordering

**RFC.** `rfcs/accepted/132-under-rtl-snora-reorders-widgets-instead-of-mirroring-them.md`. Read
the finding, R-1 to R-5, the Rulings and the principles review's additions.
**Release target.** 0.63.0 (Q-3: all of it if the mirrored row is small, else the header and tests
now and the row in 0.64).
**Where.** On `main`, by the one-`main` route. Probes are scratch, described and not committed.
**Order.** After the audit follow-up unit. **Before** the drawn-contrast Unit 0, because Q-3's
scope decision depends on this Unit 0.

## The rule

> **The order is logical, and the positions mirror.** Under both directions a row's children sit
> in reading order in the tree; under RTL they are placed from the right edge. So a direction flip
> keeps every widget's state, and Tab follows reading order (start to end) in both directions.
> Directional glyphs flip; others do not. Horizontal arrows, if snora ever answers them, follow the
> screen.

## Unit 0 — measure and prototype

1. **Every row snora constructs.** A first count finds about 30 `row!`, `Row::new` or `row_dir` in
   `snora-widgets` and `snora`, outside tests. List each, with its widget, and mark it:
   - **logical:** its order is reading order, so it mirrors;
   - or **not:** with why.

   Include the rows inside blocks: the header's title group, a button's icon and label, a toast's
   title and close mark, and a notice's action row.
2. **Measured in both directions, from bounds:**
   - the skeleton;
   - the header, with and without menus and end-controls;
   - the sidebar's tooltip side;
   - the tab bar;
   - the breadcrumb, with its separator glyph;
   - the toasts at all six positions;
   - every row marked logical.

   Say which are mirrors today and which are not.
3. **Tab order under RTL,** measured on the skeleton, the header, the tab bar and the breadcrumb.
   Is it logical or visual today? The two tests that pin today's order are
   `focus_follows_removal.rs::right_to_left_follows_the_mirrored_order` and
   `shortcuts.rs::right_to_left_next_tab_follows_tab_order`. Say what each asserts.
4. **A live flip:** focus, a scroll offset, and an open menu's state, before and after.
5. **Every glyph snora draws,** marked directional or not, and whether it flips today (R-4).
6. **The mirrored row, prototyped** (Q-2): a small layout widget in `snora-interaction`, used by
   `row_dir`, that places children in logical order from the start edge. Report:
   - its size;
   - every LTR layout test passing unchanged;
   - **RTL frames pixel-identical to today's wherever today's is right** (the skeleton, the
     toasts), and corrected where it is not (the header);
   - the flip keeping focus;
   - **whether it needs any public API.** If it does, stop and report before Unit 1.

## Unit 1

- R-1 to R-5, each with a breakage row. The flip test fails on today's reordering, and the header
  test fails on today's header.
- **Every LTR frame unchanged:** a test, before against after, on one renderer.
- The two RTL Tab-order tests rewritten to the ruled order, each listed with why.
- `row_dir`'s and `row_dir_three`'s rustdoc: the order is logical and the positions mirror.
- **CHANGELOG `[Unreleased]`:**
  - Fixed: the header under RTL, and focus across a flip;
  - Changed: Tab order under RTL if it changes, and `row_dir`'s tree order, with what applications
    that use it see.

## Acceptance criteria

1. Each test is shown able to fail. Use named `cp` backups and `sha256sum -c`; never `git checkout`
   or `git restore`.
2. `render_semantics.rs` is unmodified.
3. Every `ci.yaml` command, run locally, before pushing to `main`. Parity: say if any RTL mark moves.
4. **The book is the architect's:** `direction.md`, `keyboard.md`, and the migration guide.
