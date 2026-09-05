# Developer Handoff — RFC-096 the overlay laws

**Governing RFC.** **RFC-096** — find it under `rfcs/accepted/`, `rfcs/done/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships.)*
**Status.** Accepted (owner, 2026-09-06).
**Release target.** **0.46.0.**
**Touches.** `docs/src/reference/overlay-interaction-semantics.md`, and
`crates/snora/tests/render_semantics.rs` only if a law needs a test it does not
have. No crate code.

---

## Rulings

**Q-1 — "uncited but covered" and "uncovered" are different findings, and must
be reported differently.** Conflating them is what made my own citation count
misleading, and it is the same error in miniature as counting a citation as
coverage. Law 1 (z-stack order) is substantively covered by RFC-094 Unit 3's
`sheet_renders_above_dialog_when_both_overlap`, which asserts push order
directly — it just does not name the law. Law 4 may be genuinely uncovered.
**Those are not the same result and must not share a row.**

**Q-2 — yes, fix the numbering.** Law 1 is a `##` heading
(*"Z-stack order (Law 1)"*) while Laws 2–8 are `###` with a `Law N —` prefix, so
a reader scanning headings sees seven laws. **Checked before ruling: nothing
links to Law 1's anchor** — the only anchor links in the tree are to Law 8 and
Law 2, both `###`, both unaffected. So the rename breaks nothing. If you find a
link I missed, stop and say so rather than leaving a dangling anchor.

**Q-3 — the answer lives in each law's own text**, not in a summary table. Same
reasoning that put RFC-093's channel register next to the code: a claim and its
evidence that live in different places drift apart, and this RFC exists because
one already did.

## The work

For each of the eight laws, record one of three outcomes **in that law's own
text**:

1. **Guarded** — name the test, and say what it asserts. Not "see
   `render_semantics.rs`": the specific function.
2. **Cited but not guarded** — a test mentions the law while asserting something
   weaker or adjacent. **This is the most valuable outcome this RFC can
   produce**, because it is gate 5's defect recurring in a second document, and
   nobody has looked for it here.
3. **Not covered** — say so plainly, and do not write a test to make the row look
   better.

**Outcome 3 is a complete answer for some of these.** Law 8 says focus trapping
is *staged, not shipped* — a statement about what snora deliberately does not do,
with nothing to assert. Law 7 says keyboard dismissal is *application-owned*,
which is a division of responsibility, not engine behaviour. **Do not
manufacture tests for laws whose content is not a behavioural claim.** A row
reading "not a testable claim, and here is why" is worth more than a test that
asserts something adjacent to make the table full.

## Required evidence

**Where you name a test as guarding a law, demonstrate it failing.** Break the
behaviour the law describes, confirm that named test refuses, restore.

This is not ceremony here — it is the entire distinction the RFC is about. A test
named in a law's text but never seen to fail is a citation, and citations are the
thing this RFC exists to separate from coverage. If a test you were about to name
turns out not to fail, that is outcome 2 and it is a finding, not a setback.

## Acceptance criteria

1. All eight laws carry one of the three outcomes, in their own text.
2. Every law recorded as *guarded* has its test demonstrated failing.
3. Any law found in outcome 2 is reported prominently, not buried in the table.
4. Law 1's heading is normalized to match Laws 2–8.
5. **No CHANGELOG entry** unless a test is added that changes nothing a consumer
   observes — in which case say so in one line either way. This is documentation
   and tests.
6. Do not edit the laws' content. If a law appears **wrong** rather than
   unchecked, stop and report it — that is a different RFC and a much bigger one.
