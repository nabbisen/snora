# Developer Handoff — RFC-094 the gate sweep

**Governing RFC.** **RFC-094** — find it under `rfcs/accepted/`, `rfcs/done/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships.)*
**Status.** Accepted (owner, 2026-09-03).
**Release target.** **0.44.0.** No crate code.
**Implementation units.** One is yours — the evidence. The rulings and the
register edits are the architect's.

---

## Rulings

**Q-1 — test-backed ✅ rows only, in this RFC.** There are roughly forty rows
across the core gates and the design-track D-gates. The observed failure lives in
the test-backed ones, and a sweep that tries to cover everything will not finish.
**List the rest without sweeping them**, so the next person knows what was and
was not looked at.

**Q-2 — a dated "last re-derived" column.** A gate register is prose about
readiness and cannot be compiled, so this is the cheapest mechanism available:
staleness becomes *visible* rather than invisible. `feature-gating-criteria.md`
already does exactly this in its own status heading (*"re-derived 2026-09-03
post-tag"*), so it is a house pattern rather than a new invention.

**Q-3 — the architect rules a disputed row, not you.** RFC-087's Q-3 set this
precedent and RFC-084 is why it exists: a gate wrongly re-ticked is the defect
this whole RFC descends from. **Do not change any ✅ or ⬜**, and do not add the
dated column's values as verdicts — report evidence, and the ruling follows.

## Unit 1 — the evidence, one row at a time

For each ✅ row in `docs/src/contributing/api-freeze-review.md` whose stated
justification is tests, report:

1. **Which tests actually back it** — by name and file, found in the tree today,
   not inherited from the row's own text.
2. **Whether any of them is a negative assertion** — does anything assert that
   something is *blocked*, *absent*, or *refused*? This is the whole question.
   Gate 5 was ✅ for 24 minors on tests that were exclusively positive.
3. **What would falsify the row today**, in one sentence.
4. **Whether the row's own numbers are right.** Counts have been wrong three
   times this cycle.

The rows to check are the ones matching `✅` whose text mentions a test or an
assertion — currently seven, including *"Z-stack order documented and tested"*
and *"Direction-sensitive integration tests"*.

### Two of them are already partly done — start from these, do not redo them

- **"Direction-sensitive integration tests"** — its count said 2, actual 3
  (corrected 2026-09-03). Both original RTL tests were reachability-only; the
  third, added in 0.43.0, is the containment one. **What is not established** is
  whether the row's claim is *satisfied* by three tests of which one is negative.
- **"`keyboard::dismiss_on_escape` tested — 7 unit tests"** — **checked and it
  holds.** `crates/snora/src/keyboard.rs` has 11 `#[test]` functions; exactly 7
  cover `dismiss_on_escape` and 4 cover `cycle_zones`. Reported here so you do
  not spend time on it, and because **a sweep that reports only failures teaches
  the reader to discount it.**

## Not yours

The rulings, the ✅/⬜ values, the dated column's introduction, and any corrected
row text. Same split as RFC-090's Unit 3.

## Required evidence

The report itself is the deliverable — there is nothing to perturb here, which
makes this the one recent handoff without a failing-first demonstration.

**Two things instead.** State plainly which rows you could *not* resolve and why;
an unresolved row named is worth more than a resolved-looking one guessed. And
where a row is fine, say so in the same detail as where it is not — see
`dismiss_on_escape` above.

## Acceptance criteria

1. Every test-backed ✅ row has the four facts above, or a stated reason it could
   not be resolved.
2. Rows not in scope (Q-1) are **listed**, so the boundary is visible.
3. No ✅ or ⬜ changed, and no row text edited.
4. **No CHANGELOG entry** — this unit produces a report, not a shipped change.
   Stated so the omission is a decision. The architect's register edits may
   warrant one; that is a separate call.
