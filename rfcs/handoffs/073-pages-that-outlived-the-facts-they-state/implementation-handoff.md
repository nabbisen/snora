# Developer Handoff — RFC-073 three stale pages

**Governing RFC.** **RFC-073** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-073 — Accepted (owner, 2026-08-20).
**Release target.** 0.38.2 — **patch, documentation only.** No code.
**Implementation units.** Three, independent. Any order.

---

## 1. Purpose

Three pages state things that stopped being true. All three questions are ruled;
**none is open.** Do not reopen them.

## 2. Unit 1 — three dead links in the published book

`guides/migrations.md` links `migration-0.4-to-0.5.md`,
`migration-0.5-to-0.6.md`, `migration-0.6-to-0.7.md`. None is in `SUMMARY.md`,
so mdBook does not build them and every reader clicking gets a **404 on the
published site**.

**Ruling: add all three to `SUMMARY.md`**, in the migration list, in the same
descending order the existing entries use (the list currently begins at
`0.10 → 0.11`).

**Confirm by the built output, not the source.** All three `.md` files already
exist on disk, so every source-level link check passes today and passed while
the defect shipped. The check that matters:

```
mdbook build docs && ls docs/book/guides/migration-0.{4-to-0.5,5-to-0.6,6-to-0.7}.html
```

## 3. Unit 2 — `build-cost-budget.md`: delete the stale section

**Ruling: the `### Data integrity note (gate 9b, v0.29.0)` section is deleted,
not re-titled.**

The RFC's first draft argued for keeping it as history. The owner ruled against,
and checking supports the ruling: its derived percentages appear nowhere else in
the repository, **but their inputs are all 24 rows of committed
`compile-time.csv`**, so every figure is recomputable from data we ship.
Deleting the prose loses no fact. Keeping a hand-written percentage that drifts
from the CSV that produced it would re-create the exact defect this RFC removes.

Delete it. Its conclusions already live in the `### Data integrity note
(RFC-050)` section further down, which states the current signal
(`design_overhead_ratio`) and the current noise floor.

Then fix the two wrong statements:

- **`:110`** — *"gate 9b stays open"*. **It closed at v0.37.0.** Goes with the
  section.
- **`:136`** — *"why gate 9 is recorded as split (9a satisfied, 9b open) in
  `api-freeze-review.md`"*. This **narrates a linked page's verdict and gets it
  wrong**; `api-freeze-review.md:107` records 9b as closed at v0.37.0. Replace
  the narration with a link and let the register speak for itself.

**Do not add a rule about this** (§5).

**Check afterwards:** the page states what the trend signal is in exactly one
place. Grep it — `grep -n "9b\|trend signal" docs/src/reference/build-cost-budget.md`
— and read every hit.

## 4. Unit 3 — the accessibility checklist calls shipped work deferred

`contributing/accessibility-checklist.md:250-255` says wiring line-height into
prefab widgets *"(and/or adding `*_line_height()` style helpers) is deferred,
not blocked."*

Both halves are settled:

- **The helpers shipped in 0.38.0** — six of them, one per role (RFC-068).
- **Widget adoption is ruled, not deferred** — RFC-068 Q-2: short-label widgets
  will **not** adopt line-height, because `label` at 1.2 is tighter than iced's
  own `Relative(1.3)` default and line-height does nothing for a single line.
  Surfaces that can render wrapping prose remain open.

Rewrite the item to point at the helpers and state the ruling. **"Deferred"
tells a contributor a decision is pending; it is not.**

## 5. Explicit non-change scope

- **No new rule, policy, or checklist line.** Q-3 was ruled: fix what is wrong,
  add no ceremony. A one-line error does not need a policy attached.
- **No code.** If this touches `crates/` or `examples/`, it is wrong.
- **Do not re-title or partially preserve the v0.29.0 section** (§3).
- **Do not touch the RFC-050 section.** It is correct and is what supersedes the
  deleted one.
- **Do not answer RFC-071 Q-4** — whether a general mechanism should emit the
  figures pages claim. Still deferred, deliberately.

## 6. Required evidence

- `mdbook build docs && mdbook test docs`
- The three built migration-guide HTML files listed (§2)
- **A built-output link check**: every internal link under `docs/book` resolves
  to a file that exists. Write it however you like; state the method and the
  count checked. **This is the check that would have caught Unit 1**, and the
  source-level equivalent would not have.
- `git diff --stat -- crates/ examples/` — **expected empty**

## 7. Acceptance criteria

1. Three migration guides reachable from the published book.
2. The `(gate 9b, v0.29.0)` section is gone; both 9b statements corrected;
   `:136` links rather than narrates.
3. `build-cost-budget.md` names the trend signal in exactly one place.
4. The checklist states the ruling instead of "deferred".
5. No new rule, policy, or checklist line anywhere.
6. Built-output link check run, method and count stated.
7. `git diff -- crates/ examples/` empty; `CHANGELOG.md` `[Unreleased]` records
   all three under **Fixed**.

## 8. Required review-request format

`.git-exclude/review-request/073-pages-that-outlived-the-facts-they-state/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus:** the built-output link check (§6) — its method, not
its result. It is the only part of this work that generalises beyond the three
pages being fixed.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
