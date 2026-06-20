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
are cleaned before each measurement. iced's transitive closure remains
cached so the measurement reflects Snora's contribution, not iced's.
The workbench binary itself is also cleaned before `example_workbench_ms`.

### Limitations

- GitHub runner generation changes silently. Treat absolute numbers as
  rough guides and `delta` between consecutive releases as the signal.
- Wall-clock timing has ±5–15 % variance even on the same machine. A
  single data point cannot distinguish signal from noise; look at the
  trend over two or more releases.
- The `build_widgets_ms` measurement is the closest proxy for
  `feature-gating-criteria.md` indicator 1 (30-second threshold).

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
