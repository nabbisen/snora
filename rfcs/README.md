# Snora RFCs

This directory follows the RFC lifecycle policy in
[`done/000-rfc-lifecycle-policy.md`](./done/000-rfc-lifecycle-policy.md).
The **folder is the source of truth** for an RFC's state; each file's
Status field is kept consistent with its folder.

- `proposed/` — open for review; implementer should **not** start.
- `accepted/` — owner signed off; implementer may start; not yet shipped.
- `done/` — implemented; historical record of the design.
- `archive/` — withdrawn or superseded. Two entries: RFC-078 and RFC-091.

snora uses the lifecycle policy's **five-folder variant** (adopted
2026-08-19), because here *"the maintainer signed off"* and *"the implementer
finished"* are separate events performed by different parties, mediated by a
written handoff. Until that date accepted RFCs stayed in `proposed/` with a
`Status: Accepted` line — a state the four-folder variant does not define,
in a folder that says the design may still change. See
[`accepted/README.md`](./accepted/README.md).

An RFC's path is `proposed/` → `accepted/` → `done/`. It moves to `accepted/`
when the owner accepts it and the handoff is written, and to `done/` in the
release that ships it.

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
| 031 | [v0.20 Release Acceptance Criteria](./done/031-v020-release-acceptance.md) | v0.20.0 |
| 032 | [Notice, Chip, and Progress Primitives](./done/032-notice-chip-and-progress-primitives.md) | v0.21.0 |
| 033 | [Recipes and Dogfood Process](./done/033-recipes-and-dogfood-process.md) | v0.23.0 |
| 034 | [Promotion, Stabilization, and API Governance](./done/034-promotion-stabilization-and-api-governance.md) | v0.23.0 |
| 035 | [Documentation consistency and source-of-truth audit](./done/035-documentation-consistency-audit.md) | v0.25.3 (handoff: [`handoffs/035-…`](./handoffs/035-documentation-consistency-audit/implementation-handoff.md)) |
| 036 | [Design surface freeze review and additive-only covenant](./done/036-design-surface-freeze-and-additive-covenant.md) | v0.25.3 (handoff: [`handoffs/036-…`](./handoffs/036-design-surface-freeze-and-additive-covenant/implementation-handoff.md)) |
| 041 | [Measurement integrity and gate 9 re-assessment](./done/041-measurement-integrity-and-gate-9.md) | v0.25.3 (handoff: [`handoffs/041-…`](./handoffs/041-measurement-integrity-and-gate-9/implementation-handoff.md)) |
| 042 | [Commit `Cargo.lock`](./done/042-commit-cargo-lock.md) | v0.25.3 (handoff: [`handoffs/042-…`](./handoffs/042-commit-cargo-lock/implementation-handoff.md)) |
| 037 | [Coherent defaults for snora-rendered surfaces](./done/037-coherent-defaults-positioning.md) | v0.26.0 (handoff: [`handoffs/037-…`](./handoffs/037-coherent-defaults-positioning/implementation-handoff.md)) |
| 038 | [Token-derived `iced::Theme` emission](./done/038-token-derived-theme-emission.md) | v0.26.0 (handoff: [`handoffs/038-…`](./handoffs/038-token-derived-theme-emission/implementation-handoff.md)) |
| 043 | [The budget measurements do not measure what they claim](./done/043-measurement-methodology-measures-nothing.md) | v0.26.0 (handoff: [`handoffs/043-…`](./handoffs/043-measurement-methodology-measures-nothing/implementation-handoff.md)) |
| 039 | [Engine surfaces: the dialog card and the modal dim](./done/039-engine-surface-styling.md) | v0.27.0 (handoff: [`handoffs/039-…`](./handoffs/039-engine-surface-styling/implementation-handoff.md)) |
| 040 | [Chrome geometry: token-derived spacing and radius](./done/040-chrome-geometry.md) | v0.27.0 (handoff: [`handoffs/040-…`](./handoffs/040-chrome-geometry/implementation-handoff.md)) |
| 044 | [`RUNNER_OS` cannot be overridden](./done/044-runner-os-override-cannot-work.md) | v0.27.0 (handoff: [`handoffs/044-…`](./handoffs/044-runner-os-override-cannot-work/implementation-handoff.md)) |
| 045 | [Assistive technology: stated position, and bounding the ABDD claim](./done/045-assistive-technology-position.md) | v0.27.1 (handoff: [`handoffs/045-…`](./handoffs/045-assistive-technology-position/implementation-handoff.md)) |
| 046 | [Layout width exposure](./done/046-layout-width-exposure.md) | v0.28.0 (handoff: [`handoffs/046-…`](./handoffs/046-layout-width-exposure/implementation-handoff.md)) |
| 047 | [Stable identifiers on snora-rendered surfaces](./done/047-stable-identifiers-on-rendered-surfaces.md) | v0.28.0 (handoff: [`handoffs/047-…`](./handoffs/047-stable-identifiers-on-rendered-surfaces/implementation-handoff.md)) |
| 048 | [The dialog card: contradictory documentation and an undiscoverable capability](./done/048-dialog-card-documentation-contradiction.md) | v0.28.1 (handoff: [`handoffs/048-…`](./handoffs/048-dialog-card-documentation-contradiction/implementation-handoff.md)) |
| 049 | [`snora-dialog-card` denotes the wrong element](./done/049-dialog-identifier-denotes-the-wrong-element.md) | v0.29.0 (handoff: [`handoffs/049-…`](./handoffs/049-dialog-identifier-denotes-the-wrong-element/implementation-handoff.md)) |
| 051 | [The responsive example excludes the consumer who asked for it](./done/051-responsive-example-excludes-its-own-requester.md) | v0.30.0 (handoff: [`handoffs/051-…`](./handoffs/051-responsive-example-excludes-its-own-requester/implementation-handoff.md)) |
| 052 | [The compile-time clean never invalidates release artifacts](./done/052-clean-never-invalidates-release-artifacts.md) | v0.31.0 (handoff: [`handoffs/052-…`](./handoffs/052-clean-never-invalidates-release-artifacts/implementation-handoff.md)) |
| 053 | [`responsive_render` excludes the design path](./done/053-responsive-render-excludes-the-design-path.md) | v0.31.0 (handoff: [`handoffs/053-…`](./handoffs/053-responsive-render-excludes-the-design-path/implementation-handoff.md)) |
| 054 | [`design` requires `widgets`, and the engine surface pays for it](./done/054-design-requires-widgets.md) | closed v0.32.0 — investigation; B′ implemented by RFC-055 |
| 055 | [Extract the iced style bridge into its own crate](./done/055-extract-the-style-bridge.md) | v0.32.0 (handoff: [`handoffs/055-…`](./handoffs/055-extract-the-style-bridge/implementation-handoff.md)) |
| 056 | [Remove the `snora-widgets` style shims](./done/056-remove-the-style-shims.md) | v0.33.0 (handoff: [`handoffs/056-…`](./handoffs/056-remove-the-style-shims/implementation-handoff.md)) |
| 057 | [The typography vocabulary is complete and undiscoverable](./done/057-typography-is-undiscoverable.md) | v0.33.1 (handoff: [`handoffs/057-…`](./handoffs/057-typography-is-undiscoverable/implementation-handoff.md)) |
| 058 | [`border` contrast is untested, and `light`/`dark` ship it at ~1.3:1](./done/058-border-contrast-is-untested-and-failing.md) | v0.34.0 (handoff: [`handoffs/058-…`](./handoffs/058-border-contrast-is-untested-and-failing/implementation-handoff.md)) |
| 059 | [Two more answers filed where consumers do not read](./done/059-answers-filed-where-consumers-do-not-read.md) | v0.34.0 (handoff: [`handoffs/059-…`](./handoffs/059-answers-filed-where-consumers-do-not-read/implementation-handoff.md)) |
| 050 | [Compile-time measurement reports runner speed, not snora](./done/050-compile-time-measurement-is-runner-noise.md) | v0.35.0 (handoff: [`handoffs/050-…`](./handoffs/050-compile-time-measurement-is-runner-noise/implementation-handoff.md)) |
| 060 | [Frame-level keyboard navigation](./done/060-frame-level-keyboard-navigation.md) | v0.35.0 (handoff: [`handoffs/060-…`](./handoffs/060-frame-level-keyboard-navigation/implementation-handoff.md)) |
| 061 | [Pointer target size is a checklist rule with no assertion](./done/061-pointer-target-size-is-unasserted.md) | v0.36.0 (handoff: [`handoffs/061-…`](./handoffs/061-pointer-target-size-is-unasserted/implementation-handoff.md)) |
| 062 | [The feature-gating status table contradicts its own threshold](./done/062-feature-gating-indicators-are-uncalibrated.md) | v0.36.0 (handoff: [`handoffs/062-…`](./handoffs/062-feature-gating-indicators-are-uncalibrated/implementation-handoff.md)) |
| 063 | [The contrast pair list is hand-maintained](./done/063-contrast-pairs-are-a-hand-maintained-list.md) | v0.36.0 (handoff: [`handoffs/063-…`](./handoffs/063-contrast-pairs-are-a-hand-maintained-list/implementation-handoff.md)) |
| 064 | [`rust,ignore` is the default nobody has to justify](./done/064-ignored-doctests-are-unaudited.md) | v0.36.1 (handoff: [`handoffs/064-…`](./handoffs/064-ignored-doctests-are-unaudited/implementation-handoff.md)) |
| 065 | [The modal dim is an unmeasured surface](./done/065-the-modal-dim-is-an-unmeasured-surface.md) | v0.37.0 (handoff: [`handoffs/065-…`](./handoffs/065-the-modal-dim-is-an-unmeasured-surface/implementation-handoff.md)) |
| 066 | [The dim assertion is an endpoint check, not a sweep](./done/066-the-dim-assertion-is-an-endpoint-check.md) | v0.37.1 (handoff: [`handoffs/066-…`](./handoffs/066-the-dim-assertion-is-an-endpoint-check/implementation-handoff.md)) |
| 067 | [Withdrawing a claim does not retract it from consumers who acted on it](./done/067-withdrawing-a-claim-does-not-retract-it.md) | v0.37.2 (handoff: [`handoffs/067-…`](./handoffs/067-withdrawing-a-claim-does-not-retract-it/implementation-handoff.md)) |
| 068 | [The typography scale is half-tooled: six size helpers, no line-height helper](./done/068-line-height-has-no-helper.md) | v0.38.0 (handoff: [`handoffs/068-…`](./handoffs/068-line-height-has-no-helper/implementation-handoff.md)) |
| 069 | [Every Rust example in the book is `ignore`, and the policy blames the wrong cause](./done/069-book-examples-cannot-be-compiled.md) | v0.38.0 (handoff: [`handoffs/069-…`](./handoffs/069-book-examples-cannot-be-compiled/implementation-handoff.md)) |
| 070 | [The typography scale was never stated against iced's default](./done/070-the-scale-is-uncalibrated-against-iceds-default.md) | v0.38.1 (handoff: [`handoffs/070-…`](./handoffs/070-the-scale-is-uncalibrated-against-iceds-default/implementation-handoff.md)) |
| 071 | [The visibility floor tests a bar the palette cleared four minors ago](./done/071-the-visibility-floor-tests-a-bar-cleared-four-minors-ago.md) | v0.38.1 (handoff: [`handoffs/071-…`](./handoffs/071-the-visibility-floor-tests-a-bar-cleared-four-minors-ago/implementation-handoff.md)) |
| 072 | [Contrast values are bounded below and never above](./done/072-contrast-values-are-bounded-below-and-never-above.md) | v0.38.1 (handoff: [`handoffs/072-…`](./handoffs/072-contrast-values-are-bounded-below-and-never-above/implementation-handoff.md)) |
| 073 | [Three pages that outlived the facts they state](./done/073-pages-that-outlived-the-facts-they-state.md) | v0.38.2 (handoff: [`handoffs/073-…`](./handoffs/073-pages-that-outlived-the-facts-they-state/implementation-handoff.md)) |
| 074 | [The release checklist names two files by hand, so every other version snippet drifts](./done/074-version-snippets-are-a-hand-maintained-list.md) | v0.38.3 (handoff: [`handoffs/074-…`](./handoffs/074-version-snippets-are-a-hand-maintained-list/implementation-handoff.md)) |
| 075 | [The gate register contradicts itself, and the frozen-surface list omits seven of the functions it freezes](./done/075-the-gate-register-and-the-frozen-surface-are-both-wrong.md) | v0.38.3 (handoff: [`handoffs/075-…`](./handoffs/075-the-gate-register-and-the-frozen-surface-are-both-wrong/implementation-handoff.md)) |
| 076 | [`snora` publishes a function whose return type it does not export](./done/076-the-facade-publishes-a-type-it-does-not-export.md) | v0.39.0 (handoff: [`handoffs/076-…`](./handoffs/076-the-facade-publishes-a-type-it-does-not-export/implementation-handoff.md)) |
| 077 | [The border is not what outlines the dialog card](./done/077-the-border-is-not-what-outlines-the-card.md) | v0.39.0 (handoff: [`handoffs/077-…`](./handoffs/077-the-border-is-not-what-outlines-the-card/implementation-handoff.md)) |
| 079 | [The migration index promises a guide for every minor, and six are missing](./done/079-the-migration-index-promises-a-guide-for-every-minor.md) | v0.39.1 (handoff: [`handoffs/079-…`](./handoffs/079-the-migration-index-promises-a-guide-for-every-minor/implementation-handoff.md)) |
| 080 | [The migration guide and the release letters say the same thing](./done/080-the-migration-guide-and-the-release-letters-are-the-same-document.md) | v0.39.1 (handoff: [`handoffs/080-…`](./handoffs/080-the-migration-guide-and-the-release-letters-are-the-same-document/implementation-handoff.md)) |
| 081 | [The 12-pixel text floor is the one mandatory number nothing asserts](./done/081-the-12px-text-floor-is-asserted-nowhere.md) | v0.39.2 (handoff: [`handoffs/081-…`](./handoffs/081-the-12px-text-floor-is-asserted-nowhere/implementation-handoff.md)) |
| 082 | [Three keyboard-and-focus statements a reader cannot trust](./done/082-three-keyboard-and-focus-statements-a-reader-cannot-trust.md) | v0.39.2 (handoff: [`handoffs/082-…`](./handoffs/082-three-keyboard-and-focus-statements-a-reader-cannot-trust/implementation-handoff.md)) |
| 083 | [One workspace feature breaks docs.rs, pulls iced into the vocabulary crate, and silently enables `advanced`](./done/083-lucide-drags-iced-advanced-into-the-vocabulary-crate.md) | v0.40.0 (handoff: [`handoffs/083-…`](./handoffs/083-lucide-drags-iced-advanced-into-the-vocabulary-crate/implementation-handoff.md)) |
| 084 | [Overlays do not contain pointer events, and the dialog dismisses itself](./done/084-overlays-do-not-contain-pointer-events.md) | v0.41.0 — **Critical** (handoff: [`handoffs/084-…`](./handoffs/084-overlays-do-not-contain-pointer-events/implementation-handoff.md)) |
| 085 | [The widget layer pairs colours from different families, and no contrast suite can see it](./done/085-the-widget-layer-styles-are-unreachable-by-every-contrast-suite.md) | v0.41.0 — **Critical** (handoff: [`handoffs/085-…`](./handoffs/085-the-widget-layer-styles-are-unreachable-by-every-contrast-suite/implementation-handoff.md)) |
| 086 | [The engine's toast colours fail their own thresholds](./done/086-engine-toast-contrast.md) | v0.42.0 — High (handoff: [`handoffs/086-…`](./handoffs/086-engine-toast-contrast/implementation-handoff.md)) |
| 087 | [CI runs a subset of the tests, and a conditional deferral has been renewed by habit](./done/087-ci-runs-a-subset-of-the-tests.md) | v0.41.1 — High (handoff: [`handoffs/087-…`](./handoffs/087-ci-runs-a-subset-of-the-tests/implementation-handoff.md)) |
| 088 | [The workspace forces three iced features nobody asked for](./done/088-the-workspace-forces-three-iced-features-nobody-asked-for.md) | v0.42.0 — High (handoff: [`handoffs/088-…`](./handoffs/088-the-workspace-forces-three-iced-features-nobody-asked-for/implementation-handoff.md)) |
| 089 | [Documentation and hygiene sweep from the external audit](./done/089-documentation-and-hygiene-sweep.md) | v0.41.1 — Medium/Low (handoff: [`handoffs/089-…`](./handoffs/089-documentation-and-hygiene-sweep/implementation-handoff.md)) |
| 090 | [The release rules nothing enforces](./done/090-the-release-rules-nothing-enforces.md) | v0.42.0 — High (handoff: [`handoffs/090-…`](./handoffs/090-the-release-rules-nothing-enforces/implementation-handoff.md)) |

## Accepted

Design settled, implementation may start, not yet shipped. See
[`accepted/README.md`](./accepted/README.md). Empty is the normal resting
state.

RFC-092 came out of the 0.41.0–0.42.0 cycle, not the audit.

| ID | Title | Target |
|----|-------|--------|
| 092 | [Claims about code are not checked the way code is](./accepted/092-claims-about-code-are-not-checked-the-way-code-is.md) | v0.43.0 — High |

## Proposed

| ID | Title | Target |
|----|-------|--------|
| 093 | [A contrast gate cannot see 1.4.1, and ours never could](./proposed/093-a-contrast-gate-cannot-see-1-4-1.md) | v0.43.0 — High |

## Archive

Withdrawn or superseded — see [`archive/README.md`](./archive/README.md).

| ID | Title | Superseded by |
|----|-------|---------------|
| 078 | [Measure what iced's `advanced` feature costs](./archive/078-measure-what-iceds-advanced-feature-costs.md) | The owner's ruling that `advanced` must never be a default, plus the finding that **no consumer ever requested it** — the measurement's only question |
| 091 | [Trusted Publishing, and a deferral with a date on it](./archive/091-trusted-publishing.md) | **RFC-090**, which adopted Trusted Publishing directly — the deferral this RFC existed to keep honest turned out to be unnecessary |


_(none yet)_
