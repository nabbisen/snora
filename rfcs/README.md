# Snora RFCs

This directory follows the RFC lifecycle policy in
[`done/000-rfc-lifecycle-policy.md`](./done/000-rfc-lifecycle-policy.md).
The **folder is the source of truth** for an RFC's state; each file's
Status field is kept consistent with its folder.

- `proposed/` — open for review; implementer should not assume the design
  is final.
- `done/` — implemented; historical record of the design.
- `archive/` — withdrawn or superseded.

Numbering uses the planning-pack scheme `0NN-x` (the `NNN` groups by the
target minor; the `x` letter distinguishes RFCs within that group).
Numbers are stable forever and never reused.

## Done

| ID | Title | Shipped in |
|----|-------|------------|
| 000 | [RFC lifecycle policy](./done/000-rfc-lifecycle-policy.md) | adopted in v0.11.0 |
| 011-A | [Main Rust CI quality gate](./done/011-a-main-rust-ci-quality-gate.md) | v0.11.0 |
| 011-B | [Toast ordering semantics fix](./done/011-b-toast-ordering-semantics-fix.md) | v0.11.0 |
| 011-C | [AppLayout construction stability](./done/011-c-app-layout-construction-stability.md) | v0.11.0 |
| 011-D | [Render-semantics test harness](./done/011-d-render-semantics-test-harness.md) | v0.11.0 (initial; full matrix in v0.12) |
| 011-E | [Overlay interaction semantics](./done/011-e-overlay-interaction-semantics.md) | v0.11.0 |

## Proposed

### v0.12 — Semantic testing & ABDD maturity

| ID | Title |
|----|-------|
| 012-A | [ABDD compliance checklist](./proposed/012-a-abdd-compliance-checklist.md) |
| 012-B | [Showcase / workbench example](./proposed/012-b-showcase-workbench-example.md) |
| 012-C | [Compile-time tracking](./proposed/012-c-compile-time-tracking.md) |
| 012-D | [Documentation and doctest policy](./proposed/012-d-documentation-and-doctest-policy.md) |

### v0.13 — Gated expansion (design-first)

| ID | Title | v0.11 propagation |
|----|-------|-------------------|
| 013-A | [Anchored popover design study](./proposed/013-a-anchored-popover-design-study.md) | RFC-011-C satisfied → popover field would be additive |
| 013-B | [Public API freeze readiness](./proposed/013-b-public-api-freeze-readiness.md) | — |
| 013-C | [Tooltip vocabulary & persistent-toast helper](./proposed/013-c-tooltip-vocabulary-and-persistent-toast-helper.md) | — |

### v0.14 — Interaction & boundary clarity

| ID | Title | v0.11 propagation |
|----|-------|-------------------|
| 014-A | [Keyboard navigation & dismissal policy](./proposed/014-a-keyboard-navigation-and-dismissal-policy.md) | extends RFC-011-E Law 7 |
| 014-B | [Focus & modal accessibility boundary](./proposed/014-b-focus-and-modal-accessibility-boundary.md) | RFC-011-C satisfied; extends RFC-011-E Law 8 |
| 014-C | [Theme boundary & warning palette policy](./proposed/014-c-theme-boundary-and-warning-palette-policy.md) | documents the existing toast warning fallback |
| 014-D | [Icon, asset & feature-gating policy v2](./proposed/014-d-icon-asset-and-feature-gating-policy-v2.md) | builds on RFC-011-A feature matrix |
| 014-E | [Examples acceptance matrix](./proposed/014-e-examples-acceptance-matrix.md) | — |

### v0.15 — Public surface & adoption maturity

| ID | Title | v0.11 propagation |
|----|-------|-------------------|
| 015-A | [Versioning, migration & deprecation bridges](./proposed/015-a-versioning-migration-and-deprecation-bridges.md) | toast-fix → "Fixed" precedent set |
| 015-B | [Crate re-export & docs.rs policy](./proposed/015-b-crate-reexport-and-docsrs-policy.md) | RFC-011-C satisfied |
| 015-C | [Starter application template](./proposed/015-c-starter-application-template.md) | — |
| 015-D | [Design decision register maturity](./proposed/015-d-design-decision-register-maturity.md) | — |

### v0.16+ — Strategic evidence & long-term options

| ID | Title |
|----|-------|
| 016-A | [Alternate engine boundary assessment](./proposed/016-a-alternate-engine-boundary-assessment.md) |
| 016-B | [Performance envelope & render-cost budget](./proposed/016-b-performance-envelope-and-render-cost-budget.md) |
| 016-C | [Downstream adoption & feedback program](./proposed/016-c-downstream-adoption-and-feedback-program.md) |

## Archive

_(none yet)_
