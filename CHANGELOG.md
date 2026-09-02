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

### Added

- **"No behaviour change" becomes a checked claim, not a trusted one
  (RFC-092).** Six claims about code shipped wrong this cycle — four of
  them the architect's own, none caught by any gate, because no gate
  reads sentences. `scripts/check-docs-only.sh <rev>` prints every
  non-comment, non-blank changed line under `crates/` in a revision;
  empty output is what "documentation only" looks like, non-empty
  output refutes it — the exact filter that found F-33 and F-34 by
  hand, after 0.41.1 had already shipped claiming docs-only.

  **Ships as a CI gate, not a manual tool** (Q-1, ruled against the
  RFC's own suggestion): a commit carrying a `Docs-only: yes` trailer
  is checked automatically by the new `claims-check` job; no trailer,
  no check. The same reasoning that kept three other scripts manual for
  three releases (RFC-087, F-39) was wrong here for the same reason.
  Demonstrated on RFC-089's own commit (prints both F-33 and F-34), and
  against three constructed cases in an isolated worktree: a false
  `Docs-only: yes` claim refused and the offending lines named, a true
  one passed, and a commit with no trailer left unaffected.

  No crate code. `docs/src/contributing/`'s claim rules (Part 2, one
  sentence) are the architect's, not part of this change.

- **Workflow files are now validated before they're pushed, not only
  by GitHub after.** An unquoted step name containing `": "`
  (`183cc70`) made the entire `ci.yaml` unparseable — GitHub reported a
  bare `failure` with zero jobs, because a YAML mapping error leaves
  nothing to run. `scripts/check-workflows.sh` runs `actionlint`
  (pinned to an exact version, downloaded and SHA-256-verified if not
  already on `PATH`) against every file under `.github/workflows/`,
  locally or in CI. It fails loudly, not silently, when `actionlint`
  cannot be obtained at all — an unvalidatable tree is exactly the
  state this exists to end.

  **Wired as its own workflow, `workflow-lint.yaml`, deliberately not a
  job inside `ci.yaml`**: a broken workflow file produces no jobs, so a
  validator living inside the file it validates would have been just
  as absent during the incident above. The residual limit is stated in
  that file's own header: if it itself becomes unparseable, nothing
  reports — there is no escape from that regress, only a smaller
  surface for it (one rarely-edited file covering one edited three
  times this cycle).

  Found and fixed by the new tool itself, before it ever ran in CI:
  `binary-size.yaml`'s job-summary step had three unescaped backticks
  that `actionlint`'s embedded shellcheck reads as command
  substitution — cosmetic (a blank crate name in the binary-size job
  summary, not a build failure), but a real defect the project had no
  way to see before this.

## [0.42.0] — 2026-09-02

### Fixed

- **The engine's `Warning` and `Info` toast colours failed their own
  WCAG AA requirement, and the dismiss `×` was invisible on `Debug`
  (RFC-086, F-05, F-06).** `Warning` (white text on `WARNING_COLOR`)
  measured 3.18:1, matching the audit; text corrected to black against
  the unchanged fill — 6.60:1. The dismiss `×` was hard-coded white
  regardless of intent (1.58:1 for `Debug`'s light-gray fill); it now
  shares each intent's own text colour. **The hover/rest alpha fade is
  gone — a margin choice, not a floor fix.** With the corrected colours
  a 0.75-alpha fade clears the 3.0 floor at every intent and both stock
  themes (worst case, `Error`: 3.38:1). It is removed because doing so
  raises that worst case to 4.83:1 and makes the mark's contrast
  independent of interaction state — the mark is fully opaque at every
  status.

  **`Info` was not one of the audit's named findings.** Measuring all
  five intents (Q-3) rather than trusting the two reported found `Info`
  also under AA (4.43:1, both stock themes) — iced's own `primary`
  derivation, not a snora literal, and never checked before this
  release. Neither of iced's own paired tiers cleared AA with real
  margin, so the fill widened to `primary.strong.color` alongside a
  black text override — 5.64:1.

  This is the engine's first contrast assertion
  (`crates/snora/src/toast/contrast_tests.rs`), derived from the
  `ToastIntent` enum exhaustively (RFC-063's pattern) so a sixth intent
  cannot be added without a stated pairing. No `snora-design` or
  `snora-style` dependency — the engine stays token-free by design; see
  RFC-086 for why this is not RFC-085 restated.

  **Rendered appearance change on the default path** — see
  [`migration-0.41-to-0.42.md`](docs/src/guides/migration-0.41-to-0.42.md).

- **The workspace forced three `iced` features nobody asked for —
  `canvas`, `svg`, and `tokio` — one line below the line RFC-083 fixed
  in the same file (RFC-088, F-26).** `canvas` had zero occurrences
  anywhere in `crates/`; `svg` was used only under `snora`'s own
  `svg-icons` feature, which already declares `iced/svg` independently.
  Both removed from the workspace line. `tokio` stays: measured, not
  assumed — `snora::toast::subscription` (unconditional) calls
  `iced::time::every`, which does not compile at all without an
  executor feature, confirmed by removing it and reproducing
  `error[E0425]`. **Re-derived, not quoted:** dependency count (default
  features) 397 → 354 packages; binary size (stripped
  `snora-size-probe-engine`) −1,917,568 bytes (−1.83 MiB), matching the
  external audit's own figure independently. New CI gate
  (`scripts/check-workspace-iced-features.sh`) asserts every workspace
  `iced` feature is used, with `tokio`'s structural exemption named in
  the script rather than silently skipped — proven by perturbation
  (re-adding `canvas` fails the gate; restoring passes it).

  **Possibly breaking, the same shape as RFC-083, one release ago:** a
  consumer relying on `iced::widget::canvas` or `iced/tokio` arriving
  transitively through `snora` stops compiling. `svg-icons` consumers
  are unaffected. See
  [`migration-0.41-to-0.42.md`](docs/src/guides/migration-0.41-to-0.42.md).

### Added

- **Publishing moves into CI (RFC-090).** Three release rules previously
  enforced only by a person remembering — CI must be green, the tag
  must match `[workspace.package].version`, publish from a clean tree —
  had each failed at least once this cycle (main sat red for four
  commits; `v0.41.1` was tagged with a stray `v` prefix against 67 bare
  tags). A new `.github/workflows/release.yaml`, triggered only by a
  tag matching bare `X.Y.Z`, refuses to publish unless the tagged
  commit has an existing, completed, successful CI run (checked against
  that run, never a fresh re-run — a re-run can pass on a commit whose
  earlier run failed for a reason it doesn't reproduce) and unless the
  tag matches `Cargo.toml`'s version exactly. A dirty tree can no longer
  exist, structurally — the workflow's own checkout at the tag *is* the
  clean tree. Both refusal scripts
  (`scripts/check-commit-ci-green.sh`, `scripts/check-tag-matches-version.sh`)
  proven failing against real historical commits in this repository's
  own CI history before being wired in, not synthetic fixtures.

  Publishes via crates.io Trusted Publishing (OIDC) — no long-lived API
  token in a repo secret. No crate code affected; no version bump of its
  own.

## [0.41.1] — 2026-09-02

### Added

- **A troubleshooting guide (`docs/src/guides/troubleshooting.md`).**
  Five errors this project has actually produced for the people using
  it, each with the cause and the fix: `E0027` from the exhaustive
  `Palette`/`Typography` destructuring pattern (RFC-063), `E0638` when
  that pattern is attempted from outside the defining crate, the `E0433`
  that appears only when the `widgets` and `design` features are
  combined (RFC-054/RFC-055), `failed to select a version for the
  requirement 'snora-core = "^0.NN"'` after a minor bump, and
  `fatal: no tag message?` when signing a release tag. Linked from the
  book's guides section.

### Fixed

- **The accessibility guide claimed toast and notice tone was
  distinguishable by more than colour — it is not (F-20, RFC-089).**
  `docs/src/guides/accessibility.md` said *"Toast intents and notice
  tones are distinguishable by more than colour alone in snora's
  prefab widgets."* Checked against the actual renderers
  (`snora::toast`'s `toast_style`, `snora_widgets::design::notice`):
  both vary only background/accent colour by tone — identical title
  and body text, no icon, no textual prefix. This is a WCAG 2.1 1.4.1
  (use of colour) gap, published as solved when it was not. Moved to
  "What snora does not provide," naming the exact functions and the
  gap; adding a non-colour cue is a behaviour change and is not part
  of this fix.

  **Re-check, if this reached you before:** if you recorded that
  snora's prefab toasts or notices distinguish intent by more than
  colour, that came from us and is wrong — re-check any 1.4.1 claim
  that relied on it (RFC-067).

## [0.41.0] — 2026-09-01

### Fixed

- **Overlays did not contain pointer events — a click inside a dialog
  dismissed it, and three more surfaces had the same omission
  (RFC-084, Critical, found by an external architect's audit).**
  `render_dialog` wrapped its content in `center(...)` with no capture,
  so a click on the dialog's own padding, chrome, or plain text fell
  through the dialog layer to the modal backdrop beneath it and fired
  `on_close_modals` — not a corner case, a click anywhere on the
  dialog's own non-interactive surface dismissed it. `sheet.rs` already
  did this correctly (`opaque(body_surface)`); the dialog was the only
  modal surface that didn't. Fixed by wrapping the dialog's inner
  content in `iced::widget::opaque`, both the default and
  `design`-styled paths — validated by the architect before the
  Handoff was written, confirmed independently here: a reproduction
  test failed exactly as described before the fix (`got [CloseModals]`)
  and all twelve existing tests, including the untouched
  `outside_click_on_modal_emits_close_modals`, pass after it. **The
  same omission, three more times:** a modal shown with no
  `on_close_modals` handler blocked nothing at all (`dim_without_capture`
  was a plain, non-capturing container) — fixed by wrapping it in
  `opaque` too, so it now blocks pointer input while still producing no
  dismiss message, matching Law 8's unconditional "Pointer blocking —
  yes" without depending on whether a dismiss message was provided.
  Clicking a toast's own body (not its `×` button) pressed the widget
  underneath it — fixed the same way, wrapping each toast's own
  container in `opaque`. **Wheel-scroll measured separately, not
  assumed to follow the click fix** — and the first assumption about
  it was wrong and corrected in the open: reading `opaque`'s own
  `update` method suggested it would not stop `WheelScrolled` (it only
  checks for button presses), which would have meant scroll needed a
  separate, only-partial remedy for the no-sink case. Measuring instead
  found the opposite: `iced_widget::Stack`'s own dispatch stops passing
  the cursor to any layer beneath one whose `mouse_interaction()` is
  non-`None` while hovered, for *any* event, and `opaque` reports
  exactly that unconditionally — so it blocks scroll too, for both the
  with-sink and no-sink modal dim. The with-sink dim additionally
  gained an explicit `on_scroll` handler (mirroring its existing
  `on_press`) since `mouse_area` does not get the same unconditional
  treatment `opaque` does. **The test suite itself was the other half
  of this defect**: every containment test before this fix was
  positive only (a button inside an overlay is reachable), and none
  asked whether pointer input that should be blocked actually is. Six
  negative assertions added, derived explicitly from Law 8's own
  "Pointer blocking — yes" table row rather than invented independently
  (stated in `crates/snora/tests/render_semantics.rs`'s own module
  doc, including which of Law 8's other rows this RFC does *not*
  touch, and why). `docs/src/reference/overlay-interaction-semantics.md`'s
  Law 5 corrected — a missing close sink was documented as omitting
  capture entirely; it now correctly says it omits only the dismiss
  *message*. **Gate 5 in `api-freeze-review.md`, corrected rather than
  silently re-ticked**: it was marked satisfied and should not have
  been, since every test behind it was positive-only; the gate count
  updated from eight of ten to seven, and whether it re-ticks now that
  negative coverage exists is left to the owner. **Breaking, with a
  migration guide**: an application that relied on the old fall-through
  behavior, deliberately or not, sees a change; no configuration flag
  restores it, because the old behavior was the bug. `git diff` on
  `sheet.rs` — the one surface that was already correct — is empty.

- **The widget layer paired colours from different token families, and
  no contrast suite in the project could see it (RFC-085, Critical,
  found by an external architect's audit).** Every contrast assertion
  before this release lived in `snora-design` and tested tokens against
  roles — correct, and unable to see `snora-widgets`, which invents its
  own render-time pairings a role-based suite has no way to know exist.
  **A new suite was built first, in `snora-widgets`, and confirmed
  failing on every finding before any fix landed** — the acceptance bar
  the Handoff set. It measures every `button::Style`/`container::Style`
  this crate produces (found by grepping the crate for the return
  type, stated as the honest limit of an approach Rust's lack of
  reflection makes otherwise impossible to fully derive), on the
  background each is actually painted over, across every
  `button::Status`, both stock `iced::Theme` variants, and all four
  `design` presets. **Three named findings, plus two more the suite's
  own derived reach surfaced beyond them:** `menu_button_style` used a
  background-tier colour as text (1.89:1 light / 2.20:1 dark at rest,
  no status reached AA) — now `background.base.text`, the pairing iced
  itself guarantees. Chrome borders (header, footer, tab bar) used
  `background.weak.color` (1.02–1.48:1 against the page background,
  every preset and both stock themes) — now `background.base.text`,
  the only value derivable from a bare `iced::Theme` that clears the
  3.0:1 floor reliably (`background.strong`, tried first, measured
  short at 1.54–1.58:1). The sidebar's active highlight failed **two**
  ways: the highlight itself measured 1.89:1 (stock light) against the
  rail — and **1.51:1 under `high_contrast_dark`**, the preset that
  exists for low-vision users failing worst of all four — now
  `primary.strong.color`; and the icon/text on it used
  `background.base.text` (calibrated for the page background, not the
  highlight) — now `primary.strong.text`, iced's own pairing for the
  corrected highlight. Not named by the audit, found by the suite's own
  coverage: the active tab's label (`primary.base.color`, 2.99:1 on
  stock dark — no shade in the `primary` family reached AA against the
  page background on either stock theme, so the label now uses
  `background.base.text`, with the existing underline carrying the
  active/inactive distinction instead) and breadcrumb text (same
  pattern, same fix). **`high_contrast_dark` went from the single worst
  figure in the entire audit (1.51:1) to the best of the four design
  presets (9.96:1 worst case, versus 6.58:1 for `light` and 8.59:1 for
  `dark`)** — resolving the release blocker on its own terms. No
  `snora-design` token value changed and no new `Palette` role was
  added — `git diff -- crates/snora-design` is empty; every fix is a
  different choice of which already-correct token the widget layer
  reads. `render_semantics` unaffected. **Rendered appearance changes
  on both theme paths**, with a migration guide stating that reference
  images are invalidated, per 0.34.0's own precedent — and naming, not
  silently accepting, the visual trade-off of two fixes that had to
  drop a hover/active colour distinction because no single colour in
  the relevant family cleared AA against the actual background on both
  paths at once.

## [0.40.0] — 2026-08-21

### Fixed

- **`snora-core`'s published documentation did not build, because one
  workspace line pulled all of iced — and iced's `advanced`
  feature — into the crate whose entire identity is "no GUI
  dependency" (RFC-083).** Found from a live docs.rs build failure on
  `snora-core` 0.39.3 (`error: The platform you're compiling for is
  not supported by winit`). Root cause: the workspace declared
  `lucide-icons = { version = "1", features = ["iced"] }`;
  `lucide-icons`' own manifest maps that feature to `iced` **with
  `features = ["advanced"]` and no default features**, and every
  workspace member inherits it. Nothing in snora used the integration
  that feature provides — `snora-core` and `snora-widgets` only use
  `lucide_icons::Icon` and `LUCIDE_FONT_BYTES`, a plain enum and a byte
  slice, and `snora-widgets`' own source carries a comment instructing
  against calling the one method the feature exists to enable. Fixed
  with one line: `lucide-icons = { version = "1", default-features =
  false }`. Verified: `cargo tree -p snora-core --all-features` now
  shows `lucide-icons` and nothing else; `cargo check`/`test`/`clippy`
  all clean across the workspace (34 test-result lines, matching the
  pre-fix count exactly); iced's `advanced` feature appears in zero
  feature trees, workspace-wide. **A CI gate now holds `snora-core` to
  the same iced-free property `snora-design` already has** (matching
  that gate's exact mechanism — `cargo tree --prefix none`, grep for a
  leading `iced ` line — rather than inventing a second style),
  checked under `--all-features` specifically, since that is the
  combination that broke and the one docs.rs builds. Perturbation
  demo: re-added the removed feature, watched the gate fail naming the
  violation, restored. **Possibly breaking for a narrow case**: an
  application whose own code calls `iced::advanced::` without
  separately enabling that feature, relying on this transitive edge,
  needs to add `iced = { features = ["advanced"] }` itself — named in
  the new [0.39 → 0.40 migration guide](docs/src/guides/migration-0.39-to-0.40.md).
  `design-decisions.md`'s governance statement corrected: `advanced`
  is enabled by snora in no feature combination as of 0.40.0 — not
  merely "not by default," which left this transitive case
  unaddressed — with a record that it was not always so. Does **not**
  reopen the archived RFC-078; if anything it makes that ruling's
  premise (`advanced` must never be a default) hold without
  qualification, and records that `lucide-icons` users had been paying
  its cost, unmeasured, the whole time. **docs.rs build success is
  checked on the published release, not claimed here.**

## [0.39.3] — 2026-08-21

### Fixed

- **Two accessibility floors iterated a hand-written list of six roles.**
  The 12px text-size floor (RFC-081) and the 24px pointer-target floor
  (RFC-061) each enumerated `Typography`'s roles in a literal array, in
  the crate that defines the type, one file from `Palette::usages` doing
  exhaustive destructuring for exactly this reason (RFC-063). **A seventh
  role would have escaped both without failing anything.** Both now
  iterate `every_text_role()`, which destructures `Typography`
  exhaustively — adding a role is a compile error (`E0027`) until it is
  named. **Test-only; nothing a consumer compiles or runs changed**, and
  `git diff` on shipped code across this release is empty.

## [0.39.2] — 2026-08-20

### Fixed

- **The 12px text-size floor was stated flatly in `readability.md` and
  asserted nowhere (RFC-081).** Its two neighboring mandatory
  floors — the 24px pointer target and the contrast thresholds — are
  both enforced, one per role and padding step, the other as a compile
  error; the text floor had nothing. Found by **tekstide**. Added
  `text_size_meets_12px_floor_for_every_role`
  (`crates/snora-design/src/tests.rs`), asserting every `TextRole`'s
  `size` in all four built-in presets clears 12.0 — the failure
  message names the preset and the role, and cites
  `docs/src/guides/readability.md`, never a WCAG criterion (12px is
  snora's own rule; SC 1.4.4 is about resize, not a minimum, and this
  project has published a misattributed threshold before). No public
  validator added — presets only, per the owner's stated prior toward
  simplicity. The limit is stated in both the test's doc comment and
  `readability.md`: the assertion proves the four shipped presets
  comply and would catch a future preset edit that dropped a role
  below the floor, but cannot constrain an application's own `Tokens`
  — `Typography`'s fields are public and stay that way under RFC-036's
  covenant, so a custom `body_small: 8.0` is unreachable by any test
  snora ships. Also fixed while cross-checking every restatement of the
  floor: `docs/src/design/typography.md`'s "Accessibility floor"
  section still carried the exact pre-repair wording ("uses at least
  `body` or `body_small` — never a custom size below 12 logical
  pixels") that `readability.md` itself already documents as having
  cost knotra a remediation at roughly twice the size it needed, by
  reading as two floors instead of one — a third, unswept copy of an
  already-corrected misstatement. Corrected to match. No preset value
  changed; every role is 14–32px today and the assertion passes on
  arrival.

- **Three keyboard-and-focus statements a reader could not trust
  (RFC-082, credit tekstide for all three).** `design-decisions.md`'s
  focus-trapping row listed tekstide as a concrete app whose need "met"
  the reconsideration trigger; tekstide withdrew as a demand signal on
  2026-08-18 ("we would not switch to trapping even if you shipped
  it"), which our own 0.36.1 note had already recorded the consequence
  of. The row now says no consumer is currently a demand signal, cites
  the withdrawal, and — this does **not** resolve RFC-060 Q-1 — the
  decision stays **deferred**, only the evidence changed. (Round 2:
  the row initially pointed at RFC-078 as the pending measurement that
  would decide `advanced`'s enablement; RFC-078 was archived the same
  day, superseded by the owner's direct ruling that `advanced` will
  never be a default and no consumer ever requested it — corrected to
  match, in both the row and its prose section.) Every other row in
  the register was read against current
  source or correspondence the same day and given an "Evidence
  confirmed" date (new column) — Q-2 ruled this is stated, not
  mechanised, since the demand column depends on what consumers say,
  which no check can derive. `high-contrast.md` carried a mandatory
  checklist bullet whose own parenthetical said it could not be
  satisfied (the focus ring isn't rendered through `button::Style` in
  iced 0.14); marked `BLOCKED (iced 0.14 — no focus variant in
  button::Status)`, the exact severity label
  `accessibility-checklist.md`'s "Known limitations" section already
  defines for this shape of gap, rather than inventing a fourth
  marker. `guides/menus.md` said nothing about in-menu keyboard
  traversal in either direction; it now states the ruling and
  distinguishes the two menu-building paths — building the dropdown
  yourself (`AppLayout::header_menu`/`context_menu` with your own
  `Node`) already lets an application implement arrow-key traversal
  today, while the prefab `snora::widget::app_header` path
  (`render_menu`) currently has no channel for the application to
  express a highlighted item. No promise of a fix to the widgets path
  is made — that's a public API change with its own design question,
  scoped to a separate RFC. No code.

## [0.39.1] — 2026-08-20

### Fixed

- **`docs/src/guides/migrations.md` promised a migration guide for every
  minor release; three other documents disagreed with it, silently
  (RFC-079).** `contributing/release-process.md` carried two
  conditional checklist lines ("minor only", "if any public API broke
  or renamed") and `contributing/versioning-policy.md`'s version-level
  table required a guide only for rename/removal/feature-flag-rename —
  a reader following any of the three would reasonably skip the guide
  for an additive-only minor. `migrations.md`'s own promise was already
  correct and does not change; the other three statements (the two
  checklist lines and the table) are amended to agree with it —
  **every minor ships a guide, no condition**, even to say plainly that
  nothing is required. Six such gaps existed; the newest
  (0.38 → 0.39) was already written ahead of this RFC and is the
  worked example of the rule. The remaining five
  (0.29→0.30, 0.30→0.31, 0.31→0.32, 0.34→0.35, 0.37→0.38) are **not
  backfilled** — named explicitly in `migrations.md` as known,
  deferred gaps rather than left silently absent, per the owner's
  ruling. `scripts/check-migration-guides.sh` derives every consecutive
  minor pair from `git tag` and the filesystem and reports every gap it
  finds (18 on the current tree, including several older than the ones
  this RFC named); it fails only for a gap at or after one boundary
  constant (`ADOPTION_MINOR = 39`), so the five known-historical gaps
  don't fail a check nobody expects to be clean yet. Also fixed while
  making `mdbook test` pass cleanly (a required gate for this change,
  unrelated to the rule itself): `migration-0.38-to-0.39.md`'s
  before/after code example used free variables (`key`, `modifiers`)
  and cross-crate paths as if linked, which mdbook was actually trying
  to compile as a real doctest; marked `ignore`, matching this
  project's existing convention for the same shape of fragment
  elsewhere (RFC-064).

### Changed

- **The migration guide is now the canonical statement of what a
  release means for a consumer; a downstream letter carries only what
  is specific to that team (RFC-080).** Not a cost-cutting move — a
  reach one: the RFC-067 re-check obligation (name what to re-check
  when a claim is withdrawn or narrowed) previously depended on us
  choosing to write to a team, so it reached only the teams we decided
  to write to. In the guide it reaches everyone who reads the guide,
  including a future adopter jumping through that version who was
  never in a position to receive a letter at all. Written down in
  `contributing/release-process.md`, near the (now-unconditional,
  RFC-079) migration-guide checklist step, with the reasoning stated
  alongside the practice. A new checklist line operationalizes it:
  after the guide is written, decide correspondence per team — write
  only the team-specific part, and if no team has anything specific,
  send nothing (this is the note tekstide asked us to stop sending).
  **The correspondence bar itself does not change** — broken-now, a
  withdrawn claim a team acted on, or they asked. Every place stating
  the RFC-067 re-check obligation (`release-process.md`'s checklist
  line, `feature-gating-criteria.md`'s documentation-scope section) now
  says the same thing: the note lands in the guide, and correspondence
  may point at it but does not restate it. No code; no letter written
  or altered.

## [0.39.0] — 2026-08-20

### Added

- **`snora::focus` re-exports the zone-navigation vocabulary
  (`Cycle`, `FocusZone`, `ZonePresence`, `next_zone`) that
  `snora::keyboard::cycle_zones` already returned (RFC-076).**
  **arama** found it while shipping F6 zone navigation: the facade
  provided `cycle_zones() -> Option<snora_core::focus::Cycle>` but
  never re-exported `focus`, so a consumer depending only on `snora`
  could call the function and could not name its own return type. Fixed
  with a single module re-export, `pub use snora_core::focus;`
  (RFC-076 Q-1: the module, not the four names individually — matches
  how `keyboard` already appears as a module). `keyboard.rs`'s own doc
  comments told readers to reach past the facade to
  `snora_core::focus::next_zone` directly in three places; all three
  now use the facade path. A compiling doctest demonstrates
  `snora::focus::Cycle` is nameable using only a `snora` dependency —
  no `snora-core` needed. **Swept every public signature in `snora`
  for the same shape of gap (RFC-076 Q-2): none found** — of
  `snora-core`'s 25 public types, 22 were already re-exported by name
  and the 3 missing were exactly `focus::{Cycle, FocusZone,
  ZonePresence}`, all closed by this one re-export. Purely additive;
  nothing existing renamed or removed.

### Fixed

- **Our own rationale said the dialog card's border is what keeps it
  visible against the dimmed page behind an open dialog. It is not
  (RFC-077).** **arama** measured over real photographic content and
  found the border essentially invisible against the modal dim — 1.02:1
  in `light`, 1.23:1 in `dark`, against the 3.0:1 floor. Re-derived
  independently, swept over the full greyscale content range: **every
  preset** has a content luminance at which the border reaches exactly
  1.00:1 against the dim — there is no preset in which it reliably
  outlines the card there. What actually carries the separation is the
  card's fill against the dim, which never drops below 3.16:1 in any
  preset. The existing repair (RFC-058, 0.34.0) and the existing
  either-signal assertion (RFC-066,
  `dialog_card_distinguishable_from_modal_dim_all_presets`) were both
  already correct — `max(border ǀ dim, fill ǀ dim) ≥ 3.0` — but the
  prose around them implied two working mechanisms where arama's data
  (and this independent re-derivation) show there is only one: the fill
  branch always carries the assertion, the border branch never does.
  `docs/src/design/engine-surfaces.md` now states which mechanism does
  the work and which does not, without weakening the border requirement
  — the border still does real, required work at the card's own inner
  edge (3.38:1 `light`, **3.17:1** `dark`, previously unstated on this
  page) and against the plain undimmed page (unchanged, still
  documented). **Also measured, previously unmeasured: the sheet
  panel**, which sits over the same dim. Its border clears the dim
  floor more comfortably than the dialog's (minimum 2.41:1 vs 1.00:1)
  but is not token-styled at all — RFC-039 never restyled it — so its
  own border-to-fill contrast is only 1.02–1.35:1, well under the
  floor, using `iced::Theme::extended_palette()`'s subtle
  `background.weak` rather than a dedicated contrast-tested border
  role. No palette value, `DIM_ALPHA`, or assertion changed —
  documentation and rationale only.

## [0.38.3] — 2026-08-20

### Fixed

- **`release-process.md`'s checklist named two files by hand
  (`install.md`, `icons.md`); every other version snippet drifted
  since the line was written (RFC-074).** Worst instance: `crates/snora/src/lib.rs`
  shipped to docs.rs telling every reader of snora 0.38.2's own API
  docs to write `snora = { version = "0.25" }` — 13 minors stale,
  predating `snora-style`, the border repair, the modal dim, and the
  line-height helpers. Also stale: `README.md`'s Quick Start (`0.37`),
  `docs/src/reference/widgets.md` (`0.6`, 32 minors), and six snippets
  across `docs/src/design/feature-flags.md` (three different stale
  versions on one page). All corrected to `0.38`. A seventh instance
  the checklist could never have named: `release-process.md`'s own
  "bump snora-core's dep" example described a hand-pinned
  `path + version` dependency that no longer exists — every internal
  workspace dependency now uses `{ workspace = true }` — so the
  paragraph described a mechanism, not just a number, that was
  obsolete; corrected to state the actual current mechanism. The
  checklist line now invokes `scripts/check-version-snippets.sh`
  instead of naming files — it derives the expected minor from
  `Cargo.toml` itself and covers crate doc comments as well as
  `docs/`, so an eighth instance is discovered rather than remembered.
  Migration guides, `CHANGELOG.md`, and `rfcs/**` are excluded by
  design — they quote historical versions on purpose, and a check that
  cannot tell a live snippet from a quoted one is worse than no check.
  The seventh instance's first fix said internal dependency versions
  never need a per-crate bump — true for a patch, false for a minor:
  `[workspace.dependencies]` still carries five `version = "0.38"`
  pins that a minor must move, and the release checklist immediately
  below already said so. Corrected to state the pin is centralised,
  not absent, and point at the checklist rather than duplicate it.

- **The 1.0 gate register disagreed with itself about whether gate 9b
  was closed, and the frozen style-bridge surface omitted seven of the
  functions it freezes (RFC-075).** `api-freeze-review.md`'s own gate
  table and top summary correctly said gate 9b closed at v0.37.0, but
  the count immediately under the table said "seven of ten" (omitting
  9b), the "Remaining blockers" list still named 9b, and a third
  paragraph argued in the present tense for closing it on a stale
  25%-noise figure RFC-073 had already deleted elsewhere. All three
  corrected to agree with the table; the blocker list — a second,
  independently-drifting restatement of the same count — is removed
  rather than repaired. Separately, `api-governance.md`'s frozen
  style-bridge enumeration named 15 of `snora_style`'s 22 public
  functions, missing all six `*_line_height` helpers (RFC-068, shipped
  this same cycle) and `theme::theme` (missing since RFC-055, six
  minors). The covenant's own defining sentence — "all public
  functions of `snora_style`" — is already complete; the list added no
  governance and could only go stale, so it's deleted in favor of
  naming the six modules and pointing at the crate's rustdoc.
  `docs/src/design/stability.md` carried the same enumeration and the
  same fix. Also corrected in the same pass: `feature-gating-criteria.md`
  said "Ten downstream/review reports" before its own table and
  "Eleven" after it — the table has eleven rows, an RFC-072-era miss
  from earlier this session; `engine-surfaces.md`'s illustrative
  snippet showed the pre-RFC-065 `0.4` alpha and credited iced's public
  `is_dark` — the real constant is `DIM_ALPHA = 0.44` and the real
  `is_dark` is private and iced-free, enforced by a CI gate;
  `overlay-interaction-semantics.md` described RFC-014-A and RFC-014-B
  as future work in four places while recommending RFC-014-A's shipped
  `dismiss_on_escape` as current API on the same page — both RFCs
  landed at v0.14.0; `snora-style/src/text.rs`'s module doc still
  framed widget line-height adoption as pending an adopter's evidence —
  that evidence arrived and RFC-068 Q-2 ruled on it (short-label
  widgets will not adopt it; wrapping-prose widgets remain open);
  `snora-widgets/src/lib.rs` described its own `design` module as "the
  iced style bridge" while `design.rs`'s own module doc, one file
  down, correctly said the bridge is `snora_style` and named the
  module prefab widgets — rewritten to match; `snora-design/src/lib.rs`
  and `color.rs` still attributed the style bridge to `snora-widgets`;
  `docs/src/design/theme.md` pointed at
  `crates/snora-widgets/src/design/theme.rs`, which hasn't existed
  since RFC-055 (`crates/snora-style/src/theme.rs` is current); and
  `snora-widgets/src/design/card.rs` carried a "Cards in v0.20 are
  non-interactive" qualifier in two places — the same wording already
  fixed once, in a sibling location, at 0.22.0. Also, authorised into
  scope after the fact: `architecture.md` and
  `overlay-interaction-semantics.md` both described the modal dim as
  "40 %" with no qualifier — correct only for the unstyled
  `snora::render` path; `snora::design::render` composites it at 44%
  (`DIM_ALPHA`, RFC-065). Both now name which path each figure
  belongs to instead of stating one number as if it were the only one.

## [0.38.2] — 2026-08-20

### Fixed

- **Three pages stated things that stopped being true, and nothing was
  attached to any of them that would notice (RFC-073).** Found in a
  pre-cut audit, and confirmed with a built-output link check — every
  internal link under `docs/book` resolved to a file that exists — not
  just a source-level one, since these defects were invisible to that.
  **1. Three migration guides 404'd on the published site.**
  `guides/migrations.md` linked `migration-0.4-to-0.5.md`,
  `-0.5-to-0.6.md`, and `-0.6-to-0.7.md`; none was in `SUMMARY.md`, so
  mdBook never built them — every source-level link check passed
  while the defect shipped, because all three files exist on disk.
  Added to `SUMMARY.md`. **2. `build-cost-budget.md` carried three
  separate statements that gate 9b was still open, and one of them
  misreported `api-freeze-review.md`'s own verdict by name.** Gate 9b
  closed at v0.37.0 (verified against `api-freeze-review.md:107`
  directly). The stale "Data integrity note (gate 9b, v0.29.0)"
  section — unrevisited in nine releases, and reasoning from four rows
  a later section (RFC-050) superseded — is deleted rather than
  re-titled, since every figure in it is recomputable from the
  `compile-time.csv` rows already committed; a third statement, inside
  a separate RFC-052 note, is corrected to state that gate 9b closed
  via a different metric (`design_overhead_ratio`) entirely, not by
  the columns that note discusses reaching two comparable rows. The
  page now states its trend signal in exactly one place. **3. The
  accessibility checklist called shipped work "deferred."** The
  `*_line_height()` helpers shipped in 0.38.0 (RFC-068), and widget
  adoption isn't pending either — RFC-068 Q-2 ruled short-label
  widgets will not adopt line-height. The checklist item now states
  both facts instead of "deferred, not blocked." **A fourth broken
  link of the same shape as (1) was caught by the required built-output
  check and fixed alongside it**: `contributing/recipes.md` linked
  `design/recipes/README.md`, which mdBook's own link-target rewriting
  does not resolve to the built `index.html` the way it does for
  `SUMMARY.md`'s own nav entries — fixed by renaming the source file
  to `index.md` (not by repointing the link at a file that doesn't
  exist, which would trade a built-output break for a source-level
  one) and updating `SUMMARY.md` to match. The built-output link
  checker (`scripts/check-built-links.py`) is now a committed, manual
  tool — not wired into CI. No code change; no new CI gate, policy, or
  checklist mechanism added.

## [0.38.1] — 2026-08-20

### Fixed

- **The style bridge's own reference page still listed six helpers when
  there are twelve.** `design/iced-style-bridge.md` — the page a consumer
  reads to learn what the bridge offers — had a section titled
  "Typography sizes" enumerating `body_size` … `display_size` and never
  mentioning line-height, one release after RFC-068 added a line-height
  helper per role. Caught by the release checklist's documentation-scope
  line (a capability arrived; a page still implied its absence), not by a
  test — no test can see this. The section now lists all twelve as a
  role table, its compiled snippet applies both halves, and it points at
  [Typography](docs/src/design/typography.md) for each role stated
  against iced's own `Relative(1.3)` default, since **applying a
  line-height helper is not always an improvement over applying none**
  (RFC-070).

- **The engine-surface visibility floor tested a bar the palette
  cleared four minors ago, and the docs page that justified it was
  publishing pre-repair numbers (RFC-071).** `VISIBILITY_FLOOR` (now
  `NON_TEXT_MIN`, `3.0:1`) was `1.3:1`, justified as "what the `border`
  role actually achieves… with a small margin under the worst case
  (`light` at 1.39)." That worst case was the *pre*-repair figure — the
  border was fixed to clear `3.0:1` in 0.34.0 (RFC-058), and the
  assertion was never updated to match, so a regression from the real
  3.38:1 worst case back down to 1.31:1 would have passed silently for
  four minors. `docs/src/design/engine-surfaces.md` carried the same
  stale figures in its own table, plus a second, independently stale
  table for the modal dim (frozen at pre-RFC-065 values since 0.37.0).
  Both tables re-derived and corrected: border worst case **3.38:1**
  (`light`), dim worst case **3.2424:1** (`light`/`high_contrast_light`)
  — both clear the real `3.0:1` WCAG SC 1.4.11 floor with margin, so
  one shared constant serves both assertions without splitting it.
  Perturbation-tested: a border regressed toward `background` and a
  lowered `DIM_ALPHA` both now fail the suite, which they did not
  before. **No palette value or `DIM_ALPHA` changed** — this repairs an
  assertion and a page, not a color.

- **The typography scale was calibrated against itself and never
  stated against iced's own default line-height, and the readability
  guidance assumed applying a role's line-height is always an
  improvement — it isn't, for half the scale (RFC-070, found by
  orbok).** iced 0.14 renders text at `Relative(1.3)` whenever
  `.line_height()` is never called
  (`impl Default for LineHeight` → `Relative(1.3)`,
  `iced_core-0.14.0/src/text.rs:215-219`). Stated against that
  baseline: `body` (1.4) and `body_small` (1.35) are looser — the
  intended benefit — `title` (1.3) is **identical**, so
  `title_line_height` restates the renderer default and has no
  observable effect on any surface, and `label` (1.2), `heading`
  (1.25), and `display` (1.2) are **tighter** than doing nothing at
  all, deliberately, because larger text tolerates less relative
  leading. `readability.md`'s "apply line-height to anything that
  might wrap" rule read as uniform improvement; it named `title` and
  `heading` alongside `body`/`body_small`, and applying either of
  those two does not help the way the rule implied. Corrected, along
  with `typography.md`'s role table (now carries the baseline,
  cited to source) and `TextRole::line_height`'s and
  `title_line_height`'s own doc comments. **Measured whether `heading`
  (1.25 vs. iced's 1.3, on a 24px heading) should change under RFC-036's
  accessibility carve-out: no** — the difference is 1.2px per line
  (3.85% of a 31.2px line box), ordinary typographic tightening at
  larger sizes, not a legibility defect. **No preset value changed;**
  documentation and doc-comment corrections only.

### Changed

- **Documented that snora's contrast thresholds are floors, never
  ceilings — a property the codebase already had and had never told
  consumers (RFC-072).** Verified across the entire contrast suite:
  every assertion is `>=` (`snora-design/src/tests.rs:107`, the
  derived mandatory pairs; `:302`, primary text at AAA; and every other
  contrast assertion in the workspace, re-checked beyond just those
  two files) — there is no upper-bound contrast assertion anywhere.
  Combined with RFC-036's covenant, which permits changing a preset
  value only where a test proves it fixes a defect, the only direction
  a contrast ratio can move is up. A consumer (knotra) had asserted the
  opposite — that `border` against `surface` *stays below* AA — to
  justify excluding a notice tone; the reasoning was sound and the
  figure right, but the bound is not one snora holds, and it has
  already moved once (0.34.0 took it from 1.32 to 3.50). Stated as a
  commitment in `api-governance.md`, beside the covenant's permitted-
  changes list, with the practical consequence spelled out — do not
  assert a snora colour stays below a threshold; assert against your
  own — and the honest limit of the guarantee: a repair is judged only
  on the pair that was failing, and preserves nothing else. A short
  pointer added in `guides/accessibility.md`, not a second copy.
  Documentation only; no code, no assertion, no value changed.

## [0.38.0] — 2026-08-19

### Added

- **`snora-style::text` gained six line-height helpers, one per role
  (RFC-068).** `TextRole` carries `size` and `line_height`; the six size
  helpers (`body_size`, `body_small_size`, `label_size`, `title_size`,
  `heading_size`, `display_size`) had no line-height counterpart, so an
  application applying line-height had to reach through
  `tokens.typography.<role>.line_height` and construct
  `iced::widget::text::LineHeight::Relative` by hand — including in
  snora's own published examples. `body_line_height`, `body_small_line_height`,
  `label_line_height`, `title_line_height`, `heading_line_height`, and
  `display_line_height` now sit beside their size counterparts, same
  module, same shape, returning `LineHeight::Relative`. A two-axis
  exhaustive test (`crates/snora-style/src/text.rs`) makes both a new
  `Typography` role and a new `TextRole` field a compile error until
  tooled here, and a helper wired to the wrong role a test failure
  naming which one. **Purely additive — no rendered output changes,
  and this does not oblige snora's own prefab widgets to apply
  line-height**, which remains a separate, larger decision gated on
  adopter evidence (RFC-068 Q-2).

  **Re-check, if this reached you before:** until this release
  `docs/src/design/typography.md` stated outright that *"line-height is
  not wrapped in a helper — read the multiplier straight off
  `tokens.typography.<role>.line_height`."* That sentence was true and
  is now withdrawn. **If you wrote your own helper, a local wrapper, or
  a hand-built `LineHeight::Relative` because we told you none
  existed**, the six helpers above replace it — and if you skipped
  applying line-height at all because the token access looked like
  reaching into internals, it never was, but it is now a one-call
  helper either way. Nothing you wrote has broken; this is a
  simplification you are free to take or ignore (RFC-067).

### Changed

- **The documentation-test policy stopped blaming snippet shape for
  something structural, and 12 book examples are now actually compiled
  (RFC-069).** All 111 Rust fences in `docs/src` were `rust,ignore`,
  and the policy framed that as a per-snippet judgement —
  "full-app-shaped snippets, partial `impl` blocks, event-loop shapes."
  It wasn't: `docs/book.toml` has no `[rust]` section and the docs CI
  job runs bare `mdbook test docs`, so **no fence importing `snora`
  could compile, however small or complete.** The policy now states
  the structural cause once instead of implying it per-snippet, fixes
  a self-contradiction where its own classification table called
  `no_run` "highlighted but not compiled" (the ladder table, correctly,
  calls it "compiles, does not execute"), and does **not** extend
  RFC-064's per-fence reason rule to the book — 111 copies of one
  sentence is the exact drift failure that rule exists to prevent.
  A new `publish = false` workspace member, `examples/book_snippets`,
  holds anchored, genuinely compiled source for 12 fences on the frozen
  covenant surface (design tokens, the `snora_style` bridge) —
  `{{#include}}`-pulled into the book with **no fence tag changed
  anywhere**; CI proves these compile because a crate compiles them,
  not because a tag claims it. Three fences that showed a type's shape
  or enumerated variants (not usage) were deliberately left as prose,
  not converted — compiling what is code and leaving what is a diagram
  alone. Zero CI cost: `{{#include}}` reads raw source text and never
  invokes `cargo`, and the new crate is already swept up by the
  existing `cargo check --workspace` / `clippy --workspace` steps via
  the `examples/*` glob — no workflow file changed. Migration-guide
  fences (20 of the 111) are permanently excluded from this or any
  future compile mechanism, since their staleness is the content, not
  a defect. Documentation and CI-adjacent only — no crate code, no
  public API, no rendered output changed.

## [0.37.2] — 2026-08-19

### Changed

- **Release notes now name the re-check, not only the correction, when
  a claim is withdrawn or narrowed (RFC-067).** Two consumer-facing
  claims were withdrawn in 0.34.0 — a `text_muted` contrast exemption
  we had invented, and an over-scoped "cannot be rendered" statement
  about a focus ring on iced 0.14 — and both were explained thoroughly.
  Neither withdrawal said what a consumer who had already acted on it
  should now do, and five instances across four consumers propagated
  the old claims downstream before any of them told us: two into
  orbok's and knotra's own accessibility suites/records, one across 28
  call sites in aaai's contrast test, two into apimokka's and orbok's
  own accessibility contracts (`feature-gating-criteria.md`'s
  documentation-scope table has the full instance list). Every one was
  found by the consumer, not by us. `feature-gating-criteria.md`'s
  documentation-scope rule (RFC-048, widened by RFC-056/059) now covers
  this as a fourth case — a claim withdrawn or narrowed — and
  `release-process.md`'s checklist carries the question that fires it.
  Retroactive re-check, for the two withdrawals already known to have
  propagated:

  - **If you excluded `text_muted` from a contrast or accessibility
    suite on the strength of snora's documentation, that exemption was
    invented and is withdrawn (0.34.0) — re-check the role.** It is
    asserted at `AA_TEXT` (4.5:1) against all three surfaces as of
    0.34.0; nothing exempts it.
  - **If you recorded that a focus ring cannot be rendered on iced
    0.14, that statement was over-scoped (0.34.0) — re-check any scope
    decision that cited it.** An application owning focus as its own
    state can style it today; the accurate constraint is narrower than
    "cannot be rendered."

  We know of five propagation instances across these two withdrawals
  and cannot know the full set — four consumers told us; this note does
  not imply the list above is complete.

## [0.37.1] — 2026-08-18

### Changed

- **The modal-dim contrast assertion swept its content range instead of
  checking three discrete surfaces, and two published figures from
  0.37.0's RFC-065 entry were overstated (RFC-066).** The dim is
  painted over whatever the application actually rendered — a
  continuum — not just `background`/`surface`/`surface_raised`; for two
  of the four built-in presets the true worst case is an **interior**
  minimum (where the card's border and fill contrast cross) that a
  three-surface check cannot see. **Correction to the table published
  in the [0.37.0] entry below:** `high_contrast_light`'s recorded 7.37
  was measuring only an endpoint — the true worst case, swept, is
  **4.58** (at 82% grey content, not white); `high_contrast_dark`'s
  recorded 5.25 is similarly **4.45** (at 5% grey, not its nominal
  `surface_raised`). `light` (3.24) and `dark` (3.64) are unchanged —
  both true minima are at an endpoint, which the old check already
  measured exactly. **Nothing failed and no preset value changed**:
  both corrected figures remain comfortably above WCAG 2.1 SC 1.4.11's
  3:1, and `DIM_ALPHA` (0.44, RFC-065) is unaffected — it was chosen
  against the endpoint figures, which were always correct. The
  assertion now sweeps 1000 greyscale content steps per preset
  (`crates/snora-design/src/tests.rs`); greyscale suffices because the
  dim composites channelwise and luminance is monotonic per channel, so
  RGB content anywhere is bounded by its black/white greyscale
  endpoints. See the corrected-figure note added to the
  [0.36 → 0.37 migration guide](docs/src/guides/migration-0.36-to-0.37.md)
  (left as a correction note, not a rewrite — that guide shipped with
  0.37.0).

## [0.37.0] — 2026-08-18

### Fixed

- **The `design`-path modal dim is now a measured surface, and `light`'s
  dialog card was failing on it (RFC-065).** RFC-063 closed the *role*
  axis — no `Palette` role can ship without declaring where it renders —
  but the modal dim is not a role; it is composited at render time, so
  `Palette::usages` could not see it and nothing measured it. Measured:
  the `light` preset's dialog card was distinguishable from its own
  dimmed backdrop at **2.85:1** by either signal (border or fill), below
  WCAG 2.1 SC 1.4.11's 3:1, worst case across the three surfaces a modal
  can open over. Repaired by moving `DIM_ALPHA` from `0.40` to `0.44`
  (`snora_design::surfaces::modal_dim`, previously a private helper in
  the `snora` crate calling `iced::theme::palette::is_dark` directly, now
  a pure `snora-design` function `snora`'s renderer calls). Before/after,
  all four presets, worst backdrop:

  | preset | before | after |
  |---|---|---|
  | `light` | **2.85 — FAIL** | **3.24** |
  | `dark` | 3.18 | 3.64 |
  | `high_contrast_light` | 7.37 | 6.48 |
  | `high_contrast_dark` | 5.25 | 4.56 |

  `documentation-test-policy.md`'s sibling gap is `api-governance.md`'s:
  the *surface* axis is now recorded beside RFC-063's *role* axis — a new
  composited/derived surface carries the same declare-and-measure
  obligation a new role does, and `accessibility-checklist.md`'s Contrast
  section covers it as a class. Appearance change on the `design` path
  only; the unstyled/engine path's dim is untouched. See the
  [0.36 → 0.37 migration guide](docs/src/guides/migration-0.36-to-0.37.md).

## [0.36.1] — 2026-08-18

### Changed

- **Every `ignore`-fenced doctest in `crates/` is now audited, not just
  written and forgotten (RFC-064).** Of 19 fences, 16 are promoted to
  their strongest reachable rung: 1 to a full run (`snora-style`'s
  `to_iced_color`, which had an `assert_eq!` that had simply never
  executed), and 15 to `no_run` (compiles, catches API drift, costs no
  runtime). The 3 that genuinely cannot compile (two partial-fragment
  examples in `snora/src/keyboard.rs`, one real-event-loop example in
  `snora-style/src/theme.rs`) stay `ignore`, each with a stated reason
  above the fence. `documentation-test-policy.md` now records the
  three-rung ladder as the rule, covers all five crates (previously only
  `snora-core` and `snora-widgets` were mentioned), states the
  pure-function distinction `snora-style` missed by analogy in 0.32.0,
  and corrects a false "17 doctests, tracked in the release checklist"
  claim — the count is now genuinely tracked there. Doc comments only;
  no code, API, or behaviour changed.

## [0.36.0] — 2026-08-18

### Added

- **The pointer-target-size rule now has a height-axis assertion
  (RFC-061).** `accessibility-checklist.md` has mandated a 24×24
  logical-pixel minimum pointer target since it was written; nothing
  asserted it — RFC-058's shape exactly, and raised by tekstide, who
  noted the parallel themselves. `pointer_target_height_meets_24px_for_every_role_and_padding_step`
  (`crates/snora-design/src/tests.rs`) asserts every `TextRole` ×
  `Spacing` step combination (36 combinations × 4 presets, all passing;
  the tightest margin is `label`/`xs` at 24.8px, 0.8px above the floor
  — recorded so the next token edit meets it knowingly). The **width**
  axis is not asserted — `content_advance` depends on the rendered
  string, the font, and the shaping engine, none of which snora can
  compute without a renderer — and is documented as review-only,
  explicitly, so it cannot be mistaken for enforced.

### Fixed

- **`chip::removable`'s dismiss button was under the mandatory 24×24
  pointer-target floor on its width axis (RFC-061).** Measured directly
  against iced's shipped fallback font (FiraSans-Regular), not
  estimated: **15.0 logical pixels** wide, against the WCAG 2.5.8 floor
  of 24. Height already cleared it (24.8px, token-computed). Padding
  alone could not reliably fix this — even `spacing.sm` reaches only
  23.0px on the same font, one pixel short — so the button now has an
  explicit minimum width, computed the same way its height already
  resolves (`line_box + 2 × spacing.xs`), making it square at ~24.8px
  and keeping the fix tied to the same tokens rather than a hard-coded
  second "24". **This is an appearance change**, not a silent fix — see
  [the 0.35→0.36 migration guide](docs/src/guides/migration-0.35-to-0.36.md).
  `chip::filter` is unaffected.

### Changed

- **Feature-gating indicators recalibrated; the CI compile-time proxy
  retired (RFC-062).** `feature-gating-criteria.md`'s status table read
  indicator 1 (compile time) as "Within budget" while citing ≈96 s
  against a 30-second threshold — over budget, in the document that
  decides. Every row since 0.26.0 read 2.2×–3.5× over. The cause: the
  threshold is written against a developer's machine; the column cited
  as its proxy (`build_widgets_ms`) measures GitHub CI, which since
  RFC-043 rebuilds iced's entire closure from scratch on shared,
  noisy hardware — a different quantity, made visible by RFC-043,
  not created by it. The claim that this column is indicator 1's proxy
  is retired; indicator 1 is now recorded honestly as **unassessed**,
  with the correct developer-machine command stated. Indicator 2's
  documented method — diffing two `snora-example-hello` builds — has
  been stale since RFC-041 replaced it with the three-probe-crate
  method; corrected, with the 150 KB threshold unchanged. Every row in
  the status table now carries a measured value and a met/not-met
  verdict instead of a prose-only claim; indicators 3 and 4 were
  re-checked against current manifests (`snora-style` arrived since the
  table was last written) rather than inherited. `design-decisions.md`
  now states explicitly, with numbers: at most one of the five
  indicators could be met, short of the two the `widgets`-gate trigger
  requires — the trigger has not fired. Separately, attached the
  accessibility-tree reconsideration trigger to an actual check
  (`cargo tree -p snora --all-features | grep -i accesskit`, credit
  tekstide) — verified empty, so that trigger has not fired either.
  `release-process.md`'s checklist now points at the status table,
  which previously told readers to "update this table as part of the
  release process" with nothing pointing at the instruction for ten
  minors. Documentation only — `git diff --stat -- '**/*.rs'` is empty.

- **Contrast pairs are now derived from a compiler-enforced declaration,
  not a hand-maintained list (RFC-063).** tekstide's diagnosis after
  RFC-058 fixed two instances: *"the list did not fail because it was
  short; it failed because nothing about adding a role forces anyone to
  measure it."* `Palette::usages` (`crates/snora-design/src/palette.rs`)
  destructures `Palette` exhaustively and declares each role's intended
  rendering surfaces and threshold class — `#[non_exhaustive]`
  constrains other crates, not the one that defines it, so a role added
  without a matching declaration entry fails to compile
  (`E0027: pattern does not mention field ...`), demonstrated by a
  probe: a nineteenth field added, the error captured, then reverted
  (`git diff --stat -- crates/snora-design/` empty afterward — only
  `palette.rs`/`tests.rs`'s real changes remain).
  `mandatory_pairs` in `tests.rs` is now derived from this declaration;
  no hand-written pair list remains, and `Palette::roles()` — the same
  defect one level down, an 18-element array a nineteenth field could
  silently miss — is removed, replaced by `usages()` for both contrast
  pairing and colour-validity checking.

  Not the cross-product: each role declares where it is *intended* to
  render, not everywhere it technically could — `accent_text` declares
  only `accent`, not the three neutral surfaces, since asserting it
  there would be noise, and noise in an accessibility gate is how gates
  get ignored. True fill/surface roles (`background`, `surface`,
  `surface_raised`) declare no measurable surfaces, explicitly, rather
  than being silently omitted from the list.

  **Deriving the list surfaced one previously-unasserted pair:**
  `focus/surface_raised` — `focus` was declared against all three
  neutral surfaces for consistency with `border`'s declaration (a focus
  ring, like a border, can appear around a control on any of them), and
  the hand-written list had only ever asserted `focus/background` and
  `focus/surface`. Checked before landing, per RFC-058's discipline:
  passes on all four presets with wide margin (6.2:1–21:1) — a ratchet,
  not a defect.

  **Review found two more roles declared as measuring nothing that
  should not have been: `accent` and `danger`.** Both back *filled,
  borderless* buttons (`snora_style::button::{primary, danger}` —
  `Border::default()`, width 0), so the fill itself is the button's
  identifying boundary, the same argument RFC-058 used for the dialog
  card's border, applied to a fill instead of a stroke. Declared against
  the three neutral surfaces; all six new pairs pass with wide margin
  (5.6:1–11.8:1) — another ratchet, no value change. `success`,
  `warning`, and `info` were measured too (worst case 4.63–5.01:1,
  comfortably passing either bar) but left undeclared: their only
  usage (`snora-widgets/src/design/notice.rs`, a left accent bar and
  border) is on an informational panel that also carries the tone in
  its own text, a weaker fit for SC 1.4.11's "user interface component"
  than a filled interactive button — recorded as an open question for a
  future decision rather than resolved here. **26 pairs now asserted
  across four presets (104 assertions total, up from 19 pairs / 76
  assertions before this RFC), all passing.**

## [0.35.0] — 2026-08-18

### Added

- **Frame-level keyboard zone navigation (RFC-060).** Snora owns the
  frame; applications own what is inside a pane — and until now snora
  supplied none of its half of keyboard navigation, so every
  multi-region application reimplemented the same logic. tekstide did,
  and told us so. New, iced-free vocabulary in `snora-core::focus`:
  `FocusZone` (`Header`/`SideBar`/`Body`/`Footer`), `Cycle`
  (`Forward`/`Backward`), `ZonePresence`, and `next_zone` — a pure
  function deciding the next zone in logical order
  (`Header → SideBar → Body → Footer`, wrapping), skipping absent
  slots. Logical order is deliberately **not** direction-mirrored: under
  RTL the sidebar moves to the opposite physical edge but is still the
  start-edge rail following the header, so — unlike `ToastPosition` —
  this needs no `LayoutDirection` parameter.

  **snora does not take Tab or Shift+Tab** — Tab already means "next
  control," and claiming it would break in-pane navigation for every
  application with a form. The recommended binding is **F6 / Shift+F6**,
  via a new companion to `dismiss_on_escape`: `snora::keyboard::cycle_zones`.
  Same shape, same non-capture policy — snora installs no subscription;
  the application wires `iced::keyboard::listen()` and calls the pure
  function itself.

  Cycling is automatically **suspended** while a dialog or sheet is
  open — reported, not silently redirected to `Body`, since modal
  contents are an application-supplied `Node` snora cannot enumerate —
  and **unaffected** while only a menu is open, mirroring
  `dismiss_on_escape`'s modal-before-menu precedence exactly.

  **This ships navigation, not containment.** Nothing bounds Tab inside
  an open modal's own content yet; that needs iced's `advanced` feature
  (verified: *moving* focus is reachable without it, *querying* which
  widget is focused is not) and is a separate, measured decision, staged
  behind Q-1. The keyboard ownership table in
  `contributing/semantic-accessibility.md` is rewritten to reflect this,
  retiring the 14-minor-stale "Out of v0.20 scope (deferred, RFC-014-B)"
  row rather than adding beside it. Purely additive: new vocabulary in
  `snora-core`, one new helper in `snora::keyboard`, no existing type,
  field, or signature changed, no new iced feature enabled.

### Changed

- **Compile-time measurement gains a noise-controlled trend column,
  `design_overhead_ratio` (RFC-050).** The six absolute-millisecond
  columns `measure-compile-time.sh` has recorded since v0.20 **were
  never a trend signal** — they vary 36–60% across five releases sharing
  runner, rustc and methodology, and a documentation-only release
  (0.33.1, RFC-057, zero code changed) moved every one of them 36% to
  55%. That is runner noise, not snora. `design_overhead_ratio` =
  `example_workbench_ms / example_hello_ms`, computed same-run from
  measurements already collected (no new build), cancels it: 2.5%
  spread on the same five releases, and it held through 0.34.0 (RFC-058,
  which *did* change code) moving every absolute column −11% to −21% in
  the opposite direction. The trend watch points move to the ratio; the
  absolute 30-second ceiling on `build_widgets_ms` stays, unchanged,
  since it is a developer-experience ceiling, not a trend. Two ratios
  considered and rejected on the evidence: `widgets_design_ratio`
  (14.9% spread post-fix, worse than several raw columns) and any ratio
  built from the two sub-second columns (`build_engine_only_ms`,
  `build_widgets_design_ms` — dominated by process startup, not compiled
  code). Historical CSV rows are **not** edited or padded (RFC-041 N-1);
  the next appended row is the first with all eleven fields. Gate 9b
  (`api-freeze-review.md`) does **not** close with this change — it
  closes when ≥2 released versions carry `design_overhead_ratio`.

## [0.34.0] — 2026-08-18

### Fixed

- **`border` contrast repaired in `light` and `dark` (RFC-058).** snora's
  contrast suite asserted twelve pairs; `border` was in none of them, and
  the untested role was failing WCAG 2.1 SC 1.4.11's 3.0:1 minimum for
  non-text boundaries that identify a component — a border is that
  boundary for the RFC-039 dialog card, which was deliberately chosen as
  border-defined rather than shadow-defined. Reported by tekstide,
  verified independently. Measured before repair (`cargo test -p
  snora-design`, failing):

  | preset | border/background | border/surface | border/surface_raised |
  |---|---|---|---|
  | `light` | 1.39:1 | 1.28:1 | 1.39:1 |
  | `dark` | 1.43:1 | 1.32:1 | 1.19:1 |

  After (`light` border `#898C8F`, `dark` border `#69717D`, chosen to clear
  the *binding* pair per preset — `surface` for `light`, `surface_raised`
  for `dark` — not merely the easiest one):

  | preset | border/background | border/surface | border/surface_raised |
  |---|---|---|---|
  | `light` | 3.38:1 | 3.12:1 | 3.38:1 |
  | `dark` | 3.81:1 | 3.50:1 | 3.17:1 |

  `high_contrast_light` and `high_contrast_dark` are unchanged — they
  already pass at 19.8–21:1. This is the first exercise of RFC-036's
  accessibility carve-out: the assertion was added and its failure
  captured *before* the palette values changed, per the carve-out's
  required order. **This is an appearance change**, not a silent fix —
  borders in `light`/`dark` render visibly more present. See
  [the 0.33→0.34 migration guide](docs/src/guides/migration-0.33-to-0.34.md).

  **A second role, `text_muted`, was found untested in the same pass and
  is fixed here too.** `Palette`'s doc comment previously claimed
  `text_muted` was "exempt from the mandatory body-text contrast checks"
  — an exemption WCAG 2.1 SC 1.4.3 does not grant (its exemptions are
  incidental/decorative/invisible text, logotypes, and large text at
  3:1; rendered, readable "non-essential" text is not among them). That
  invented exemption is why the role went untested, and it is withdrawn:
  `text_muted` is now asserted at `AA_TEXT` against all three surfaces,
  the same as every other text role. Measured before repair:
  `light/surface` **4.4626:1** (failing). After (`light`'s `text_muted`
  repaired to `#6A717E`): `light/surface` **4.55:1**,
  `light/{background,surface_raised}` **4.93:1**. `dark` is **unchanged**
  — its worst pair, `dark/surface_raised`, already passes at **4.53:1**,
  and RFC-036's carve-out permits a value change only where a contrast
  test proves a defect; there is none in `dark`. That margin is thin
  enough to note here as a fact for the next palette edit to respect, not
  a surprise to rediscover. `text_secondary/surface_raised`, the one
  remaining unasserted pair from the same body-text family, is also now
  asserted (all four presets pass with wide margin, 7.6:1 or better) —
  a free ratchet completing the class, not a repair.

### Changed

- **The checklist's 3.0:1 non-text-boundary rule generalized (RFC-058).**
  It previously existed only under *Focus visibility*, attached to the
  `focus` role — which is why a second role (`border`) carrying the same
  obligation went unassessed for several releases. Moved into *Contrast*
  as a rule about the class of non-text boundaries.

- **Two answers snora already had, moved to where a consumer can find
  them (RFC-059).** tekstide evaluated snora end to end and declined to
  adopt it; neither finding was about missing capability.

  **The `BLOCKED` focus-ring claim was over-scoped, at four sites.** A
  review label instructed reviewers to record a missing focus ring as
  `BLOCKED (iced 0.14 — no focus variant in button::Status)` and
  explicitly "do not file it as a bug." That is true only for a widget
  that lets **iced** own focus. It is not a property of iced 0.14: a
  `container` style closure is an arbitrary `Fn(&Theme) -> Style`, so an
  application that already owns focus as its own state can style it
  today — colour *and* width — with no snora change. Re-scoped at all
  four sites carrying the claim, the most important of which
  (`design/iced-style-bridge.md`) was not in this RFC's own original
  scope list — found by a grep the RFC's own review ran, the same class
  of miss RFC-056 and RFC-058 each had once. `FocusTokens`
  (`crates/snora-design/src/focus.rs`) is now documented with this
  present-day audience, not only a future one.

  **The token-surface stability guarantee had no consumer-facing page.**
  RFC-036's additive-only covenant answers "does the token surface
  churn?" with a contractual "no" — and tekstide named exactly that
  question as *"changing our calculus more than any feature would,"*
  then declined partly for want of an answer that already existed in
  `contributing/api-governance.md`. New page:
  [`design/stability.md`](docs/src/design/stability.md), linked from
  `feature-flags.md` and `tokens.md`, states what is frozen, what is
  not, and — explicitly — that the covenant does not promise
  upgrade-mechanical-painlessness, only that the surface itself does not
  move.

  **The documentation-scope rule widened a second time.**
  `feature-gating-criteria.md`'s rule (RFC-048, widened to cover removals
  by RFC-056) now covers a third case: a governance answer that exists
  only in a contributor document. `release-process.md` now points at it,
  so following the release checklist reaches the rule without anyone
  having to remember it exists.

  Documentation only: `git diff --stat -- 'crates/**/*.rs'` shows one
  file (`focus.rs`), doc-comment lines only.

## [0.33.1] — 2026-08-15

### Changed

- **Typography is now discoverable (RFC-057).** snora has carried a six-role
  text scale — `body`, `body_small`, `label`, `title`, `heading`, `display`,
  each with a size **and** a line-height multiplier — since v0.20. It is
  tested, demonstrated in the design workbench, and fully usable today through
  public API with no change to snora. Nothing told you it existed: there was no
  typography page, typography was absent from the book's navigation entirely,
  the consumer accessibility guide said nothing about text, and the README's
  only mention was a disclaimer. Two new pages fix that —
  [`design/typography.md`](docs/src/design/typography.md) for the vocabulary
  and [`guides/readability.md`](docs/src/guides/readability.md) for the task —
  and the accessibility guide links to the latter rather than absorbing it,
  since accessibility and readability are different concerns.
- **Corrected a false claim that line-height was unusable.** Four places said,
  in effect, that `TextRole.line_height` was vocabulary-only because iced 0.14
  does not expose line-height. It does:
  `iced::widget::text::LineHeight::Relative` takes exactly the multiplier each
  role stores. One of the four was an item in the **contributor accessibility
  checklist** — a review gate instructing reviewers to skip line-height, which
  is why the gap survived several design releases.

  **Nothing about rendering changed.** snora's own prefab widgets still use two
  of the six roles (`label` and `body`), and the notice still renders its title
  at `label_size`. The typography page says so plainly rather than implying a
  hierarchy snora does not render. Applying more of the scale to snora's own
  chrome is deferred — it is an appearance change, and the one consumer
  exercising those widgets has an earlier appearance change still unadopted.

  Documentation only: the two changed `.rs` files carry doc-comment lines and
  nothing else.

## [0.33.0] — 2026-08-15

### Removed

- **`snora_widgets::design::style` and `snora_widgets::design::theme`
  removed (RFC-056).** RFC-055 (0.32.0) relocated the iced style bridge
  to a new peer crate, `snora-style`, and kept these two paths as
  compatibility re-exports so nothing broke mid-move. Its precondition
  for removing them is met: `snora::design::style::*` and
  `snora::design::theme` — the documented consumer route through the
  `snora` facade — already point at `snora-style` directly and are
  **completely unaffected** by this change. Removed rather than
  deprecated: `#[deprecated]` on a bare `pub use` re-export emits no
  warning at all in this workspace (verified — a deprecation cycle
  would have required wrapping the re-export in a local module purely
  to carry the attribute, machinery for a warning the hypothetical
  audience would likely never see anyway, since no documentation ever
  directed anyone to depend on `snora-widgets` directly). Breaking only
  for direct `snora-widgets` consumers importing these paths, of which
  none are known. See
  [the 0.32→0.33 migration guide](docs/src/guides/migration-0.32-to-0.33.md)
  for the two-row old→new path table.

## [0.32.0] — 2026-08-15

### Changed

- **Extracted the iced style bridge into its own crate, `snora-style`
  (RFC-055). No import path moved.** `snora-widgets/src/design/style/`
  and `snora-widgets/src/design/theme.rs`
  — six modules mapping `Tokens` to plain `iced` style structs or a
  complete `iced::Theme`, no `Element`, no widget — had three consumers
  (the prefab widgets, the engine chrome's dialog card, and
  applications via `snora::design::style::*` / `snora::design::theme`)
  while living inside one of them. RFC-054's investigation established
  the style layer is structurally *below* the widget layer (it imports
  nothing from the widget layer; widget-layer modules import it), so
  its placement in `snora-widgets` was the accident — `theme` has the
  identical property and joined the move for the same reason, one
  review round later. Moved to a new peer crate, `snora-style`,
  depending on `iced` and `snora-design` only. Both
  `snora_widgets::design::{style::*, theme}` and
  `snora::design::{style::*, theme}` re-export it at their existing
  paths — **every import that worked before this release still works,
  unchanged; no migration guide is needed.**

  **`design` no longer requires `widgets`.** `snora --features design`
  (with `default-features = false`) now compiles — `design::render`,
  `design::responsive_render`, `design::style::*`, and `design::theme`
  are reachable without pulling in the ~46 KB `snora-widgets` crate a
  consumer with zero `snora::widget::*` call sites never used. This is
  the configuration `examples/responsive_body` (v0.30.0) was built to
  match and could not itself run — and, per the review that shaped this
  entry, the reporting consumer uses `theme` specifically with zero
  widget call sites, which is why it moved alongside the style
  functions rather than staying gated behind `widgets`. The
  prefab-widget re-exports under `snora::design` (`widget`, `button`,
  `card`, `notice`, `chip`, `progress`) still require `widgets` and are
  `#[cfg]`-gated for it.

  **The default configuration is unaffected — proven by measurement,
  not assertion.** `snora-style` stays an optional dependency of
  `snora-widgets`, gated by the same `design` feature as before
  extraction: `size_probe_widgets` and `size_probe_design` are
  byte-for-byte identical before and after this change.

  **`snora_widgets::design::{style::*, theme}` are now compatibility
  re-exports**, not deprecated in this release. The `snora::design`
  paths applications actually use are re-exported independently,
  directly from `snora-style`, not routed through the widgets path;
  deprecating the widgets-crate paths is a separate future decision,
  recorded here so it stays visible rather than becoming silent debt.

## [0.31.0] — 2026-08-15

### Added

- **`snora::design::responsive_render` — the `design`-path pair to
  `snora::responsive_render` (RFC-053).** `snora::responsive_render`
  renders through the engine path unconditionally, so a `design`-path
  application that adopted it silently lost the styled dialog card and
  the token-derived modal dim — including the `high_contrast_dark`
  visibility fix RFC-039 shipped. Reported by apimokka, whose entire
  0.28 adoption existed to deliver that accessibility fix, and who
  therefore could not adopt width exposure at all without regressing
  it. `snora::design::responsive_render(build, &tokens)` wraps
  `snora::design::render` the same way the engine-path function wraps
  `snora::render` — same shape, `&Tokens` as a second argument, no new
  composition path. `snora::responsive_render`'s own documentation now
  states plainly that it renders through the engine path.

### Fixed

- **`measure-compile-time.sh`'s per-measurement clean never invalidated
  `release`/`release-baseline` artifacts (RFC-052).** `cargo clean -p
  <package>` reaches the dev profile only — `-r`/`--release` and
  `--profile <NAME>` are separate, required flags. Four of the six
  compile-time columns build `release` or `release-baseline`; two of
  them, `build_engine_only_ms` and `build_widgets_design_ms`, ran
  immediately after another `release`-profile measurement in the same
  script invocation and silently rode on its still-warm artifacts
  instead of measuring a rebuild — confirmed by `Compiling`-line
  evidence, not by a timing delta, since snora's small crates can
  compile fast enough either way that the numbers alone don't show it.
  Specifically, `snora-core` — the one crate with no feature-set
  variation between measurement steps — was the only one left stale by
  the bug; the other crates in each measurement were already being
  rebuilt for unrelated reasons (a different top-level package, or a
  changed feature flag) and so were never actually spared by it. Found
  while answering RFC-050's Q-1.
  Fixed by cleaning all three profiles unconditionally before every
  measurement, decoupling a measurement from its own profile so a
  future one cannot be added with the wrong clean. **Rows before this
  fix and after it are not comparable** for `build_engine_only_ms` and
  `build_widgets_design_ms` — no historical row is edited or
  back-filled — and gate 9b's compile-time trend closure condition
  resets again, the third such discontinuity after RFC-043 and
  RFC-044. See `docs/src/reference/build-cost-budget.md`'s RFC-052 note
  for the full evidence.

## [0.30.0] — 2026-08-15

### Added

- **`examples/responsive_body` — an engine-only responsive example
  composed the way snora's known consumers actually build (RFC-051).**
  `responsive_render` shipped in v0.28.0 because apimokka asked for it,
  but every existing demonstration teaches it through `AppLayout::side_bar`
  built with `snora::widget::app_side_bar` — apimokka uses neither (zero
  `snora::widget::*` call sites; `side_bar` is on their ignored list).
  The new example varies `body`'s own composition (a tab bar) by width
  instead, with no prefab widgets and no `side_bar`/`footer` anywhere in
  its source — compiler-enforced via `default-features = false`, not
  merely by discipline. For readers who compose their own chrome into
  `body`, this is the copyable one; `examples/responsive` (slot-based,
  unchanged) remains the one for `side_bar` collapse.

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

[Unreleased]: https://github.com/nabbisen/snora/compare/0.42.0...HEAD
[0.42.0]: https://github.com/nabbisen/snora/compare/0.41.1...0.42.0
[0.41.1]: https://github.com/nabbisen/snora/compare/0.41.0...0.41.1
[0.41.0]: https://github.com/nabbisen/snora/compare/0.40.0...0.41.0
[0.40.0]: https://github.com/nabbisen/snora/compare/0.39.3...0.40.0
[0.39.3]: https://github.com/nabbisen/snora/compare/0.39.2...0.39.3
[0.39.2]: https://github.com/nabbisen/snora/compare/0.39.1...0.39.2
[0.39.1]: https://github.com/nabbisen/snora/compare/0.39.0...0.39.1
[0.39.0]: https://github.com/nabbisen/snora/compare/0.38.3...0.39.0
[0.38.3]: https://github.com/nabbisen/snora/compare/0.38.2...0.38.3
[0.38.2]: https://github.com/nabbisen/snora/compare/0.38.1...0.38.2
[0.38.1]: https://github.com/nabbisen/snora/compare/0.38.0...0.38.1
[0.38.0]: https://github.com/nabbisen/snora/compare/0.37.2...0.38.0
[0.37.2]: https://github.com/nabbisen/snora/compare/0.37.1...0.37.2
[0.37.1]: https://github.com/nabbisen/snora/compare/0.37.0...0.37.1
[0.37.0]: https://github.com/nabbisen/snora/compare/0.36.1...0.37.0
[0.36.1]: https://github.com/nabbisen/snora/compare/0.36.0...0.36.1
[0.36.0]: https://github.com/nabbisen/snora/compare/0.35.0...0.36.0
[0.35.0]: https://github.com/nabbisen/snora/compare/0.34.0...0.35.0
[0.34.0]: https://github.com/nabbisen/snora/compare/0.33.1...0.34.0
[0.33.1]: https://github.com/nabbisen/snora/compare/0.33.0...0.33.1
[0.33.0]: https://github.com/nabbisen/snora/compare/0.32.0...0.33.0
[0.32.0]: https://github.com/nabbisen/snora/compare/0.31.0...0.32.0
[0.31.0]: https://github.com/nabbisen/snora/compare/0.30.0...0.31.0
[0.30.0]: https://github.com/nabbisen/snora/compare/0.29.0...0.30.0
[0.29.0]: https://github.com/nabbisen/snora/compare/0.28.1...0.29.0
[0.28.1]: https://github.com/nabbisen/snora/compare/0.28.0...0.28.1
[0.28.0]: https://github.com/nabbisen/snora/compare/0.27.1...0.28.0
[0.27.1]: https://github.com/nabbisen/snora/compare/0.27.0...0.27.1
[0.27.0]: https://github.com/nabbisen/snora/compare/0.26.0...0.27.0
[0.26.0]: https://github.com/nabbisen/snora/compare/0.25.3...0.26.0
[0.25.3]: https://github.com/nabbisen/snora/compare/0.25.2...0.25.3
[0.25.2]: https://github.com/nabbisen/snora/compare/0.25.1...0.25.2
[0.25.1]: https://github.com/nabbisen/snora/compare/0.25.0...0.25.1
[0.25.0]: https://github.com/nabbisen/snora/compare/0.24.0...0.25.0
[0.24.0]: https://github.com/nabbisen/snora/compare/0.23.0...0.24.0
[0.23.0]: https://github.com/nabbisen/snora/compare/0.22.0...0.23.0
[0.22.0]: https://github.com/nabbisen/snora/compare/0.21.0...0.22.0
[0.21.0]: https://github.com/nabbisen/snora/compare/0.20.0...0.21.0
[0.20.0]: https://github.com/nabbisen/snora/compare/0.19.1...0.20.0
[0.19.1]: https://github.com/nabbisen/snora/compare/0.19.0...0.19.1
[0.19.0]: https://github.com/nabbisen/snora/compare/0.18.3...0.19.0
[0.18.3]: https://github.com/nabbisen/snora/compare/0.18.2...0.18.3
[0.18.2]: https://github.com/nabbisen/snora/compare/0.18.1...0.18.2
[0.18.1]: https://github.com/nabbisen/snora/compare/0.18.0...0.18.1
[0.18.0]: https://github.com/nabbisen/snora/compare/0.17.0...0.18.0
[0.17.0]: https://github.com/nabbisen/snora/compare/0.16.0...0.17.0
[0.16.0]: https://github.com/nabbisen/snora/compare/0.15.0...0.16.0
[0.15.0]: https://github.com/nabbisen/snora/compare/0.14.0...0.15.0
[0.14.0]: https://github.com/nabbisen/snora/compare/0.13.0...0.14.0
[0.13.0]: https://github.com/nabbisen/snora/compare/0.12.0...0.13.0
[0.12.0]: https://github.com/nabbisen/snora/compare/0.11.0...0.12.0
[0.11.0]: https://github.com/nabbisen/snora/compare/0.10.0...0.11.0
[0.10.0]: https://github.com/nabbisen/snora/compare/0.9.0...0.10.0
[0.9.0]: https://github.com/nabbisen/snora/compare/0.8.0...0.9.0
[0.8.0]: https://github.com/nabbisen/snora/compare/0.7.0...0.8.0
[0.7.0]: https://github.com/nabbisen/snora/releases/tag/0.7.0
