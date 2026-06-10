# RFC-014-E — Examples Acceptance Matrix

Status: Proposed  
Target release: v0.14; depends on workbench/showcase progress  
Priority: Medium-high  
Type: Examples / QA / release acceptance

## 1. Summary

Turn examples from informal demos into a release acceptance matrix that proves major surfaces, feature flags, and ABDD behavior remain usable.

## 2. Motivation

Snora examples are important because the framework is intentionally small and slot-oriented. Users learn by seeing how to assemble `AppLayout`, overlays, widgets, toasts, and direction changes. As the project matures, examples should become part of release confidence, not only documentation. The planned workbench example should be the center of this matrix.

## 3. Goals

- Define which examples must build under which feature sets.
- Use examples to prove the public workflow remains ergonomic.
- Ensure at least one example exercises every major surface.
- Require LTR/RTL demonstration in examples touching logical placement.
- Keep examples small enough to remain maintainable.

## 4. Non-Goals

- Do not turn examples into product applications.
- Do not add complex domain logic.
- Do not require screenshots as release blockers initially.
- Do not test pixel-perfect layout.
- Do not use examples to justify broad widget expansion.

## 5. External Design

Acceptance matrix:

| Example | Purpose | Required features | Must demonstrate |
|---|---|---|---|
| `hello` | minimal body + render | no-default or default | smallest working app |
| `header` | prefab header | widgets | slot injection |
| `menus` | header/context menus | widgets | close sink, backdrop |
| `overlays` | dialog/sheet | default/widgets | modal dim, outside close |
| `toasts` | toast lifecycle | default | transient/persistent toasts |
| `direction` | ABDD | default/widgets | LTR/RTL mirroring |
| `icons` | icon features | lucide/svg variants | feature-gated usage |
| `workbench` | integrated dogfood | default plus optional icon features | all major surfaces together |

Release checklist:

```text
cargo check --examples --workspace --all-features
cargo check -p snora --example hello --no-default-features
cargo check -p snora --example workbench --features widgets
```

Exact commands may differ based on example crate placement.

## 6. Internal Design

Repository changes:

- Add `examples/README.md` with the matrix.
- Add per-example top comments explaining the surface being demonstrated.
- Add CI job or step for examples.
- Keep example names stable where possible so docs links remain stable.

Implementation principle:

Examples should consume public API exactly as downstream apps do. They should not reach into private modules or use test-only shortcuts.

## 7. Testing and Acceptance

Acceptance criteria:

- Every example in the matrix builds in CI under its intended features.
- Workbench can be manually run and used as visual smoke test.
- Examples covering direction have an obvious RTL toggle or fixed RTL variant.
- Examples covering close sinks demonstrate both open and close paths.
- The release checklist references the examples matrix.

## 8. Documentation Updates

Update:

- `examples/README.md`
- `docs/src/SUMMARY.md` if examples are linked from mdBook
- relevant guide pages to point to examples
- release process checklist

Docs should say examples are acceptance references, not a replacement for real apps.

## 9. Compatibility and Migration

Compatible.

Renaming examples can break docs links and user habits; avoid unless the example set is still clearly pre-1.0 and migration links are updated.

## 10. Open Questions

- Should workbench be one large example or a small example app under `examples/workbench/`?
- Should example screenshots be generated in CI later?
- Should examples use only default features unless demonstrating a specific feature?
