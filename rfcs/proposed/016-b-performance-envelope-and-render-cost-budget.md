# RFC-016-B — Performance Envelope and Render-Cost Budget

Status: Proposed  
Target release: v0.16+ after compile-time tracking  
Priority: Medium  
Type: Performance observability / non-functional requirements

## 1. Summary

Define a lightweight performance envelope for Snora’s layout composition and overlay rendering, complementing binary-size and compile-time tracking.

## 2. Motivation

Snora is small, but GUI framework helpers can still accumulate render-cost regressions through unnecessary cloning, large example patterns, or expensive widget composition. The existing binary-size budget and proposed compile-time tracking cover build artifacts; this RFC adds a narrow runtime/render-cost perspective without premature micro-optimization.

## 3. Goals

- Define what runtime performance Snora cares about.
- Track render/layout composition cost at a coarse level.
- Avoid pixel or GPU benchmarking complexity.
- Catch accidental O(n^2) behavior in toasts/menus/examples.
- Preserve readability over micro-optimization.

## 4. Non-Goals

- Do not benchmark iced itself.
- Do not promise FPS targets.
- Do not add a game-loop model.
- Do not add profiling dependencies to normal users.
- Do not optimize before measuring.

## 5. External Design

Performance envelope:

| Area | Expected property |
|---|---|
| `render(AppLayout)` | linear in number of populated surfaces and toasts |
| Toast rendering | linear in toast count |
| Direction helpers | constant or linear in child count only |
| Prefab widgets | no hidden background work |
| Examples | usable as smoke tests, not benchmark claims |

Initial measurement approach:

- Add a benchmark-like dev tool or example that repeatedly builds representative `AppLayout` values.
- Measure wall-clock at coarse level in CI only if stable enough.
- Prefer local script first; do not block PRs on noisy timing initially.

## 6. Internal Design

Possible files:

- `scripts/measure-render-cost.sh`
- `examples/render_cost.rs` or `benches/render_cost.rs`
- `docs/src/reference/performance-envelope.md`

Avoid adding criterion or other benchmark dependencies unless accepted through feature-gating criteria. A simple release-mode example that constructs layouts may be enough initially.

Suggested scenarios:

1. base skeleton only;
2. skeleton + menus;
3. skeleton + dialog + sheet;
4. 1 toast, 10 toasts, 100 toasts;
5. LTR and RTL variants;
6. workbench-like layout.

Measurements should be trend indicators, not absolute promises.

## 7. Testing and Acceptance

Acceptance criteria:

- Performance envelope doc exists.
- Measurement script is optional and non-blocking initially.
- No runtime dependency is added to public crates.
- Any blocking threshold must be introduced only after several data points.
- The script output is documented and easy to compare manually.

## 8. Documentation Updates

Update:

- reference performance page
- contributing feature-gating criteria
- release process if measurements become release tasks
- binary-size/compile-time docs to cross-link build-cost and runtime-cost concepts

Docs must avoid making claims unsupported by measurements.

## 9. Compatibility and Migration

Compatible.

If a benchmark dependency is added, keep it dev-only and evaluate compile-time impact.

## 10. Open Questions

- Is a simple script enough, or should proper Rust benchmarks be added?
- Should render-cost tracking ever become a CI gate?
- What toast count is reasonable for stress testing without implying recommended usage?
