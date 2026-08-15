# Developer Handoff — RFC-050 compile-time measurement is runner noise

**Governing RFC.** [RFC-050](../../proposed/050-compile-time-measurement-is-runner-noise.md)
**Status.** Inherited from RFC-050 — Accepted, not yet implemented.
**Release target.** 0.30.0 (minor — the CSV gains columns).
**Implementation units.** One. Independent of RFC-051; either may land first.

---

## 1. Task title

Add two derived ratio columns to the compile-time measurement, move the trend
watch points onto them, and record why the absolute milliseconds are not a
trend signal.

## 2. Purpose

The six compile-time columns each vary **20–27%** across four releases measured
on identical runner, rustc and methodology. A **documentation-only** release
moved them 8–11%. The numbers report which GitHub runner the job landed on.

Gate 9b is open because of this. This task is its route to closure.

## 3. Background — read first

- `rfcs/proposed/050-compile-time-measurement-is-runner-noise.md` in full,
  especially §"The finding this RFC turns on" — the reasoning constrains the
  implementation and you should not re-derive it.
- `docs/src/reference/build-cost-budget.md`, including its three existing
  data-integrity notes (RFC-041, RFC-043, RFC-044). **This will be the
  fourth.** Match their tone: state what was wrong, what the evidence was,
  and what closes it.
- `scripts/measure-compile-time.sh` — small, readable, and the only place the
  row is produced.

Conventions: English only. `cargo fmt --all --check` now passes on a clean
tree and is enforced in CI as of 0.28.1 — the delta-check workaround older
handoffs describe is obsolete.

## 4. The reasoning you must not lose

The fix is ratios **because the noise is common-mode**. Two facts establish
that, and both belong in the doc note you write:

1. Four of six columns rank the four releases identically
   (`0.28.0 < 0.28.1 < 0.27.0 < 0.27.1`).
2. The coefficient of variation is 8.6%–11.5% across columns spanning three
   orders of magnitude (389 ms to 139 545 ms).

That is a multiplicative whole-run factor. Dividing two same-run measurements
cancels it.

**This is why the two ratios are specified rather than left to your judgement:**

| Ratio | spread on existing data |
|---|---|
| `example_workbench_ms / example_hello_ms` | **3.6%** (from ~27%) |
| `build_widgets_design_ms / build_widgets_ms` | **7.3%** (from ~25%) |
| `build_engine_only_ms / build_widgets_ms` | 23.5% — **does not collapse** |

The third fails because `build_engine_only_ms` is ~389 ms, where process
startup and timer granularity are *additive* constants that a ratio cannot
cancel. **Ratios must be between comparable, similarly-sized measurements.**
Do not add a third ratio involving the sub-second columns.

## 5. Change scope

| File | Purpose |
|---|---|
| `scripts/measure-compile-time.sh` | compute and emit two ratio columns |
| `docs/src/reference/build-cost-budget/compile-time.csv` | header only — see §7 |
| `docs/src/reference/build-cost-budget.md` | data-integrity note; watch points |
| `.github/workflows/build-cost.yaml` | job summary (Q-2) |
| `docs/src/contributing/api-freeze-review.md` | gate 9b closure condition |
| `CHANGELOG.md` | `[Unreleased]` **Changed** |

## 6. Required implementation

### Step 1 — Answer Q-1 first, before anything else

`build_engine_only_ms` is ~389 ms for a release build of `snora` +
`snora-core`, and its ordering disagrees with the other columns.

**Determine whether it is measuring a real rebuild.** Run
`scripts/measure-compile-time.sh` locally and check whether
`cargo clean -p snora-core -p snora-design -p snora-widgets -p snora`
actually invalidates what the subsequent `cargo build -p snora
--no-default-features --release` recompiles — `cargo build -v` or timing a
manual clean/build pair will show it.

- If it is a genuinely small compile, say so with the evidence.
- If `cargo clean -p` is not invalidating what the script assumes, **stop and
  report**. That is a second instance of RFC-043's defect class — a
  measurement that does not measure what it claims — and it changes what this
  task should do.

Do not skip this because the ratios do not depend on it. RFC-043 happened
because nobody checked.

### Step 2 — Emit the ratios

In `scripts/measure-compile-time.sh`, after the existing measurements:

- `design_overhead_ratio` = `example_workbench_ms / example_hello_ms`
- `widgets_design_ratio` = `build_widgets_design_ms / build_widgets_ms`

Computed from values already collected in the same run. **No new builds, no
extra CI time.** Append both to the emitted row, after `example_workbench_ms`
and before `rustc` — see §7 for why position matters.

Bash has no floating point; use `awk` or `bc`. Emit **5 significant figures**
(observed values are ~0.042 and ~0.0057, so `%.5f` is adequate but check that
`widgets_design_ratio` does not lose resolution).

Guard against divide-by-zero: if a denominator is `0` or empty, emit `N/A`
rather than crashing the release measurement. A missing ratio must not fail a
release.

### Step 3 — CSV header

Add both columns to the header row. **Do not touch any existing data row** —
append-only, RFC-041 N-1. Historical rows will have fewer fields than the
header; that is correct and expected, and the next appended row will be the
first complete one.

If you believe existing rows need `N/A` padding to stay parseable, say so in
the review request with the reason — but the default is *do not edit history*.

### Step 4 — The data-integrity note

Add to `build-cost-budget.md`, matching the three existing notes. It must
carry:

- the CV table (all six columns, mean/stdev/CV);
- the 0.28.0 → 0.28.1 evidence — a release that changed **no code** and moved
  the numbers 8–11%;
- the common-mode reasoning from §4;
- the explicit statement that **absolute columns are runner-dominated and are
  raw record, not trend signal**.

### Step 5 — Watch points

`build-cost-budget.md`'s "Watch points" currently trigger on absolute
milliseconds. Replace the **trend** watch points with ratio-based ones and
state what sensitivity each actually has.

**Keep the absolute 30 000 ms ceiling on `build_widgets_ms`.** It is a
different kind of check — an absolute bound on developer experience, mapped to
`feature-gating-criteria.md` indicator 1 — and a 10% measurement error does not
matter to it. Say why it stays, so a later reader does not "finish the job" by
removing it.

### Step 6 — Job summary (Q-2)

`build-cost.yaml` prints absolute ms in the GitHub job summary. Print the
ratios too, labelled as the comparable figures. Keep the absolutes: a human
skimming a release has intuition for milliseconds and none for `0.042`.

The summary reads the row positionally via `IFS=',' read -r …`. **That read
will break when the row gains two fields** — it currently unpacks 10 into 10
named variables. Update it; this is the one place in-repo that parses the row
positionally.

### Step 7 — Gate 9b

Update the gate 9b row in `api-freeze-review.md` to record the new closure
condition: **≥2 releases measured with the ratio columns present.**

Gate 9b is **not** closed by this task landing. Do not mark it satisfied.

## 7. The one thing that will break

`build-cost.yaml`'s "Write job summary" step does:

```bash
IFS=',' read -r v ws_ms wid_ms eng_ms hel_ms wid_design_ms wb_ms rustc os date <<<"$ROW"
```

Ten fields, ten variables. Adding two columns **before** `rustc` shifts
`rustc`, `runner_os` and `date` — so the summary would print a ratio where it
says "Rustc:" and silently mislabel the rest.

Placing the ratios at the **end** would avoid touching this, and is the wrong
choice: the row's trailing three fields are provenance (`rustc`, `runner_os`,
`date`) and belong last, as they are in every existing row and in
`binary-size.csv`. Keep the schema coherent and fix the reader.

## 8. Explicit non-change scope

Do **not**:

- **Add repeat runs or medians.** Considered and declined in the RFC —
  it costs CI minutes and corrects within-runner jitter rather than the
  between-runner bias the evidence points at.
- **Edit or back-fill any historical CSV row.** Append-only (RFC-041 N-1).
- **Add a CI failure gate on compile time.** It still fails no build.
- **Add a ratio involving `build_engine_only_ms`** (§4).
- **Touch `binary-size.csv`, `measure-binary-size.sh`, or gate 9a.** Binary
  size does not have this problem — engine size moved −0.0008% across the same
  documentation-only release.
- **Re-enable dependency caching** in `build-cost.yaml`. The uncached build is
  deliberate (RFC-043) and its rationale is in the workflow comments.

## 9. Required tests

```bash
bash -n scripts/measure-compile-time.sh
scripts/measure-compile-time.sh 0.0.0-test        # run it; inspect the row
mdbook build docs && mdbook test docs
cargo fmt --all --check
```

The script is not covered by `cargo test`; **running it and reading the row is
the test.** Include the emitted row in your evidence, and confirm by hand that
each ratio equals the quotient of the two columns in that same row.

Also verify the job-summary parse: paste a 12-field row through the updated
`IFS=',' read` line and confirm every variable lands in the right place.

## 10. Acceptance criteria

RFC-050 §Acceptance criteria 1–5:

1. Both ratio columns are emitted, computed from same-run values.
2. The CSV header documents both; no existing row is edited.
3. `build-cost-budget.md` carries the data-integrity note with the CV table
   and the 0.28.0 → 0.28.1 evidence; trend watch points move to the ratios;
   the absolute 30 s ceiling stays with a stated reason.
4. **Q-1 is answered with evidence either way** (§6 step 1).
5. Gate 9b records the ≥2-releases-with-ratios closure condition and is **not**
   marked satisfied.

## 11. Prohibited shortcuts

- Do not skip Q-1 because the ratios do not depend on it.
- Do not "fix" historical rows to match the new header.
- Do not move the ratios to the end of the row to avoid updating the job
  summary (§7).
- Do not report the ratios as satisfying gate 9b. Two releases must pass first.
- Do not delete the absolute columns. They are the raw record and the only
  thing comparable to the 30 s ceiling.

## 12. Compatibility and security

**Compatibility.** The CSV gains two columns. Anything parsing it positionally
past field 7 breaks; in-repo, that is exactly one place (§7) and it is in
scope. No library API change, no user-visible behaviour change.

**Security.** No new data flow, dependency, or integration.

## 13. Required evidence

- The full diff of `measure-compile-time.sh`.
- **A real emitted row** from a local run, with the two ratios verified by
  hand against the columns in that same row.
- Your Q-1 finding, with the command output behind it.
- The job-summary parse check (§9).
- The `build-cost-budget.md` diff in full.
- `mdbook build` / `mdbook test` output.

## 14. Required review-request format

Per workflow policy §9.2 and the packaging convention: `README.md` entry
point, full `review-request.md`, `evidence/`, under
`.git-exclude/review-request/050-compile-time-measurement-is-runner-noise/`.
**State the single entry-point path to hand to the reviewer** in the
completion summary.

**Requested review focus:** Q-1. If `build_engine_only_ms` is not measuring a
real rebuild, that finding is worth more than the ratios, and the ratios are
the easy part of this task.
