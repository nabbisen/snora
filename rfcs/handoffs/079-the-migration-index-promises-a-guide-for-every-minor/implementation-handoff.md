# Developer Handoff — RFC-079 a guide for every minor

**Governing RFC.** **RFC-079** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-079 — Accepted (owner, 2026-08-20).
**Release target.** 0.39.1 — documentation and one script. **No code.**
**Implementation units.** Two.

---

## 1. The rule, ruled

**Every minor release ships a migration guide. No condition.** A guide for a
minor that broke nothing says so in a sentence.

Two earlier formulations were rejected and are recorded in the RFC because how
each failed is the point. **Do not reintroduce either** — in particular, do not
add a clause of the form "a guide when there is something to say." That reads as
one rule and behaves as a per-release judgement call, which is how the defect
happened.

**`docs/src/guides/migrations.md` already states the rule correctly** — *"Each
minor release ships a focused migration guide"* — and **does not change.** The
other two documents are amended to match it:

- **`contributing/release-process.md:82`** — make the checklist line
  unconditional and unambiguous for minors, and point it at Unit 2's check.
- **`contributing/versioning-policy.md`** — its version-level table gives
  additive API a *"Docs update"* and requires a guide only for
  rename/removal/feature-flag-rename. **That is the statement that disagrees.**
  Amend it so a migration guide is required for every minor, with the
  rename/removal rows keeping their *additional* requirements (deprecation
  alias, etc.).

**Nothing is invented.** The double standard dies by deleting two dissenting
statements, not by writing a third.

## 2. Unit 1 — the five remaining gaps are NOT backfilled

**Owner's ruling: 0.38 → 0.39 only, and it is already written and shipped**
(`docs/src/guides/migration-0.38-to-0.39.md`). It is also the worked example of
the rule: nothing required, said in a sentence, with the two things worth
knowing anyway.

Still missing: **0.29→0.30, 0.30→0.31, 0.31→0.32, 0.34→0.35, 0.37→0.38.**

**Name them in `migrations.md` as known gaps.** A stated gap is honest; a silent
one is the defect this RFC exists to close. One line is enough — that these
predate the rule's adoption and are not yet written.

**Do not write them.** Not "if you have time". They are deferred by the owner.

## 3. Unit 2 — the check, and the trap in it

*For every minor tag, a guide file exists.* Derivable from `git tag` and the
filesystem; it would have caught all six.

**The trap: five gaps exist right now.** A check that fails on the day it ships,
and is *expected* to fail, is a check people learn to ignore — and we have no way
to notice one that someone quietly stopped running.

**Resolve it with a single boundary value, not a list of exceptions.** An
exceptions list is a hand-maintained enumeration and is the RFC-063 shape this
project keeps deleting.

- The check reports **every** gap it finds, with the count.
- It **exits non-zero only for minors at or after the rule's adoption version**
  (`0.39`), which is one constant in the script.
- Earlier gaps are reported as **known historical**, and the script says so
  in its own output rather than being silent about why they do not fail.

Same shape as the two checks already in `scripts/`: committed, runnable,
inventoried in `scripts/README.md`, **not a CI gate.**

## 4. Explicit non-change scope

- **`migrations.md`'s promise sentence does not change.** It was right.
- **Do not write the five missing guides.**
- **No conditional clause in the rule** (§1).
- **No CI gate.**
- **No code.** If this touches `crates/`, it is wrong.

## 5. Required evidence

- The three documents quoted side by side afterwards, showing they agree
- The check's output on the current tree: five known-historical gaps, zero at or
  after 0.39, exit zero
- **A perturbation demo, both directions:** delete
  `migration-0.38-to-0.39.md`, run the check, see it **fail** (a post-adoption
  minor); restore. And confirm the five historical gaps do **not** fail it.
- `mdbook build docs && mdbook test docs`; `scripts/check-built-links.py` clean
- `git diff --stat -- crates/` — **expected empty**

## 6. Acceptance criteria

1. All three documents state the same rule; `migrations.md` unchanged.
2. No conditional clause anywhere in the rule's wording.
3. The five gaps are named in the index as known and unwritten; none is written.
4. The check exists, uses one boundary constant, reports all gaps, fails only on
   post-adoption ones.
5. Both perturbation directions captured.
6. Script committed, inventoried, not in CI. `CHANGELOG.md` `[Unreleased]`.

## 7. Required review-request format

`.git-exclude/review-request/079-the-migration-index-promises-a-guide-for-every-minor/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus: the boundary constant and the two-direction demo.**
Whether the check survives its own first day is the only part of this that can
fail quietly.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
