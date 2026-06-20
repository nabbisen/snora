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

## Post-0.17: toward 1.0

Six of ten 1.0 gates are now satisfied. The remaining path:

1. One iced major upgrade completed. ⬜
3. At least one third-party or production-grade app. ⬜
7. Public API freeze review completed. ✅ v0.18
9. Binary-size and compile-time trends (≥2 data points). ⬜ first point v0.17

Work on these proceeds alongside any v0.18+ feature work. There is no
scheduled date for 1.0.

## Snora Design System (v0.19 foundation shipped — RFC-020 … RFC-030)

The Snora Design System foundation has landed in v0.19 as an **opt-in**
layer (`design` feature; `snora-design` published as of v0.19.0):

- Iced-free `snora-design` crate with `Tokens`, `Palette`, `Color`,
  `Spacing`, `Typography`, `Radius`, `FocusTokens`, `Tone`/`Emphasis`/
  `Size`/`Density`; four presets with automated WCAG AA contrast tests.
- iced style bridge (`snora::design::style`): button + card + text helpers.
- Pilot button helpers: `primary`, `secondary`, `ghost`, `danger` + `*_maybe`.
- Pilot card helpers: `surface`, `raised`, `selected`.
- CI quality gates: `design-isolation` job (Q3 iced-free), feature matrix.
- Accessibility checklist, semantic construction policy, API governance docs.
- Design workbench example.
- RFC-031 (v0.20 release gate), RFC-032/033/034 (future-phase) remain
  in `rfcs/proposed/`.

Remaining design-track sequence:

- **v0.20** — activate and publish `snora-design`; measure binary-size /
  build-cost; implement RFC-031 release acceptance gate.
- **v0.21+** — notice/chip/progress primitives (RFC-032).
- **v0.22+** — recipes and dogfooding (RFC-033).
- **v0.23+** — promotion / stabilization and API governance (RFC-034).

## Recently shipped

- **0.19** — Snora Design System foundation (RFC-020–030, opt-in `design`
  feature): `snora-design` crate, style bridge, pilot button/card helpers,
  CI quality gates, accessibility docs, API governance, design workbench.
  `publish = false` until v0.20 activation. Docs gate fixed (book.toml,
  fence-tag policy). Migration guide: 0.18 → 0.19.
- **0.18** — Documentation maturity: contributing index; version snippets updated to 0.17; Gate 7 ✅ (API freeze review complete, 7/10 gates satisfied).
- **0.17** — 1.0 gate advancement: `Icon` gains `PartialEq`; two RTL
  render-semantics integration tests (10 total); keyboard.rs doc fence fix;
  first build-cost data points in all three CSVs; api-freeze-review.md
  fully updated (6/10 gates satisfied); Gate 2 ✅ (vocabulary stable
  v0.13–v0.16).
- **0.16** — Strategic evidence: alternate engine boundary doc; performance
  envelope + render-cost script; downstream feedback and feature-request
  issue templates; feedback-and-scope guide; README contribution section.
- **0.15** — Public surface and adoption maturity: docs.rs metadata;
  install.md version fix; versioning policy + migration template; decision
  index; starter application example.
- **0.14** — Interaction and boundary clarity: `dismiss_on_escape` helper;
  warning color const; overlay accessibility boundary; icons feature-gating
  policy; examples acceptance matrix.
- **0.13** — Design expansion: anchored popover study; API freeze review
  tracker; tooltip/persistent-toast evidence check.
- **0.12** — Semantic testing and ABDD maturity: RFC-011-D full acceptance;
  ABDD checklist; workbench example; compile-time tracking; doc test policy.
- **0.11** — Foundation hardening: main Rust CI; toast ordering bugfix;
  `AppLayout` `#[non_exhaustive]`; overlay semantics; render-semantics
  test harness; RFC directory.

## Longer-term: 1.0

Snora hits 1.0 when the API surface has been stable across a few
releases and we are confident it will not need a wholesale redesign.

The full readiness checklist is in
[`docs/src/contributing/api-freeze-review.md`](docs/src/contributing/api-freeze-review.md).

**Summary of 1.0 gates** (✅ = satisfied):

1. One iced major upgrade completed and lived on ≥1 minor. ⬜
2. Two consecutive minors without vocabulary churn. ✅ v0.13–v0.16
3. At least one third-party or production-grade app. ⬜
4. AppLayout construction policy decided. ✅ v0.11
5. Render-semantics tests cover z-stack, dismissal, toast, RTL. ✅ v0.17
6. Feature-matrix CI stable. ✅ v0.11
7. Public API freeze review completed. ✅ v0.18
8. Showcase/workbench example exercises all major surfaces. ✅ v0.12
9. Binary-size and compile-time trends monitored (≥2 data points). ⬜ first point v0.17
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
