# RFC 031 — v0.20 Release Acceptance Criteria

**Status.** Implemented (v0.20.0)
**Tracks.** Snora Design System migration; v0.20 release gate.
**Touches.** release checklist, changelog, migration notes.

## Summary

This RFC defines the v0.20 release acceptance criteria.

v0.20 is the Design System Foundation milestone, not a complete design
system.

### Roadmap position

v0.19 is reserved for 1.0-gate advancement (iced major upgrade, third-party
app, second build-cost data point) and is **not** a design release. The
design-system foundation is v0.20. Design work may proceed on a branch
before v0.19 ships, but does not claim the v0.19 release number.

## Required deliverables

### Philosophy

- Boundary statement added.
- Non-goals updated.
- Minimal/default/custom paths documented.

### Architecture

- `snora-design` crate or equivalent exists.
- `snora-design` has no iced dependency.
- `snora-widgets` owns iced implementation.
- Minimal path remains green.
- Default features are `["widgets"]`; `design` is opt-in. Binary-size and
  build-cost are measured with and without `design` before any decision to
  make `design` default-on; the CSV budgets are re-baselined for the new
  `snora-design` crate.
- crates.io publish order is `snora-core → snora-design → snora-widgets →
  snora`, and the release-process doc is updated to match.

### Token model requirements

Status text roles (`success_text`, `warning_text`, `danger_text`,
`info_text`) are part of `Palette`. `Tokens` and `Palette` are
`#[non_exhaustive]`.

### Required token types

- `Tokens`
- `Palette`
- `Color`
- `Typography`
- `TextRole`
- `Spacing`
- `Radius`
- `FocusTokens`
- `Tone`
- `Emphasis`
- `Size`
- `Density`

### Presets

- `Tokens::light()`
- `Tokens::dark()`
- `Tokens::high_contrast_light()`
- `Tokens::high_contrast_dark()`

### Tests

- token sanity tests;
- automated contrast tests;
- feature matrix;
- docs build;
- examples compile.

### iced bridge

- color conversion;
- button style functions;
- card/container style functions;
- text/line-height mapping policy.

### Pilot helpers

- basic button helper/wrapper;
- basic card helper/wrapper.

### Accessibility

- checklist;
- semantic construction policy;
- high-contrast docs;
- visual-fit workbench review.

## Allowed deferrals

May defer:

- density implementation;
- progress helper;
- root facade token-only design feature;
- custom iced theme conversion;
- notice/chip/progress primitives;
- recipes;
- OS accessibility setting sync;
- snapshot visual tests.

Must not defer:

- minimal path;
- iced-free `snora-design`;
- high-contrast presets;
- contrast tests;
- typography line-height;
- boundary docs.

## Release process

1. Freeze scope.
2. Complete RFC-020 through RFC-030.
3. Implement foundation.
4. Run gates (including the feature matrix and contrast tests).
5. Measure binary-size / build-cost with and without `design`.
6. Perform visual-fit workbench review.
7. Update changelog and migration notes (new crate, new `design` feature).
8. Publish in dependency order:
   `snora-core → snora-design → snora-widgets → snora`.
9. Tag release.

## Acceptance criteria

v0.20 can release only when all non-deferrable items are complete and gates pass.
