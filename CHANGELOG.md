# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

While the crate version is below 1.0, breaking changes are signaled by a
**minor** bump (e.g. `0.7.0` → `0.8.0`); patch releases (`0.7.0` → `0.7.1`)
are bug fixes and additive changes only.

This file begins its history at the 0.7.0 release. Earlier release notes
are recorded in the per-version migration guides under
[`docs/guides/`](docs/src/guides/).

## [0.29.0] — 2026-08-15

### Changed

- **`snora-dialog-card`'s referent changed — it now names the actual
  card, not the window (RFC-049).** Since v0.27.0, `snora-dialog-card`
  was attached to the dialog's full-window centring container, not the
  styled card RFC-039 introduced; the card itself carried no identifier.
  Resolving `snora-dialog-card` always returned window-sized bounds,
  never the card's — a stable identifier that was "present" on every
  render but pointed at the wrong element, exactly the failure mode this
  system exists to prevent. Fixed by splitting the name: the centring
  container is now `snora-dialog` (always present, both paths), and
  `snora-dialog-card` is **re-pointed**, not retired, to the actual
  styled card (present only on `snora::design::render`). This is a
  **minor** bump, per the [versioning policy's rendered-surface-
  identifiers rule](docs/src/contributing/versioning-policy.md#rendered-surface-identifiers)
  — the first rename exercised under that rule. **No deprecation bridge
  is possible**: these are plain strings, not Rust symbols, so a
  downstream test asserting on the old referent does not fail on
  upgrade — it silently starts resolving the card instead of the window.
  Accepted for this one release only because no known consumer had
  adopted 0.28.0 identifiers yet. See the [migration
  guide](docs/src/guides/migration-0.28-to-0.29.md) for the exact
  before/after table.

## [0.28.1] — 2026-08-15

### Changed

- **Corrected a self-contradicting documentation claim about the dialog
  card (RFC-048).** A downstream team (**arama**, credited — this is
  their finding) reported that `render_dialog`'s module doc "is
  documented as producing *the centered modal card*, but draws no card."
  The behavior was correct and deliberate, and the card they wanted had
  already shipped in v0.27.0 via `snora::design::render` (RFC-039) — they
  were on v0.25.0. **The documentation was what was wrong, and it was
  wrong before v0.27.0 too:** `docs/src/guides/overlays.md` promised "a
  centered modal card" at one line and denied any card chrome eleven
  lines later, in the same file, since at least v0.25.0 — verified
  against the v0.25.0 tag directly, not inferred. This is not a case of a
  consumer missing documentation that contradicted the code; it is
  snora's own page contradicting itself, which leaves no way for a reader
  to tell which half binds. Corrected seven sites (the six identified,
  plus one the RFC's own grep missed — an `architecture.md` prose claim
  split across a line wrap, undetectable by a single-line pattern match)
  and the four z-stack tables describing layer order, distinguishing what
  the default `snora::render` path draws (no card, only centering and the
  dim) from what `snora::design::render` draws (a token-styled card) for
  both the card and the dim — the dim's 40% alpha is unchanged on both
  paths and was left alone; only its color derivation is path-specific.
  `docs/src/guides/overlays.md` — the page a consumer with exactly
  arama's problem would read — now states the card is available and
  links `docs/src/design/engine-surfaces.md`, rather than stating "snora
  is a positioner, not a styler" unscoped, which read as a promise the
  capability would never exist. `docs/src/contributing/
  feature-gating-criteria.md` now records the rule whose absence produced
  this: when a `design`-gated capability lands, every default-path page
  that states the capability is absent is part of the change scope, not
  only the new page documenting the capability itself — the same
  omission, three releases apart, that also produced RFC-045's
  discoverability gap. Documentation only: no executable code changed
  (`git diff --stat -- 'crates/**/*.rs'` touches doc-comment lines only),
  `render_semantics` passes unmodified, no API or gate-row change.

## [0.28.0] — 2026-08-04

### Added

- **`snora::responsive_render` — expose the layout's available width
  (RFC-046).** snora had no window-size awareness of any kind; every
  consumer wanting breakpoint-style behavior had to write window
  observation from scratch. `responsive_render(build)` wraps
  `iced::widget::Responsive` (reachable through `iced::widget::*`'s glob
  re-export — not separately documented by `iced`, confirmed by
  compiling against it rather than by reading its source) and calls
  `build` with the available width in logical pixels, in the same way
  `snora::render` and `snora::design::render` already work: the
  application supplies a closure, snora renders the result through the
  **same** shared z-stack `render` uses — no second copy of the layer
  composition. Exposes width only, not the full `Size` iced's own
  `Responsive` provides: width is what motivated this, and is the
  narrower, more conservative contract. Deliberately adds **no**
  `Breakpoint` type, threshold, or auto-collapse behavior — the
  application decides its own thresholds and what changes at them,
  matching every other "snora positions and stacks, the application
  decides" boundary in the project (no theming layer, no form widgets).
  A new guide (`docs/src/guides/responsive.md`) and runnable example
  (`examples/responsive`) demonstrate a threshold the example itself
  chooses. Not `design`-gated — this is engine capability, in the
  default surface alongside `render`. `render_semantics` passes
  unmodified; no new dependency, no `AppLayout` field.

- **Stable identifiers on every surface snora renders itself
  (RFC-047).** A downstream team building scripted GUI verification found
  no widget identifiers, no semantic names, and no state query — the
  window title was the only readable signal, used as a de facto
  accessibility API because nothing else existed. snora now attaches a
  documented `iced::widget::Id` to every surface it composes: the menu
  backdrop, the modal dim (both the click-capturing and non-capturing
  variants — deliberately the same identifier, since it names the
  surface, not its interactive behavior), the dialog's centered
  container, the sheet panel, the toast stack and each individual toast
  (`snora-toast-{id}`, derived deterministically from `Toast::id`), and
  the four skeleton regions (header/sidebar/body/footer — labeling the
  *slot* snora composed, never the application's own content inside it).
  Naming convention: `snora-` prefix, kebab-case, so snora's identifiers
  never collide with an application's own in a tree the application also
  populates. Always-on, not feature-gated — an `Id` has no rendering
  effect, and gating it would make tests pass or fail depending on
  feature selection. Every identifier is documented in a new reference
  page, `docs/src/reference/rendered-surface-identifiers.md`, verified
  against the actual emitted set by a dedicated drift test rather than
  hand-maintained — the failure mode this guards against is a stale
  reference asserting on a name that no longer exists. **From this
  release, these identifiers are a compatibility surface**: renaming or
  removing one is a minor version bump, not a patch, recorded in
  `docs/src/contributing/versioning-policy.md`. This provides labels on
  snora-rendered output only — not a test harness, not a state-query API,
  and not accessibility semantics (an `Id` is not a role; see RFC-045 for
  snora's separate, iced-blocked position on an actual accessibility
  tree). `render_semantics` passes unmodified, proving no appearance or
  behavior changed.

## [0.27.1] — 2026-08-04

### Changed

- **snora's assistive-technology position stated; the ABDD claim bounded
  (RFC-045).** A downstream team preparing UX acceptance evidence asked
  whether snora has a position on AccessKit or considers assistive
  technology out of scope — verified there is no accessibility tree, no
  AccessKit integration, and no semantic identifiers anywhere in the
  crates. The gap is defensible (iced 0.14 exposes no accessibility API
  for a layout framework to consume); snora's own framing was not — the
  name "Accessible By Default and by Design" invited a broader reading
  than the implementation supports. `docs/src/contributing/
  semantic-accessibility.md` now states the position: snora integrates an
  accessibility tree when iced exposes one, and will not build an interim
  abstraction of its own — recorded in `design-decisions.md` with a
  reconsideration trigger (iced exposes an accessibility API). `README.md`
  and `docs/src/getting-started/05-when-to-use.md` bound the "accessible"
  claim at its two overclaiming sites to layout-direction and visual
  accessibility specifically. A new consumer-facing
  `docs/src/guides/accessibility.md` page — linked from the Guides
  section, not buried under `contributing/` — states what snora provides
  and does not, for a reader who does not already know the existing
  `contributing/` documents exist. This bounds a claim; it does not
  retreat from a capability — the contrast-tested presets, four built-in
  token sets, and ABDD layout discipline are unchanged. Documentation
  only: no code, API, or gate-row change.

## [0.27.0] — 2026-08-04

### Added

- **`snora::design::render(layout, &tokens) -> Element` — token-derived
  engine surfaces (RFC-039).** The dialog and the modal dim are surfaces
  the engine renders itself, not primitives an application builds; RFC-038
  made chrome *colours* follow the emitted theme but deliberately left
  these untouched. `snora::design::render` is a sibling to `snora::render`,
  sharing one z-stack implementation (layer order, conditions, backdrop
  wiring written exactly once) parameterized by style — with `design`
  inactive, `snora::render`'s output is byte-for-byte unchanged, proven by
  `render_semantics` passing without modification. The dialog gets a real
  card: fill `surface_raised`, border `border`, radius `radius.lg`,
  padding `spacing.lg` — reusing `snora::design::style::container::
  card_raised` (RFC-029) directly, with its drop shadow zeroed out, since
  a border-defined card (not a shadow-defined one) is what actually works
  in the high-contrast presets. Two of the four built-in presets have
  `surface_raised == background` by the token data's own design, so the
  card is visible there *only* because of its border — tests assert the
  border's contrast against `background`, not the fill's, since a
  fill-vs-background assertion would be false by construction in half the
  presets. The modal dim replaces the hardcoded `rgba(0, 0, 0, 0.4)` with
  a color chosen from `background`'s own darkness (`iced::theme::palette::
  is_dark`) rather than a fixed pole — black-on-black in a dark preset was
  close to invisible before, the same class of defect RFC-038's
  `shift_away_from` was built to prevent for derived theme tiers, here for
  a fixed constant instead of a derived one. Neither derivation extends
  `snora-design`'s frozen token surface (RFC-036); both work within it, per
  the owner-confirmed recommendation over adding a scrim or elevation
  role. Purely additive and feature-gated behind `design`; nothing calls
  it automatically. No file under `crates/snora-core/` or
  `crates/snora-design/` changed.

- **`snora::design::widget::*` — token-derived chrome geometry
  (RFC-040).** RFC-038 made chrome *colours* follow the emitted theme;
  geometry could not, since an `iced::Theme` carries no spacing or
  radius, so the prefab header/sidebar/footer/tab-bar/breadcrumb widgets
  still hardcoded unrelated magic numbers, and `radius: 0.0` was a large
  part of why stock snora chrome read as flat and dated. Styled variants
  (`app_header`, `app_side_bar`, `app_footer`, `app_tab_bar`,
  `app_breadcrumb`) take `&Tokens` first, then the same parameters as
  their `snora::widget::*` counterparts, and map padding/gaps/radii to
  the `Spacing`/`Radius` scales. Every widget's body is extracted into
  exactly one `pub(crate)` builder parameterized by a small geometry
  struct; the unstyled functions pass that struct's `::unstyled()`
  constructor (today's literals, unchanged) and the styled functions
  pass a token-derived one — drift between the two paths is structurally
  impossible, since there is nowhere for a second copy of a widget body
  to live. The mapping is deliberate, not reverse-engineered to match
  today's numbers: each value maps to the `Spacing`/`Radius` token whose
  own documented semantic fits it, several values landing on today's
  exact literal because the original numbers already loosely followed
  this scale (stated per value in `docs/src/design/chrome-geometry.md`,
  not hidden behind it), others deliberately not reproducing today's
  number where the semantic mapping calls for a different token (e.g.
  the sidebar's icon-button gap moves from 16 to `Spacing::md`, 12).
  Header, footer, and the tab bar's own border move from the hardcoded
  `radius: 0.0` to a shared `Radius::sm` — the specific defect this RFC
  names directly. Where a literal has no clean `Spacing`/`Radius`
  equivalent (several border *widths*, and the tab bar's structural
  zero vertical padding), it stays a literal, documented as such rather
  than forcing an invented token match. `Density::Compact` has no
  resolved `Spacing` scale in `snora-design` as of this release (only
  `comfortable()` exists) — geometry is derived from `tokens.spacing`/
  `tokens.radius` directly rather than branching on `tokens.density`,
  so it is already density-correct in the sense that matters: whatever
  a future compact scale resolves to flows through with no widget-level
  branch to keep in sync, verified with hand-mutated `Tokens` in
  `design/widget/tests.rs` since no built-in preset yet offers two
  distinct `Spacing` values to compare. The design workbench gains a
  chrome-geometry section rendering every widget unstyled next to
  styled for direct visual comparison. Purely additive and
  feature-gated behind `design`; the unstyled `snora::widget::*` set's
  geometry is unchanged, proven by a regression test asserting every
  unstyled builder still receives today's exact literal. No file under
  `crates/snora-core/` or `crates/snora-design/` changed.

### Fixed

- **`runner_os` override corrected; release checklist now verifies row
  contents (RFC-044).** RFC-043's `env: RUNNER_OS: ubuntu-latest` step
  override never took effect: GitHub Actions reserves the entire
  `RUNNER_*` variable namespace and silently ignores any attempt to
  overwrite it. Both the 0.25.3 and 0.26.0 rows in `binary-size.csv` and
  `compile-time.csv` read `Linux`, not `ubuntu-latest` — the workflow file
  looked correct on inspection and two review passes confirmed it; only
  the emitted row revealed the defect, and no post-fix row existed until
  the 0.26.0 tag. Fixed by routing the value through `SNORA_RUNNER_OS`
  (outside the reserved namespace) in both `binary-size.yaml` and
  `build-cost.yaml`, consumed by both measurement scripts as
  `${SNORA_RUNNER_OS:-${RUNNER_OS:-unknown}}`. Neither historical `Linux`
  row is edited or back-filled; both budget docs now record the
  discontinuity. Separately, the release checklist previously verified
  only that a new CSV row *exists* after a tag — exactly the gap that let
  this defect, and RFC-043's original `widgets_diff_bytes = 0` defect,
  both ship unnoticed. It now also checks row *contents*: `runner_os`
  must read `ubuntu-latest`, `widgets_diff_bytes` must be non-zero, and
  `check_workspace_ms` must be plausibly cold (≥ 10,000 ms). This
  criterion is not closed by this change alone — it closes only when the
  next tagged release's row is confirmed to read `ubuntu-latest`. No
  crate source, public API, or feature flag changed; no CSV row edited,
  deleted, or back-filled.

## [0.26.0] — 2026-08-03

### Added

- **`snora::design::theme(&Tokens) -> iced::Theme` (RFC-038).** Derives a
  complete iced theme from a Snora Design token bundle, so stock iced
  widgets (`text_input`, `pick_list`, `scrollable`, …) and the window
  background follow the same palette as snora's design primitives instead
  of needing a second, separately maintained `iced::Theme`. Built with a
  custom `Theme::custom_with_fn` generator that constructs every `Pair` as
  a struct literal from a verified token role — never via iced's
  `Pair::new`, which would silently replace contrast-tested colors with a
  heuristic approximation. Every set's `base` tier equals its source token
  role exactly; the `weak`/`strong` tiers and `Background`'s seven
  non-base tiers are derived by deterministic, contrast-verified
  transforms rather than collapsed onto `base` — an earlier revision made
  every tier identical, which silently removed hover/pressed feedback from
  every stock iced button, since `button::primary` reads `primary.base` at
  rest and `primary.strong` on hover. `secondary` derives from
  `surface`/`text_primary` (iced's own neutral-chrome shape for that set),
  not from `info`/`info_text` as originally proposed — iced derives
  `Secondary` from background+text as a neutral set, not a semantic
  accent, so the `info` mapping would have rendered every stock secondary
  control in an unintended hue. `Background.strong` — the one tier iced's
  stock widgets read as a border/separator color, not a text background —
  is derived by growing its shift amount until it clears a `1.5:1`
  contrast floor against `Background.base` itself, rather than a fixed
  amount: an earlier revision used a fixed amount keyed off the paired
  text's darkness, which produced borders darker than an already
  near-black background in the `dark` preset (invisible), and would have
  stayed invisible for any preset whose background sits at a luminance
  extreme even with a fixed amount keyed the other way, since the same
  OKLCH-lightness delta yields a far smaller WCAG contrast gain near a
  luminance extreme than anywhere else on the scale. Fidelity tests assert
  every `base` color equals its source token role exactly, for all four
  presets; determinism tests independently recompute every derived tier;
  contrast tests assert every emitted pair (base and derived) meets WCAG
  AA against its own paired text (AAA where the underlying tokens already
  do); an adjacent-surface test asserts `Background.strong` clears its
  contrast floor against `Background.base`; distinctness tests assert
  `base`/`weak`/`strong` are pairwise distinct within every semantic set.
  Purely additive and feature-gated behind `design`; nothing calls it
  automatically, so applications that don't opt in see no change. The full
  mapping is documented in `docs/src/design/theme.md`. No file under
  `crates/snora-core/` or `crates/snora-design/` changed; RFC-036's
  additive-only covenant is unaffected.

### Changed

- **Design-system boundary extended to snora's own rendered surfaces, and
  DEC-02 amended to permit theme emission (RFC-037).** RFC-020's boundary
  ("Snora positions and stacks; Snora Design styles what applications
  build") covered only primitives applications construct themselves — it
  never contemplated snora styling its *own* chrome, overlays, and
  notification surfaces, which is the larger part of what users actually
  see. `docs/src/design/overview.md` now carries the amended boundary
  statement, discharging RFC-020's own acceptance criterion ("boundary
  statement is added to docs") that was never satisfied at the time.
  Separately, DEC-02 ("theme-aware, not theme-owning") is split:
  **theme-owning** — a parallel theming abstraction, snora holding theme
  state, applications configuring appearance through snora instead of
  iced — **remains permanently declined**; **theme-producing** — a pure
  `Tokens -> iced::Theme` function the application calls, owns, and hands
  to iced itself, with snora holding no state — is now **Accepted**,
  which is what RFC-038's `snora::design::theme` (above) relies on.
  `docs/src/contributing/feedback-and-scope.md` and `README.md` are
  updated to match. Documentation and governance only: no source file
  changed, and the gating invariant — with `design` inactive, snora's
  rendered output is unchanged from v0.25 — is stated explicitly as a
  compatibility promise. Surface coverage remains incremental: as of
  v0.26.0 only chrome colors follow the emitted theme; overlay styling
  and layout geometry are not yet token-derived (tracked by RFC-039/040).

### Fixed

- **Binary-size and build-cost measurement methodology corrected
  (RFC-043).** The 0.25.3 first-real-data-point (RFC-041) exposed that the
  instrument itself was never calibrated: `widgets_diff_bytes` measured
  `0` because all three size-probe crates were byte-identical and never
  *called* `snora::widget::*`/`snora::design::*` — Rust's linker strips
  compiled-but-unused code, so the diff measured the marginal cost of
  compiling a feature in, not of adopting it. `size_probe_widgets` now
  wires `app_header`/`app_side_bar` into its layout; `size_probe_design`
  additionally calls `design::button::primary` and
  `design::style::container::card_surface`. Local remeasurement:
  `widgets_diff_bytes` 0 → 43,520; `design_diff_bytes` 128 → 2,560 — both
  now non-trivial. Separately, `build-cost.yaml` measured "cold" compile
  time with a restored dependency cache in place, so only snora's own four
  crates were ever actually rebuilt — the 56 s → 5.5 s change between
  0.19.1 and 0.25.3 was this caching artifact, not a real trend; the cache
  step is now removed from that workflow, and CI compile-time runs will
  take minutes rather than under a minute going forward, correctly.
  `runner_os` is stabilized to `ubuntu-latest` in both workflows (explicit
  `RUNNER_OS` override; GitHub's auto-populated value is `Linux`, which
  the 0.25.3 rows alone carry). Both budget docs annotate 0.25.3 as
  simultaneously the first row from the fixed tag-automation and the last
  row from the broken methodology — not comparable to what follows. No
  CSV row deleted, edited, or back-filled; **gate 9 remains ⬜** and
  re-satisfies only once ≥2 real rows exist under this corrected
  methodology. `actions/checkout` bumped `v4` → `v6` in
  `binary-size.yaml`, `build-cost.yaml`, and `unpinned-build.yaml`,
  matching `ci.yaml`/`docs.yaml`. No crate source, public API, or feature
  flag change.

## [0.25.3] — 2026-08-03

### Fixed

- **Documentation consistency and source-of-truth audit (RFC-035).**
  Corrected in-tree documentation that still described a three-crate
  architecture five releases after `snora-design` shipped: both
  architecture pages, the README feature list, the `snora` crate module
  doc, and the design-decision register now name all four crates
  (`snora-core`, `snora-design`, `snora-widgets`, `snora`) and the
  correct dependency direction. Fixed a contributor procedure that
  pointed at the wrong crate for new prefab widgets and added a
  matching procedure for design primitives. Brought the release
  checklist's workflow table, CI job names, and gate list in line with
  what CI actually runs. Fixed a structurally broken decision register
  (orphaned section heading, misplaced link definitions) and corrected
  a stale claim that iced's palette has no `warning` pair (it does, as
  of iced 0.14 — see `design-decisions.md`). Updated the API freeze
  review header and D-gate spans to the current version. Fixed the RFC
  index table rendering and added a note-on-first-use for `archive/`.
  No public API, feature flag, or runtime behavior changed.
  **F-6 (`Cargo.lock` policy) was withdrawn**, not fixed: the finding
  asserted the lockfile was tracked while `.gitignore` and the
  architecture page denied it. Verification showed it was *not* tracked
  (deliberately removed in `b7af344`), so both of those were already
  correct and needed no change. The underlying question was taken up
  separately and resolved in this same release — see RFC-042 under
  **Changed**, which now tracks the lockfile for measurement and MSRV
  reasons.

- **Measurement workflow tag-pattern mismatch fixed; 1.0 gate 9 reopened
  (RFC-041).** Both `binary-size.yaml` and `build-cost.yaml` triggered and
  gated their append/commit-back steps on `refs/tags/v*`, while all 38 of
  the project's release tags carry no `v` prefix — so the measurement
  automation has never executed on a release tag in the project's history.
  `binary-size.csv` held three rows in which every measurement column was
  `N/A`; `compile-time.csv` held two, one on a non-CI sandbox runner. Fixed
  both workflows' trigger filters and version-extraction/step-guard logic
  to accept the project's real tag shape (and a `v`-prefixed shape, for
  future-proofing). Replaced two unfalsifiable release-checklist items
  ("confirm the workflow succeeded") with falsifiable ones (confirm a row
  for the new version actually exists in the CSV on `main`). Corrected 1.0
  gate 9 and the two related release-hygiene rows in `api-freeze-review.md`
  from a false "satisfied" to reopened (⬜); the honest core-gate count is
  **7 of 10**, not 8. Annotated both budget docs: pre-0.25.3 rows predate
  the fix, are not comparable to post-0.25.3 rows (0.25.2's
  `resolver = "2"` → `"3"` change plus no committed `Cargo.lock` means
  dependency resolution isn't pinned across the boundary), and the
  `compile-time.csv` `0.17.0` row is flagged suspect (duplicate value
  across two different metrics). No CSV row deleted, edited, or
  back-filled. **The MSRV portion of RFC-041 was corrected and completed
  in round 2**: the RFC's original value (`rust-version = "1.85"`) was
  verified false against the resolved dependency graph and escalated
  rather than shipped; the owner-confirmed corrected value, `rust-version
  = "1.88"` (`iced`/`wgpu` require it; `cargo +1.87 check` fails,
  `cargo +1.88 check` passes), is now declared in `[workspace.package]`
  and inherited by all four crates. `docs/src/getting-started/01-install.md`
  corrected to state the real floor. The bump policy (inherited rise =
  patch, chosen rise = minor) is recorded in `versioning-policy.md`. No
  source, public API, or feature-flag change.

### Changed

- **Design surface freeze review; design gates D-3 and D-4 closed (RFC-036).**
  Recorded a freeze review spanning six consecutive minors (v0.20 → v0.25)
  showing the Snora Design token model and iced style bridge stable except
  for two deliberate SemVer-hardening changes (`Palette::roles()` narrowed
  to test-only; `composite_over` gained a debug-only precondition) that
  altered neither the token model nor the style bridge. Closes D-3 and D-4
  in `api-freeze-review.md`. Establishes the **additive-only covenant** in
  `api-governance.md`, which itemizes the frozen design surface and
  constrains what may change in it without reopening the gate — the
  prerequisite for the v0.26 appearance milestone (RFC-037…040) to proceed
  without hollowing out the gates it just closed. No source, public API,
  or feature-flag change.

- **`Cargo.lock` is now tracked (RFC-042).** Reverses `b7af344` ("remove
  Cargo.lock from vcs"), a deliberate prior decision, because the
  workspace is also a measurement harness: with no lockfile, CI resolved
  dependencies fresh on every run, so binary-size/build-cost deltas mixed
  snora's own changes with whatever upstream published in between, and the
  MSRV floor could move unobserved (as it did — RFC-041 found the
  documented 1.85 was actually 1.88). A committed lockfile fixes both.
  **The cost accepted:** CI no longer notices on its own that a fresh
  resolution would have broken; a new scheduled `unpinned-build` workflow
  (weekly + manual dispatch) replaces that lost signal by re-resolving in
  a throwaway checkout, checking the workspace and the declared MSRV still
  hold, and failing loudly — it never commits or pushes the updated
  lockfile. **Downstream consumers are unaffected in either direction**: a
  library's committed lockfile is ignored by Cargo when the library is
  used as a dependency. No dependency range in any `Cargo.toml` changed.

## [0.25.2] — 2026-06-21

### Changed

- **Workspace feature resolver moved from `"2"` to `"3"`.** Switches the
  workspace to Cargo's MSRV-aware feature resolver.
- **`[workspace].members` changed from an explicit 21-entry list to the
  globs `crates/*` and `examples/*`.** New crates and example directories
  now join the workspace automatically instead of requiring a manual
  `Cargo.toml` edit.
- User-facing version snippets updated to `0.25` in `README.md`,
  `docs/src/design/feature-flags.md`, and `docs/src/design/overview.md`.

**No file under `crates/*/src/` changed in this release** (verified via
`git diff 0.25.1 0.25.2 -- crates/`, which is empty). The published crates
are functionally identical to 0.25.1 for downstream consumers; the
`resolver` key affects only this workspace's own builds, not consumers'
dependency resolution.

## [0.25.1] — 2026-06-20

### Added

- **`snora::design::contrast` re-export (facade completeness fix).**
  `snora::design::contrast::{relative_luminance, contrast_ratio,
  composite_over}` now resolves when the `design` feature is enabled.
  Previously the `contrast` module was the only public `snora-design` module
  not reachable through the `snora::design` facade, forcing downstream apps
  that wanted contrast utilities to add a direct `snora-design` dependency.
  No new capability is introduced — the functions are already public in
  `snora-design`; this only adds the intended facade path.

- **Smoke test `design_facade_tests::contrast_ratio_black_white_via_facade`.**
  Verifies the facade path resolves and that `contrast_ratio(black, white)`
  returns the WCAG 2.1 value of ~21.0. Guards the re-export against
  accidental future removal.

- **`snora::design` module doc updated** to list `contrast` alongside
  `style`, `button`, `card`, `notice`, `chip`, and `progress` in the
  "Exposes:" section.

## [0.25.0] — 2026-06-20

### Fixed

- **RFC-031 missing from `rfcs/README.md` Done table (M-1).** The row for
  RFC-031 was absent despite the file existing in `rfcs/done/`. Added between
  RFC-030 and RFC-032.

- **Binary-size measurement methodology (M-2).** Replaced the previous
  approach (comparing `snora-example-hello` against
  `snora-example-design-workbench`) with three identical size-probe crates:
  `snora-size-probe-engine`, `snora-size-probe-widgets`,
  `snora-size-probe-design`. All probes contain identical application code;
  diffs now measure the marginal cost of each feature in isolation.
  `binary-size.csv` schema updated; pre-v0.25 rows carry `N/A`.

- **Build-cost measurement not cold for design (M-3).** `snora-design`
  is now included in the `cargo clean` list in `measure-compile-time.sh`.
  `snora-example-design-workbench` is also cleaned before its measurement.

- **Measurement docs inconsistent with implementation (M-4).** Updated
  `binary-size-budget.md` (column table, methodology description),
  `build-cost-budget.md` (six columns documented), `scripts/README.md`
  (all three scripts described with new probe approach), and
  `append-binary-size-row.sh` (comments and example row corrected).

- **`examples/README.md` invalid workbench command (M-5).** Removed
  the invalid `--features design` flag from the design-workbench run
  command. The example's `Cargo.toml` already includes the `design`
  feature; passing it at the CLI is a no-op at best, an error at worst.

- **ROADMAP stale design-track section (M-6).** Rewrote the Snora Design
  System section with accurate per-version history. Removed stale
  "remain in `rfcs/proposed/`" sentence.

### Changed

- **`snora/Cargo.toml` `design` feature comment.** Updated from
  "Opt-in until binary-size/build-cost are measured (roadmap Option B: v0.20)"
  to accurately describe the policy: opt-in by deliberate decision;
  default-on requires an explicit review and RFC.

- **CHANGELOG reference links** updated from `v0.10.0...HEAD` through
  the full release history to v0.24.0.

- **`docs/src/design/chips.md`** updated to describe the solid accent
  background + `accent_text` foreground selected state (was "tinted accent
  background").

- **Design workbench banner** text updated from "Visual-fit QA for Snora
  Design v0.20" to "Visual-fit QA for Snora Design".

- **`feature-gating-criteria.md`** current status updated to v0.25.

- **`api-freeze-review.md`** D-3/D-4 rationale clarified: vocabulary and
  style bridge stable across five consecutive minors (v0.20–v0.24); gates
  close after a dedicated design-freeze review at or after v0.25.

### Added

- **Three size-probe example crates** in `examples/size_probe_engine/`,
  `examples/size_probe_widgets/`, `examples/size_probe_design/`. All contain
  identical minimal app code; only the `snora` feature set differs.
  Used by `scripts/measure-binary-size.sh` to produce valid marginal-cost
  measurements.

## [0.24.0] — 2026-06-20

### Breaking changes

- **`Palette::roles()` is now `#[cfg(test)] pub(crate)`** (was `pub`).
  The method returned `[Color; 18]`, which locks the role count into the
  public API, conflicting with `Palette` being `#[non_exhaustive]`. The
  method is now test-only and crate-internal. Access palette roles directly
  via fields (`palette.text_primary`, `palette.accent`, etc.).

### Fixed

- **Chip selected contrast (M-4, accessibility bug).** `chip::filter` and
  `chip::removable` selected-state hover/pressed colors failed WCAG AA
  (4.5:1) in light and dark presets. Replaced the semi-transparent accent
  tint (α=0.15–0.30) with a solid `accent` background + `accent_text`
  foreground. Measured contrast ≥6.7:1 across all four built-in presets.
  New tests: `chip_selected_text_over_accent_background_meets_aa_all_presets`
  and `chip_selected_text_hover_pressed_meets_aa_all_presets`.

- **RFC README lifecycle inconsistency (M-1).** `rfcs/README.md` `##
  Proposed` section listed RFC-031 (already in `rfcs/done/`). Section now
  correctly reads `_(none)_`.

- **Stale version snippets (M-3).** `README.md` quick-start (`0.10` →
  `0.24`); `crates/snora/src/lib.rs` engine-only doc (`0.18` → `0.24`);
  `docs/src/design/overview.md` and `feature-flags.md` (`0.19` → `0.24`).

- **`composite_over` debug assertion (N-2).** Added
  `debug_assert!(bg.is_opaque())` with an explanatory message.

- **`release-process.md` stale note (S-5).** Removed "flip `publish =
  false` at release time" note; `snora-design` has been published since v0.20.

- **Script comments (S-6).** `measure-binary-size.sh` and
  `append-binary-size-row.sh` comments updated to document the 9-field
  schema (`version,widgets_on,widgets_off,diff,design_on,design_diff,rustc,runner_os,date`).

- **Relative links (S-2).** `.github/pull_request_template.md` links
  corrected to `../docs/src/...`; `migration-0.21-to-0.22.md` design doc
  links corrected to `../design/...`.

- **"35 design-track RFCs" → "15" (S-3).** Fixed in ROADMAP and
  `migration-0.22-to-0.23.md`.

### Added

- **Binary-size and build-cost measurement for `design` feature (M-2).**
  `measure-binary-size.sh` now measures three configurations: widgets ON,
  widgets OFF, and `widgets+design` via `snora-example-design-workbench`.
  New columns: `design_on_bytes`, `design_diff_bytes` in `binary-size.csv`;
  `build_widgets_design_ms`, `example_workbench_ms` in `compile-time.csv`.
  Existing rows backfilled with `N/A`. CI job summaries updated. First real
  design measurements will appear after the next CI tag.

- **Design workbench in examples acceptance matrix (M-6).** Added
  `snora-example-design-workbench` to `examples/README.md` with a dedicated
  manual QA checklist.

### Changed

- **SUMMARY.md navigation (S-1).** Recipes moved under `# Snora Design`
  section (were under a separate `## Recipes` heading after Contributing).
  API governance remains under Contributing.

- **`api-freeze-review.md` updated (M-7).** Header reflects v0.24.0 status
  (eight of ten core gates). D-8 (`snora-design` published) marked ✅ v0.20.
  D-3/D-4 rationale updated: vocabulary and style bridge have been stable
  v0.20–v0.24.

- **README Snora Design mention (S-4).** "Skeleton, not styling" updated to
  describe the opt-in Snora Design layer.

- **`notice.rs` and `chip.rs` (S-7).** Module comments now explicitly
  document the `"×"` accessible-label limitation and note it as a future
  customization point.

- **Compile-only test helpers annotated (S-8).** `_notice_compiles_...`
  and `_progress_compiles_...` now carry `#[allow(dead_code)]` with
  explanatory comments.

## [0.23.0] — 2026-06-20

### Added

- **Four initial recipes in `docs/src/design/recipes/` (RFC-033).** Each
  follows the nine-section format (Purpose, When to use, When not to use,
  Data the app owns, Snora primitives used, Accessibility notes, Code
  example, Customization points, Promotion status):
  - **Empty state** — placeholder card with optional CTA. Uses
    `card::surface`, `button::primary`, text size helpers.
  - **Background task card** — progress bar + optional cancel/pause.
    Uses `card::surface`, `progress::row`, `button::ghost`.
  - **Friendly error recovery notice** — inline error with recovery action.
    Uses `notice::Notice` with `Tone::Danger` or `Tone::Warning`.
  - **Result card** — selectable list-item card with metadata chips.
    Uses `card::surface`/`selected`, `chip::filter`. Documents the
    outer-`button`-wrapping-a-card pattern for keyboard reachability.

- **Recipe index** at `docs/src/design/recipes/README.md` with promotion
  status table. All four recipes are status **Recipe**.

- **`contributing/recipes.md` updated** — candidate table replaced with
  links to published recipes; directory listing updated to match.

- **RFC-033 and RFC-034 closed** — both moved to `rfcs/done/` (Status:
  Implemented v0.23.0). RFC-034 (API governance) was already fully
  implemented in `docs/src/contributing/api-governance.md`; this release
  formally records its completion.

### Design API changes

```
New APIs:        none
Experimental:    none
Promoted:        none
Deprecated:      none
Removed:         none
Recipes added:   empty-state, background-task, error-recovery, result-card
Recipes promoted: none
Scope concerns:  none
```

## [0.22.0] — 2026-06-20

### Changed

- **`chip::removable` — removed duplicate `style_fn_rm` variable.** The
  second closure reference to the style function was an unnecessary copy of
  the first. Both now share the same `style_fn` function pointer. No
  behaviour change.

- **`chip` — hover/pressed states use `darken` helper.** The inline
  per-channel arithmetic (`c.r = (c.r - 0.04).max(0.0)`) has been replaced
  with a private `darken(color, amount)` helper consistent with the pattern
  in `style/button.rs`. No behaviour change.

- **Stale version reference removed.** `snora::design::card` doc comment
  said "Cards in v0.20 are non-interactive"; the version qualifier removed.

- **`v021-primitives.md` updated** from a planning document ("Proposed API")
  to a shipped-primitives reference page linking out to the new dedicated
  pages.

### Added

- **Test coverage for `chip` style functions.** 4 new unit tests in
  `design::chip::tests`: all `button::Status` variants across all four
  token presets for both selected and unselected styles; `darken` clamping.

- **Compile-time tests for `notice` and `progress`.** `notice::tests` covers
  all `Tone` variants, all preset tokens, and all builder combinations.
  `progress::tests` covers all variants and includes a `value_clamps_within_range`
  runtime assertion.

- **Three new design doc pages** in `docs/src/design/`:
  `notices.md`, `chips.md`, `progress.md` — each covering usage, accessibility
  (RFC-027 five questions), and visual fit. `v021-primitives.md` renamed to
  serve as a cross-reference overview.

## [0.21.0] — 2026-06-20

### Added

- **Notice primitive (`snora::design::notice::Notice`, RFC-032).** Builder-style
  notice banner: tone (`Tone::Info/Success/Warning/Danger/Accent`), optional
  title, body, optional action button, optional dismiss button. Action and
  dismiss controls are `iced::widget::button` — keyboard-reachable. Tone
  colors use palette status roles verified by the automated contrast tests.

- **Chip primitives (`snora::design::chip`, RFC-032).** Two functions:
  `filter` (toggle chip, tinted accent background when selected) and
  `removable` (chip label + separate × button). Both backed by
  `iced::widget::button`.

- **Progress primitives (`snora::design::progress`, RFC-032).** Two layout
  variants: `row` (compact inline) and `card` (wrapped in `card::surface`).
  Backed by `iced::widget::progress_bar`. `None` value = indeterminate,
  rendered as 0% with "…" suffix (iced 0.14 has no native indeterminate
  animation — documented limitation, not a regression). `Tone` parameter
  colors the bar via `style::progress::toned`.

- **`style::progress::toned` style function.** Maps `Tone` to a
  `progress_bar::Style` using token palette roles.

- **Design workbench updated.** Notice, chip, and progress sections added,
  exercising all tone variants and indeterminate state.

- **RFC-032 closed** — moved to `rfcs/done/` (Status: Implemented v0.21.0).

## [0.20.0] — 2026-06-20

### Added

- **`snora-design` published to crates.io (RFC-031).** The iced-free design
  token crate introduced as groundwork in v0.19 is now a first-class published
  dependency. The v0.20 release satisfies all RFC-031 non-deferrable criteria:
  iced-free `snora-design`, high-contrast presets, automated contrast tests,
  iced style bridge, pilot button/card helpers, accessibility checklist,
  semantic construction policy, and boundary docs. `design` remains opt-in
  (`default = ["widgets"]`); binary-size/build-cost measurement with and
  without `design` is still pending — the current scripts and CSVs measure
  `widgets_on/off` only, not `design_on/off`. Measurement columns for the
  design feature will be added before any decision to make `design` default-on.

- **RFC-031 closed** — moved to `rfcs/done/` (Status: Implemented v0.20.0).

### Changed

- `docs/src/contributing/release-process.md`: removed stale "flip
  publish=false first" note for `snora-design` (already done in v0.19.1).

## [0.19.1] — 2026-06-20

### Fixed

- **`scripts/measure-compile-time.sh` — missing space caused CI failure.**
  Line 43 read `measure_ms"build_engine_only"` (no space), which the shell
  parsed as a call to the non-existent command `measure_msbuild_engine_only`,
  causing `build-cost.yaml` to fail with exit code 127. Single character fix:
  added the space before the argument.

- **`binary-size.csv` schema corrected.** The header declared 6 columns
  (`lto` as the fifth), but the v0.17.0 row had always been 7 columns
  (`rustc` and `runner_os` in positions 5–6). Fixed header, updated
  `measure-binary-size.sh` to emit 7 fields (replacing the unused `lto`
  argument with `rustc` and `runner_os`), updated
  `append-binary-size-row.sh` field-count validation from 6 to 7, and
  updated `binary-size.yaml` job summary parsing accordingly.
  `binary-size-budget.md` column table updated to match.

### Added

- **Gate 9 fully satisfied — build-cost data point (v0.19.1).** Appended
  the CI measurement from the v0.19.1 run to `compile-time.csv`
  (check_workspace 56,150 ms; build_widgets 96,000 ms; build_engine_only
  330 ms; example_hello 153,000 ms; ubuntu-latest). Combined with the
  v0.17.0 sandbox row, Gate 9 now has ≥2 data points in both budget CSVs.
  Remaining 1.0 blockers: gate 1 (iced major upgrade) and gate 3
  (confirmed third-party production app).

- **Binary-size Gate 9 data point (v0.19.1).** Appended a second real CI
  row to `binary-size.csv`: 15,813,712 bytes stripped, diff = 0
  (ubuntu-latest).

## [0.19.0] — 2026-06-20

### Added

- **v0.21 primitives design doc (RFC-032).** New
  `docs/src/design/v021-primitives.md` documents the planned notice, filter
  chip, and progress primitives — proposed API, internal model, events, and
  per-primitive accessibility requirements. These primitives are **not
  implemented** in v0.19; they are listed to make the design visible before
  the v0.21 implementation cycle.

- **Recipes and dogfood process (RFC-033).** New
  `docs/src/contributing/recipes.md` defines the nine-section recipe format,
  the candidate recipe catalog (result card, empty state, error recovery
  notice, etc.), the dogfood validation requirement that guards promotion, and
  the feedback template for downstream applications.

- **API governance (RFC-034).** New
  `docs/src/contributing/api-governance.md` defines the five API states
  (recipe, experimental, stable, deprecated, removed), the six-condition
  promotion criteria, the twelve-item stable-API review checklist, the
  deprecation policy, the per-release review template, and the eight
  Snora Design 1.0 gates (D-1 through D-8). Gates D-1 through D-8 appended
  to `docs/src/contributing/api-freeze-review.md` alongside the core gates.

- **Design workbench example (RFC-030).** New `snora-example-design-workbench`
  crate exercises all four token presets (light / dark / HC light / HC dark),
  all button variants (enabled + disabled), all card variants, the full
  typography scale, and palette swatches. Preset is stored in app state to
  avoid lifetime friction. Serves as the visual-fit QA surface for RFC-027's
  accessibility checklist.

- **Snora Design documentation section (RFC-030).** Seven new pages under
  `docs/src/design/`: overview, feature flags, tokens, high contrast, buttons,
  cards, and the iced style bridge. Covers minimal / default / design usage
  paths and documents the iced 0.14 focus-ring limitation prominently.

- **Pilot button helpers (RFC-028)** in `snora_widgets::design::button` (and
  `snora::design::button` at the facade). Eight functions: `primary`,
  `secondary`, `ghost`, `danger` (take `on_press: Message`) and their
  `*_maybe` variants (take `Option<Message>`, disabled when `None`). All four
  wrap `iced::widget::button`; tokens are cloned once into the style closure
  so callers are lifetime-free. Focus rings absent in iced 0.14 —
  documented limitation.

- **Pilot card helpers (RFC-029)** in `snora_widgets::design::card` (and
  `snora::design::card` at the facade). Three functions: `surface`, `raised`,
  `selected`. All wrap `iced::widget::container` with token-derived padding,
  radius, border, and background. Cards are non-interactive visual grouping
  surfaces in v0.20; application behaviour lives outside the card.

- **New `snora-design` crate (iced-free) — Snora Design token foundation
  (RFC-022 / RFC-023 / RFC-024).** Defines `Color`, a semantic `Palette`
  (18 roles including paired status-text foregrounds `success_text` /
  `warning_text` / `danger_text` / `info_text`), `Spacing`, `Typography` /
  `TextRole`, `Radius`, `FocusTokens`, and the `Tone` / `Emphasis` / `Size` /
  `Density` variant vocabulary, bundled into a `Tokens` struct with four
  built-in presets (`light`, `dark`, `high_contrast_light`,
  `high_contrast_dark`). `Tokens` and `Palette` are `#[non_exhaustive]`. Ships
  a pure-Rust `contrast` module and an automated contrast test suite covering
  all mandatory WCAG AA pairs (including `danger_text on danger`). The crate
  has **no iced dependency** (CI gate Q3; enforced by the new
  `design-isolation` CI job). Groundwork only: not yet wired into the `snora`
  facade; `publish = false`; activation targets v0.20.

- **`snora-widgets` `design` feature — iced style bridge (RFC-025).** Adds an
  opt-in `design` feature to `snora-widgets` and the root `snora` crate.
  When enabled, exposes `snora_widgets::design::style` with:
  `color::to_iced_color` (explicit boundary function); four semantic button
  styles (`primary`, `secondary`, `ghost`, `danger`) covering iced 0.14's
  `Active / Hovered / Pressed / Disabled` statuses; three card/container styles
  (`card_surface`, `card_raised`, `card_selected`); six typography-size helpers.
  Root `snora::design` re-exports the full token vocabulary and the style
  sub-modules (enumerated, not glob). 12 style-bridge unit tests; all feature
  isolation checks pass. **iced 0.14 note:** `button::Status` has no
  `Focused` variant; custom focus rings on standard buttons/cards are not
  deliverable in v0.20 — documented limitation, not a regression.

- **CI quality gates for the design feature (RFC-026).** Extended `ci.yaml`:
  three new design feature-matrix entries (`widgets,design`;
  `widgets,design,lucide-icons`; `widgets,design,svg-icons`) in the existing
  `feature-matrix` job; new `design-isolation` job enforcing Q2-B (widgets
  compiles without design) and Q3 (no iced in `snora-design`); `rust-quality`
  job now runs `cargo test -p snora-design` on every PR (Q4 token sanity +
  Q5 mandatory contrast).

- **Accessibility checklist and semantic construction policy (RFC-027).**
  Two new contributing docs: `accessibility-checklist.md` (required review
  gate for every Snora Design primitive, covering contrast, high-contrast, focus
  visibility, keyboard reachability, semantic construction, pointer target size,
  typography, directionality, reduced motion, disabled states, loading/error
  states, and plain-language wording); `semantic-accessibility.md` (core
  "prefer native iced controls" rule, primitive construction table, the five
  required RFC/PR questions, the iced 0.14 focus-state limitation stated
  formally, and the keyboard ownership table). Both pages are indexed in
  `SUMMARY.md` and `contributing/README.md`.

- **Snora Design System RFCs (RFC-020 … RFC-034)** under `rfcs/proposed/`,
  with per-RFC and global implementation-handoff material in
  `design-system-handoff/`.

### Changed

- Opened the **0.19.0** development line (`0.18.3` was published; workspace
  version and inter-crate pins bumped `0.18` → `0.19`).
- `rfcs/README.md`: documented flat sequential RFC numbering from RFC-020
  onward; indexed RFC-020 … RFC-034 in the Proposed section.
- `docs/src/contributing/release-process.md`: updated publish order to
  `snora-core → snora-design → snora-widgets → snora` (RFC-031); added
  `snora-design` to the `cargo package` checklist with a note to flip
  `publish = false` at v0.20.
- `docs/src/SUMMARY.md`: added Snora Design section with seven doc pages.
- `docs/book.toml`: removed deprecated `multilingual = false` key and the
  `git-repository-icon` that referenced a non-existent font.
- `docs/src/reference/vocabulary.md`, `reference/widgets.md`,
  `contributing/anchored-popover-design.md`: converted `rust,no_run` type-
  signature fences to `rust,ignore` per RFC-012-D policy; `mdbook test docs`
  now passes cleanly.

## [0.18.3] — 2026-06-17

- Re-export `lucide_icons::LUCIDE_FONT_BYTES`.

## [0.18.2] — 2026-06-10

### Fixed

- **`keyboard.rs` doc example used `iced::keyboard::on_key_press`**, which
  does not exist in iced 0.14. Example updated to the correct
  `iced::keyboard::listen().map(...)` pattern matching the workbench and
  starter examples.

- **`snora/src/lib.rs` engine-only doc snippet** showed version `"0.6"`.
  Updated to `"0.18"`.

- **`layout.rs` doc fences** used bare `ignore` instead of `rust,ignore`,
  violating the RFC-012-D documentation test policy. Both fences corrected.

- **`guides/overlays.md` z-stack** collapsed layers 2 and 3 into one
  entry, misrepresenting the 8-layer stack. Expanded to match `render.rs`
  and `overlay-interaction-semantics.md` exactly.

- **User-facing version snippets** in `install.md` and `icons.md` still
  showed `"0.17"` after the v0.18.1 patch. Updated to `"0.18"`.

- **`render_semantics.rs`** `#[allow(dead_code)]` comment was stale
  ("reserved for v0.12 expansion"); removed since all `Msg` variants are
  actively used.

### Added

- **`context_menu` integration test** (`context_menu_content_reachable`).
  Layer 3 of the z-stack was the only layer without render-semantics
  coverage. The new test verifies `context_menu` content is findable and
  interactive. Integration test total: **11**.

- **`Icon::PartialEq` unit tests** (3 unconditional + 2 under
  `svg-icons` feature). `Icon::PartialEq` was added in v0.17.0 without
  corresponding tests. `snora-core` unit test total: **20**.

## [0.18.1] — 2026-06-10

### Fixed

- **`Icon::Lucide` rendering failed to compile** when a downstream app had
  `lucide-icons` enabled and iced's dependency graph contained multiple
  `iced_core` versions. The previous code called `lucide_const.widget()`
  which returns `iced::widget::Text` parameterised against lucide-icons'
  own internal iced_core, causing an unsatisfied `From` trait bound when
  converting `.into()` the snora-widgets element type.

  **Fix:** extract the unicode codepoint via the stable `From<Icon> for char`
  conversion (which has no iced dependency) and construct the `Text` widget
  using snora-widgets' own `iced::widget::text()` call with
  `iced::Font::with_name("lucide")`. This matches the visual output of
  the previous code exactly while using only the snora-widgets iced
  dependency. (RFC-019-A; reported by downstream user nabbisen)

## [0.18.0] — 2026-06-10

### Added

- **Contributing overview** (`docs/src/contributing/README.md`). Grouped
  index of all 13 contributing pages with reading paths for new contributors,
  designers, and maintainers. First item in the Contributing SUMMARY section.
  (RFC-018-C)

### Changed

- **User-facing version snippets** in `docs/src/getting-started/01-install.md`
  and `docs/src/guides/icons.md` updated from `"0.14"` (and two `"0.5"` in
  `icons.md`) to `"0.17"`. The release checklist now includes a version-snippet
  update step. (RFC-018-A)

- **`api-freeze-review.md`** Gate 7 marked ✅ v0.18: all checklist sections
  green, API declared ready for 1.0 pending gates 1, 3, and 9. Seven of ten
  1.0 gates now satisfied. (RFC-018-B)

- **ROADMAP** Gate 7 updated to ✅ v0.18. Post-0.18 section lists the three
  remaining 1.0 blockers. (RFC-018-B)

## [0.17.0] — 2026-06-10

### Added

- **`Icon` now implements `PartialEq`** across all feature combinations.
  Without `lucide-icons`: derived automatically. With `lucide-icons`: a
  manual impl compares `Text` and `Svg` variants by value and `Lucide`
  variants by discriminant (since `lucide_icons::Icon` does not itself
  derive `PartialEq`). (RFC-017-A)

- **Two RTL render-semantics integration tests** in
  `crates/snora/tests/render_semantics.rs`:
  `sheet_end_edge_reachable_under_rtl` and
  `toast_dismiss_reachable_under_rtl`. Total integration tests: 10
  (was 8). Gate 5 now reads "10 tests including 2 RTL." (RFC-017-B)

- **First build-cost data points** recorded in all three budget CSVs:
  `binary-size.csv`, `compile-time.csv`, `render-cost.csv`. Values are
  from the sandbox build environment; CI on real hardware will produce
  representative numbers. Gate 9 infra proven. (RFC-017-D)

### Changed

- **`api-freeze-review.md`** fully updated to reflect v0.17.0 state: six
  of ten 1.0 gates now satisfied (added Gates 2 and 5); type-names audit
  complete; all documentation and release-hygiene rows updated. (RFC-017-E)

- **ROADMAP** 1.0 gate table updated: Gate 2 ✅ (v0.13–v0.16 vocabulary
  stable), Gate 5 ✅ (v0.17, 10 tests including RTL), Gate 9 first point
  noted. (RFC-017-A, RFC-017-B, RFC-017-D)

### Fixed

- **`keyboard.rs` doc comment fence** changed from bare `ignore` to
  `rust,ignore` per the RFC-012-D documentation test policy. (RFC-017-C)

## [0.16.0] — 2026-06-10

### Added

- **Alternate engine boundary doc**
  (`docs/src/contributing/alternate-engine-boundary.md`). Explains why
  `snora-core` is iced-free (vocabulary stability, testability, and
  architectural clarity), what a hypothetical alternate engine would require
  (capability table), what is iced-specific and not portable, and the
  conservative public wording: Snora does not currently promise alternate
  renderer support. (RFC-016-A)

- **Performance envelope reference**
  (`docs/src/reference/performance-envelope.md`). Documents Snora's
  algorithmic performance commitments (all O(n) or O(1)), six reference
  scenarios, and the render-cost CSV at
  `docs/src/reference/performance-envelope/render-cost.csv` (header-only
  until first tag run). (RFC-016-B)

- **`scripts/measure-render-cost.sh`**. Times the release-baseline builds
  of `examples/hello` and `examples/workbench` as a proxy for layout
  composition cost. Mirrors `measure-compile-time.sh` output conventions.
  (RFC-016-B)

- **Downstream feedback issue template**
  (`.github/ISSUE_TEMPLATE/downstream-feedback.yml`). Structured form for
  developers using Snora in real applications. Directly feeds the 1.0
  "third-party adoption" gate. (RFC-016-C)

- **Feature request issue template**
  (`.github/ISSUE_TEMPLATE/feature-request.yml`). Includes scope-triage
  question and pre-submission checklist against the off-the-roadmap
  non-goals. (RFC-016-C)

- **Feedback and scope guide**
  (`docs/src/contributing/feedback-and-scope.md`). Defines the
  framework layer, feature-request triage table, what counts as a
  third-party production app for the 1.0 gate, and the "Snora does not
  grow into a widget library" commitment. (RFC-016-C)

### Changed

- **README** — new "Contributing and feedback" section with links to
  both issue templates and the feedback guide. (RFC-016-C)

- **`docs/src/reference/build-cost-budget.md`** — cross-link to the new
  performance-envelope page. (RFC-016-B)

## [0.15.0] — 2026-06-10

### Added

- **Starter application example** (`examples/starter/`, `snora-example-starter`).
  177 ELOC covering the minimal-but-complete Snora patterns: header menu with
  close sink, dialog with Escape close via `snora::keyboard::dismiss_on_escape`,
  transient toast, LTR/RTL toggle, tab bar, and sidebar. Workspace member;
  compiles in CI. Companion getting-started page added
  (`docs/src/getting-started/07-starter-application.md`). (RFC-015-C)

- **Versioning policy** (`docs/src/contributing/versioning-policy.md`).
  Documents the change-type table, the "Fixed vs Changed" rule (behavior fixes
  that restore documented invariants are **Fixed**), the "at least two minor
  releases" deprecation bridge rule, and the four questions any PR touching
  public API must answer. (RFC-015-A)

- **Migration guide template** (`docs/src/guides/migration-template.md`).
  Seven-section template (`Who is affected`, `What changed`, `Why it changed`,
  `Mechanical migration`, `Behavioral migration`, `Deprecated aliases and
  removal schedule`, `Examples before/after`). (RFC-015-A)

- **Decision index** at the top of `docs/src/contributing/design-decisions.md`.
  Status table covering all 17 current decisions with status labels (*Firm
  boundary*, *Accepted*, *Deferred*) and concrete reconsideration triggers.
  (RFC-015-D)

### Changed

- **`crates/snora/Cargo.toml`** now has `[package.metadata.docs.rs]` with
  `all-features = true` and `rustdoc-args = ["--cfg", "docsrs"]`. docs.rs will
  now build `snora` with all features, making feature-gated items (widgets,
  Lucide constants, keyboard module) visible in the rendered docs. (RFC-015-B)

- **`docs/src/getting-started/01-install.md`** updated: version references
  corrected from `"0.10"` to `"0.14"`; new "Which crate should I depend on?"
  section added. (RFC-015-B)

- **`docs/src/guides/migrations.md`** extended with a link to the migration
  template and the versioning policy. (RFC-015-A)

- **Release checklist** updated: versioning-policy questions and migration
  guide check added. (RFC-015-A)

## [0.14.0] — 2026-06-10

### Added

- **`snora::keyboard::dismiss_on_escape`** — public helper implementing
  the Snora overlay dismissal priority for the `Escape` key: modal before
  menu, `None` when no overlay is open or when the relevant close sink is
  absent. Seven unit tests cover all six cases plus non-Escape keys. The
  workbench example now wires this helper via `iced::keyboard::listen()`.
  (RFC-014-A)

- **`examples/README.md`** — acceptance matrix listing all twelve example
  crates with their purpose, demonstrated surfaces, and the icons-gap note.
  Includes instructions for adding new examples and the workbench manual QA
  reference. (RFC-014-E)

### Changed

- **`crates/snora/src/toast.rs`** — the warning fallback color is now a
  named private const `WARNING_COLOR` with a doc comment explaining why it
  exists and that it is not a public design token. (RFC-014-C)

- **`docs/src/reference/overlay-interaction-semantics.md`** — new
  "Keyboard dismissal" section with the `dismiss_on_escape` usage table
  and a complete code snippet. (RFC-014-A)

- **`docs/src/guides/overlays.md`** — new "Accessibility responsibilities"
  section with the application modal checklist and the ABDD-is-layout
  boundary statement. (RFC-014-B)

- **`docs/src/guides/icons.md`** — new "Why icons are feature-gated" and
  "Supported feature combinations" sections. (RFC-014-D)

- **`docs/src/contributing/feature-gating-criteria.md`** — icon and asset
  feature policy cross-reference section added. (RFC-014-D)

- **`docs/src/contributing/design-decisions.md`** — three new entries:
  theme-aware-not-owning + style review checklist (RFC-014-C); focus-trap
  deferred decision (RFC-014-B). (RFC-014-B, RFC-014-C)

- **Release checklist** updated with examples matrix and workbench QA
  steps. (RFC-014-E)

- **`examples/workbench`** updated: `Escape` key wires overlay dismissal
  via `snora::keyboard::dismiss_on_escape`; `NoOp` message variant handles
  non-key-press keyboard events cleanly. (RFC-014-A)

## [0.13.0] — 2026-06-10

### Added

- **Anchored popover design page**
  (`docs/src/contributing/anchored-popover-design.md`). Records the
  complete design study: eight internal questions answered (iced `operate`
  geometry API, application-provided-only geometry, new `on_close_popovers`
  sink needed, non-modal, single popover, no collision detection, additive
  `AppLayout` field, layer between context_menu and modal dim). Decision:
  defer implementation until a concrete consuming app exists. (RFC-013-A)

- **Public API freeze review**
  (`docs/src/contributing/api-freeze-review.md`). Living readiness
  tracker for 1.0. Four of ten gates are now satisfied (AppLayout
  stability, render-semantics tests, feature-matrix CI, workbench
  example); six remain (iced major upgrade, vocabulary stability,
  third-party adoption, freeze review completion, build-cost data,
  docs.rs polish). (RFC-013-B)

### Changed

- **ROADMAP 1.0 gates** expanded to ten items with current satisfaction
  status. Gates 4, 5, 6, 8, 10 marked ✅; remaining five are real
  blockers. Pointer added to `api-freeze-review.md`. (RFC-013-B)

- **`design-decisions.md`** extended with two deferred-feature records:
  tooltip vocabulary (trigger: second consumer type) and persistent-toast
  helper (trigger: two apps repeat the pattern). (RFC-013-C)

### Notes

- Tooltip vocabulary (`RFC-013-C` Candidate A): trigger not met as of
  v0.13. `SideBarItem.tooltip: String` remains the only consumer. Watch
  for a second consumer type.
- Persistent-toast helper (`RFC-013-C` Candidate B): trigger not met.
  No example calls `.persistent()` yet.

## [0.12.0] — 2026-06-10

### Added

- **Render-semantics test expansion** (RFC-011-D full acceptance).
  Three new integration tests in `crates/snora/tests/render_semantics.rs`:
  menu backdrop dismissal (`outside_click_on_menu_emits_close_menus`),
  dialog+sheet coexistence (`dialog_and_sheet_coexist_sheet_content_reachable`),
  and sheet opaque-wrapper interaction. Engine test suite is now 8 integration
  tests covering all invariants from the RFC-011-D full-acceptance table.
  Five new `toast.rs` unit tests cover RTL `horizontal_align` for all
  Start/End/Center positions — full ABDD regression coverage at the unit level.

- **ABDD compliance checklist** (`docs/src/contributing/abdd-checklist.md`).
  Normative review gate for direction-sensitive changes. Covers scope
  determination, logical-edge API, public naming, example/doc requirements,
  test requirements, and accessibility wording. Linked from SUMMARY,
  direction guide, and adding-an-overlay guide. (RFC-012-A)

- **PR template** (`.github/pull_request_template.md`). Two-checkbox ABDD
  prompt plus a docs fence classification reminder, visible on every PR.
  (RFC-012-A)

- **Workbench example** (`examples/workbench/`). A single application
  exercising all major Snora surfaces together: header with File menu and
  RTL toggle, sidebar, breadcrumb, tab bar, four tab-body panels
  (Overview, Overlay Lab, Toast Lab, Direction Lab), all five toast intents,
  all six toast positions, dialog, sheet (End-anchored, mirrors under RTL),
  context menu, and footer status bar. Workspace member; compiles in CI.
  (RFC-012-B)

- **Workbench getting-started page** (`docs/src/getting-started/06-workbench.md`).
  Surface-by-surface reference table and manual QA checklist.

- **Compile-time tracking** (`scripts/measure-compile-time.sh`,
  `.github/workflows/build-cost.yaml`, `docs/src/reference/build-cost-budget.md`,
  `docs/src/reference/build-cost-budget/compile-time.csv`).
  Complements the existing binary-size budget. Measures four cold-build
  configurations per release and appends a row to the CSV on every tag,
  mirroring the `binary-size.yaml` commit-back pattern. No CI failure gate
  initially — trend signal only. (RFC-012-C)

- **Documentation test policy** (`docs/src/contributing/documentation-test-policy.md`).
  Defines code fence classifications and the no-bare-`rust`-fence rule.
  `mdbook test docs` added to the CI docs job as enforcement. (RFC-012-D)

### Changed

- **All 54 bare `rust` fences in `docs/src` classified** (RFC-012-D).
  Type-declaration excerpts → `rust,no_run` (15 fences in `vocabulary.md`
  and `widgets.md`). App-shaped partials → `rust,ignore` (41 fences across
  the remaining 15 files). Zero bare `rust` fences remain; `mdbook test`
  now passes on the docs tree.

- **CI docs job extended** with `mdbook test docs` step. (RFC-012-D)

- **Feature-gating-criteria indicator 1** updated to point at the new
  `compile-time.csv` and `build-cost-budget.md` instead of the previous
  ad-hoc measurement instruction. (RFC-012-C)

- **Release checklist** updated with the `build-cost` workflow post-tag
  verification step. (RFC-012-C)

- **README** updated with workbench reference and link. (RFC-012-B)

## [0.11.0] — 2026-06-10

### Added

- **Main Rust CI workflow** (`.github/workflows/ci.yaml`). Enforces the
  documented local-verification commands on every pull request and push
  to `main`: workspace check, clippy with `-D warnings`, `snora-core`
  tests, `snora` engine tests (including render-semantics), engine-only
  build, a six-combination feature matrix, and a mdBook docs build. The
  new workflow is the quality gate; `docs.yaml` and `binary-size.yaml`
  retain their existing deployment and measurement responsibilities.
  See `docs/src/contributing/release-process.md` for the relationship
  between the three workflows. (RFC-011-A)

- **Render-semantics test harness** (`crates/snora/tests/render_semantics.rs`).
  Six headless integration tests using `iced_test` verify the engine's
  runtime behavioral contract: skeleton reachability, outside-click
  dismissal, dialog and sheet interactive content, missing-close-sink
  graceful degradation, toast visibility above a modal, and sheet opaque
  wrapper behavior. `iced_test` is a `[dev-dependencies]` entry only —
  no impact on public API or binary size. (RFC-011-D)

- **RFC directory** (`rfcs/`). Adopts the RFC lifecycle policy (RFC-000)
  with `done/`, `proposed/`, and `archive/` folders and a `README.md`
  index. All 24 forward RFCs (011-A … 016-C) are filed in `proposed/`;
  RFC-000 and the five v0.11 RFCs move to `done/` with this release.

- **Overlay interaction semantics reference page**
  (`docs/src/reference/overlay-interaction-semantics.md`). Normative
  documentation for overlay coexistence, the z-stack order, the two
  close sinks, modal dim behavior, Law 5 (missing close sink), Law 6
  (toasts above modals), Law 7 (keyboard app-owned), and Law 8 (focus
  out of scope). Linked from `SUMMARY.md`. `render.rs` doc comments
  updated to match. (RFC-011-E)

- **Migration guide 0.10 → 0.11**
  (`docs/src/guides/migration-0.10-to-0.11.md`) covering the
  `#[non_exhaustive]` change and the toast ordering fix.

### Changed

- **`AppLayout` is now `#[non_exhaustive]`**. Struct literal
  construction from outside `snora-core` is no longer permitted.
  The canonical construction path — `AppLayout::new(body)` plus
  chainable builder methods — is unchanged and is the stable contract.
  Field reads remain unrestricted. This allows future overlay surfaces
  (e.g. popover, focus policy) to be added as non-breaking minor
  releases. An in-tree audit confirmed no examples or in-tree code used
  struct literals; all already used the builder. See the migration
  guide. (RFC-011-C)

- **Feature-gating criteria doc** updated with the supported
  feature-combination matrix and the subordinate-feature note for
  `lucide-icons` / `svg-icons`. (RFC-011-A)

- **Testing guide** updated with "What Snora tests internally" section
  describing the render-semantics test harness and the `snora-test`
  non-goal. (RFC-011-D)

- **`render.rs` doc comments** corrected: layer 7 description now says
  "configured `ToastPosition`, newest toast closest to the anchor edge"
  instead of the stale "bottom-end". (RFC-011-B / RFC-011-E)

- **`toast.rs` module doc** corrected: removes the stale "bottom-end
  only" description; accurately describes the six-position support and
  the newest-closest-to-anchor invariant. (RFC-011-B)

### Fixed

- **Toast ordering**. The newest toast now correctly appears closest to
  the configured anchor edge, matching the documented `ToastPosition`
  invariant. Previously the iteration predicate was inverted (`is_bottom()`
  instead of `is_top()`), causing both top and bottom anchor families to
  display in the wrong order. Applications that pushed toasts in
  chronological order (newest at the back) will now see the correct
  visual result. Applications that relied on the inverted order should
  update. (RFC-011-B)

## [0.10.0] — 2026-06-10

### Added

- **Binary size budget.** snora now tracks the size of its canonical
  example binary (`examples/hello`) at every release, with and
  without the optional `widgets` feature, to catch unintended size
  regressions early.
  - New reference page
    [`docs/src/reference/binary-size-budget.md`](docs/src/reference/binary-size-budget.md)
    explains the why/how and links to the data.
  - The data itself lives in a CI-managed CSV at
    `docs/src/reference/binary-size-budget/binary-size.csv`
    (header-only until the first tagged release writes a row).
  - New `binary-size` GitHub Actions workflow measures on every
    push and pull request (job summary + 30-day artifact, no file
    changes), and on every release tag appends one row to the CSV
    and commits it back to `main` with `[skip ci]`.
  - New `[profile.release-baseline]` Cargo profile (inherits
    `release`, `lto = false`, `codegen-units = 16`) gives the
    workflow a fast, consistent measurement profile.
  - New scripts `scripts/measure-binary-size.sh` and
    `scripts/append-binary-size-row.sh`, documented in
    `scripts/README.md`.
- `feature-gating-criteria.md` indicator (2) is now wired to the
  budget: its "Current status" row points at the CSV and the
  150 KB `diff_bytes` threshold.

### Changed

- `docs/src/contributing/release-process.md` gained a post-tag-push
  checklist item confirming the budget row was appended and the
  threshold respected.

### Notes

- The first CSV row is produced by CI on the first `v0.10.0` tag
  push; the shipped tree carries only the CSV header. This follows
  the page's own rule that humans never hand-edit the data file.

## [0.9.0] — 2026-04-29

### Added

- **Doctest coverage for `snora-core` vocabulary.** Public types now
  ship with executable `///` examples that `cargo test --doc` runs
  alongside the unit suite. The additions cover the 0.7-era
  newcomers (`Tab`, `TabBar`, `TabAction`, `Crumb` +
  `Crumb::ancestor` / `Crumb::leaf`, `BreadcrumbAction`) and key
  pre-existing types (`Sheet`, `SheetEdge::is_vertical`,
  `SheetSize::as_ratio` / `as_pixels`, `Toast::new`,
  `Toast::persistent`, `ToastPosition::is_top`,
  `LayoutDirection::flipped`). Total: 17 new doctests, all passing
  alongside the 17 unit tests.
- **A single Migration index page**
  ([`docs/src/guides/migrations.md`](docs/src/guides/migrations.md))
  collects the per-version migration guides and groups them under
  one entry in `SUMMARY.md` and `docs/README.md`. The individual
  `migration-X.Y-to-X.Z.md` files are unchanged; the index simply
  trims the navigation tree.

### Changed

- `docs/src/SUMMARY.md` no longer carries a top-level "Migration"
  part with one bullet per version. Migrations are now a single
  entry in the Guides part. Per-version detail lives one click
  away on the index page.

### Deferred

- **Binary-size baseline.** Originally planned for 0.9, deferred to
  0.10 — the release-mode iced compile (with workspace LTO) was
  too slow to run reliably in this release's environment. The
  measurement methodology is unchanged; see
  [`docs/src/contributing/feature-gating-criteria.md`](docs/src/contributing/feature-gating-criteria.md)
  indicator (2).

### Tests

- 34 tests in `snora-core`: 17 unit + 17 doctests (was 17 unit).

## [0.8.0] — 2026-04-29

### Added

- **Documentation is now an mdBook.** The `docs/` directory has been
  reorganized into a standard mdBook layout (`docs/book.toml`,
  `docs/src/`, `docs/src/SUMMARY.md`). All long-form documentation is
  authored as before; the new structure adds a searchable, themed,
  hosted view at <https://nabbisen.github.io/snora/>.
  - The Markdown source remains GitHub-readable. Internal cross-links
    use relative paths so both render targets work.
  - `docs/book/` is git-ignored; only the source under `docs/src/` is
    versioned.
- **GitHub Actions docs workflow.** A new
  `.github/workflows/docs.yaml` builds the mdBook on every push to
  `main` and deploys the result to GitHub Pages. The workflow
  status is exposed as a Docs badge in the README.
- **Project-level GitHub conventions.** `.github/` now ships:
  - `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`.
  - Issue templates (`ISSUE_TEMPLATE/{bug_report,feature_request,question}.yml`)
    and `config.yml`.
- **README Quick start now points to runnable examples.** A direct
  link to <https://github.com/nabbisen/snora/tree/main/examples>
  sits alongside the existing pointer to the getting-started chapter,
  so readers who want to skim working code rather than tutorials
  get there in one click.

### Changed

- The `docs/README.md` entry page was rewritten as the entry to the
  full snora documentation (not as an mdBook welcome). It links into
  `docs/src/...` and explains how to read the docs locally
  (`mdbook serve docs --open`) or in CI-published form on GitHub
  Pages.
- `docs/src/contributing/release-process.md` gained an `mdbook build
  docs` step in the release checklist so the book is validated as
  part of every release.

### Tests

- 17 unit tests in `snora-core` (unchanged from 0.7.0).

## [0.7.0] — 2026-04-29

### Removed

- The deprecated 0.6 aliases for the renamed sheet API are gone:
  - `BottomSheet` (use `Sheet`).
  - `SheetHeight` (use `SheetSize`).
  - `AppLayout::bottom_sheet(...)` (use `AppLayout::sheet(...)`).

  Code that compiled cleanly under 0.6 (without `#[deprecated]`
  warnings) compiles unchanged on 0.7.

### Added

- **Tab bar widget.** New navigation primitive for peer-level views.
  - Vocabulary in `snora-core`: `Tab<TabId>`, `TabBar<TabId>`,
    `TabAction<TabId>`.
  - Renderer in `snora-widgets`: `app_tab_bar(bar, on_action, direction)`.
  - Re-exported through `snora` and `snora::widget`.
  - Direction-aware: tab order mirrors under `LayoutDirection::Rtl`.
  - Active tab is rendered with a colored underline drawn from the
    theme's primary palette.
- **Breadcrumb widget.** New navigation primitive for ancestor-level
  navigation.
  - Vocabulary in `snora-core`: `Crumb<CrumbId>`, `BreadcrumbAction<CrumbId>`,
    plus the `Crumb::ancestor(...)` and `Crumb::leaf(...)` constructors.
  - Renderer in `snora-widgets`: `app_breadcrumb(crumbs, on_action, direction)`.
  - Re-exported through `snora` and `snora::widget`.
  - Direction-aware: order mirrors *and* the separator glyph flips
    (`›` under LTR, `‹` under RTL).
- New focused examples: `snora-example-tab` and `snora-example-breadcrumb`.
- New contributor doc:
  [`docs/contributing/feature-gating-criteria.md`](docs/src/contributing/feature-gating-criteria.md)
  records the indicators that would justify splitting the coarse
  `widgets` feature into per-widget gates. The decision for 0.7 is
  to keep the coarse gate; the document captures the criteria for
  revisiting it in future releases.
- New migration guide:
  [`docs/guides/migration-0.6-to-0.7.md`](docs/src/guides/migration-0.6-to-0.7.md).

### Changed

- `docs/contributing/design-decisions.md` gained two sections:
  - "Why `Tab` and `Crumb` are separate vocabulary, not one navigation type."
  - "Why widget feature gating is coarse, not per-widget."
- `docs/contributing/architecture.md` source-layout diagram updated for
  the two new modules (`snora-core/src/{tab.rs, crumb.rs}` and
  `snora-widgets/src/{tab.rs, crumb.rs}`).

### Tests

- 17 unit tests in `snora-core` (12 inherited from 0.6 + 2 tab + 3 crumb).

[Unreleased]: https://github.com/nabbisen/snora/compare/v0.25.1...HEAD
[0.25.1]: https://github.com/nabbisen/snora/compare/v0.25.0...v0.25.1
[0.25.0]: https://github.com/nabbisen/snora/compare/v0.24.0...v0.25.0
[0.24.0]: https://github.com/nabbisen/snora/compare/v0.23.0...v0.24.0
[0.23.0]: https://github.com/nabbisen/snora/compare/v0.22.0...v0.23.0
[0.22.0]: https://github.com/nabbisen/snora/compare/v0.21.0...v0.22.0
[0.21.0]: https://github.com/nabbisen/snora/compare/v0.20.0...v0.21.0
[0.20.0]: https://github.com/nabbisen/snora/compare/v0.19.1...v0.20.0
[0.19.1]: https://github.com/nabbisen/snora/compare/v0.19.0...v0.19.1
[0.19.0]: https://github.com/nabbisen/snora/compare/v0.18.3...v0.19.0
[0.18.3]: https://github.com/nabbisen/snora/compare/v0.18.2...v0.18.3
[0.18.2]: https://github.com/nabbisen/snora/compare/v0.10.0...v0.18.2
[0.10.0]: https://github.com/nabbisen/snora/releases/tag/v0.10.0
[0.9.0]: https://github.com/nabbisen/snora/releases/tag/v0.9.0
[0.8.0]: https://github.com/nabbisen/snora/releases/tag/v0.8.0
[0.7.0]: https://github.com/nabbisen/snora/releases/tag/v0.7.0
