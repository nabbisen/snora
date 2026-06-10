# RFC-016-B — Performance Envelope and Render-Cost Budget

**Status.** Implemented (v0.16.0)
**Tracks.** Performance observability.
**Touches.** `scripts/measure-render-cost.sh` (new),
`docs/src/reference/performance-envelope.md` (new),
`docs/src/SUMMARY.md`, `docs/src/reference/build-cost-budget.md` (cross-link).

> Depends on RFC-012-C (compile-time tracking infra) — satisfied since v0.12.

## 1. [Decisions] Open questions answered

### Q: Simple script or proper Rust benchmarks?

Simple shell script first. `criterion` would add a non-trivial dependency
that itself takes time to compile. A timed `cargo build` of `examples/hello`
plus a release-mode timing of the workbench gives useful trend data without
the complexity.

### Q: Should render-cost tracking ever become a CI gate?

Not in this RFC. Record data, watch trends. Gate only after several data
points show a concerning pattern.

### Q: What toast count for stress testing?

100 toasts is the documented stress scenario (from the planning draft).
That is unrealistically high for any real UI, which means it will be easy
to stay well under any limit — useful for catching O(n²) regressions.

## 2. Script design

`scripts/measure-render-cost.sh <version>`:

Measures wall-clock time for:
1. `cargo build --profile release-baseline -p snora-example-hello` (skeleton only)
2. `cargo build --profile release-baseline -p snora-example-workbench` (all surfaces)

Both times are already covered by the binary-size script for the hello case.
This script is complementary: it records workbench build time as a proxy
for "complex layout compilation cost."

Output CSV row:
```
version,hello_ms,workbench_ms,rustc,runner_os,date
```

Separate CSV at `docs/src/reference/performance-envelope/render-cost.csv`.

## 3. Documentation

`performance-envelope.md` explains:
- what Snora considers its performance commitments (algorithmic, not FPS);
- the six reference scenarios from the planning draft;
- how to run the script and interpret results;
- that no CI gate exists yet.

## 4. Acceptance criteria

- `measure-render-cost.sh` exists and is executable.
- `performance-envelope.md` exists with the envelope table and the script docs.
- `build-cost-budget.md` cross-links to it.
- No new public crate dependencies.
