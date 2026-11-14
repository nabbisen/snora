# RFC-132 — Under RTL, snora reorders widgets instead of mirroring them: the header is not a mirror, and a flip loses focus

**Status.** Accepted 2026-11-15 (owner), every open question ruled as suggested (see Rulings), with the refinements of the principles review below.
**Handoff.** [`handoffs/132-…`](../handoffs/132-under-rtl-snora-reorders-widgets-instead-of-mirroring-them/implementation-handoff.md).
**Raised.** 2026-11-15, architect, from the 0.63 audit's request B (W2-D15, W2-D24), verified by
the architect against the code.
**Release target.** 0.63.0 (Q-3).
**Priority.** ABDD is a requirement (direction-agnostic design). The header is on every snora
window that has one, and a keyboard user who switches direction loses their place.

---

## The finding

**1. `app_header` is not a mirror image under RTL.** `build_header` (`crates/snora-widgets/src/header.rs:157-169`)
builds `end_side = row![filler, end_controls]`, then `row_dir(direction, start_group, end_side)`.
- `row_dir` swaps the **two blocks** and nothing inside them.
- So under RTL the row is `[filler, end_controls, start_group]`. The end-controls sit **beside
  the title group**, not at the end (left) edge.
- The title group's own order, `[title, spacer, menus]`, is not mirrored either.
- The audit's probe (800 px, `design::render`): under LTR the end-control is flush at the right.
  Under RTL with no menu it is at x 629..675, beside the title. With one menu it reaches the right
  edge by another route. Both paths, styled and unstyled, behave the same.
- `direction.md` said "title group on `Start`, end-controls on `End`". It now names the defect.

**2. A live LTR ↔ RTL flip loses keyboard focus.**
- snora mirrors by **reordering children** (`row_dir`, the skeleton row), and iced matches a
  widget's state by its position in the tree.
- After a flip the focused control is at a new position, so nothing is focused. The audit's probe
  shows a sidebar control focused before (`[0]`) and nothing after (`[]`).
- `direction.md` said "no per-widget reset needed". It now names the defect.

**3. Unknown, and it must be known: does Tab follow the logical order under RTL?**
- Reordering children changes tree order, and Tab follows tree order. So under RTL Tab may move
  left to right on screen, which is logical **end** to **start**.
- Platform convention (a document's order versus its visual direction) is that keyboard order
  follows **reading order**: start to end, so right to left under RTL.
- snora's tests pin something here (`focus_follows_removal.rs::right_to_left_follows_the_mirrored_order`,
  `shortcuts.rs::right_to_left_next_tab_follows_tab_order`), but no page says which order is
  meant. Unit 0 measures it.

## The cause, as one sentence

**snora implements "mirror" as "reverse the children"**, which changes what a reader sees, as
mirroring should, and also changes:
- what the tree is, so state is lost;
- what Tab follows, perhaps;
- inside a block, nothing, so the header is not mirrored.

## Proposal

**R-1. Mirroring is a layout, not a reordering.** Children stay in **logical order** in the tree,
under both directions, and a direction-aware row places them from the start edge: left under LTR,
right under RTL.
- `row_dir` and `row_dir_three` keep their signatures, and their meaning becomes this.
  - Their rustdoc says "the order is reversed" today; it will say the positions mirror and the
    order stays logical.
  - The skeleton row and the header use the same row.
- **Then a flip keeps every widget's state,** focus included, because no child moves in the tree.
- **And Tab follows the logical order in both directions** (Q-1), because the tree is the logical
  order.
- **Every row snora builds whose order is logical uses it, not only the outer blocks** (principles
  review). The header's title group is the case that showed a block mirrored outside and not
  inside. Unit 0 lists every row snora constructs, about 30 by a first count, and marks each
  logical or not. A row that is not logical, with no reading order (rare), says why in a comment.
- **`row_dir` is public, so its change is a change for applications.** An application row built
  with it keeps logical tree order from then on, so its Tab order and widget state under RTL
  change with ours. The migration guide says so. `direction.md` teaches the rule: the order is
  logical, and the positions mirror.

**R-2. The header is mirrored whole:**
- the title at the start edge;
- the menus after it, in logical order;
- the end-controls at the end edge.

**R-3. Every direction claim is tested in both directions,** read from bounds:
- the skeleton row, the header (with and without menus and end-controls), the sidebar's tooltip
  side, the toast positions, the sheet (done in 0.62), the tab bar and the breadcrumb;
- a live flip that keeps focus;
- Tab order under RTL.

The audit's probes show the skeleton and the toasts mirror correctly today. Their tests guard
that.

**R-4. Directional glyphs follow the direction, and the rule is written down** (principles
review). The breadcrumb's separator flips today (`›` to `‹`, `crumb.rs`). Unit 0 lists every
glyph snora draws and marks it directional or not:
- **a directional glyph** (a chevron, a "next" arrow) flips under RTL, with a test each way;
- **a non-directional one** does not flip, with a test that it does not.

**R-5. The keyboard rule for horizontal arrows, stated before it is needed** (principles review).
snora answers no ← or → today: `router.rs` only recognises arrows so that a text field keeps them.
If snora ever answers them (tab-bar arrows, RFC-116 L-11), **they follow the visual direction:
→ moves right on screen under both directions,** as platforms do. Tab, F6 and the tab shortcuts
stay logical. Writing it down now means a later RFC cannot pick the other way by accident.

## Open questions, with suggested rulings

**Q-1 — Tab order under RTL.** **Suggest: logical order (start to end) in both directions,**
which is right to left on screen under RTL.
- It is the platform convention, and what a screen reader's order would be.
- It matches F6, which is already logical and does not mirror (RFC-060, RFC-126).

If Unit 0 finds snora's Tab is visual under RTL today, that changes for RTL users, and the
migration guide says so. The two tests above that pin today's order are rewritten to the ruled
order, each listed with why.

**Q-2 — how the mirrored row is built.** **Suggest:** Unit 0 prototypes it, as a small layout
widget in `snora-interaction`, used by `row_dir`, and reports:
- its size;
- that it passes the existing layout tests unchanged under LTR;
- that the frame under RTL is pixel-identical to today's reordering wherever today's is right.

If it cannot be built in iced 0.14 without public API, Unit 0 says so before Unit 1.

**Q-3 — 0.63.0.** **Suggest: yes, all of it,** if Unit 0 shows the row is small. If it is not, the
header (R-2, a fix inside `header.rs`) and the tests go in 0.63, and R-1 in 0.64, with the flip
named as a known limit until then.

## Rulings, 2026-11-15 (owner)

Accepted, with each question as suggested:
- **Q-1:** Tab follows reading order, start to end, in both directions.
- **Q-2:** Unit 0 prototypes the mirrored row as a small layout widget in `snora-interaction`, used
  by `row_dir`, and reports before Unit 1 if it needs public API.
- **Q-3:** 0.63.0 for all of it if the row is small. Otherwise the header and the tests go in 0.63,
  and the mirrored row in 0.64, with the flip named as a known limit.

The owner asked for a careful check against the fundamental philosophy. That check added:
- every logical row, not only the outer blocks;
- the public change to `row_dir`, named for applications;
- directional glyphs (R-4);
- the horizontal-arrow rule (R-5).

## Review against snora's principles

- **ABDD.** This is the principle, applied to its own mechanism: the logic (tree, Tab, state) is
  direction-free, and only placement follows the direction.
- **Nobody is confused.** A reader under RTL gets the same header as under LTR, mirrored. A
  keyboard user's place survives a direction switch.
- **Robust.** One row for every mirrored place, so there is no second mechanism to drift.
- **Keyboard.** Tab follows reading order in both directions. F6 does already.
- **Predictable.** One sentence: the order is logical, and the positions mirror.

## What this is not

- Not a change to LTR layout: every LTR frame is unchanged.
- Not text shaping or bidi inside a string: that is iced's text layer.
- Not whether an icon should mirror (an arrow does, a clock does not): a separate question.

## Acceptance criteria

1. **Unit 0:**
   - the header, the skeleton, the sidebar's tooltip, the tab bar, the breadcrumb and the toasts
     measured in both directions;
   - Tab order under RTL measured;
   - the flip measured;
   - the mirrored row prototyped (Q-2).
2. **Unit 1:**
   - R-1 to R-3, each with a breakage row;
   - every LTR frame unchanged (a test);
   - the existing tests that pin today's RTL order, listed with why each changes.
3. **The book,** by the architect: `direction.md`, `keyboard.md`, and the migration guide.
