# RFC 050 — Compile-time measurement reports runner speed, not snora

**Status.** Done — shipped in v0.35.0 (2026-08-18). Parked 2026-08-15 pending
post-RFC-052 data; unparked and re-derived 2026-08-18** on five post-fix rows.
The re-derivation changed the design — see §"What the post-fix data changed".
**Tracks.** Measurement integrity (continues RFC-041, RFC-043, RFC-044).
Closes gate **9b**.
**Touches.** `scripts/measure-compile-time.sh`,
`.github/workflows/build-cost.yaml`,
`docs/src/reference/build-cost-budget/compile-time.csv` (schema),
`docs/src/reference/build-cost-budget.md`,
`docs/src/contributing/api-freeze-review.md`.
**Release target.** 0.35.0 (minor — the CSV gains a column).

## Summary

The compile-time budget's six columns each vary **36–60%** across five releases
measured on identical runner, rustc and methodology. A documentation-only
release moved every one of them **36–55%**. The numbers are dominated by which
GitHub runner the job landed on, not by anything about snora.

The variance is **common-mode**: the columns move together, so a *ratio*
between two measurements from the same run can cancel the runner and leave a
signal about snora. One such ratio reduces spread from 41% to **2.5%**.

This RFC adds that derived ratio column, moves the trend watch points onto it,
and keeps the absolute columns as raw record. It does **not** add repeat runs.

## Motivation

Gate 9 was split in v0.29.0: **9a (binary size) satisfied, 9b (compile time)
open**, because "trend monitored" would claim more than the data supports. This
RFC is 9b's route to closure.

The five rows sharing runner, rustc and post-RFC-052 methodology are 0.31.0,
0.32.0, 0.33.0, 0.33.1 and 0.34.0:

| Column | mean (ms) | raw spread |
|---|---:|---:|
| `check_workspace_ms` | 53 138 | **60.4%** |
| `build_widgets_ms` | 90 819 | **54.8%** |
| `build_engine_only_ms` | 448 | **42.5%** |
| `example_hello_ms` | 143 227 | **40.7%** |
| `build_widgets_design_ms` | 552 | **36.4%** |
| `example_workbench_ms` | 6 061 | **41.6%** |

The decisive observation is **0.33.0 → 0.33.1**, which changed **no code at
all** — RFC-057 was documentation and doc comments — yet moved every column:

| Column | 0.33.0 | 0.33.1 | change |
|---|---:|---:|---:|
| `check_workspace_ms` | 39 270 | 59 139 | **+50.6%** |
| `build_widgets_ms` | 67 296 | 104 190 | **+54.8%** |
| `build_engine_only_ms` | 355 | 504 | **+42.0%** |
| `example_hello_ms` | 111 226 | 156 549 | **+40.7%** |
| `build_widgets_design_ms` | 453 | 618 | **+36.4%** |
| `example_workbench_ms` | 4 690 | 6 564 | **+40.0%** |

Whatever that measures, it is not snora.

**Note the direction of travel.** The same analysis on the pre-RFC-052 rows
gave 21–27% spread and an 8–11% documentation-only move. Both have roughly
doubled. **Collecting more rows is making this gate less satisfiable, not
more** — which is the strongest argument that the methodology, not the sample
size, is the problem.

## The finding this RFC turns on

**The variance is common-mode, not per-measurement.** The columns rank the
releases near-identically — 0.33.0 is the fastest row on all six columns,
0.33.1 among the slowest on all six — and the spread is of a similar order
across columns spanning **three orders of magnitude** (448 ms to 143 227 ms).
That is the signature of a *multiplicative* factor applied to the whole run,
which is exactly what "this job landed on a busier or slower host" looks like.

If the effect is multiplicative and shared, dividing one column by another from
the same run cancels it.

## What the post-fix data changed

The parked note required the ratio *selection* to be re-derived once RFC-052's
clean fix had produced comparable rows. It has been, on all fifteen candidate
ratios. **Two of the original RFC's conclusions did not survive.**

### 1. `widgets_design_ratio` must be dropped

| Ratio | pre-RFC-052 | **post-RFC-052** |
|---|---:|---:|
| `example_workbench / example_hello` | 5.1% | **2.5%** |
| `build_widgets_design / build_widgets` | 5.9% | **14.9%** |

The parked note suspected exactly this: *"its observed 7.3% stability reflects
cargo's startup consistency, not snora's compile cost."* Confirmed. With the
clean fixed, the ratio is worse than several columns' raw spread and cannot
detect a 10% regression. **It is not a signal and this RFC no longer proposes
it.**

### 2. The stated selection principle was wrong

The original RFC concluded that both ratios must be *"between comparable,
similarly-sized measurements"*. The full sweep refutes this:

| Ratio | size ratio | spread |
|---|---:|---:|
| `example_workbench / example_hello` | 23.7× | **2.5%** |
| `build_widgets / example_hello` | 1.6× | 10.0% |
| `check_workspace / example_hello` | 2.6× | 14.7% |

The best pair is the one with the **largest** size disparity of the three, and
two near-equally-sized pairs perform six to eight times worse. Similar
magnitude is neither sufficient nor apparently necessary.

**The actual distinguishing property is that numerator and denominator do the
same kind of work under the same profile.** `example_workbench_ms` and
`example_hello_ms` are the only two measurements that are both
`--profile release-baseline` builds of an example *binary*. Every other pair
mixes a `cargo check` (dev, no codegen) with a `--release` library build, or a
library build with a binary link — different work, whose costs do not scale
together under a common runner factor.

This matters beyond bookkeeping: it is the rule for adding any future ratio.
**Pair like with like, or the cancellation does not happen.**

## Evidence that the surviving ratio works

Computed on the five post-RFC-052 rows:

| | 0.31.0 | 0.32.0 | 0.33.0 | 0.33.1 | 0.34.0 | spread |
|---|---|---|---|---|---|---|
| `example_workbench_ms` raw | 6 641 | 6 561 | 4 690 | 6 564 | 5 851 | 41.6% |
| `example_hello_ms` raw | 155 591 | 156 534 | 111 226 | 156 549 | 136 236 | 40.7% |
| **ratio** | 0.04268 | 0.04191 | 0.04217 | 0.04193 | 0.04295 | **2.5%** |

A **16-fold** noise reduction. **No other candidate comes close** — the next
best of fifteen is 8.8%. This is one usable signal, not a family of them, and
the RFC is scoped to what the data supports.

### The 0.34.0 row is an out-of-sample test, and the ratio passed it

0.34.0 was measured **after** this ratio was selected, on data that did not
exist when the selection was made. It is therefore a genuine out-of-sample
check rather than more of the fit:

- The raw columns moved **−10.9% to −21.2%** from 0.33.1 — in the *opposite*
  direction from the +36% to +55% the documentation-only 0.33.1 had produced,
  and of comparable magnitude.
- 0.34.0 **did** change code (four preset colour values, RFC-058) where 0.33.1
  changed none. The raw columns cannot tell those two releases apart: one moved
  them sharply up with no code, the other sharply down with code.
- Through both, the ratio stayed inside 0.0419–0.0430.

That is the thesis stated as cleanly as the data can state it: the absolute
columns respond to the runner, not to snora, in both directions and regardless
of what shipped — and the ratio does not.

For contrast, `binary-size.csv` over the same release: engine size moved
15 740 880 → 15 741 008 bytes, **+0.0008%**. Gate 9a's series behaves; 9b's
does not.

## Why not repeat runs and take a median

The obvious fix — run each measurement N times, take the median — was
considered and is **not** proposed.

- It multiplies CI cost. `example_hello_ms` alone is ~145 s; the job already
  runs uncached by design (RFC-043), and tripling it for a median is minutes
  per release for a trend signal that fails no build.
- It reduces *variance* but does not remove the *bias*: three runs on the same
  slow host still report a slow host. Median-of-N addresses within-runner
  jitter; the evidence above says the dominant effect is between-runner speed,
  which repetition on one runner cannot see.
- The ratio approach needs **no** additional CI time and demonstrably works on
  data we already have.

If the ratio proves insufficient after ≥2 releases, repeat runs remain
available and this RFC's data makes the case assessable rather than
speculative.

## Design

### Keep every existing column

The absolute milliseconds stay, unchanged, appended as now. They are the raw
record, they are what the 30 s watch point in `feature-gating-criteria.md`
indicator 1 is written against, and the append-only policy (RFC-041 N-1)
forbids rewriting history regardless.

They are simply **not the trend signal**.

### Add one derived ratio column

| Column | Definition | Reads as |
|---|---|---|
| `design_overhead_ratio` | `example_workbench_ms / example_hello_ms` | what the design workbench costs relative to a minimal snora binary |

Computed by `measure-compile-time.sh` from values it already collects, in the
same run. **No new measurements, no extra build time.** Emitted at 5
significant figures (observed values ~0.042).

### Move the trend watch points onto the ratio

`build-cost-budget.md`'s "Watch points" currently trigger on absolute
milliseconds, which this RFC shows cannot distinguish a 50% regression from a
slow runner. Replace the trend points with a ratio threshold, and state the
sensitivity it actually has.

The absolute 30 s watch point on `build_widgets_ms` **stays** — it is a
different kind of check (an absolute ceiling on developer experience, not a
trend), and runner error does not matter to it.

### Record what the sub-second columns are for

`build_engine_only_ms` (~448 ms) and `build_widgets_design_ms` (~552 ms) are
legitimate measurements — `iced` is a non-optional dependency of `snora`, and
these rebuild snora's own crates against a warm dependency graph — but at that
magnitude they are dominated by process startup, cargo metadata resolution and
timer granularity. **They are raw record only, and no ratio may be built from
them.** State this in `build-cost-budget.md` so the next person does not
re-derive the dropped ratio.

## Non-goals

- **No repeat runs.** See above; revisit only if the ratio proves insufficient.
- **No `widgets_design_ratio`.** Dropped on post-fix evidence.
- **No rewriting or back-filling historical rows** — including not padding them
  with `N/A` to match the new header. Historical rows simply have fewer fields
  than the header; the next appended row is the first complete one. Append-only
  (RFC-041 N-1).
- **No CI failure gate.** Compile cost still fails no build; it is a trend
  signal read by humans.
- **No change to binary-size measurement.** Gate 9a is satisfied and
  `binary-size.csv` does not have this problem — engine size moved
  **−0.0008%** across a documentation-only release, because binary size is
  deterministic given a toolchain and wall-clock time is not.
- **No new dependency, and no change to the uncached build policy** (RFC-043).
- **No new measurement columns.** Adding one would reset the comparability
  clock again; this RFC derives from what is already collected.

## Open questions

**Q-1 — Should the ratio replace the absolute in the job summary?**
The GitHub job summary currently prints absolute ms. Showing a ratio nobody has
an intuition for may be worse for a human skimming a release. Suggest printing
both, with the ratio labelled as the comparable one.

**Q-2 — Does gate 9b's closure condition need the noise statement attached?**
Two post-change data points satisfy the letter. But a future reader finding
"trend monitored ✅" should be able to see immediately that the *absolute*
columns remain runner-dominated and only the ratio is a trend. Suggest the gate
row state both.

## Acceptance criteria

1. `measure-compile-time.sh` emits `design_overhead_ratio`, computed from
   same-run values, at 5 significant figures.
2. The CSV header documents it, **appended as the last field, not inserted** —
   see Compatibility. **No existing row is edited or padded** — historical rows
   stay short (this supersedes the earlier draft's contradictory "historical
   rows carry `N/A`").
3. `build-cost-budget.md` states that absolute columns are runner-dominated,
   with the post-RFC-052 spread table and the 0.33.0 → 0.33.1
   documentation-only evidence, and moves the trend watch points onto the ratio
   while keeping the absolute 30 s ceiling.
4. `build-cost-budget.md` records that the sub-second columns are raw record
   only and that `widgets_design_ratio` was derived, tested on post-fix data at
   14.9%, and rejected — so it is not proposed again.
5. Gate 9b's row in `api-freeze-review.md` records the new closure condition:
   **≥2 releases measured with `design_overhead_ratio` present.**

Gate 9b is **not** closed by this RFC landing. It closes when two post-change
data points exist — the same discipline RFC-044 applied to itself.

## Compatibility and security

**Compatibility.** The CSV gains a column. ~~Nothing in-repo parses it
positionally — the drift and budget tooling read by header.~~

**Wrong, corrected during implementation review (2026-08-18).**
`release-process.md`'s release checklist does `cut -d, -f9` on this exact file
to content-check `runner_os`, and `-f2` to sanity-check `check_workspace_ms`.
The claim was not verified before it was written.

The consequence is a design constraint rather than a caveat: **the column is
appended as the last field, never inserted.** Historical rows are deliberately
short (RFC-041 N-1), so inserting mid-row would make a given field number mean
different things depending on a row's age — old rows and new rows would need
different parsing, in a file whose entire RFC lineage (041, 043, 044, 052)
exists to keep measurements trustworthy. Appending keeps fields 1–10 stable for
every row ever written and leaves the existing `cut` checks correct untouched.

No library API changes; no user-visible behaviour changes.

**Security.** No new data flow, dependency, or integration.

## Release implications

**0.35.0, minor.** No API change, but the CSV schema is a documented artifact
and gains a column, which is more than a patch should carry. No migration guide
is required — nothing downstream consumes this file.
