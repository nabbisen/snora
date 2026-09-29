# Performance envelope

Snora makes algorithmic performance commitments, not FPS or latency targets.
It does not benchmark iced itself. The measurements here track Snora's own
contribution to build and runtime cost.

## Algorithmic commitments

| Area | Expected property |
|---|---|
| `render(AppLayout)` | Linear in number of populated surfaces and toasts |
| Toast rendering | Linear in toast count |
| Direction helpers (`row_dir`, `horizontal_align`) | Constant or linear in child count |
| Prefab widgets | No hidden background work; pure view functions |
| `sweep_expired` | Linear in toast queue length |
| `dismiss_on_escape` | O(1) — three comparisons |

None of these should ever grow superlinearly. If a regression appears
(e.g. a toast loop that clones the queue O(n) times per render), it is a
bug, not a trade-off.

## Measured at scale, and asserted (RFC-105)

The commitments above are asserted, not only stated. `crates/snora/tests/bounded_work.rs`
lays out **N = 1,000** and **10 × N** toasts and asserts that the larger run
costs **under 30×** the smaller: linear is about 10×, and quadratic about 100×.
It is `#[ignore]`d and runs in release as its own CI step. A ratio, rather than a
time limit, keeps it independent of the machine.

Recorded measurement, not a threshold (release build, tiny-skia, Ryzen 9 9950X,
2026-09-26):

| Toasts | 100 | 1,000 | 10,000 | 100,000 |
|---|---|---|---|---|
| layout | 0.6 ms | 5.9 ms | 62.9 ms | 656 ms |

The ratio assertion measured 11.63–11.96 on that machine, and **92.18 against a
deliberate quadratic edit**, which is what the test exists to catch. snora caps
nothing: prefabs render every item the caller passes, so capping a list is the
application's job.

## Build-time proxies

`scripts/measure-render-cost.sh` measures the release-baseline build time
of two examples as a proxy for layout-composition compilation cost:

| Metric | What it measures |
|---|---|
| `hello_ms` | Minimal skeleton: smallest Snora app |
| `workbench_ms` | All surfaces: header, sidebar, menus, dialog, sheet, toasts, tabs, breadcrumb |

The `workbench_ms` delta over `hello_ms` reflects the cost of the full
surface set in user code. These are **trend signals**, not gates.

Per-release values (appended on tags):
[`performance-envelope/render-cost.csv`](performance-envelope/render-cost.csv)

For binary size and compile time of the framework crates themselves, see
[build cost budget](build-cost-budget.md).

## Reference scenarios

These are the six scenarios checked qualitatively before each release:

1. Base skeleton (header + body + footer, no overlays).
2. Skeleton + menu backdrop.
3. Skeleton + dialog + sheet coexisting.
4. 1 toast, 10 toasts, 100 toasts (toast render is linear — 100 is a
   stress test, not a realistic limit).
5. LTR and RTL variants of the full workbench layout.
6. Workbench-like layout (all surfaces populated simultaneously).

None of these should cause noticeable latency. If they do, open an issue.

## Running locally

```bash
scripts/measure-render-cost.sh 0.16.0
```

The script emits one CSV row to stdout. To append:

```bash
scripts/measure-render-cost.sh 0.16.0 >> \
  docs/src/reference/performance-envelope/render-cost.csv
```

Do not hand-edit the CSV.

## Watch points

No CI gate exists yet. Investigate if:

- `workbench_ms` exceeds 3× `hello_ms` unexpectedly.
- Either number grows step-change between releases without a corresponding
  dependency addition.
- Any surface addition causes a measurable non-linear increase in a
  release-baseline build.
