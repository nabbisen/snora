# RFC 094 — The gate register has rows nobody re-derived

**Status.** Accepted (owner, 2026-09-03). Handoff written — see
[`handoffs/094-…`](../handoffs/094-the-gate-register-has-rows-nobody-re-derived/implementation-handoff.md).
**Q-1 ruled** — test-backed rows only. **Q-2 ruled** — a dated "last re-derived"
column. **Q-3 ruled** — the architect rules a disputed row, per RFC-087's Q-3.
**Tracks.** 1.0 readiness / evidence integrity. **Severity: Medium.**
**Found by** answering "what remains for the next release" against the register
rather than from memory.
**Touches.** `docs/src/contributing/api-freeze-review.md`. No crate code.
**Release target.** 0.44.0.

## The finding

**RFC-084 corrected gate 5 and stopped at gate 5.**

Gate 5 — *"Render-semantics tests cover z-stack, dismissal, toast, RTL"* — was
marked ✅ at v0.17 and was wrong for **24 minors**, because every render-semantics
test behind it was positive-only: reachability, never containment. An external
audit found four Criticals living in that gap at once.

RFC-084 fixed the code, added the negative assertions, and corrected the row. It
did not ask **which other rows rest on the same tests, or on the same era of
evidence.** Nobody has since.

At least two do:

| Row | Ticked on | Problem |
|---|---|---|
| *"Z-stack order documented and tested"* | RFC-011-D/E, RFC-012 | Ticked roughly **thirty minors** before any negative assertion existed. Z-stack is one of gate 5's own four dimensions |
| *"Direction-sensitive integration tests"* | RFC-017 — "2 RTL render-semantics tests" | Both were reachability-only until 0.43.0 added a containment one. **The count was also stale**: 2 claimed, 3 actual (corrected 2026-09-03) |

**Both are arguably justified today**, and that is the uncomfortable part. Gate
5's negative assertions cover z-stack; 0.43.0's RTL containment test covers
direction. They became justified **by work done for a different gate,
accidentally, not because anyone re-checked them.** A tick that is right by luck
is indistinguishable from one that is right by evidence until someone looks.

## This is not a witch hunt, and the RFC should not become one

One row was checked while writing this and **held**: *"`keyboard::dismiss_on_escape`
tested — 7 unit tests"*. `crates/snora/src/keyboard.rs` has 11 `#[test]`
functions, of which exactly 7 cover `dismiss_on_escape`; the other 4 cover
`cycle_zones`, a different public function. The row is accurate.

Recording that matters. A sweep that only reports failures teaches the reader
that everything is broken, and the next person discounts it.

## Why the register is where this bites

The gate register is the document that answers *"is snora ready for 1.0"*. Its
rows are read at release time — through `release-process.md`'s own checklist —
and at no other point. A row that was true when written and quietly stopped being
true has, in this project, an observed half-life of **24 minors**.

**Every gate row is a claim about code, asserted in prose, checked by nothing.**
RFC-092 established that this is the failure class here; this is that class
sitting in the one document that decides 1.0.

## Proposal

**A one-time sweep of every ✅ row, and a rule for keeping them honest.**

1. For each ✅ row, record **what would falsify it today** and whether that was
   checked, or whether the tick is inherited. Not a re-audit of the underlying
   work — a statement of which kind of tick it is.
2. Rows whose evidence turns out to be insufficient are **corrected, not quietly
   re-ticked** — RFC-084's own precedent, and its Q-3 instruction.
3. Rows justified only by accident get their real justification written in, so
   the next reader sees evidence rather than luck.

## Non-goals

- **Not re-auditing the code behind each gate.** The question is whether the row
  states why it is true, not whether snora is good.
- **Not un-ticking rows to be safe.** A gate wrongly reopened costs as much
  credibility as one wrongly closed, and RFC-084 made the opposite mistake
  visible enough to learn from.
- **Not touching gates 1 and 3.** Both are ⬜ and neither is ours to close.

## Open questions

**Q-1 — scope: every ✅ row, or only those whose evidence is tests?** There are
roughly forty rows across the core gates and the design-track D-gates. The
test-backed ones are where the observed failure lives. **Suggest: test-backed
rows first, in this RFC; the rest listed but not swept**, so the work finishes.

**Q-2 — does this get a mechanism, or is it a one-time sweep?** RFC-092's
lesson says a rule with no mechanism decays. But a gate register is prose about
readiness and cannot be compiled. **Suggest: a dated "last re-derived" column**,
so staleness becomes visible rather than invisible — the cheapest available
mechanism, and the one `feature-gating-criteria.md` already uses in its own
status heading.

**Q-3 — who rules a row that turns out unjustified?** RFC-087's Q-3 said the
re-tick is the owner's, not the implementer's. **Suggest the same here**, with
this RFC producing evidence and the owner ruling each disputed row.

## Acceptance criteria

1. Every test-backed ✅ row states what would falsify it and whether that was
   checked or inherited.
2. Any row found unjustified is **corrected with its reason**, not silently
   re-ticked, and its disposition is the owner's.
3. Q-2's ruling is implemented, whatever it is.
4. CHANGELOG entry, or one line saying why not.
