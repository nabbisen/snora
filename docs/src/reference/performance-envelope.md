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
