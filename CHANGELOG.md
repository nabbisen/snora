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

## [Unreleased]

Nothing yet.

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

[Unreleased]: https://github.com/nabbisen/snora/compare/v0.10.0...HEAD
[0.10.0]: https://github.com/nabbisen/snora/releases/tag/v0.10.0
[0.9.0]: https://github.com/nabbisen/snora/releases/tag/v0.9.0
[0.8.0]: https://github.com/nabbisen/snora/releases/tag/v0.8.0
[0.7.0]: https://github.com/nabbisen/snora/releases/tag/v0.7.0
