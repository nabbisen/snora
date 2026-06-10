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

## Near-term: 0.16+ (next phase)

Primary goal: strategic evidence — alternate engine boundary assessment,
performance envelope, and downstream adoption feedback.

See RFC-016-A through RFC-016-C in `rfcs/proposed/` for details.

## Recently shipped

- **0.15** — Public surface and adoption maturity: `snora` docs.rs metadata
  (`all-features`); install.md version fix + "which crate?" section;
  versioning policy doc + migration template; decision index with
  status/trigger labels; starter application example (177 ELOC, workspace
  member).
- **0.14** — Interaction and boundary clarity: `snora::keyboard::dismiss_on_escape`
  helper + workbench Escape wiring; warning color named const; overlay
  accessibility boundary; icons.md feature-gating policy; examples acceptance
  matrix.
- **0.13** — Design expansion: anchored popover design study; API freeze review
  tracker; tooltip/persistent-toast evidence check.
- **0.12** — Semantic testing and ABDD maturity: RFC-011-D full acceptance;
  ABDD checklist + PR template; workbench example; compile-time tracking;
  documentation test policy.
- **0.11** — Foundation hardening: main Rust CI; toast ordering bugfix;
  `AppLayout` `#[non_exhaustive]`; overlay semantics reference page;
  render-semantics test harness; RFC directory.

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
