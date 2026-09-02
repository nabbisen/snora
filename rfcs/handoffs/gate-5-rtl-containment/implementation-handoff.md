# Developer Handoff — Gate 5's last positive-only dimension

**Governing document.** Not an RFC — one test. Gate 5 in
[`docs/src/contributing/api-freeze-review.md`](../../../docs/src/contributing/api-freeze-review.md)
names it as the condition for its own re-tick.
**Status.** Approved by the owner, 2026-09-02.
**Release target.** **0.43.0.**
**Touches.** `crates/snora/tests/render_semantics.rs` only. No crate code.
**Size.** One test function.

---

## Why this exists

Gate 5 — *"Render-semantics tests cover z-stack, dismissal, toast, RTL"* — was
marked ✅ at v0.17 and **stayed wrong for 24 minors**, because every
render-semantics test was positive-only: things are reachable, clicks fire. None
asked whether input that should be *blocked* actually is. It was not, in four
places at once (RFC-084, F-01 to F-04, external audit) — a click inside the
dialog dismissed it, a modal with no close sink blocked nothing, the dim did not
block scrolling, and clicking a toast pressed the widget beneath it.

RFC-084 fixed all four and added negative assertions. Gate 5 was reopened and
its re-tick left to the owner.

**Three of gate 5's four dimensions now have negative coverage. RTL does not.**
Both RTL tests assert only reachability:

- `sheet_end_edge_reachable_under_rtl` — *"sheet button must fire SheetAction"*
- `toast_dismiss_reachable_under_rtl` — *"toast dismiss must fire"*

Neither asserts that anything is blocked. That is the exact condition that made
the gate wrongly ✅, surviving at smaller scale in the one dimension nobody
revisited.

## Why this is not ceremony

RTL mirrors layout geometry: a `ToastPosition::TopEnd` toast renders on the
opposite side. Containment is implemented with `opaque`, which is itself
direction-agnostic — **but the wrapper's position is not.** A containment wrapper
that is mispositioned only under a mirrored layout would pass every existing
test, because every existing containment test runs LTR.

That is a real hole, not a hypothetical one, and it is cheap to close.

## The test

Mirror `toast_body_click_does_not_reach_content_beneath` under RTL. That test is
the model — same shape, same assertion, `LayoutDirection::Rtl` added and
`ToastPosition::TopEnd` set so the toast actually moves:

- a full-bleed `mouse_area` body that fires `Msg::BodyPressed` on press;
- one toast;
- `.direction(LayoutDirection::Rtl)` and `.toast_position(ToastPosition::TopEnd)`;
- click the toast's own title text;
- assert `Msg::BodyPressed` is **not** among the messages.

Name it so the dimension it covers is obvious — e.g.
`toast_body_click_does_not_reach_content_beneath_under_rtl`.

## Required evidence

**Prove it fails before calling it done.** Standing rule on this project for a
defect-specific test, and it has caught three checks this cycle that could not
fail — RFC-087's D-1, RFC-086's alpha-blind assertion, RFC-088's silent exit.

A passing test alone is not acceptable evidence. Break containment deliberately
— remove the `opaque` wrapper from the toast surface in
`crates/snora/src/toast.rs`, or make it conditional on direction — confirm the
new test fails and names the direction, restore, confirm it passes. Put the
transcript in the review package.

If the test passes with containment removed, it is not testing what it claims —
say so rather than adjusting the assertion until it goes green.

## Acceptance criteria

1. The new test exists in `crates/snora/tests/render_semantics.rs`.
2. It is demonstrated **failing** on deliberately broken containment, then
   passing after restore, with the transcript as evidence.
3. `cargo test -p snora --test render_semantics` green.
4. **No CHANGELOG entry** — a test with no consumer-visible effect. Stated here
   so the omission is a decision rather than a gap; if you disagree, say so in
   one line rather than adding one silently.
5. Do **not** edit gate 5's row in `api-freeze-review.md`. That is the
   architect's, and the re-tick is the owner's judgement.
