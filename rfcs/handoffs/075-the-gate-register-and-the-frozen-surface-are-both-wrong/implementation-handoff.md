# Developer Handoff — RFC-075 the gate register and the frozen surface

**Governing RFC.** **RFC-075** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Source report.** `.git-exclude/review-request/docs-and-comments-staleness-audit/`
**Status.** Inherited from RFC-075 — Accepted (owner, 2026-08-20).
**Release target.** 0.38.3 — **patch.** Documentation and doc comments only.
**Implementation units.** Four, independent. Any order.

---

## 1. Before anything: re-verify, do not inherit

The source report is good — I checked four of its load-bearing claims against
source and all four held. **Check the rest the same way.** If any finding does
not reproduce, say so; a report being mostly right is the condition under which
a wrong item slips through unexamined.

## 2. Unit 1 — the gate register must agree with itself

`contributing/api-freeze-review.md`. Its table is correct; three statements
around it are not.

| line | problem |
|---|---|
| `:110` | "Gates satisfied: … 9a, 10 = **seven of ten**, plus the binary-size half of gate 9" — omits 9b; the count is **eight of ten** |
| `:159` | "**Remaining blockers:** … compile-time measurement noise (**gate 9b**)" — 9b is not a blocker |
| `:168` | "satisfying 9b **now** … would be a quieter instance of the same mistake" — present tense about a closed gate, on a **25%** noise figure RFC-073 deleted elsewhere as stale |

**Q-2 ruled: state the count once, immediately under the table, and delete the
second restatement rather than repairing it.** Two derived copies of one fact is
how this happened; repairing both leaves two copies to drift again.

`:168`'s paragraph is about **RFC-041's precedent** and is worth keeping — but
rewrite it in the past tense so it argues about how 9b *was* handled, not about
whether to close it. Drop the 25% figure; it is the one RFC-073 removed.

`:162`'s *"the honest count was never eight of ten"* is a **correct historical
statement** about a past wrong claim. Keep it — and make sure the live count
beside it no longer contradicts it by accident.

## 3. Unit 2 — the frozen-surface list

**Q-1 ruled: delete the enumeration.**

`api-governance.md:167` already defines the surface as **"all public functions
of `snora_style`"**. That sentence is complete and cannot go stale. The bullet
list beneath it adds no governance and is the only part that can be wrong — it
names 15 of 22, missing the six `*_line_height` helpers and `theme::theme`
(absent since RFC-055, six minors).

Replace the list with one sentence naming the modules (`color`, `button`,
`container`, `progress`, `text`, `theme`) and pointing at the crate's rustdoc
for the current set.

**Do not extend the list by hand.** A hand-listed definition of a frozen surface
is a governance defect waiting to recur, and this is its second occurrence. If
you believe a reader needs the enumeration inline, stop and say so rather than
typing it — generating it is a different task with a different cost.

## 4. Unit 3 — the module that is described two ways

`snora-widgets/src/lib.rs:44-48` documents `pub mod design` as *"iced style
bridge … color conversion, semantic button styles, and card/container styles."*
`design.rs`'s own module doc, one file down, says it is prefab **widgets** and
that *"the iced style bridge … is `snora_style`, not this module."*

`design.rs` is right. Rewrite `lib.rs`'s comment to match it. **The two must
describe the same module the same way** — that is acceptance criterion 4, and it
is checkable by reading them side by side.

Then sweep the remaining RFC-055/056 relocation instances (C1–C5 in the report),
each verified against source.

## 5. Unit 4 — the rest

- **B2** `feature-gating-criteria.md` — "Ten" before the table, "Eleven" after.
  The table has 11 rows. Fix the one that is wrong; **check there is not a
  third copy** before declaring it done.
- **B3** `engine-surfaces.md:110-111` — the snippet shows `a: 0.4` (the constant
  is `DIM_ALPHA = 0.44`) and the prose credits *"iced's own public
  `iced::theme::palette::is_dark`"*. The real `is_dark` is **private**, local,
  and computes luminance itself — **`snora-design` has no iced dependency, and a
  CI gate fails if one appears** (RFC-021/022 Q3). As written the page describes
  an architecture that would fail that gate. Correct both.
- **D1** `snora-style/src/text.rs` — still says widget adoption is *"gated on an
  adopter's deferred typography assessment landing its own evidence."* **The
  evidence arrived; RFC-068 Q-2 was ruled on 2026-08-19**: short-label widgets
  will not adopt line-height, because `label` at 1.2 is tighter than iced's own
  1.3 default and line-height does nothing for a single line. Wrapping-prose
  surfaces remain open. State the ruling, not a pending gate.
- **D2** `overlay-interaction-semantics.md` — RFC-014-A described as future work
  three times while its shipped deliverable is recommended as current API.
- **E1** `snora-widgets/src/design/card.rs:8,93` — version-scoped qualifier on a
  still-true fact; the identical qualifier was fixed in a sibling location at
  0.22.0 and left here. Fix both halves this time.

## 6. Explicit non-change scope

- **No code.** `git diff -- crates/` must show doc comments only.
- **Do not extend the frozen-surface list** (§3).
- **Do not add a "one page, two claims" check.** Q-3 ruled: not yet. The count
  goes to RFC-071 Q-4, which already holds that question. Adding a fifth data
  point to it is more useful than a check nobody has specified.
- **Do not touch Category A** — that is RFC-074.
- **Do not edit any migration guide, CHANGELOG entry, or RFC** to make a
  statement current. Those are historical records.

## 7. Required evidence

- Your own re-verification of each finding, against source (§1) — including any
  that did not reproduce
- `git diff --stat -- crates/` with the doc-comment-only claim demonstrated
- `cargo test --workspace --all-features`, `cargo doc --no-deps` clean
- `mdbook build docs && mdbook test docs`; `scripts/check-built-links.py` clean
- `grep -n "9b" docs/src/contributing/api-freeze-review.md` with **every hit
  read and quoted** in the report — this is how the third statement was missed
  the first time

## 8. Acceptance criteria

1. Every gate-9b statement in `api-freeze-review.md` agrees with its own table;
   the satisfied count appears **once**.
2. The frozen-surface enumeration is **gone**, not extended.
3. `lib.rs` and `design.rs` describe the same module the same way.
4. `text.rs` states RFC-068 Q-2's ruling, not a pending gate.
5. B2, B3, C1–C5, D2, E1 corrected and individually verified.
6. `git diff -- crates/` is doc comments only.
7. `CHANGELOG.md` `[Unreleased]` under **Fixed**.

## 9. Required review-request format

`.git-exclude/review-request/075-the-gate-register-and-the-frozen-surface-are-both-wrong/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus: Unit 1's completeness.** Quote every `9b` hit and say
why each is now correct. The defect being fixed is precisely that someone
corrected the statements they were shown and not the ones they were not.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
