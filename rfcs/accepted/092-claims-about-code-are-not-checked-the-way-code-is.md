# RFC 092 — Claims about code are not checked the way code is

**Status.** Accepted (owner, 2026-09-02). Handoff written — see
[`handoffs/092-…`](../handoffs/092-claims-about-code-are-not-checked-the-way-code-is/implementation-handoff.md).
**Q-1 ruled** — the script ships as a **gate**, keyed on a `Docs-only: yes`
commit trailer, *against this RFC's own suggestion*: "CI cannot know which
claims were made" is false if the claim is written where a machine reads it, and
that reasoning is what kept three scripts manual for three releases (RFC-087,
F-39). **Q-2 ruled** — both parts adopted, Part 2 as one sentence in existing
documentation. **Q-3 ruled** — it binds the architect's review results too.
**Tracks.** Review integrity. **Severity: High.**
**Found by** the 0.41.0–0.42.0 cycle. Six instances, four of them the
architect's own.
**Touches.** `scripts/` (one new script), `docs/src/contributing/`. No crate
code.
**Release target.** 0.43.0.

## The finding

This project gates code exhaustively. Contrast floors, feature matrices, an
`E0027` on a new palette field, link checks, version snippets, migration-guide
coverage, and now three release refusals. A wrong *line of code* has a hard time
reaching main.

**A wrong sentence about that code has no such difficulty.**

Every one of these shipped or nearly shipped this cycle:

| Claim | Where | Actually |
|---|---|---|
| *"No behaviour change"* | RFC-089's commit message | F-33 removed a `match`, F-34 restructured `sheet.rs` |
| *"the RFC-089 sweep touched rustdoc text, lints and metadata"* | the 0.41.1 release commit, at the tag | same two code changes; the architect had read 3 of 29 files |
| *"the fade drops four of five intents under the 3.0 floor"* | RFC-086 submission, CHANGELOG, migration guide | zero of ten; worst case 3.38:1. The table cited was the pre-fix mark |
| *"confirmed passing on a clean tree"* | RFC-087's gate acceptance | true on a developer clone with 68 tags; the gate could not run in CI at all |
| *"110 of 110 fences"* | RFC-069 | 111 |
| *"all 39 audit findings verified"* | ROADMAP draft | 8 |

Three were caught by review, two by the implementer re-checking their own work,
and one by a downstream reader. **None was caught by a gate**, because no gate
reads sentences.

## Why this is not a discipline problem

The tempting response is "check claims more carefully." That is a prose control,
and RFC-090 established what this project's prose controls are worth: the rules
a machine refuses have never been broken here, and the rules a person must
remember have been broken every time it mattered.

The deeper reason is economic. **Verifying a summary costs as much as producing
it.** "No behaviour change" over a 29-file diff is only checkable by reading 29
files — which is why the architect read three and generalised, and would do so
again under the same pressure. A control that asks for the expensive thing every
time will be skipped exactly when the diff is large, which is when it matters.

So the design question is not "how do we check claims" but **"which claims can be
produced by a command instead of by a person."**

## Proposal

### Part 1 — mechanize the three claim classes that can be

**A. "Documentation only" / "no behaviour change."** A script, not a judgement:

    scripts/check-docs-only.sh <rev>

Prints every non-comment, non-blank changed line under `crates/`. Empty output
is the claim; non-empty output refutes it. This is the exact filter that found
F-33 and F-34 by hand, after the release shipped. Any RFC, handoff, or commit
message asserting docs-only quotes its output.

**B. Counts.** Any number in a claim — fences, findings, re-exports, tests,
packages — is produced by a command that is quoted alongside it. Three of the
six failures above are miscounts, all from counting by eye.

**C. Measurement claims name the tree they were measured on.** RFC-086's error
was not a bad measurement; the numbers were right. It cited a **pre-fix** table
to support a claim about **post-fix** behaviour. A measurement claim states the
commit or tree state, so that mismatch is visible rather than inferable.

### Part 2 — for everything else, provenance instead of verification

A claim that cannot be produced by a command is **labelled as an inference**, not
asserted flatly.

This already worked here, unprompted. The implementer's Q-2 research on Trusted
Publishing separated *"the originating RFC lists monorepo support under Future
Possibilities"* (confirmed, source named) from *"one token probably covers all
five"* (inference from the shape of the API, labelled as such). **That labelling
is why the ruling took one pass instead of three**, and why archiving RFC-091
was defensible — the risk being accepted was legible.

The rule is cheap because it asks for less, not more: not "verify everything,"
but "say which kind of thing you are saying."

## What this RFC cannot do, stated plainly

**Part 2 is a prose control, and this document has just argued that prose
controls fail here.** That is not a flaw in the write-up; it is the honest shape
of the problem. No compiler reads a commit message.

Two things make it worth adopting anyway, and neither is optimism:

1. **Part 1 removes the highest-frequency cases entirely.** Four of the six
   failures above are a docs-only claim or a miscount — both mechanizable today.
2. **Part 2 has an artifact.** Every claim that matters passes through a review
   result, which is a file, written by a named party, that the owner reads. A
   rule attached to an artifact someone already produces is weaker than a
   compiler and stronger than a good intention.

**Scoping this RFC to Part 1 alone is a legitimate outcome** — see Q-2.

## Non-goals

- **Not a review checklist.** Checklists are the control category that failed.
- **Not evidence for every sentence.** The rule applies to claims about *what
  changed* and *what was measured*, not to reasoning or recommendation.
- **No new CI job.** `check-docs-only.sh` is run by a human making a claim; it
  gates nothing, because nothing in CI knows which claims were made.

## Open questions

**Q-1 — does `check-docs-only.sh` belong in CI at all?** I think not: CI cannot
know that a commit *claimed* docs-only. It is a tool for the claimant and the
reviewer. **Suggest: script only, no gate** — but this is exactly the reasoning
that kept three scripts manual for three releases (RFC-087 F-39), so it deserves
a second look rather than my say-so.

**Q-2 — adopt Part 2, or scope this RFC to Part 1?** Part 1 is mechanical and
uncontroversial. Part 2 is a writing rule with an artifact and no enforcement.
Adopting it half-heartedly is worse than not adopting it — it becomes another
sentence in a document that people stop reading.

**Q-3 — does this bind the architect's own review results?** It should: four of
the six failures were the architect's, and two were in review results or release
commits rather than implementation. A rule that exempts the reviewer would miss
most of the observed instances.

## Acceptance criteria

1. `scripts/check-docs-only.sh` exists, is documented in `scripts/README.md`, and
   is **demonstrated on the RFC-089 commit** — where it must print F-33 and F-34,
   the two changes that made "no behaviour change" false.
2. The claim rules land in `docs/src/contributing/` where the review and release
   process already live, not in a new page nobody opens.
3. Whatever Q-2 rules, the RFC's own text says which part was adopted.
4. CHANGELOG entry, or one line saying why not.
