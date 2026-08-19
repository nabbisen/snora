# Build cost budget

Snora tracks two build-cost metrics per release:

| Metric | CSV file | Workflow |
|---|---|---|
| Stripped binary size | [`binary-size-budget/binary-size.csv`](binary-size-budget/binary-size.csv) | `binary-size.yaml` |
| Compile time (cold) | [`build-cost-budget/compile-time.csv`](build-cost-budget/compile-time.csv) | `build-cost.yaml` |

Both are trend signals, not strict gates. The goal is to catch
unintended growth before it becomes invisible.

## Compile-time measurements

`scripts/measure-compile-time.sh` records six cold-build durations:

| Column | What it measures |
|---|---|
| `check_workspace_ms` | `cargo check --workspace --all-features` |
| `build_widgets_ms` | `cargo build -p snora-widgets --release` |
| `build_engine_only_ms` | `cargo build -p snora --no-default-features --release` |
| `example_hello_ms` | `cargo build --profile release-baseline -p snora-example-hello` |
| `build_widgets_design_ms` | `cargo build -p snora-widgets --features design --release` |
| `example_workbench_ms` | `cargo build --profile release-baseline -p snora-example-design-workbench` |
| `design_overhead_ratio` (RFC-050) | derived: `example_workbench_ms / example_hello_ms`, at 5 significant figures. **This is the trend signal** — see the note below. No new build; both inputs are already collected above. |

"Cold" means `snora-core`, `snora-design`, `snora-widgets`, and `snora`
are cleaned before each measurement, across every profile a measurement
in this run builds with — dev, `release`, and `release-baseline` — as of
the RFC-052 fix; see that note below for why this was not true before it
for two of the six columns. **And**, from the release after 0.25.3
onward (RFC-043), the CI job runs with no dependency cache at all,
so iced's transitive closure is rebuilt from scratch too. Through 0.25.3,
`build-cost.yaml` restored a
`Swatinem/rust-cache` between runs; because the per-measurement clean only
covered snora's own four crates, the cached iced build meant the "cold"
numbers were actually warm — this is what produced the 56 s → 5.5 s
"improvement" between 0.19.1 and 0.25.3, which was a caching artifact, not
a real trend (see the RFC-043 data integrity note below). Expect run times
in the minutes, not under a minute; that reflects a real cold build.
The workbench binary itself is also cleaned before `example_workbench_ms`.

### Limitations

- GitHub runner generation changes silently. Treat absolute numbers as
  rough guides and `delta` between consecutive releases as the signal.
- Wall-clock timing has ±5–15 % variance even on the same machine. A
  single data point cannot distinguish signal from noise; look at the
  trend over two or more releases.
- The `build_widgets_ms` measurement is the closest proxy for
  `feature-gating-criteria.md` indicator 1 (30-second threshold).

### Data integrity note (RFC-041)

`compile-time.csv` holds only two rows through 0.25.2 (0.17.0, 0.19.1), and
no row has been appended by CI on a release tag: the workflow's trigger
matched `v*.*.*`, but the project's tags carry no `v` prefix, so the
append-on-tag step never fired on any of the 38 release tags. This was
fixed in 0.25.3 (RFC-041). Rows before 0.25.3 predate the fix and were not
produced by the tag automation. The series also spans the 0.25.2
`resolver = "2"` → `"3"` workspace change; at the time of the 0.25.3
measurement, `Cargo.lock` was not yet committed for any *prior* release,
so dependency resolution was not pinned between the CI runs that produced
pre-0.25.3 rows — pre-0.25.3 and post-0.25.3 rows are **not comparable**.
`Cargo.lock` is committed as of 0.25.3 itself (RFC-042), so this caveat
does not apply going forward.

Additionally, row `0.17.0` was measured with `runner_os = unknown` (a
sandbox run, not CI) and reports `example_hello_ms = 182000` — the same
value `render-cost.csv` reports for `hello_ms` at `0.17.0`, a different
metric. Two independent measurements landing on an identical value
suggests a script or transcription fault at that data point; treat the
`0.17.0` row as suspect rather than a real signal.

### Data integrity note (RFC-043)

The 0.25.3 row is doubly transitional, the same as the binary-size
series: it is the **first** row produced by the fixed tag-automation
(RFC-041) and the **last** row produced by a warm-cache "cold" build.
`build-cost.yaml` restored a dependency cache between runs through
0.25.3, so the timed steps only ever rebuilt snora's own four small
crates, not iced's transitive closure — the 56 s → 5.5 s drop between
0.19.1 and 0.25.3 reflects that, not a real improvement. Fixed by
removing the cache entirely from the release after 0.25.3 onward. The
0.25.3 row's timings are **not comparable** to any row from the next
release onward, which reflects a genuinely cold build. RFC-043 also
attempted to stabilize `runner_os` to `ubuntu-latest` via an explicit
`RUNNER_OS` override on that release onward — see the RFC-044 note
below for why that attempt did not take effect.

### Data integrity note (RFC-044)

RFC-043's `runner_os` override did not work: GitHub Actions reserves the
`RUNNER_*` variable namespace and silently ignores any attempt to
overwrite it via a step's `env:` block. Both the 0.25.3 and 0.26.0 rows
read `Linux`, not `ubuntu-latest` — the workflow file looked correct
on inspection (the override was present and well-commented, and two
review passes confirmed it); only the emitted row revealed the defect,
and no post-fix row existed until the 0.26.0 tag. Fixed in RFC-044 by
routing the value through `SNORA_RUNNER_OS`, a name GitHub does not
reserve, consumed by `measure-compile-time.sh` as
`${SNORA_RUNNER_OS:-${RUNNER_OS:-unknown}}`. The two `Linux` rows are not
edited or back-filled — they stand as a recorded discontinuity, per
the append-only policy (RFC-041 N-1). This criterion is not closed by the
fix landing; it closes only when the next tagged release's row is
confirmed to read `ubuntu-latest`.

### Data integrity note (RFC-052)

**`cargo clean -p <package>` only reaches the dev profile.** `-r`/`--release`
and `--profile <NAME>` are separate, required flags — `cargo clean --help`
documents this directly. The script's per-measurement clean,
`cargo clean -p snora-core -p snora-design -p snora-widgets -p snora`, never
passed either, so it cleaned dev-profile artifacts only, while four of the
six measurements build `release` or `release-baseline`. Found while
answering RFC-050's Q-1 ("is `build_engine_only_ms` measuring anything?").

Confirmed directly:

```text
$ cargo build -p snora-core --release
$ ls target/release/libsnora_core.rlib          → present
$ cargo clean -p snora-core -p snora-design -p snora-widgets -p snora
     Removed 0 files                            ← the dev profile only
$ ls target/release/libsnora_core.rlib          → STILL PRESENT
```

And the consequence — reproducing the script's own step order, so a
`release`-profile measurement runs immediately after another one already
warmed the same profile:

```text
# BEFORE the fix (dev-only clean, `release` artifacts survive):
$ cargo build -p snora --no-default-features --release
   Compiling snora v0.30.0 (...)
    Finished `release` profile [optimized] target(s) in 0.18s   ← snora-core NOT among the Compiling lines

# AFTER the fix (dev + release + release-baseline all cleaned):
$ cargo build -p snora --no-default-features --release
   Compiling snora-core v0.30.0 (...)
   Compiling snora v0.30.0 (...)
    Finished `release` profile [optimized] target(s) in 0.24s   ← genuinely cold
```

`snora` itself rebuilt in both cases — its own fingerprint had already
changed for a reason unrelated to the clean (see below), so the bug did
not spare it. Only `snora-core` rode on a stale `release` artifact
before the fix, and only `snora-core` is new in the `Compiling` output
after it.

The same pattern holds for `build_widgets_design_ms`: before the fix,
`snora-design`/`snora-widgets` compiled (forced by the `design`-feature
fingerprint change, independent of the clean bug) while `snora-core`
again rode on a stale `release` artifact; after the fix, `snora-core`
compiles too.

**In both cases the fix recovers exactly one crate: `snora-core`.** It
is the only one of the three with no feature-set or package-selection
variation between measurement steps, so it is the only one cargo's own
fingerprinting left fresh regardless of the clean bug — `snora` and
`snora-widgets`/`snora-design` were already being rebuilt for reasons
unrelated to the clean (a different top-level package, or a changed
feature flag), and so were never actually spared by it.

**Which columns are affected — not all four release/release-baseline
columns, only two:**

| Column | Profile | Affected? |
|---|---|---|
| `check_workspace_ms` | dev | No — the dev-only clean already reached it; confirmed unaffected (RFC-052 Q-1) |
| `build_widgets_ms` | `release` | No — the first `release`-profile build in an uncached job (RFC-043), cold regardless of the clean bug |
| `build_engine_only_ms` | `release` | **Yes** — previously a partial rebuild that silently omitted `snora-core` (`snora` itself did rebuild; only its one dependency was stale) |
| `example_hello_ms` | `release-baseline` | No — the first `release-baseline` build in the run, cold regardless |
| `build_widgets_design_ms` | `release` | **Yes** — same pattern: `snora-design`/`snora-widgets` already rebuilt via the feature-flag change; `snora-core` now correctly joins them |
| `example_workbench_ms` | `release-baseline` | No, materially — real compilation already happened via the `design`-feature difference despite the no-op clean (RFC-052 Q-2) |

A local before/after full-script comparison (two complete runs, same reset
starting point, no interleaving) put both affected columns' magnitude
change within normal run-to-run noise on this machine — `build_engine_only_ms`
moved from 190 ms to 217 ms, `build_widgets_design_ms` from 250 ms to 256 ms.
**This is not surprising and is not the evidence for the fix**: per the
mechanical account above, the fix recovers compiling exactly one crate,
`snora-core`, which has no dependencies at all — a small addition on top
of a measurement that was already compiling something else regardless of
the bug. That the total time barely moves is consistent with the fix,
not evidence against it — the same magnitude-vs-mechanism gap RFC-052's
own local run showed (248 ms against a 323–421 ms historical range, the
same order). The `Compiling` lines above are the proof; a small or noisy
local delta does not undermine that, and CI is the authority for
magnitude, not a local machine with warm rustc/target caches from
repeated testing.

**Rows before this fix and rows after it are not comparable for
`build_engine_only_ms` and `build_widgets_design_ms`.** No historical row
is edited or back-filled (RFC-041 N-1) — the discontinuity is recorded,
not repaired. This is the **third** methodology discontinuity for these
columns (after RFC-043, RFC-044), and at the time reset the
≥2-comparable-post-fix-rows clock for them again. **Gate 9b closed
anyway, at v0.37.0** — not by these absolute columns reaching two
comparable rows, but by a different metric entirely,
`design_overhead_ratio` (established below, RFC-050), which does not
depend on `build_engine_only_ms` or `build_widgets_design_ms` at all.
See [`api-freeze-review.md`](../contributing/api-freeze-review.md) for
the closure record.

RFC-050's `widgets_design_ratio` (`build_widgets_design_ms / build_widgets_ms`)
was already unsound before this fix: it divides a snora-crates-only rebuild
by an iced-plus-snora cold build, two different quantities sharing a unit.
The defect fixed here made the numerator additionally wrong in a second way
(silently omitting `snora-core` from what it rebuilt); RFC-050's ratio
selection is being re-derived on post-fix data rather than implemented on
the columns as they stood.

### Data integrity note (RFC-050) — the absolute columns are runner noise; `design_overhead_ratio` is the trend signal

**The absolute millisecond columns are not, on their own, a usable trend.**
Five rows now share runner, rustc and post-RFC-052 methodology — 0.31.0,
0.32.0, 0.33.0, 0.33.1, 0.34.0:

| Column | mean (ms) | min | max | spread |
|---|---:|---:|---:|---:|
| `check_workspace_ms` | 53 138 | 39 270 | 62 987 | **60.4%** |
| `build_widgets_ms` | 90 819 | 67 296 | 104 190 | **54.8%** |
| `build_engine_only_ms` | 448 | 355 | 506 | **42.5%** |
| `example_hello_ms` | 143 227 | 111 226 | 156 549 | **40.7%** |
| `build_widgets_design_ms` | 552 | 453 | 618 | **36.4%** |
| `example_workbench_ms` | 6 061 | 4 690 | 6 564 | **41.6%** |

Two directional controls, both against the same numbers, make the case that
this is runner noise and not signal:

- **0.33.0 → 0.33.1 changed no code at all** (RFC-057 was documentation and
  doc comments only) and every column still moved **+36.4% to +54.8%**.
- **0.33.1 → 0.34.0 changed code** (four preset colour values, RFC-058) and
  every column moved **−10.9% to −21.2%** — the opposite direction, of
  comparable magnitude. The absolute columns cannot tell these two releases
  apart: one shipped nothing and jumped up sharply; the other shipped a
  real, deliberate accessibility fix and dropped sharply.

**`design_overhead_ratio` = `example_workbench_ms / example_hello_ms`
cancels this**, because the variance is common-mode (a shared runner-speed
factor moving the whole run) and a same-run ratio divides it out:

| | 0.31.0 | 0.32.0 | 0.33.0 | 0.33.1 | 0.34.0 | spread |
|---|---|---|---|---|---|---|
| `design_overhead_ratio` | 0.04268 | 0.04191 | 0.04217 | 0.04193 | 0.04295 | **2.5%** |

A 16-fold reduction versus the raw columns' 36–60%. The pair was chosen
because both measurements are `--profile release-baseline` builds of an
example *binary* — the same kind of work under the same profile — not
because they are similarly sized (`example_workbench_ms` is 23.7× smaller
in raw magnitude, and the two nearest-in-size pairs score 6–8× worse).
0.34.0 is a genuine out-of-sample check: it was measured after this ratio
was selected, and stayed inside 0.0419–0.0430 through a release that moved
every absolute column double digits.

**`build_engine_only_ms` (~448 ms) and `build_widgets_design_ms` (~552 ms)
are raw record only — no ratio may be built from either.** At that
magnitude both are dominated by process startup, cargo metadata resolution
and timer granularity rather than by compiled code, and neither pairs with
another column under the "same kind of work, same profile" rule above.

**`widgets_design_ratio` (`build_widgets_design_ms / build_widgets_ms`) was
derived, tested, and rejected — do not re-derive it.** It divides a
snora-crates-only rebuild by an iced-plus-snora cold build, two different
quantities under different profiles. Its apparent pre-fix stability (5.9%)
was cargo's own startup consistency, not a snora signal: on post-RFC-052
data it measures **14.9%**, worse than several raw columns, and cannot
reliably detect a 10% regression.

**Columns are always appended, never inserted.** `design_overhead_ratio`
is the CSV's **last** field, after `date`, not slotted in among the
existing seven. Historical rows are deliberately left short rather than
padded to match a new header (RFC-041 N-1), so inserting a new column
mid-row would split what a given field number *means* depending on the
row's age — pre-0.35.0 rows would have `rustc` at field 8, post-0.35.0
rows would have `design_overhead_ratio` there instead. That is exactly
the kind of measurement-integrity defect this file's own RFC lineage
(RFC-041, RFC-043, RFC-044, RFC-052) exists to prevent, and it would
have broken `release-process.md`'s positional `cut -d, -f9` check for
`runner_os` for every row from 0.35.0 onward. Appending keeps every
field number meaning one thing for every row in the file; the new
column is simply absent (a short row) on anything before 0.35.0. Apply
the same rule to any future column.

**The ratio is only comparable against rows from the same CI cold-build
path.** A ratio computed from a local run (warm dependency cache, no
per-measurement `cargo clean` reaching a truly cold iced rebuild) is a
smoke test of the *arithmetic*, not a data point for the series — do
not compare it against the CSV's CI-produced rows.

### Watch points

No CI failures are triggered by compile time in the first iteration.

**Trend — watch `design_overhead_ratio`, not the absolute columns.** Its
spread on identical-runner data is ~2.5%, so a ~5% move between releases is
visible above noise; the absolute columns cannot distinguish a real 50%
regression from ordinary runner variance (see the note above). Investigate
a step-change in the ratio without a corresponding, explicable change to
`example_workbench_ms`'s or `example_hello_ms`'s own dependency graph.

**Absolute ceiling — `build_widgets_ms` exceeding 30 000 ms** on the GitHub
`ubuntu-latest` runner still applies, unchanged. This maps to indicator 1
in the feature-gating criteria, and — unlike the trend watch points above —
it is a ceiling on developer experience, not a trend measurement, so
runner-to-runner noise does not weaken it: 30 s is 30 s regardless of which
host measured it.

### Running locally

```bash
scripts/measure-compile-time.sh 0.12.0
```

The script writes one CSV row to stdout. Redirect to append:

```bash
scripts/measure-compile-time.sh 0.12.0 >> \
  docs/src/reference/build-cost-budget/compile-time.csv
```

Do not hand-edit the CSV; let the script and the CI workflow manage it.

## Binary size budget

See [`binary-size-budget.md`](binary-size-budget.md) for the binary-size
tracking policy and the 150 KB `diff_bytes` threshold.

## Related: render-cost budget

For runtime/layout-composition cost (build-time proxies for Snora's own
example compilation), see [performance-envelope.md](performance-envelope.md).
