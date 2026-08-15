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

**Seven** of ten 1.0 gates are satisfied. The remaining path:

1. One iced major upgrade completed. ⬜
3. At least one third-party or production-grade app. ⬜
7. Public API freeze review completed. ✅ v0.18
9. Binary-size and compile-time trends (≥2 data points). ⬜ **reopened
   v0.25.3** — the measurement workflows had never fired on a release tag
   (RFC-041), and the methodology itself measures ~0 marginal cost
   (RFC-043). Previously recorded as satisfied at v0.19.1; that was not
   supported by the data.

Work on these proceeds alongside any v0.18+ feature work. There is no
scheduled date for 1.0.

## Snora Design System (complete — RFC-020 … RFC-040)

The Snora Design System shipped across five minor releases:

- **v0.19** — Foundation groundwork (RFC-020–030): iced-free `snora-design`
  crate, token model, WCAG AA contrast tests, iced style bridge, pilot
  button/card helpers, CI quality gates, accessibility and governance docs.
  `snora-design` was `publish = false` in v0.19.
- **v0.20** — Activation (RFC-031): `snora-design` published; release gate
  satisfied; `design` confirmed opt-in.
- **v0.21** — Primitives (RFC-032): notice, chip, and progress.
- **v0.23** — Recipes and governance (RFC-033, RFC-034): four initial recipes,
  API governance process.
- **v0.24** — Architect-review cleanup: chip contrast fix, measurement
  methodology improvements, documentation corrections.

All RFC-020 through RFC-034 are in `rfcs/done/`.

**Appearance milestone (RFC-037 … RFC-040) — complete.** A second design
phase, opened after real-world use showed that an application built on
snora looked flat and unresponsive: snora's own chrome never participated
in the token system. v0.26.0 delivered the colour half (an emitted
`iced::Theme` that chrome follows transitively); v0.27.0 the surfaces and
geometry (dialog card, derived modal dim, chrome spacing and radius).

**Elevation was never delivered and is not planned.** `Tokens` carries no
shadow or elevation scale, and adding one is a frozen-surface change under
RFC-036's covenant that no RFC in this milestone chose to pay for. Early
scoping listed it; that was an assumption about a vocabulary that does not
exist.

The `design` feature remains **opt-in** (`default = ["widgets"]`), and from
v0.27.0 the new surfaces are opt-in *per call site* on top of that — with
`design` inactive, rendering is unchanged. Making `design` default-on
requires an explicit size/build-cost review and a future RFC or release
decision — measurement alone does not automatically flip the default.

Future design work is governed by
[`api-governance.md`](docs/src/contributing/api-governance.md) and the
D-gates.

## Recently shipped

- **Gate 9 split (recorded v0.29.0).** Binary-size trend monitoring is
  **satisfied** — four rows on one runner and methodology, and the series
  tracks real change. Compile-time monitoring stays **open**: the same four
  rows spread 21–27%, and the documentation-only 0.28.1 moved them 8–11%
  with zero code changed, so the variance exceeds any per-release signal.
  Recorded as 9a/9b rather than ticked whole — RFC-041 exists because a gate
  was once declared satisfied on data that did not support it.
- **0.30.0** — `examples/responsive_body`: an engine-only responsive example
  composed the way snora's known consumers actually build (RFC-051).
  `responsive_render` shipped in 0.28.0 because a downstream team asked for
  it, yet every demonstration of it taught `AppLayout::side_bar` built from
  prefab widgets — which that team uses none of. The new example varies
  `body`'s own composition instead, with engine-only enforced by the compiler
  rather than by discipline. Additive; nothing else changed.
  **Also fixes** version snippets in the icons guide, stale since 0.28.0
  because the 0.29.0 release missed a checklist item that names that file.
- **0.29.0** — `snora-dialog-card` now names the card. Since 0.27.0 it was
  attached to the dialog's full-window centring container, so resolving it
  returned window-sized bounds and the actual card carried no identifier at
  all — a stable identifier that was present on every render and pointed at
  the wrong element. The centring container is now `snora-dialog`; the card
  keeps the `snora-dialog-card` name, re-pointed (RFC-049). **First rename
  exercised under the identifier compatibility policy** shipped in 0.28.0:
  it is a minor bump, announced in the CHANGELOG and migration guide rather
  than avoided. No deprecation bridge is possible for string identifiers, so
  a test asserting the old referent changes meaning silently — accepted only
  because no consumer had adopted 0.28.0 identifiers yet.
- **0.28.1** — From a second downstream field report (**arama**). snora's
  documentation contradicted itself about the dialog card: `overlays.md`
  promised "a centered modal card" and denied any card chrome eleven lines
  later, in the same file, since at least v0.25.0. The behaviour was correct
  and the card had shipped in 0.27.0 — the docs were what was wrong. Seven
  claim sites and four z-stack tables now distinguish what `snora::render`
  draws from what `snora::design::render` draws, and the dialogs guide
  surfaces the card instead of implying it will never exist (RFC-048). The
  rule whose absence caused this — **when a `design`-gated capability lands,
  every default-path page that denies it is in scope** — is now written down.
  Documentation only; no code changed.
- **0.28.0** — From the first substantive downstream field report.
  `snora::responsive_render` exposes the layout's available width — snora
  observed window size nowhere before, so every consumer wanting
  breakpoints wrote that plumbing themselves (RFC-046). Stable identifiers
  on every surface snora renders itself — the modal dim, backdrops, dialog
  card, sheet panel, toasts, skeleton regions — which are exactly the
  surfaces an application cannot label, because it never sees them
  (RFC-047). Identifier names are now a compatibility surface.
  **snora deliberately prescribes no breakpoint thresholds**; that is
  deferred until there is evidence about what real applications converge
  on, which this release makes gatherable.
- **0.27.1** — snora's assistive-technology position stated: it will
  integrate an accessibility tree when iced exposes one, and will not build
  a parallel abstraction meanwhile. The ABDD claim is bounded where it
  overclaimed — "accessibility correct from day one" now reads
  "layout-direction and visual accessibility … not assistive-technology
  support" (RFC-045). Documentation only.

- **0.27.0** — Appearance milestone complete. `snora::design::render`
  gives the dialog a real card and derives the modal dim from tokens —
  fixing a latent defect where the hardcoded 40% black scrim was
  *completely invisible* over `high_contrast_dark`'s pure-black background
  (RFC-039). `snora::design::widget::*` derives chrome spacing and radius
  from the token scales, replacing seven unrelated padding shapes and
  square corners (RFC-040). Both are opt-in **per call site**, not per
  feature flag: existing call sites keep their exact appearance.
  Measurement `runner_os` fixed — GitHub reserves `RUNNER_*`, so v0.26's
  override was silently ignored (RFC-044).

- **0.26.0** — Appearance milestone, first half. `snora::design::theme`
  emits an `iced::Theme` from Snora Design tokens, so stock iced widgets
  and the window background follow the same palette as snora's design
  primitives (RFC-038); DEC-02 amended to split theme-*producing*
  (permitted under `design`) from theme-*owning* (still declined), and
  RFC-020's long-unsatisfied "boundary statement in docs" criterion
  discharged (RFC-037); budget measurement methodology corrected — the
  size probes never called the features they measured, so
  `widgets_diff_bytes` had been reporting 0 (RFC-043).

- **0.25.3** — Documentation accuracy, measurement integrity, build
  reproducibility. Corrected in-tree docs that still described a
  three-crate workspace five releases after `snora-design` shipped
  (RFC-035); design-surface freeze review closing D-3/D-4 with an
  additive-only covenant (RFC-036); fixed measurement workflows that had
  **never** run on a release tag and reopened gate 9 (RFC-041); declared
  `rust-version = "1.88"` — the documented 1.85 was false; committed
  `Cargo.lock` with a weekly unpinned-build job (RFC-042). No public API,
  feature-flag, or runtime behavior change.
- **0.25.2** — Workspace resolver `"2"` → `"3"`; member globbing; version
  snippets. No source change.
- **0.25.1** — `snora::design::contrast` facade re-export.
- **0.25.0** — Measurement methodology fix (size-probe crates replacing hello/workbench diff); build-cost cold-clean fix; RFC-031 index fix; docs corrections.
- **0.24.0** — Architect-review cleanup: M-4 chip contrast fix, M-1/M-2/M-3/M-5/M-6/M-7 must-fixes, S-series should-fixes, all from v0.23 review.
- **0.23.0** — Recipes and dogfood process (RFC-033): four initial recipes (empty-state, background-task, error-recovery, result-card); RFC-034 governance formally closed. All 15 design-track RFCs complete.
- **0.22.0** — Code quality and doc audit: chip dedup, new tests (chip/notice/progress), three new design doc pages (notices/chips/progress), stale version refs cleaned.
- **0.21.0** — Notice, chip, and progress primitives (RFC-032); design workbench updated.
- **0.20.0** — Snora Design activation: `snora-design` published; RFC-031 release gate satisfied; `design` opt-in confirmed; migration guide 0.19→0.20.
- **0.19.1** — CI fixes: `measure-compile-time.sh` missing-space bug fixed;
  `binary-size.csv` schema corrected (7-field rows); Gate 9 ✅ (binary-size
  and build-cost both have ≥2 real CI data points). Remaining 1.0 blockers:
  gate 1 (iced upgrade) and gate 3 (third-party app).
  **This gate-9 claim was false and was retracted in 0.25.3 (RFC-041):**
  the workflows had never run on a release tag, and every row was `N/A`.
  Left here as written rather than edited, per the append-only record
  policy — see 0.25.3 above and 0.29.0 for where the gate actually stands.
- **0.19.0** — Snora Design System foundation (RFC-020–030, opt-in `design`
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
9. Binary-size and compile-time trends monitored (≥2 data points). ⬜
   (reopened v0.25.3 — see above and RFC-041 / RFC-043)
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
