# Developer Handoff — RFC-094 Unit 2: make the reopened rows true

**Governing RFC.** **RFC-094** — find it under `rfcs/accepted/`, `rfcs/done/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md).
**Status.** Accepted (owner, 2026-09-03). Follows Unit 1's evidence report.
**Release target.** **0.44.0.**
**Touches.** `.github/workflows/ci.yaml`, `crates/snora/src/toast.rs`, possibly
`crates/snora/src/render.rs`.

---

## Your Unit 1 report is why these three exist

Two rows were reopened and one narrowed on your evidence. **Nothing here is a
correction of your work** — these are the remedies that make the reopened rows
true again. The rulings are in
`.git-exclude/reviewed/094-the-gate-register-has-rows-nobody-re-derived/review-result.md`.

## 1. A `design`-alone entry in the feature matrix

`crates/snora/Cargo.toml` documents `design` as independent of `widgets` —
*"everything else works with `design` alone"* — and
`--no-default-features --features design` appears in no CI job. You found this;
I confirmed it, and confirmed it is **not broken**: it compiles and its suites
pass (42 / 17 / 7).

Add it to `feature-matrix` as a tenth combination.

**Use `cargo check`, matching its nine siblings.** Do not make this one entry run
tests to compensate for the row's wording — you correctly noted that "CI-tested"
is imprecise when every entry is a compile check. **That wording is mine to fix,
not yours to work around**, and I will fix it when the row re-ticks. A matrix
that is consistent is worth more than one entry that is special.

## 2. A test for `toast::subscription`

The row names two lifecycle helpers and only `sweep_expired` has a test.

**The mechanism, since the function looks untestable and is not.**
`Subscription` has no `PartialEq`, so the branches cannot be compared directly —
but `iced_futures::Subscription::units(&self) -> usize` is public and counts
recipes. `Subscription::none()` has **zero**; a live one has **one**. That
distinguishes `subscription`'s only branch:

| Input | Expected `units()` |
|---|---|
| empty slice | 0 |
| one persistent toast | 0 |
| one transient toast | 1 |
| one transient + one persistent | 1 |

Two of those are the negative cases — the assertion that no subscription is
produced when nothing needs sweeping, which is the half a doctest cannot reach.

**Confirm `units()` is reachable through `iced`'s own re-export** before building
on it. If it is not public at that path, say so and propose the alternative
rather than reaching into `iced_futures` directly — the engine depends on `iced`,
not on its internals.

## 3. `render()`'s layer sequence — assess, do not assume

The z-stack row was narrowed because its tests assert **pairwise consequences**
of the layer order, not the order. Swapping layers 5 and 6 in `render()` would
fail nothing in the tree.

The obvious remedy is a decision-shaped test, the way `render_order_for` makes
toast ordering testable: extract the sequence as data, test it by equality, and
have `render()` consume it.

**I am not convinced that pays for itself, and I want your judgement, not your
compliance.** `render()` is the one place composition happens, the module doc
documents the order extensively, and the refactor introduces an enum and a
decision function into the single most load-bearing function in the engine.

Report either:

- **a test**, if the extraction is cheap and does not distort `render()`; or
- **a stated reason not to**, naming what would have to go wrong for the missing
  test to matter and why that is unlikely enough to accept.

**A well-argued "no" is a complete deliverable here**, and it goes in the row so
the next person does not re-ask. What is not acceptable is silence, or a test
that asserts the order by re-stating the same `if let` chain in the test file —
that tests the copy, not the code.

## Required evidence

1 and 2 get the usual failing-first demonstration: break the thing, watch the
check refuse, restore.

- For 1: remove `design` from `snora`'s features locally, confirm the new matrix
  entry fails, restore.
- For 2: invert the `has_transient` condition, confirm the test catches it,
  restore.

For 3, if you write a test: the same. If you argue against one, the argument is
the deliverable and there is nothing to demonstrate.

## Acceptance criteria

1. The matrix has a `design`-alone entry, `cargo check`, demonstrated failing.
2. `subscription` has a test covering both branches including the empty and
   all-persistent cases, demonstrated failing.
3. Item 3 answered either way, with reasoning if the answer is no.
4. CHANGELOG entry, or one line saying why not — these are tests and CI, so I
   expect not, but say which.
