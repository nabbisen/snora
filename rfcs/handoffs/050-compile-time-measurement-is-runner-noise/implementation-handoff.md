# Developer Handoff — RFC-050 compile-time measurement

**Governing RFC.** [RFC-050](../../proposed/050-compile-time-measurement-is-runner-noise.md)
**Status.** Accepted (owner, 2026-08-15); parked, then unparked and re-derived
2026-08-18.
**Release target.** 0.35.0 (minor — the CSV gains one column).
**Implementation units.** One.

---

## 1. Task title

Emit `design_overhead_ratio` from `measure-compile-time.sh`, move the trend
watch points onto it, and record why the absolute columns are not a trend.

## 2. Purpose

Six compile-time columns vary **36–60%** across five releases on identical
runner, rustc and methodology, and a **documentation-only** release moved every
one of them 36–55%. The variance is common-mode, so a ratio between two
same-run measurements cancels the runner. One ratio does this well:
**2.5% spread** across five rows, a 16-fold noise reduction — and it held on
the 0.34.0 row, which was measured after the ratio was chosen.

Gate 9b has now been reopened or reset three times (RFC-041, RFC-043,
RFC-052) and the noise has **roughly doubled** since the RFC was first written.
More rows are not converging on a satisfiable gate. This is 9b's route to
closure.

## 3. Read this before writing code — the RFC was re-derived and two of its earlier conclusions are dead

The version of RFC-050 you are implementing is **not** the one from 2026-08-15.
The re-derivation on post-RFC-052 data reversed two things:

1. **`widgets_design_ratio` is dropped.** It measured 5.9% pre-fix and
   **14.9% post-fix** (unchanged when the fifth row landed) — worse than several columns' raw spread. Its earlier
   apparent stability was cargo's startup consistency, not snora's compile
   cost, exactly as the parked note suspected. **Do not implement it.** If you
   find it referenced anywhere, that reference is stale.
2. **"Similarly-sized measurements" was the wrong selection rule.** The winning
   pair has a 23.7× size disparity; two near-equal-sized pairs score 10.0% and
   14.7%. The real property is that numerator and denominator **do the same
   kind of work under the same profile** — `example_workbench_ms` and
   `example_hello_ms` are the only two measurements that are both
   `--profile release-baseline` builds of an example *binary*.

Implement one ratio. Do not add a second on your own judgement; if you think
you have found one, report it rather than shipping it (§9).

## 4. What to implement

### 4.1 `scripts/measure-compile-time.sh`

Emit a seventh data field, `design_overhead_ratio` =
`example_workbench_ms / example_hello_ms`, at **5 significant figures**
(observed values ~0.042). Both inputs are already collected in the same run —
**no new measurement, no extra build time.**

Append it to the CSV line and to the documented output format in the header
comment. Mind the column order against the existing header.

`bash` has no floating-point arithmetic. Use `awk` or `bc`; whichever you pick,
make the precision explicit rather than inherited from a default, and handle a
zero denominator without emitting a malformed row.

### 4.2 `docs/src/reference/build-cost-budget/compile-time.csv`

Header gains the column. **Do not touch a single existing row** — see §5.

### 4.3 `docs/src/reference/build-cost-budget.md`

- State that the **absolute columns are runner-dominated**, with the
  post-RFC-052 spread table (36–60%) and **both** directional controls: the
  documentation-only 0.33.1 moved every column **+36% to +55%** with zero code
  changed, and 0.34.0 moved them **−10.9% to −21.2%** *with* code changed. The
  raw columns cannot distinguish those two releases; the ratio held through
  both.
- **Move the trend watch points onto the ratio.** State the sensitivity it
  actually has (~2.5% spread, so a 5% move is visible; the absolute columns
  cannot see 50%).
- **Keep the absolute 30 s watch point on `build_widgets_ms`.** It is an
  absolute ceiling on developer experience, not a trend, and runner error does
  not matter to it.
- Record that `build_engine_only_ms` (~448 ms) and `build_widgets_design_ms`
  (~552 ms) are **raw record only** and no ratio may be built from them — they
  are dominated by process startup and timer granularity.
- Record that `widgets_design_ratio` **was derived, tested at 14.9% on
  post-fix data, and rejected.** This is the point of the note: so it is not
  proposed a third time.

### 4.4 `docs/src/contributing/api-freeze-review.md`

Gate 9b's row gains the closure condition: **≥2 releases measured with
`design_overhead_ratio` present.** Per Q-2, state alongside it that the
absolute columns remain runner-dominated and only the ratio is a trend — a
future reader seeing "✅" should not conclude the milliseconds became
trustworthy.

**Gate 9b does not close with this RFC.** It closes when two post-change data
points exist. Do not tick it.

### 4.5 `.github/workflows/build-cost.yaml`

Per Q-1: print **both** the absolute values and the ratio in the job summary,
with the ratio labelled as the comparable one. Nobody has an intuition for
0.042 yet.

### 4.6 `CHANGELOG.md`

Under **Changed**. Say plainly that the absolute numbers were never a trend
signal, rather than presenting the ratio as a refinement of something that
worked.

## 5. The append-only rule, and a contradiction that is now resolved

RFC-041 N-1: **historical rows are never edited, rewritten, or back-filled.**

The earlier draft of RFC-050 contradicted itself — its acceptance criterion 2
said *"historical rows carry `N/A`"* while its non-goals forbade *"padding them
with `N/A` to match the new header."* **Resolved in favour of the non-goal:**
historical rows stay short. A row written before the column existed has fewer
fields than the header, and that is the honest record. The next appended row is
the first complete one.

If you find tooling that cannot read a short row, fix the tooling.

## 6. Change scope

| File | Purpose |
|---|---|
| `scripts/measure-compile-time.sh` | emit the ratio (§4.1) |
| `docs/src/reference/build-cost-budget/compile-time.csv` | header only (§4.2) |
| `docs/src/reference/build-cost-budget.md` | the noise statement, watch points, both rejection notes (§4.3) |
| `docs/src/contributing/api-freeze-review.md` | gate 9b closure condition (§4.4) |
| `.github/workflows/build-cost.yaml` | job summary (§4.5) |
| `CHANGELOG.md` | **Changed** (§4.6) |

**`docs/src/contributing/feature-gating-criteria.md:67` — checked, and it needs
care rather than a blanket fix.** It reads *"The `build_widgets_ms` column is
the indicator 1 proxy"*, and indicator 1 (line 54) triggers a crate split when
compile time *"exceeds 30 seconds on a developer's machine of average specs"*.

Do **not** simply move indicator 1 onto the ratio. It is an **absolute
ceiling**, and RFC-050 explicitly keeps absolute ceilings — a ratio cannot
answer "does this exceed 30 seconds". Two real caveats do belong there:

- the proxy column carries **54.8% spread**, so a single reading near the
  threshold decides a crate split on runner luck; and
- the threshold is written against *a developer's machine* while the proxy is
  *CI*, which was already a loose substitution before this RFC quantified it.

Add the caveat and leave the check absolute. If you conclude something stronger
is needed, report it — changing a 1.0-gate-adjacent decision rule is not this
task's call.

## 7. Explicit non-change scope

Do **not**:

- **Implement `widgets_design_ratio`** or any ratio using a sub-second column.
- **Add a new measurement column.** It would reset the comparability clock a
  fourth time. This RFC derives from what is already collected.
- **Add repeat runs / median-of-N.** Considered and rejected in the RFC: it
  costs CI minutes and addresses within-runner jitter, when the evidence says
  the dominant effect is *between*-runner speed.
- **Edit, pad, reorder or back-fill any historical CSV row** (§5).
- **Tick gate 9b.**
- **Change the binary-size measurement.** 9a is satisfied and does not have
  this problem.
- **Add a CI failure gate.** Compile cost fails no build.
- **Change the uncached build policy** (RFC-043).

## 8. Required tests

There is no unit-test harness for this script, so the evidence *is* the test:

```bash
scripts/measure-compile-time.sh 0.0.0-test     # full run; slow, expected
```

- Show the emitted row, and that `design_overhead_ratio` equals
  `example_workbench_ms / example_hello_ms` to the precision emitted —
  recompute it by hand from the row's own two fields.
- Show the header and the row have the **same field count**, and that a
  historical short row still parses in whatever reads this file.
- `mdbook build docs && mdbook test docs`.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
  --all-features -- -D warnings` (unchanged by this RFC, but the release gate
  runs them).

A `shellcheck` pass on the modified script if available; report if not.

## 9. Required evidence

- The script diff, and one **real emitted row** from an actual run.
- The hand-recomputed ratio from that row's own fields, matching.
- The CSV header diff, plus `git diff` proving **no existing row changed**.
- The `build-cost-budget.md` diff, showing all four required statements
  (§4.3).
- The gate 9b row, showing the condition recorded and the gate **not ticked**.
- The job-summary output (§4.5).
- What you found in `feature-gating-criteria.md` indicator 1 (§6).

## 10. Acceptance criteria

RFC-050 §Acceptance criteria 1–5. The two most likely to be got wrong:

- **2** — no existing row edited or padded. The earlier draft asked for the
  opposite; §5 governs.
- **4** — the rejection notes for both the sub-second columns and
  `widgets_design_ratio`. Omitting them means the next person re-derives a
  ratio that has now failed twice.

## 11. Required review-request format

Per workflow policy §9.2: `README.md` entry point, full `review-request.md`,
`evidence/`, under
`.git-exclude/review-request/050-compile-time-measurement-is-runner-noise/`.
**State the single entry-point path** in the completion summary.

**Requested review focus:** the emitted ratio's precision and correctness
against its own row, and whether `build-cost-budget.md` now makes it impossible
to read the absolute columns as a trend. The failure mode is a reader who sees
six millisecond columns, a ratio, and no statement of which one means anything.
