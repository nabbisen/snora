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

From **RFC-020 onward**, numbering is **flat sequential** (RFC-020, RFC-021,
…) rather than the `0NN-x` grouping; the leading number no longer encodes a
target minor. Existing `0NN-x` RFCs keep their IDs and are not renumbered.
Numbers remain stable forever and are never reused.

## Done

| ID | Title | Shipped in |
|----|-------|------------|
| 000 | [RFC lifecycle policy](./done/000-rfc-lifecycle-policy.md) | adopted in v0.11.0 |
| 011-A | [Main Rust CI quality gate](./done/011-a-main-rust-ci-quality-gate.md) | v0.11.0 |
| 011-B | [Toast ordering semantics fix](./done/011-b-toast-ordering-semantics-fix.md) | v0.11.0 |
| 011-C | [AppLayout construction stability](./done/011-c-app-layout-construction-stability.md) | v0.11.0 |
| 011-D | [Render-semantics test harness](./done/011-d-render-semantics-test-harness.md) | v0.11.0 initial; v0.12.0 full acceptance |
| 011-E | [Overlay interaction semantics](./done/011-e-overlay-interaction-semantics.md) | v0.11.0 |
| 012-A | [ABDD compliance checklist](./done/012-a-abdd-compliance-checklist.md) | v0.12.0 |
| 012-B | [Showcase / workbench example](./done/012-b-showcase-workbench-example.md) | v0.12.0 |
| 012-C | [Compile-time tracking](./done/012-c-compile-time-tracking.md) | v0.12.0 |
| 012-D | [Documentation and doctest policy](./done/012-d-documentation-and-doctest-policy.md) | v0.12.0 |
| 013-A | [Anchored popover design study](./done/013-a-anchored-popover-design-study.md) | v0.13.0 (design study; implementation deferred) |
| 013-B | [Public API freeze readiness](./done/013-b-public-api-freeze-readiness.md) | v0.13.0 |
| 013-C | [Tooltip vocabulary & persistent-toast helper](./done/013-c-tooltip-vocabulary-and-persistent-toast-helper.md) | v0.13.0 (both candidates deferred; triggers unmet) |
| 014-A | [Keyboard navigation & dismissal policy](./done/014-a-keyboard-navigation-and-dismissal-policy.md) | v0.14.0 |
| 014-B | [Focus & modal accessibility boundary](./done/014-b-focus-and-modal-accessibility-boundary.md) | v0.14.0 |
| 014-C | [Theme boundary & warning palette policy](./done/014-c-theme-boundary-and-warning-palette-policy.md) | v0.14.0 |
| 014-D | [Icon, asset & feature-gating policy v2](./done/014-d-icon-asset-and-feature-gating-policy-v2.md) | v0.14.0 |
| 014-E | [Examples acceptance matrix](./done/014-e-examples-acceptance-matrix.md) | v0.14.0 |
| 015-A | [Versioning, migration & deprecation bridges](./done/015-a-versioning-migration-and-deprecation-bridges.md) | v0.15.0 |
| 015-B | [Crate re-export & docs.rs policy](./done/015-b-crate-reexport-and-docsrs-policy.md) | v0.15.0 |
| 015-C | [Starter application template](./done/015-c-starter-application-template.md) | v0.15.0 |
| 015-D | [Design decision register maturity](./done/015-d-design-decision-register-maturity.md) | v0.15.0 |
| 016-A | [Alternate engine boundary assessment](./done/016-a-alternate-engine-boundary-assessment.md) | v0.16.0 |
| 016-B | [Performance envelope & render-cost budget](./done/016-b-performance-envelope-and-render-cost-budget.md) | v0.16.0 |
| 016-C | [Downstream adoption & feedback program](./done/016-c-downstream-adoption-and-feedback-program.md) | v0.16.0 |
| 017-A | [1.0 gate advancement](./done/017-a-1-0-gate-advancement.md) | v0.17.0 |
| 017-B | [RTL render-semantics tests](./done/017-b-rtl-render-semantics-tests.md) | v0.17.0 |
| 017-C | [Documentation test policy enforcement](./done/017-c-documentation-test-policy-enforcement.md) | v0.17.0 |
| 017-D | [First build-cost data point](./done/017-d-first-build-cost-data-point.md) | v0.17.0 |
| 017-E | [API freeze review update](./done/017-e-api-freeze-review-update.md) | v0.17.0 |
| 018-A | [Version number maintenance](./done/018-a-version-number-maintenance.md) | v0.18.0 |
| 018-B | [Gate 7 close-out](./done/018-b-gate-7-close-out.md) | v0.18.0 |
| 018-C | [Contributing index and cross-links](./done/018-c-contributing-index-and-cross-links.md) | v0.18.0 |
| 019-A | [Lucide icons type-parameter fix](./done/019-a-lucide-icons-type-parameter-fix.md) | v0.18.1 |

| 020 | [Design System Boundary and Philosophy Amendment](./done/020-design-system-boundary-and-philosophy.md) | v0.19.0 |
| 021 | [Crate and Feature Architecture](./done/021-crate-and-feature-architecture.md) | v0.19.0 |
| 022 | [Snora Design Token Data Model](./done/022-token-data-model.md) | v0.19.0 |
| 023 | [Palettes, High Contrast, and Automated Contrast Tests](./done/023-palettes-high-contrast-and-contrast-tests.md) | v0.19.0 |
| 024 | [Typography, Leading, Spacing, Radius, Focus, and Density](./done/024-typography-leading-spacing-radius-focus-density.md) | v0.19.0 |
| 025 | [iced Style Bridge](./done/025-iced-style-bridge.md) | v0.19.0 |
| 026 | [Feature Matrix CI and Quality Gates](./done/026-feature-matrix-ci-and-quality-gates.md) | v0.19.0 |
| 027 | [Accessibility and Semantic Construction Policy](./done/027-accessibility-and-semantic-construction.md) | v0.19.0 |
| 028 | [v0.20 Pilot Button Helper](./done/028-pilot-button-helper.md) | v0.19.0 |
| 029 | [v0.20 Pilot Card Helper](./done/029-pilot-card-helper.md) | v0.19.0 |
| 030 | [Documentation, Examples, and Design Workbench](./done/030-docs-examples-and-design-workbench.md) | v0.19.0 |


## Proposed

The following design-system RFCs remain in review (target v0.20+):

| ID | Title |
|----|-------|
| 031 | [v0.20 Release Acceptance Criteria](./done/031-v020-release-acceptance.md) | v0.20.0 |
| 032 | [Notice, Chip, and Progress Primitives](./proposed/032-notice-chip-and-progress-primitives.md) |
| 033 | [Recipes and Dogfood Process](./proposed/033-recipes-and-dogfood-process.md) |
| 034 | [Promotion, Stabilization, and API Governance](./proposed/034-promotion-stabilization-and-api-governance.md) |

## Archive

_(none yet)_
