# Developer Handoff — RFC-036 design surface freeze and additive-only covenant

**Governing RFC.** [RFC-036](../../proposed/036-design-surface-freeze-and-additive-covenant.md)
**Status.** Inherited from RFC-036 (Proposed; accepted by the owner).
**Release target.** 0.25.3 (patch), alongside RFC-035 and RFC-041.
**Implementation units.** One. Documentation and governance only.

---

## 1. Task title

Record the design-surface freeze review, close design gates D-3 and D-4,
and write the additive-only covenant that constrains the v0.26 work.

## 2. Purpose

D-3 and D-4 have been eligible to close since v0.22 and have stayed open
for want of a recorded review. Meanwhile the v0.26 milestone will extend
the design layer. Closing a stability gate immediately before expanding the
surface it covers would hollow out the gate — so the closure and the
constraint that keeps it honest land together.

## 3. Background

RFC-034 defines the design D-gates. RFC-036 §Evidence records a review
spanning six consecutive minors (v0.20 → v0.25). Read RFC-036 in full
before starting; this handoff executes it and does not restate its
reasoning.

Read before starting:

- `rfcs/proposed/036-design-surface-freeze-and-additive-covenant.md`
- `docs/src/contributing/api-governance.md` (where the covenant lands)
- `docs/src/contributing/api-freeze-review.md` (where the gates live)

Conventions (the owner's rules document is not in the repository):

- **English only** for all prose.
- No code changes at all in this handoff, so the `cargo fmt` ordering rule
  does not apply — but do not run workspace-wide `cargo fmt`: this repo has
  ~152 hunks of pre-existing drift unrelated to any current work.

## 4. Applicable requirements

- **RFC-034** design gate definitions (D-1 … D-8)
- **DEC-12** `Palette::roles()` narrowed to test-only
- **NF-1** `snora-core` / `snora-design` iced-free
- Requirements §1.7 "Preserve the *why*"

## 5. Change scope

| File | Purpose |
|---|---|
| `docs/src/contributing/api-freeze-review.md` | D-3 and D-4 rows → ✅ |
| `docs/src/contributing/api-governance.md` | The additive-only covenant |
| `CHANGELOG.md` | `[Unreleased]` entry |

## 6. Explicit non-change scope

Do **not** touch:

- Any source file. This handoff changes zero code.
- Any **core** 1.0 gate row (gates 1–10). Only D-3 and D-4 change here.
  Gate 9 is RFC-041's business — do not touch it in this change even
  though it is on the same page.
- D-1, D-2, D-5, D-6, D-7, D-8 rows.
- Any file under `rfcs/done/`.
- `crates/snora-design/` or `crates/snora-widgets/src/design/` — the
  covenant *describes* these surfaces; it does not modify them.

## 7. Required implementation

### Step 1 — Re-verify the evidence yourself

Do not take RFC-036's evidence table on trust. Run:

```bash
git diff 0.20.0 0.25.2 -- crates/snora-design/src/
git diff 0.20.0 0.25.2 -- crates/snora-widgets/src/design/style/
```

Confirm:

- `snora-design` changed in exactly two files — `palette.rs` (`roles()`
  narrowed from `pub` to `#[cfg(test)] pub(crate)`) and `contrast.rs`
  (`composite_over` gained a debug-only precondition plus docs).
- The 18 `Palette` role fields are unchanged.
- The style bridge changed by addition only (`style::progress`, v0.21).

**If any of this does not hold, stop and escalate.** The gate closure
depends on it. Report what you found either way.

### Step 2 — Close D-3 and D-4

In `docs/src/contributing/api-freeze-review.md`, in the "Snora Design gate
set" table:

- **D-3** → `✅ v0.20–v0.25 (token model unchanged across six consecutive
  minors; freeze review RFC-036)`
- **D-4** → `✅ v0.20–v0.25 (style bridge additive-only across six
  consecutive minors; freeze review RFC-036)`

Add a short paragraph beneath the table noting that the closure is
**qualified**: one public item was removed (`Palette::roles()`, DEC-12)
and one contract tightened (`composite_over`'s debug precondition) during
the span. Both were deliberate SemVer hardening and neither changed the
token model — but the record must say so rather than claim an unbroken
surface. RFC-036 §Evidence has the wording; do not overstate it.

### Step 3 — Write the covenant

In `docs/src/contributing/api-governance.md`, add a section titled
**"Additive-only covenant (design surface)"** containing, from RFC-036
§"The frozen surface" and §"The additive-only covenant":

1. The **frozen surface**, itemised — all public items of `snora-design`,
   and all public functions of `snora_widgets::design::style`. List them
   explicitly; a vague description is not a covenant.
2. An explicit note that the design **primitives** (`button`, `card`,
   `notice`, `chip`, `progress`) are **not** frozen — they remain under
   the RFC-034 promotion lifecycle.
3. What is **permitted** (additions; preset value changes only where a
   contrast test proves an accessibility fix, recorded as **Fixed**).
4. What is **forbidden** (removal, rename, retype, signature change, any
   `Palette` role change, and *meaning* changes to an existing token even
   where the type is untouched).
5. The **reopening obligation**: an RFC needing a forbidden change must say
   so and **reset D-3/D-4 to open in the same change** — it may not proceed
   and rationalise afterwards.

### Step 4 — CHANGELOG

Add to `[Unreleased]` under **Changed** (governance, not a bug fix): the
design-surface freeze review is recorded, D-3/D-4 close, and the
additive-only covenant now governs the design surface. One short entry.

## 8. Required tests

No new tests; this handoff changes no code.

```bash
mdbook build docs
mdbook test docs
```

Both must exit 0.

## 9. Acceptance criteria

RFC-036 §Acceptance criteria, items 1–5. In particular:

- No gate row other than D-3 and D-4 changes value.
- The covenant lists the frozen surface item by item.
- The two `git diff` commands reproduce RFC-036's Evidence section.

## 10. Prohibited shortcuts

- Do not close D-3/D-4 without running Step 1 yourself.
- Do not describe the surface as "unchanged" — it is not, and RFC-036 is
  explicit that the qualification is recorded rather than smoothed over.
- Do not write the covenant as prose that gestures at "be careful with the
  design surface". It must be an enumerated list a reviewer can check
  mechanically.
- Do not touch gate 9 — RFC-041 owns it, and both changes target the same
  file. Coordinate ordering with the architect if both land together.

## 11. Compatibility and security

Neither is affected: no code, no API, no dependency, no data flow change.
State this explicitly in the review request.

## 12. Known risks

Per RFC-036 §Risks. The one you control: reading "additive" loosely. The
forbidden list names *meaning* changes, not only signature changes — write
it that way.

## 13. Required evidence

- Output of both Step 1 `git diff` commands.
- Diff of the two documentation files.
- `mdbook build docs` and `mdbook test docs` output.
- Explicit confirmation that no gate row other than D-3/D-4 changed.

## 14. Required review-request format

Per workflow policy §9.2 and the packaging convention: a `README.md` entry
point, a full `review-request.md`, and an `evidence/` directory, under
`.git-exclude/review-request/036-design-surface-freeze-and-additive-covenant/`.
Report paths relative to the project root.
