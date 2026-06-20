# Binary size budget

**Latest values and full history:**
[binary-size-budget/binary-size.csv](binary-size-budget/binary-size.csv)

Released values are appended to that CSV automatically on every
release tag push by the
[`binary-size`](https://github.com/nabbisen/snora/actions/workflows/binary-size.yaml)
GitHub Actions workflow.

## Why this page exists

Snora targets desktop GUI applications, where executable size has
real consequences for distribution — installers, app stores,
auto-updaters, end-user disk space. Adding a feature to the
framework should never quietly cost users megabytes of bloat.

This page tracks the size of the canonical example binary
(`examples/hello`, the smallest possible snora app) at every
release tag, with and without the optional `widgets` feature. The
diff between the two values is the floor of "what snora widgets
cost you" — anything further you do (sidebar, header, sheet, tabs)
builds on that floor.

Tracking the number across releases gives us **drift detection**:
if 0.10 → 0.11 grows the binary by 200 KB without a corresponding
new feature, that is a regression to investigate before publish,
not a surprise users find six months later.

The threshold values that govern when to act on the data live in
[`feature-gating-criteria.md`](../contributing/feature-gating-criteria.md)
indicator (2). This page is the data; that page is the policy.

## How this is updated

The flow is split between automation and human discipline:

- **CI runs the measurement on every push to `main` and every
  pull request.** Results land in the workflow's job summary and a
  30-day artifact. *No file is modified for these runs.* This
  gives developers visibility into accidental size regressions
  without polluting the repository history.

- **CI appends one row to the budget CSV on every release tag
  (`v*.*.*`)**, then commits the change back to `main` with
  `[skip ci]`. This is the only path by which the CSV grows. One
  release, one row.

- **Humans never edit the CSV directly.** Manual edits would
  shadow the bot's commits and could be lost on the next release;
  there is also no scenario where the value being recorded is
  better measured by hand than by the CI's own scripts.

The script that produces a row is
[`scripts/measure-binary-size.sh`](https://github.com/nabbisen/snora/blob/main/scripts/measure-binary-size.sh).
It is intentionally simple — `cargo build` × 2, `strip --strip-all`
× 2, `stat -c '%s'` × 2 — so that the values are reproducible
locally:

```text
scripts/measure-binary-size.sh 0.10.0
```

## Reading the numbers

Each CSV row records:

| Column | Meaning |
|---|---|
| `version` | snora version this row is for, e.g. `0.24.0`. |
| `widgets_on_bytes` | Stripped size of `snora-example-hello` built with default features (widgets ON). |
| `widgets_off_bytes` | Same example built with `--no-default-features`. |
| `diff_bytes` | `widgets_on_bytes − widgets_off_bytes`. Marginal cost of opting into `snora-widgets`. |
| `design_on_bytes` | Stripped size of `snora-example-design-workbench` with `features = ["widgets", "design"]`. `N/A` for releases before v0.24. |
| `design_diff_bytes` | `design_on_bytes − widgets_on_bytes`. Marginal cost of opting into `snora-design`. `N/A` for releases before v0.24. |
| `rustc` | Rust toolchain version used, e.g. `rustc_1.96.0_(ac68faa20_2026-05-25)`. |
| `runner_os` | CI runner OS, e.g. `ubuntu-latest`. |
| `date` | UTC date of the measurement (`YYYY-MM-DD`). |

The 150 KB threshold from
[`feature-gating-criteria.md`](../contributing/feature-gating-criteria.md)
indicator (2) applies to `diff_bytes`. If a release crosses that
threshold, the criteria document specifies what to do (it does
not unilaterally trigger a per-widget feature split — see the
document for the full rule).

### Build profile

Measurements use the `[profile.release-baseline]` Cargo profile, which
inherits from `[profile.release]` but with `lto = false` and
`codegen-units = 16`. This keeps CI build time to 2–4 minutes per
configuration rather than 10–20 minutes for a full LTO build.

The cost is that `release-baseline` binaries are **20–40% larger than what
a user actually ships** with the default `[profile.release]`. That is
acceptable for the budget's purpose: every row is measured under the same
profile, so the diff between consecutive rows accurately reflects the change
in the framework's contribution. The `rustc` and `runner_os` columns allow
filtering out measurements taken under different toolchains or platforms.

## Frequently checked questions

**Where does this page get edited by humans?** Above this line.
Everything below the CSV link at the top of the page is prose,
maintained by hand. Inside `binary-size-budget/binary-size.csv`,
nothing is maintained by hand — it is bot-only.

**Why no chart on this page?** Plotting was deferred. The CSV is
the data store; visualization tooling can be added later (Plotly,
a `gh-pages` chart, an external dashboard) without changing how
the data flows. Keeping the data store separable from the
visualization is exactly why the CSV lives in its own subfolder.

**Why is the CSV empty / missing my release?** The first row is
written by CI the first time a `v*.*.*` tag is pushed after this
budget was introduced (0.10.0). Until that first tagged release
runs through the workflow, the CSV holds only its header. If a
release shipped and no row appeared, the `binary-size` workflow
failed on that tag — check the workflow run; the failure mode is
almost always a build issue rather than a measurement bug.
