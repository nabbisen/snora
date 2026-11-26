# Handoff — 0.67 audit: is every claim on the pages adopters rely on true, and guarded?

**Ruled.** The owner, 2026-11-26 ("Yes. Accepted."), on the architect's scoping of 0.66, with the scope as revised
against the owner's philosophy.
**What it produces.** Four review requests, each a claims table and findings. **No code and no book change in this
work.** The architect corrects the book. What the reviews rule becomes 0.67.
**Method.** 0.62's audit (`rfcs/handoffs/audit-0-62-keyboard-and-overlay-claims/implementation-handoff.md`): its verdict
words, its columns and "What counts as a claim" apply unchanged. 0.63's "What differs" points 1 to 4 (`audit-0-63-visual-
guarantees/implementation-handoff.md`) apply wherever a page makes a visual or direction claim. This handoff says only
what differs.
**When.** It starts now and runs during 0.66, because it is read-only. **RFC-138 comes first if both are waiting:** it
ships in 0.66.
**Where.** On `main`, by the one-`main` route, **after the review approves it** (not that the audit commits anything). A
probe that has to run code goes in a scratch test that is described in the request and not committed.

## Why

- **0.62's audit** (8 pages, 508 claims) found 15 false. One was a defect since 0.4: a sheet on the right drawn at the
  wrong width.
- **0.63's** found the RTL mirroring and contrast defects.
- **Every real defect since 0.54 was found by a reader or an audit,** never by a test, because no test checked the
  sentence.
- **The pages below are the ones adopters meet first, and those whose promises they plan around.** None has been read.
  The owner's rule is that nobody should be confused or misunderstand. A false sentence on a first-contact page is the
  likeliest way that happens.

## Scope: four requests, about 5,100 lines

| Request | Pages | Lines |
|---|---|---|
| **A, first contact** | `introduction.md`; `getting-started/01` to `07`; `guides/icons.md`, `responsive.md`, `testing.md`, `troubleshooting.md`, `migrations.md` (the index, not the per-version guides) | 1,239 |
| **B, design pages and recipes** | `design/cards.md`, `feature-flags.md`, `iced-style-bridge.md`, `overview.md`, `progress.md`, `v021-primitives.md`; `design/recipes/*` (5) | 860 |
| **C, the reference** | `reference/architecture.md`, `binary-size-budget.md`, `build-cost-budget.md`, `performance-envelope.md`, `rendered-surface-identifiers.md`, `threat-model.md`, `vocabulary.md`, `widgets.md` | 1,803 |
| **D, promises to adopters** | `contributing/versioning-policy.md`, `api-governance.md`, `feature-gating-criteria.md`, `consumer-requirements.md`, `feedback-and-scope.md` | 1,177 |

**Not in scope:**
- the pages 0.62 and 0.63 read (cite their row instead);
- the per-version migration guides;
- released changelog entries, history and *Corrected* lines, rationale, future wanted, RFCs;
- the other contributing pages (they are snora's own process);
- **the public rustdoc** (the next audit, after 0.67's follow-up);
- **0.62's backlog** (the coverage release).

## What differs from 0.62 and 0.63

**1. A completeness claim is checked by enumeration, never by search.**
- "Every", "all", "the full list", an inventory, and a list that presents itself as whole are compared against **what
  exists**: the public API (the crate's exports, read from `lib.rs` and the re-exports, or from `cargo doc`'s index), or
  what is drawn.
- A list that is right for its stated scope but silent where a reader would look is a finding. Say what the reader looked
  for. `reference/widgets.md` was the example, fixed on 2026-11-26.
- RFC-128 R-2's source search missed the inactive tab mix. A search finds what you thought of.

**2. A first-contact page is checked by following it.**
- In a new scratch crate (outside the workspace, with snora by `path`), do what each getting-started page says, in order.
- Record each step that does not work as written, and each sentence that leaves a new user guessing.
- The snippets are compiled (RFC-136), so this checks the prose around them: the step order, the claimed output, what
  the window shows.

**3. A number the reference prints is a claim about a measurement.**
- `binary-size-budget.md`, `build-cost-budget.md` and `performance-envelope.md`: for each number, say what measures it and
  whether a gate re-checks it (a workflow's CSV row, a threshold in a script).
- A number nothing re-measures is **Unguarded**, even if it is true today. Say when it was last measured.
- Do not re-measure on this machine as evidence of a before and after (the cross-machine lesson). Name the gap.

**4. A security claim names its guard.** For `threat-model.md`, each mitigation is:
- a gate (`cargo-deny`, the advisory workflow, `check-workflows.sh`, a script),
- a test,
- or nothing.

A claim with no guard is a finding, not a pass. If one looks like a vulnerability, describe its class, and stop and
report it first.

**5. A covenant is guarded by a check, or it is not** (0.63 point 5). It applies to D's versioning and governance
promises ("additive-only", "deprecated before removed", "the matrix is verified on every push"):
- read the check that enforces it (`check-changelog.sh`, the feature matrix in `ci.yaml`, a source read), or say there is
  none;
- `feature-gating-criteria.md`'s matrix is compared, row by row, against `ci.yaml`'s jobs.

**6. A status in `consumer-requirements.md` is checked against the tree.** Each "met" or "not met" is a claim about the
code today: confirm it, or report it stale, with the evidence.

## Four review requests, in this order

1. `.git-exclude/review-request/audit-0-67-a/`
2. `.git-exclude/review-request/audit-0-67-b/`
3. `.git-exclude/review-request/audit-0-67-c/`
4. `.git-exclude/review-request/audit-0-67-d/`

**Each is reviewed before the next is sent.** A finding in A may change how B is read.

Each request has a `claims.md` with 0.62's columns and verdict words. Its README carries:
- the counts;
- **every false claim first;**
- the first-contact walk's results (A);
- the proposed tests, grouped by the suite they would join.

**Stop at a false claim that is a defect in snora** (not only in the sentence), and report it in the first lines. The
architect decides whether it is a defect to fix or a sentence to correct.

## Acceptance criteria

1. Every normative sentence of the scoped pages is in a `claims.md` row, or listed as not a claim with a one-word
   reason.
2. Every "Guarded" names its test or gate. One in five is shown failing by a breakage, using named `cp` backups and
   `sha256sum -c`; never `git checkout` or `git restore`.
3. Every "Unguarded" has a probe result or a reading. Every pixel claim names its renderer.
4. Every completeness claim says what it was enumerated against.
5. No product code or book page changed, and nothing committed.
