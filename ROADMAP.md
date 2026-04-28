# Roadmap

This document records the directions snora's maintainers expect to
take in upcoming releases, with rough priority and reasoning. It is
**not** a commitment — order and scope can change in response to
real-world usage and feedback. Items move from this document into
[CHANGELOG.md](CHANGELOG.md) when they ship.

For released history, see [CHANGELOG.md](CHANGELOG.md). For the
*why* behind closed design decisions, see
[`docs/contributing/design-decisions.md`](docs/contributing/design-decisions.md).

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

## Near-term: 0.8 (next release)

These items have either concrete demand or a small enough scope
that they fit a single release.

### Likely

- **Establish a binary-size baseline.** Per
  [feature-gating-criteria.md](docs/contributing/feature-gating-criteria.md),
  indicator (2) needs a baseline measurement (`snora-example-hello`
  with and without `--no-default-features`) recorded so future
  releases can detect drift.
- **Doctest coverage for the new widget vocab.** `Tab`, `TabBar`,
  `Crumb` should have small `cargo test --doc` examples to lock
  the surface against accidental signature change.

### Maybe

- **Persistent-toast acknowledgement helper.** The persistent-toast
  pattern (used in apps for "Export complete" style notifications)
  recurs enough that a small ergonomic helper might be worth
  shipping. Not a vocabulary change — pure ergonomics.
- **Tooltip vocabulary.** Currently `SideBarItem.tooltip: String` is
  the only typed tooltip in snora. If a second widget grows a
  tooltip slot, it warrants pulling tooltips out into shared
  vocabulary (`Tooltip { text: String, side: Edge }` or similar).
  Watch for the second consumer.

## Middle-term: 0.9 — 0.10

Things we expect to want but that need design work or signal from
real applications first.

### Watch list

- **Status bar widget.** Considered for 0.7 and deferred — current
  position is "the existing `app_footer` + `row_dir_three` covers
  this well enough that a dedicated `app_status_bar` would be a
  thin wrapper without earning its keep". A concrete app whose
  needs do not fit `app_footer` would change that.
- **Anchored popover overlay.** `Dialog` is centered; `Sheet` is
  edge-anchored. Neither covers "popover anchored to the widget
  the user clicked" (combobox dropdowns, hover cards). Designing
  this requires deciding how the anchor point is communicated to
  the engine — an iced `Point` or a layout reference. Out of scope
  until a concrete need arrives.
- **Command palette overlay.** Discussed in
  [adding-an-overlay.md](docs/contributing/adding-an-overlay.md) as
  a hypothetical example. If multiple applications request it, it
  becomes a real candidate.
- **`#[doc(cfg(feature = ...))]` polish.** Make sure every feature-
  gated item is correctly tagged so docs.rs renders the toggles.

### Investigation

- **Per-widget feature gates.** Coarse gating
  ([feature-gating-criteria.md](docs/contributing/feature-gating-criteria.md))
  is the current decision. Re-evaluate at each release; split when
  the documented thresholds are met.
- **Snapshot of compile time and binary size in CI.** Once the
  baseline from 0.8 exists, automate the measurement so the
  feature-gating thresholds are checked without human effort.

## Longer-term: 1.0

Snora hits 1.0 when the API surface has been stable across a few
releases and we are confident it will not need a wholesale redesign.

Concrete prerequisites:

1. **iced 0.15 (or whatever the next major iced is) integration is
   done.** The next iced bump is the most likely source of breaking
   changes; we want at least one minor cycle on the new iced before
   committing to 1.0.
2. **The vocabulary set has stopped growing rapidly.** If two
   consecutive minor releases ship without a new vocabulary type
   being added (or worse, with one being renamed), the surface is
   stable enough to commit to.
3. **At least one third-party application in production.** "It is
   used by something other than its examples and the maintainer's
   own projects."

We are explicitly **not** rushing to 1.0. Pre-1.0 SemVer is
serving snora well — minor versions can carry small breaking
changes when justified, with deprecation aliases bridging two
releases (the pattern used at 0.5 → 0.6 and 0.6 → 0.7).

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
  [testing guide](docs/guides/testing.md) covers what `pub` fields
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
