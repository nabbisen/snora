# RFC 050 — Compile-time measurement reports runner speed, not snora

**Status.** Accepted (owner, 2026-08-15) — **blocked on
[RFC-052](../done/052-clean-never-invalidates-release-artifacts.md)**; ratio
selection must be re-derived on post-fix data. See §"Blocked" below.
**Tracks.** Measurement integrity (continues RFC-041, RFC-043, RFC-044).
Closes gate **9b**.
**Touches.** `scripts/measure-compile-time.sh`,
`.github/workflows/build-cost.yaml`,
`docs/src/reference/build-cost-budget/compile-time.csv` (schema),
`docs/src/reference/build-cost-budget.md`,
`docs/src/contributing/api-freeze-review.md`.
**Release target.** 0.30.0 (minor — the CSV gains columns).

## Parked — do not implement yet (2026-08-15)

RFC-052 shipped in v0.31.0, so the clean is fixed and the two post-fix rows
(0.31.0, 0.32.0) now exist. **That is not enough data**, for two reasons:

- **Two points do not support this RFC's analysis.** The common-mode finding
  below was derived from *four* rows spanning three orders of magnitude. Two
  rows give a range, not a variance estimate.
- **The two rows bracket a crate extraction.** v0.32.0 added `snora-style`
  (RFC-055) and changed what the widgets and design builds compile, so the
  delta between them mixes real structural change with runner noise — the
  exact confound this RFC exists to separate.

**Wait for roughly two more releases of quiet data**, then re-derive. Forcing
a trend rule out of two non-comparable points would repeat the error RFC-041
was raised to end.

Gate 9b remains open and its clock is unchanged by this.

## Blocked — read before implementing

Answering Q-1 established that `cargo clean -p` **never invalidates
release-profile artifacts**, so `build_engine_only_ms` and
`build_widgets_design_ms` measure cargo's freshness check rather than a build.
[RFC-052](../done/052-clean-never-invalidates-release-artifacts.md) fixes that and
must land first.

**What survives:** the common-mode noise analysis below, computed from
`check_workspace_ms`, `build_widgets_ms` and `example_hello_ms` — all genuine
measurements — and the 0.28.0 → 0.28.1 documentation-only evidence.

**What does not:** the ratio *selection*. `widgets_design_ratio` divides a
snora-only rebuild by an iced-plus-snora cold build, and its numerator is
currently not a build at all. Its observed 7.3% stability reflects cargo's
startup consistency, not snora's compile cost. Re-derive both ratios on
post-fix data rather than reinstating these.

## Summary

The compile-time budget's six columns each vary **20–27%** across four
releases measured on identical runner, rustc and methodology. A
documentation-only release moved them **8–11%**. The numbers are dominated by
which GitHub runner the job landed on, not by anything about snora.

The variance is **common-mode**: the columns move together, so a *ratio*
between two measurements from the same run cancels the runner and leaves a
signal about snora. Measured on the existing data, that reduces spread from
27% to **3.6%**.

This RFC adds derived ratio columns, moves the watch points onto them, and
keeps the absolute columns as raw record. It does **not** add repeat runs.

## Motivation

Gate 9 was split in v0.29.0: **9a (binary size) satisfied, 9b (compile time)
open**, because "trend monitored" would claim more than the data supports.
This RFC is 9b's route to closure.

The four rows sharing runner, rustc and methodology are 0.27.0, 0.27.1,
0.28.0 and 0.28.1:

| Column | mean | stdev | CV |
|---|---:|---:|---:|
| `check_workspace_ms` | 52 342 | 4 711 | **9.0%** |
| `build_widgets_ms` | 88 975 | 9 156 | **10.3%** |
| `build_engine_only_ms` | 389 | 34 | **8.6%** |
| `example_hello_ms` | 139 545 | 16 099 | **11.5%** |
| `build_widgets_design_ms` | 502 | 47 | **9.5%** |
| `example_workbench_ms` | 5 865 | 662 | **11.3%** |

The decisive observation is **0.28.0 → 0.28.1**, which changed **no code at
all** — RFC-048 was documentation and doc comments, proven by
`git diff --stat -- 'crates/**/*.rs'` touching only comment lines — yet moved
`check_workspace_ms` **+11.3%**, `build_widgets_ms` **+9.5%** and
`example_hello_ms` **+7.7%**.

Whatever that measures, it is not snora.

## The finding this RFC turns on

**The variance is common-mode, not per-measurement.** Two pieces of evidence.

**1. The columns rank the releases identically.** Four of six produce exactly
the same ordering:

```text
0.28.0  <  0.28.1  <  0.27.0  <  0.27.1
```

The two exceptions are `build_engine_only_ms` and `build_widgets_design_ms` —
the sub-second columns, discussed below. A per-measurement random effect would
not reproduce one ordering six times; a whole run being fast or slow does.

**2. The coefficient of variation is nearly constant** — 8.6% to 11.5% across
columns spanning **three orders of magnitude** (389 ms to 139 545 ms). That is
the signature of a *multiplicative* factor applied to the whole run, which is
exactly what "this job landed on a busier or slower host" looks like.

If the effect is multiplicative and shared, dividing one column by another
from the same run cancels it.

## Evidence that ratios work

Computed on the existing four rows — no new data required:

| Ratio | 0.27.0 | 0.27.1 | 0.28.0 | 0.28.1 | spread |
|---|---|---|---|---|---|
| `example_workbench / example_hello` | 0.04271 | 0.04122 | 0.04226 | 0.04199 | **3.6%** |
| `build_widgets_design / build_widgets` | 0.00577 | 0.00538 | 0.00568 | 0.00575 | **7.3%** |
| `build_engine_only / build_widgets` | 0.00432 | 0.00394 | 0.00442 | 0.00487 | 23.5% |

The first two collapse from ~27% and ~25% raw to **3.6%** and **7.3%**. That
is the difference between a number that cannot detect a 10% regression and one
that can.

**The third does not collapse, and that is informative rather than a
counterexample.** `build_engine_only_ms` is ~389 ms. At that magnitude the
measurement is dominated by process startup, `cargo` metadata resolution and
timer granularity — effects that are *additive constants*, not multiplicative,
so a ratio does not cancel them. It is a poor numerator and a worse
denominator.

**Both ratios chosen as signals must therefore be between comparable,
similarly-sized measurements from the same run.** That constraint is the
design, not an implementation detail.

## Why not repeat runs and take a median

The obvious fix — run each measurement N times, take the median — was
considered and is **not** proposed.

- It multiplies CI cost. `example_hello_ms` alone is ~140 s; the job already
  runs uncached by design (RFC-043), and tripling it for a median is minutes
  per release for a trend signal that fails no build.
- It reduces *variance* but does not remove the *bias*: three runs on the same
  slow host still report a slow host. Median-of-N addresses within-runner
  jitter; the evidence above says the dominant effect is between-runner speed,
  which repetition on one runner cannot see.
- The ratio approach needs **no** additional CI time and demonstrably works on
  data we already have.

If ratios prove insufficient after ≥2 releases, repeat runs remain available
and this RFC's data makes the case assessable rather than speculative.

## Design

### Keep every existing column

The absolute milliseconds stay, unchanged, appended as now. They are the raw
record, they are what the 30 s watch point in
`feature-gating-criteria.md` indicator 1 is written against, and the
append-only policy (RFC-041 N-1) forbids rewriting history regardless.

They are simply **not the trend signal**.

### Add two derived ratio columns

| Column | Definition | Reads as |
|---|---|---|
| `design_overhead_ratio` | `example_workbench_ms / example_hello_ms` | what the design workbench costs relative to a minimal snora binary |
| `widgets_design_ratio` | `build_widgets_design_ms / build_widgets_ms` | what enabling `design` adds to a widgets build |

Both are computed by `measure-compile-time.sh` from values it already
collects, in the same run. **No new measurements, no extra build time.**

Emitted with enough precision to be useful (5 significant figures; the
observed values are ~0.042 and ~0.0057).

### Move the watch points onto the ratios

`build-cost-budget.md`'s "Watch points" currently trigger on absolute
milliseconds, which this RFC shows cannot distinguish a 25% regression from a
slow runner. Replace with ratio-based thresholds, and state the sensitivity
each one actually has.

The absolute 30 s watch point on `build_widgets_ms` **stays** — it is a
different kind of check (an absolute ceiling on developer experience, not a
trend), and a 10% error does not matter to it.

## Non-goals

- **No repeat runs.** See above; revisit only if ratios prove insufficient.
- **No rewriting or back-filling historical rows** — including not padding
  them with `N/A` to match the new header. Historical rows simply have fewer
  fields than the header; the next appended row is the first complete one.
  Append-only (RFC-041 N-1).
- **No CI failure gate.** Compile cost still fails no build; it is a trend
  signal read by humans.
- **No change to binary-size measurement.** Gate 9a is satisfied and
  `binary-size.csv` does not have this problem — engine size moved
  **−0.0008%** across the same documentation-only release, because binary size
  is deterministic given a toolchain and wall-clock time is not.
- **No new dependency, and no change to the uncached build policy** (RFC-043).

## Open questions

**Q-1 — Is `build_engine_only_ms` measuring anything?**
At ~389 ms for a release build of `snora` + `snora-core`, it is suspiciously
fast, and its ordering disagrees with the other columns. Either it is a
legitimately small compile, or `cargo clean -p` is not invalidating what the
script assumes. **Determine which before relying on it for anything.** If it
is not measuring a real rebuild, that is a second instance of RFC-043's defect
class and should be reported rather than quietly dropped.

**Q-2 — Should the ratio replace the absolute in the job summary?**
The GitHub job summary currently prints absolute ms. Showing a ratio nobody
has an intuition for may be worse for a human skimming a release. Suggest
printing both, with the ratio labelled as the comparable one.

## Acceptance criteria

1. `measure-compile-time.sh` emits `design_overhead_ratio` and
   `widgets_design_ratio`, computed from same-run values.
2. The CSV header documents both; historical rows carry `N/A`; no existing
   row is edited.
3. `build-cost-budget.md` states that absolute columns are runner-dominated,
   with the CV table and the 0.28.0 → 0.28.1 evidence, and moves the trend
   watch points onto the ratios while keeping the absolute 30 s ceiling.
4. Q-1 is answered in the review request with evidence either way.
5. Gate 9b's row in `api-freeze-review.md` records the new closure condition:
   **≥2 releases measured with ratio columns present**.

Gate 9b is **not** closed by this RFC landing. It closes when two post-change
data points exist — the same discipline RFC-044 applied to itself.

## Compatibility and security

**Compatibility.** The CSV gains columns; anything parsing it positionally
past column 9 breaks. Nothing in-repo does — the drift and budget tooling read
by header. No library API changes; no user-visible behaviour changes.

**Security.** No new data flow, dependency, or integration.

## Release implications

**0.30.0, minor.** No API change, but the CSV schema is a documented artifact
and gains columns, which is more than a patch should carry. No migration guide
is required — nothing downstream consumes this file.
