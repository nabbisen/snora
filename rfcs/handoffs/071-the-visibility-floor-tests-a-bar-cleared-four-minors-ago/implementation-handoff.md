# Developer Handoff — RFC-071 the visibility floor

**Governing RFC.** **RFC-071** — find it under `rfcs/done/`, `rfcs/accepted/` or `rfcs/archive/` via the index at [`rfcs/README.md`](../../README.md). *(Not linked by folder: an RFC moves between folders as it ships, and a folder-bearing link here goes stale the moment it does — 14 of them had.)*
**Status.** Inherited from RFC-071 — Accepted (owner, 2026-08-19).
**Release target.** 0.38.1 — **patch**. Test constant and documentation only;
no public API, no palette value, no rendered output.
**Implementation units.** One. **Do this before RFC-070** — it is an assertion
that currently guards nothing.

---

## 1. Task title

Raise `VISIBILITY_FLOOR` to `NON_TEXT_MIN`, and replace both stale contrast
tables on `engine-surfaces.md` with re-derived figures.

## 2. Purpose

`docs/src/design/engine-surfaces.md` carries two "measured contrast" tables.
**Both are stale, each frozen at the values from before the repair that changed
them.**

| table | published | actual | stale since |
|---|---|---|---|
| border vs background, `light` | 1.39:1 | **3.38:1** | 0.34.0 (RFC-058) |
| border vs background, `dark` | 1.43:1 | **3.81:1** | 0.34.0 (RFC-058) |
| dim over background, `light` | 2.85:1 | **3.2424:1** | 0.37.0 (RFC-065) |
| dim over background, `dark` | 3.83:1 | **4.3798:1** | 0.37.0 (RFC-065) |
| dim over background, `hc_light` | 2.85:1 | **3.2424:1** | 0.37.0 |
| dim over background, `hc_dark` | 3.66:1 | **4.2529:1** | 0.37.0 |

The high-contrast border figures (21.0:1) are correct.

**Re-derive every one of these yourself.** The table above is a cross-check, not
a source — RFC-071's own first draft copied a figure off the page and was wrong
within the hour. If yours disagree, **yours win and the RFC is wrong**; say so
rather than reconciling to it.

`DIM_ALPHA`'s doc comment in `crates/snora-design/src/surfaces.rs` already
carries the correct `3.24:1` for `light`. **The constant and the page have
disagreed since the day RFC-065 shipped.** That is a useful independent check on
your derivation.

## 3. Why the assertion matters more than the tables

`VISIBILITY_FLOOR = 1.3` (`crates/snora/src/design/render/tests.rs:50`) is
justified in its own doc comment as *"what the `border` role … actually
achieves across all four built-in presets, with a small margin under the worst
case (`light` at 1.39)"*, and as deliberately below WCAG SC 1.4.11's `3.0`
because a real token value could not reach it.

Every clause is false. The worst case is 3.38. The margin is 2.6×. And the real
token clears 3.0 in all four presets.

**So `light`'s border could regress from 3.38 to 1.31 and this assertion passes.**
The 0.34.0 repair — RFC-036's first use of the accessibility carve-out, a
palette value changed *because a test proved a defect* — is unprotected by the
test that measures it. That is the defect worth fixing; the tables are the
paper trail that hid it.

## 4. Q-1 and Q-3 are decided — do not reopen them

**Q-1 — keep the constant shared. Do not split it.**

The RFC's first draft proposed splitting, because the dim appeared stuck at
2.85 and unable to reach 3.0. **It reaches 3.2424.** Both assertions clear
`NON_TEXT_MIN` with margin — border worst 3.38, dim worst 3.2424 — so one
constant serves both and the conceptual argument for sharing ("is this element
visually distinct from the page behind it") needs no compromise.

**Q-2 — the floor is `NON_TEXT_MIN` (3.0).** It already exists as a constant
(RFC-058). Use it; do not introduce a second spelling of 3.0.

Know what this commits us to, and write it in the doc comment: **under RFC-036,
the `border` values and `DIM_ALPHA` become effectively frozen at ≥3.0**,
changeable only through the accessibility carve-out. That is not a new
constraint being invented — 0.34.0's border repair and RFC-065's `DIM_ALPHA`
were both chosen specifically to clear 3.0. **This asserts an intent we already
acted on twice.**

**Q-3 — the dim's table was stale too.** Answered by re-deriving it. The
instruction that produced the answer stands as the rule for this work:
**re-derive; never assume newer means current.**

## 5. The doc comment is part of the fix

`VISIBILITY_FLOOR`'s current comment is wrong in every factual clause, and it
is *why* nobody noticed — it reads as a considered choice. Replace it with one
that states:

- the floor and that it is `NON_TEXT_MIN` / WCAG SC 1.4.11;
- **the measured worst case of each assertion, and the release it was measured
  at** — so the next drift is visible rather than inferred;
- that both assertions share it deliberately, and why.

**A justification that cannot go stale silently is the actual deliverable.**

## 6. Change scope

| File | Change |
|---|---|
| `crates/snora/src/design/render/tests.rs` | `VISIBILITY_FLOOR` → `NON_TEXT_MIN`; the doc comment per §5 |
| `docs/src/design/engine-surfaces.md` | Both tables re-derived; the prose that cites 1.39 as the worst case; the claim that a real token value sits below 3.0 |
| `CHANGELOG.md` | `[Unreleased]` → **Fixed** |

## 7. Explicit non-change scope

- **No palette value changes. No `DIM_ALPHA` change.** This repairs an
  assertion and a page, not a colour. If a preset fails the new floor, **stop
  and report** — that is a finding, not a licence to adjust the colour.
- **Do not split `VISIBILITY_FLOOR`** (§4).
- **Do not touch the 0.34.0 CHANGELOG table.** It is correct — verified against
  source this week after a consumer reported otherwise — and changelog entries
  are historical records, not live documentation.
- **Do not extend this to other pages' figures.** RFC-071 Q-4 asks whether a
  general mechanism is warranted and explicitly defers it.

## 8. Required tests

The floor change is the test. What is required beyond it is proof it fires:

**A perturbation demo.** Move `light`'s `border` toward `background` until it
crosses 3.0, capture the failure naming the preset and the ratio, restore.
Then do the same for the dim by lowering `DIM_ALPHA` — the dim assertion has
never failed either, and it is the one whose margin is thinnest (3.24 against
3.0, 8%).

Two captured failures, two restores, `git status` clean afterwards.

## 9. Required evidence

- `cargo test -p snora --all-features` — pass
- `cargo test --workspace --all-features` — pass
- The two perturbation captures, each showing the failure **and** the restored
  green run
- **Your own derivation of all ten figures**, as a table, with the method
  stated — this is the deliverable the tables depend on
- `mdbook build docs && mdbook test docs`
- `git diff --stat -- 'crates/snora-design/src'` — **expected empty** (no
  palette, no `DIM_ALPHA`)

## 10. Acceptance criteria

1. `VISIBILITY_FLOOR` is `NON_TEXT_MIN`, not a second spelling of 3.0, and not
   split.
2. Its doc comment states each assertion's measured worst case **and the
   release measured at**.
3. Both `engine-surfaces.md` tables carry re-derived figures; the "worst case
   1.39" prose and the "below 3.0" rationale are gone.
4. Both perturbation demos captured and restored.
5. No palette value and no `DIM_ALPHA` changed.
6. `CHANGELOG.md` records it under **Fixed** — an assertion that could not
   catch what it exists for is a defect, not a change.

## 11. Required review-request format

`.git-exclude/review-request/071-the-visibility-floor-tests-a-bar-cleared-four-minors-ago/`,
`README.md` as entry point, evidence under `evidence/`, relative paths, single
entry-point path in the completion summary.

**Requested review focus:** your independent derivation of the ten figures
(§9). Everything else follows from it, and it is the one part where copying
rather than deriving would reproduce the exact defect being fixed.

**Scope deviations are authorized by the architect, in writing, and by nobody
else.**
