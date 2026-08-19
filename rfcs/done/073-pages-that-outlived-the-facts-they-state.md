# RFC 073 — Three pages that outlived the facts they state, and the pattern behind them

**Status.** Done — shipped in v0.38.2 (2026-08-20).
[Handoff](../handoffs/073-pages-that-outlived-the-facts-they-state/implementation-handoff.md).
**Tracks.** Documentation integrity.
**Found by** the architect, in a pre-cut audit of `docs/` requested before
0.38.1 was tagged. **No test can catch any of the three.**
**Touches.** `docs/src/SUMMARY.md`, `docs/src/reference/build-cost-budget.md`,
`docs/src/contributing/accessibility-checklist.md`.
**Release target.** 0.38.2 — documentation only.

## Summary

Three defects, unrelated in subject and identical in shape: **a page states
something that stopped being true and nothing was attached to it that would
notice.**

## Defect 1 — three dead links in the published book

`guides/migrations.md` links to `migration-0.4-to-0.5.md`,
`migration-0.5-to-0.6.md` and `migration-0.6-to-0.7.md`. **None of the three is
in `SUMMARY.md`, so mdBook never builds them**, and a reader clicking any of the
three gets a 404 on the published site.

**Source-level link checking does not see this.** Every relative `.md` link in
`docs/src` resolves on disk — all three target files exist. The defect lives
only in the built output: `docs/book/guides/migration-0.4-to-0.5.html` and its
two siblings are absent.

SUMMARY's migration list currently begins at `0.10 → 0.11`, so the omission is
plausibly deliberate for pre-0.7 history — but `migrations.md` links them
anyway, and a deliberate exclusion that another page links into is
indistinguishable from an accident.

## Defect 2 — `build-cost-budget.md` says gate 9b is open. It closed at v0.37.0.

Two statements, both wrong:

- **`:110`** — *"These numbers are not yet a usable trend, and gate 9b stays
  open because of it."*
- **`:136`** — *"why gate 9 is recorded as split (9a satisfied, 9b open) in
  `api-freeze-review.md`"* — which **misreports a linked page's current
  content**, by name. `api-freeze-review.md:107` records 9b as
  **✅ closed at v0.37.0**.

The section is headed *"Data integrity note (gate 9b, v0.29.0)"* and reasons
from four rows — 0.27.0 through 0.28.1. It has not been revisited in nine
releases.

**And the page now contradicts itself.** It carries two data-integrity notes on
the same subject:

| | says |
|---|---|
| `:108` (v0.29.0) | not a usable trend; 9b open; a step-change under **~25%** is indistinguishable from runner variance |
| `:246` (RFC-050) | the absolute columns are runner noise; **`design_overhead_ratio` is the trend signal**, spread **2.5%** across five rows |

The second is right and is what closed the gate. The first is what a reader
meets 138 lines earlier. This is the third instance of one page carrying two
notes that disagree — after `documentation-test-policy.md` on `no_run`
(RFC-069) and `engine-surfaces.md`'s two contrast tables (RFC-071).

## Defect 3 — the accessibility checklist calls shipped work "deferred"

`contributing/accessibility-checklist.md:250-255`:

> Snora's own prefab widgets do not yet apply it to the text they render
> internally; wiring it in (**and/or adding `*_line_height()` style helpers**)
> is **deferred, not blocked**.

**The helpers shipped in 0.38.0** (RFC-068) — six of them, one per role. And
"wiring it in" is no longer deferred either: **RFC-068 Q-2 decided it**, and
short-label widgets will not adopt line-height, because applying `label` at 1.2
is tighter than iced's own default and does nothing for a single line.

"Deferred" tells a contributor a decision is pending. Both halves are settled —
one shipped, one ruled — and this is the checklist a contributor works through.

## The pattern, and what this RFC does *not* do

All three are a claim with nothing attached that would fire. So were RFC-062's
status table, RFC-063's pair list, RFC-069's fence policy, RFC-071's two tables,
and the `iced-style-bridge.md` helper list caught in the same audit and fixed in
0.38.1.

**RFC-071 Q-4 deferred the general question** — whether a mechanism is warranted
that emits the figures a page claims. **This RFC does not answer it and must not
try.** It fixes three pages. What it adds is the third and fourth data points,
recorded so the question is decided on evidence rather than on the irritation of
the moment.

## Resolved questions — all three ruled by the owner, 2026-08-20

**Q-1 — add the three guides to `SUMMARY.md`.** They are written and already
linked; a migration index that silently omits three jumps is worse than a longer
sidebar.

**Q-2 — correct the stale section. Do not preserve it.**
**The owner's rule: information that is wrong *now* gets corrected, not
labelled.** This RFC's first draft argued for keeping the v0.29.0 measurements
as history. That was wrong, and checking dissolved the argument rather than
supporting it:

- The derived figures in that section — the 21–27% spread, the 8–11% move on a
  zero-code release — **appear nowhere else in the repository.** So the concern
  was real.
- But **their inputs are all 24 rows of committed `compile-time.csv`.** Every
  one of those percentages is recomputable from data we ship. Nothing is lost by
  deleting the prose.
- And a hand-written derived percentage sitting in prose, drifting from the CSV
  that produced it, **is the exact artifact this RFC exists to remove.**
  Preserving it would have re-created the defect while fixing it.

So: the section goes. Its conclusions survive where they belong — in the
RFC-050 note that supersedes it, which states the current signal and the current
noise floor.

**Q-3 — fix what is clearly wrong; add no rule.** `:136` narrates
`api-freeze-review.md`'s verdict and gets it wrong. Correct it to a link.
**No written rule, no mechanism** — a one-line error does not need a policy
attached, and this project has enough rules that fire on things that have
happened once.

## Acceptance criteria

1. All three migration guides build and are reachable from the published book;
   Q-1's choice recorded with its reason.
2. The `(gate 9b, v0.29.0)` section is **gone**, not re-titled. Both gate-9b
   statements are corrected, and `:136` links to `api-freeze-review.md` instead
   of narrating its verdict.
3. `build-cost-budget.md` states the trend signal in exactly one place, and
   nothing on the page disagrees with it.
4. The checklist no longer calls the `*_line_height()` helpers deferred, and
   states RFC-068 Q-2's ruling for widget adoption rather than "not yet".
5. **No new rule, policy, or checklist line is added by this RFC.**
6. **A built-output link check** — every internal link in `docs/book` resolves
   to a file that exists. The source-level check passes today and missed this.
7. No code change.

## Compatibility and security

**Compatibility.** Documentation only. **Security.** None.
