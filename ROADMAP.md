# Roadmap

This document records the directions snora's maintainers expect to
take in upcoming releases, with rough priority and reasoning. It is
**not** a commitment — order and scope can change in response to
real-world usage and feedback. Items move from this document into
[CHANGELOG.md](CHANGELOG.md) when they ship.

For released history, see [CHANGELOG.md](CHANGELOG.md). For the
*why* behind closed design decisions, see
[`docs/src/contributing/design-decisions.md`](docs/src/contributing/design-decisions.md).

## Guiding principles (unchanging)

These constrain what *can* be on the roadmap:

- **Snora is a framework, not a UI components library.** New
  components only ship if they fit the skeleton + overlay model,
  serve typical desktop chrome (header / sidebar / body / footer
  / tabs / breadcrumbs / status), and do not pull snora toward
  being a generic widget library.
- **ABDD is non-negotiable.** Every layout-affecting addition must
  use logical edges (`Edge::Start` / `Edge::End`) and respect
  `LayoutDirection`. We do not accept widgets with hardcoded
  left/right.
- **`snora-core` stays iced-free.** Anything that needs iced goes
  into `snora` (engine) or `snora-widgets` (visuals).
- **Vocabulary over flags.** New configuration is expressed via a
  named enum, not a `bool` or magic constant.
- **No silent drops.** If an `AppLayout` field is populated, the
  engine renders it; a missing close sink only suppresses the
  click-outside backdrop, never the content.

## Near-term: 0.14 (next release)
  (`reference/binary-size-budget/binary-size.csv`) appended on each
  release tag, a `release-baseline` Cargo profile for fast
  measurement, the `binary-size` GitHub Actions workflow, and the
  `binary-size-budget.md` reference page wiring it to
  feature-gating-criteria indicator (2).
- **0.9** — Doctest coverage for `snora-core` vocabulary (17 new
  doctests across `Tab`, `TabBar`, `TabAction`, `Crumb`,
  `BreadcrumbAction`, `Sheet`, `SheetEdge`, `SheetSize`, `Toast`,
  `ToastPosition`, `LayoutDirection`); migration guides collapsed
  to a single index entry in the SUMMARY.
- **0.8** — mdBook documentation, GitHub Pages deployment, Docs CI
  workflow, project-level GitHub conventions
  (`.github/CONTRIBUTING.md`, security policy, code of conduct,
  issue templates).
- **0.7** — `Tab` + `Crumb` vocabulary and widgets; removal of
  the deprecated 0.6 sheet aliases; documented feature-gating
  criteria.
- **0.6** — `Sheet` overlay generalization (4 edges, axis-relative
  size); 3-crate workspace split (`snora-core` / `snora-widgets` /
  `snora`).

## Near-term: 0.14 (next release)

Primary goal: interaction and boundary clarity — keyboard behavior,
accessibility, theme documentation, icon policy, and examples acceptance.

See RFC-014-A through RFC-014-E in `rfcs/proposed/` for details.

## Recently shipped

- **0.13** — Design expansion: anchored popover design study (eight
  internal questions answered, decision to defer recorded); public API
  freeze review doc (`api-freeze-review.md`) with current gate status;
  tooltip/persistent-toast evidence check (both deferred — triggers unmet).
- **0.12** — Semantic testing and ABDD maturity: RFC-011-D full acceptance
  (8 render-semantics integration tests, 5 RTL unit tests); ABDD compliance
  checklist and PR template; workbench example exercising all surfaces;
  compile-time tracking script + workflow + docs; documentation test policy
  (54 fences classified, `mdbook test` in CI).
- **0.11** — Foundation hardening: main Rust CI quality gate (three-job
  workflow covering check/clippy/tests/feature-matrix/docs); toast ordering
  bugfix; `AppLayout` is `#[non_exhaustive]`; overlay interaction semantics
  reference page; render-semantics test harness; RFC directory adopted.

## Middle-term: 0.15

Public surface and adoption maturity — versioning policy, re-export and
docs.rs cleanup, starter template, design-decision register maintenance.
See RFC-015-A through RFC-015-D.

## Longer-term: 1.0

Snora hits 1.0 when the API surface has been stable across a few
releases and we are confident it will not need a wholesale redesign.

The full readiness checklist is in
[`docs/src/contributing/api-freeze-review.md`](docs/src/contributing/api-freeze-review.md).

**Summary of 1.0 gates** (✅ = satisfied):

1. One iced major upgrade completed and lived on ≥1 minor. ⬜
2. Two consecutive minors without vocabulary churn. ⬜
3. At least one third-party or production-grade app. ⬜
4. AppLayout construction policy decided. ✅ v0.11
5. Render-semantics tests cover z-stack, dismissal, toast, RTL. ✅ v0.12
6. Feature-matrix CI stable. ✅ v0.11
7. Public API freeze review completed. ⬜ in progress
8. Showcase/workbench example exercises all major surfaces. ✅ v0.12
9. Binary-size and compile-time trends monitored (≥2 data points). ⬜
10. No hidden feature-combination failures. ✅ (CI gate)

We are explicitly **not** rushing to 1.0. Pre-1.0 SemVer is serving
snora well; minor versions can carry small breaking changes when
justified, with deprecation bridges across two releases.

## Off the roadmap (deliberately not pursued)

These come up in discussion and are repeatedly declined. Listed
here so the answer is visible.

- **Form widgets** (`text_input` wrappers, validation primitives,
  `field` / `section`). iced's primitives do this; snora wrapping
  them adds layers without value. Form-heavy apps stay viable on
  snora — the AppLayout slots accept any iced element — but snora
  does not provide form shortcuts.
- **Data display widgets** (`data_table`, `chart`, `card_grid`).
  Out of snora's "framework" scope — these are UI library territory.
  Use iced canvas or a dedicated data-viz crate.
- **Decorative widgets** (`avatar`, `badge`, `chip`). Trivial enough
  to write in a few lines; absorbing them into snora would expand
  the surface without commensurate value.
- **A `snora-test` crate.** The
  [testing guide](docs/src/guides/testing.md) covers what `pub` fields
  on the vocabulary types already enable. A dedicated test-helper
  crate would freeze internal shapes into the public API.
- **Game-loop or real-time rendering support.** snora is
  retained-mode / event-driven. Real-time rendering belongs to iced
  canvas or a different framework.

If you have a use case that lands in one of the categories above
but you think snora *should* support it, open an issue with a
concrete scenario — these decisions are not absolute, just strongly
held defaults.

## How to influence this roadmap

- **Open an issue** describing your use case. Concrete app stories
  carry far more weight than abstract requests.
- **Send a PR** that demonstrates the design. Code is the most
  legible argument.
- **Reach out to the maintainer** at the email address in the
  workspace `Cargo.toml`.

The roadmap is updated alongside each release, typically in the
same PR that bumps the workspace version. Stale items are not a
sign of abandonment; they are a sign that something more pressing
arrived.
