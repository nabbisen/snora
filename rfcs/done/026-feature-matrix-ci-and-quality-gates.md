# RFC 026 — Feature Matrix CI and Quality Gates

**Status.** Implemented (v0.19.0)
**Tracks.** Snora Design System migration; v0.20 foundation.
**Touches.** CI workflows, quality gates, release checks.

## Summary

This RFC defines CI and quality gates for the design-system migration.

## Goals

- Preserve minimal path.
- Verify widgets-only path.
- Verify design paths.
- Keep icon features independent.
- Enforce contrast tests.
- Add visual-fit gate.
- Avoid CI explosion.

## PR CI

Required every PR once design work begins:

```text
cargo check -p snora --no-default-features
cargo check -p snora --no-default-features --features widgets
cargo check -p snora
cargo check --workspace --all-features
cargo test -p snora-design
cargo test -p snora-core
```

## Release / scheduled CI

```text
widgets + design
widgets + lucide-icons
widgets + svg-icons
widgets + design + lucide-icons
widgets + design + svg-icons
```

If root token-only design is unsupported, check `snora-design` directly.

## Gates

### Q1 — Minimal path

`snora --no-default-features` compiles.

### Q2 — Feature matrix

PR and release matrices pass.

### Q2-B — Feature isolation

- `widgets` without `design` compiles.
- base widgets do not expose `snora-design` types.
- icons do not require design.
- design modules are cfg-gated.

### Q3 — No iced in `snora-design`

Dependency review or automated check.

### Q4 — Token sanity

No invalid colors/spacing/radius/line-height/focus.

### Q5 — Automated contrast

Built-in presets pass pure Rust contrast tests.

### Q6 — Semantic construction

Primitives are reviewed for native control use and accessibility limits.

### Q7 — Documentation

No public feature without docs/example/migration note.

### Q8 — Scope

Reject domain-specific components or keep as recipes.

### Q9 — Visual fit

Workbench/live QA checks focus, clipping, line-height, and high-contrast fit.

## Process lifecycle

```text
PR opened
  -> PR CI
  -> token/contrast tests
  -> docs/examples check
  -> semantic review if primitive
  -> scope review
  -> merge
  -> scheduled/release broad matrix
```

## Internal design

CI may be split:

```text
.github/workflows/ci.yaml
.github/workflows/design-matrix.yaml
.github/workflows/docs.yaml
```

If matrix grows, an `xtask` can be introduced later.

## Acceptance criteria

- CI covers minimal/widgets/default/all-features.
- Contrast tests are mandatory.
- Feature isolation is checked.
- Release matrix covers icon/design combinations.
