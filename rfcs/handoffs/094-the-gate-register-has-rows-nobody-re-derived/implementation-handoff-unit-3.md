# Developer Handoff — RFC-094 Unit 3: two overlap tests

**Governing RFC.** **RFC-094** — find it under `rfcs/accepted/`, `rfcs/done/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md).
**Status.** Accepted (owner, 2026-09-03). Follows Unit 2.
**Release target.** **0.44.0.** The last item before the cut.
**Touches.** `crates/snora/tests/render_semantics.rs` only. No crate code.
**Size.** Two test functions.

---

## This is your own closing offer, taken up — not item 3 reopened

Item 3's "no" is **accepted on its argument**, and this handoff does not reverse
it. There is no decision function here and no `LAYER_ORDER` constant. You wrote:

> the cheap, additive option … is more overlap-based reachability tests in the
> existing `render_semantics.rs` style … That is strictly smaller than the
> decision-function refactor and would close the two named gaps directly, if the
> owner wants them closed.

The owner wants them closed. Both, and for different reasons.

## 1. Dialog and sheet, spatially overlapping

`dialog_and_sheet_coexist_sheet_content_reachable` uses a centered dialog and a
`SheetEdge::Bottom` sheet that **do not overlap in the fixture**, so it proves
both are reachable and cannot distinguish push order between layers 5 and 6. You
found that in Unit 1; it is why the z-stack row now says "consequences" rather
than "order".

Make them contest the same space. `SheetSize::Full` (or a `Ratio` near 1.0)
against a centered dialog is the obvious way — check `SheetSize`'s variants and
pick whichever actually produces overlap in the simulator rather than assuming
`Full` does.

The assertion is the ordinary one: with both present and overlapping, a click
where they contest reaches the layer that is pushed **later**. Name in the test's
doc comment which layer that is and why, the way `sheet_content_button_reachable`
already does.

**If overlap turns out not to be constructible** with the current `Sheet` API —
say so and stop. That is a finding about the fixture surface, not a failure, and
it is worth more than a test that passes without contesting anything.

## 2. Menu and modal together — the one that is not about order

**No test in `render_semantics.rs` constructs a menu and a modal at the same
time.** I checked all of them. So what is unverified is not their relative order
but, in your own words, *"whether the application can even show a menu and a
modal together without something breaking."*

**That is item 1's shape, not item 3's** — a documented, supported combination
with zero coverage. `design`-alone turned out to work. This one is unknown in
either direction, which is worse than known-untested.

Construct a layout with a header or context menu **and** a dialog or modal dim,
and assert what should be true: both render, and the interaction that should
reach each one does. If something is broken, that is the finding and you should
stop and report it rather than shaping a test around it.

## Required evidence

The usual, for each test that lands: break the thing it guards, watch it fail,
restore.

For test 1 that means swapping the dialog and sheet `layers.push()` calls in
`render.rs` — **which is exactly the change nothing currently catches**, so it is
the demonstration that matters most in this unit. Restore afterwards and confirm
byte-identical.

For test 2, if it passes first time, there is no natural break to introduce;
demonstrate instead that it fails if the menu layer is not pushed at all.

## Acceptance criteria

1. Both tests exist, or a stated reason one could not be constructed.
2. Test 1 demonstrated failing on a dialog/sheet push-order swap.
3. Test 2 demonstrated failing on a missing menu layer.
4. **No CHANGELOG entry** — tests only, no consumer-observable change. Stated so
   the omission is a decision, per RFC-092.
5. Do not touch `api-freeze-review.md`. If test 1 lands, the z-stack row can say
   "order" again honestly — **that edit is mine**, and it is the point of the
   whole exercise.
