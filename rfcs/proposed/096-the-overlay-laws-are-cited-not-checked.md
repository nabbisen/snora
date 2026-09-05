# RFC 096 — The overlay laws are cited, not checked

**Status.** Proposed (2026-09-06).
**Tracks.** Contract integrity. **Severity: Medium.**
**Found by** RFC-094's Unit 3, which added the first test confirming the engine
obeys Law 2 — and by then measuring what the other laws actually have.
**Touches.** `crates/snora/tests/render_semantics.rs`,
`docs/src/reference/overlay-interaction-semantics.md`. No crate code expected.
**Release target.** 0.46.0.

## The finding

`overlay-interaction-semantics.md` states **eight laws** governing how overlays
compose. They are the engine's behavioural contract — the document RFC-084
derived its negative assertions from, and the one a consumer reads to know what
snora guarantees.

**They are prose, and RFC-094 established what this project's prose is worth
when nothing checks it.**

RFC-094's Unit 3 found the first instance without looking for it: menu-plus-modal
had **never been constructed in any test**. The behaviour was correct — the modal
dim dominates the menu, exactly as Law 2 says — but nothing confirmed the engine
did what the law claimed. The implementer's framing is this RFC's thesis:
*"a claim in prose, unchecked — precisely RFC-094's whole thesis applied one
level down."*

## What is actually there, measured rather than assumed

**I previously told the owner that six of the eight laws had never been checked.
That was inferred and it is wrong.** Counted:

| Law | Cited in `render_semantics.rs` |
|---|---|
| 1 — Z-stack order | **none** |
| 2 — Menus below modal state | 2 |
| 3 — Dialog and sheet may coexist | 1 |
| 4 — Close sinks are global per overlay class | **none** |
| 5 — Missing close sink does not hide content | 2 |
| 6 — Toasts are above modal state | 1 |
| 7 — Keyboard dismissal is application-owned | 1 |
| 8 — Modal focus trapping is staged | 5 |

So six of eight are cited somewhere, not one. **Laws 1 and 4 are cited nowhere at
all.**

**But RFC-094's own finding was that a citation is not coverage.** Gate 5 cited
tests for twenty-four minors while every one of them was positive-only. A law
mentioned in a doc comment above a test is evidence that someone had the law in
mind — not that the test would fail if the engine stopped obeying it.

That is the question this RFC asks, and nobody has asked it of this document.

## Proposal

**For each of the eight laws, name the test that would fail if the engine
violated it — or record that none exists.**

Three outcomes per law, and all three are acceptable deliverables:

1. **A test exists and would fail.** Name it in the law's own text, so the
   contract points at its own evidence.
2. **A test exists but would not fail** — it cites the law while asserting
   something weaker or adjacent. This is the gate-5 shape and the most valuable
   thing this RFC can find.
3. **No test exists.** Record it in the law's text plainly, and decide separately
   whether to write one. **Not every law needs a test** — Law 8 says focus
   trapping is *staged, not shipped*, which is a statement about what snora does
   not do, and there is nothing to assert about absent behaviour.

## Non-goals

- **Not writing a test per law by reflex.** Law 8's shape shows why. A law
  describing a deliberate absence, or a division of responsibility (Law 7 —
  keyboard dismissal is *application-owned*), may have nothing testable in this
  workspace.
- **Not re-auditing overlay behaviour.** RFC-084 did that and found four
  Criticals. This asks whether the document's claims are checked, not whether the
  engine is correct.
- **Not editing the laws themselves.** If a law turns out to be wrong rather than
  unchecked, that is a finding to report, not to fix inside this RFC.

## Open questions

**Q-1 — do Laws 1 and 4 differ from the rest, or only in citation?** Law 1
(z-stack order) is now substantively covered by RFC-094 Unit 3's
`sheet_renders_above_dialog_when_both_overlap`, which asserts push order
directly — it simply does not cite the law by name. Law 4 (close sinks are global
per overlay class) may be genuinely untested. **Suggest treating "uncited but
covered" and "uncovered" as different findings**, since conflating them is what
made the citation counts misleading in the first place.

**Q-2 — does the law numbering want fixing?** Law 1 is a `##` heading
(*"Z-stack order (Law 1)"*) while Laws 2–8 are `###` with a `Law N —` prefix. A
reader scanning headings sees seven laws, not eight. Cosmetic, adjacent, and
cheap — but out of scope unless the owner says otherwise.

**Q-3 — where does the answer live?** In each law's own text, or in a table at
the top? **Suggest each law's own text**, so the claim and its evidence cannot
drift apart — the same reasoning that put the channel register next to the code
in RFC-093 rather than in a separate document.

## Acceptance criteria

1. All eight laws have one of the three outcomes recorded, in the law's own text.
2. Any law found in outcome 2 — cited but not actually guarded — is reported
   explicitly, since that is the gate-5 defect recurring in a second document.
3. Where a test is named, it is **demonstrated failing** on a deliberate
   violation, per this project's standing rule. A named test that has never been
   seen to fail is a citation, which is the thing this RFC exists to distinguish
   from coverage.
4. CHANGELOG entry, or one line saying why not.
