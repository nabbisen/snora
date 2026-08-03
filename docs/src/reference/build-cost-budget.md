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

"Cold" means `snora-core`, `snora-design`, `snora-widgets`, and `snora`
are cleaned before each measurement, **and**, from the release after
0.25.3 onward (RFC-043), the CI job runs with no dependency cache at all,
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
release onward, which reflects a genuinely cold build. `runner_os` is
also stabilized to `ubuntu-latest` (explicit `RUNNER_OS` override) from
that release onward — GitHub's auto-populated value is `Linux`, which the
0.25.3 row carries and every row before it does not.

### Watch points

No CI failures are triggered by compile time in the first iteration.
Investigate when:

- `build_widgets_ms` exceeds **30 000 ms** on the GitHub `ubuntu-latest`
  runner. This maps to indicator 1 in the feature-gating criteria.
- `build_engine_only_ms` grows toward `build_widgets_ms`. The
  engine-only build should remain materially faster.
- Any column shows a step-change jump without a corresponding
  dependency addition.

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
