# Migrating between versions

Snora is in pre-1.0 SemVer: minor version bumps may carry small
breaking changes when justified, with deprecation aliases bridging
two consecutive releases where possible. Each minor release ships a
focused migration guide describing exactly what to change and why.

This page is the index — pick the guide that matches your jump.

## Per-version guides

- [0.41 → 0.42](migration-0.41-to-0.42.md) — **rendered appearance
  changes on both theme paths.** Menu text, sidebar highlight, chrome
  borders, tab labels, and breadcrumb text all had contrast failures no
  suite could see — a new widget-layer contrast suite found and fixed
  them. `high_contrast_dark` went from the worst preset (1.51:1) to the
  best (9.96:1 worst case). Reference images are invalidated.
- [0.40 → 0.41](migration-0.40-to-0.41.md) — **breaking: a real bug
  fixed.** Overlays did not contain pointer input — a click inside a
  dialog dismissed it, a modal with no close sink blocked nothing, and
  clicking a toast pressed the widget beneath it. All four fixed; no
  configuration flag restores the old (buggy) behavior.
- [0.39 → 0.40](migration-0.39-to-0.40.md) — **possibly breaking, for a
  narrow case.** `lucide-icons` no longer transitively enables iced's
  `advanced` feature (or pulls in `iced`/`winit` at all in `snora-core`);
  an application relying on that transitive edge for its own
  `iced::advanced::` calls needs to enable `advanced` itself, one line.
  Fixes a docs.rs build failure on `snora-core`.
- [0.38 → 0.39](migration-0.38-to-0.39.md) — **nothing required.**
  `snora::focus` re-exports the zone-navigation vocabulary, so F6 users can drop
  a direct `snora-core` dependency; and our explanation of what makes the dialog
  card visible against the modal dim was wrong and is corrected. No value, no
  rendered output, no API removed.
- **0.37 → 0.38 — known gap, guide not yet written** (RFC-079). Predates
  this page's rule being enforced; not backfilled, per the owner's
  ruling — deferred, not forgotten.
- [0.36 → 0.37](migration-0.36-to-0.37.md) — the `design`-path modal dim's
  alpha repaired for WCAG 2.1 SC 1.4.11 (was 2.85:1 in `light`, now
  >= 3.0:1 in all four presets) — an accessibility fix, not a restyle;
  the unstyled/engine path unaffected.
- [0.35 → 0.36](migration-0.35-to-0.36.md) — `chip::removable`'s dismiss
  button gained a fixed minimum width — a WCAG 2.5.8 pointer-target-size
  repair (measured 15.0px wide against the shipped font, now ~24.8px
  square); an appearance change, not an API change.
- **0.34 → 0.35 — known gap, guide not yet written** (RFC-079). Same
  status as 0.37 → 0.38 above.
- [0.33 → 0.34](migration-0.33-to-0.34.md) — `light`/`dark`'s `border` colour
  repaired for WCAG 2.1 SC 1.4.11 (was 1.19-1.43:1 against surfaces, now
  >= 3.0:1) — an accessibility fix, not a restyle; `high_contrast_*`
  unaffected.
- [0.32 → 0.33](migration-0.32-to-0.33.md) — `snora_widgets::design::{style,
  theme}` removed; use `snora_style::*` directly, or
  `snora::design::style::*`/`snora::design::theme` (unaffected — the
  documented consumer route already pointed at `snora-style`).
- **0.31 → 0.32, 0.30 → 0.31, 0.29 → 0.30 — known gaps, guides not yet
  written** (RFC-079). Crossed by a real jump this month (knotra,
  0.25 → 0.39), so not merely historical — but not urgent, and nobody
  is blocked; deferred by the owner, same as the other gaps above.
- [0.28 → 0.29](migration-0.28-to-0.29.md) — `snora-dialog-card` was
  attached to the wrong element since v0.27.0; re-pointed to the actual
  card, with a new `snora-dialog` identifier for the centring container
  it used to name.
- [0.27 → 0.28](migration-0.27-to-0.28.md) — layout width exposure
  (`responsive_render`) and stable identifiers on snora-rendered surfaces.
  Both additive; identifier names become a compatibility surface.
- [0.26 → 0.27](migration-0.26-to-0.27.md) — appearance milestone
  completed: dialog card, derived modal dim, token-derived chrome
  geometry. Opt-in **per call site**, not per feature flag.
- [0.25 → 0.26](migration-0.25-to-0.26.md) — `snora::design::theme`
  emits an `iced::Theme` from Snora Design tokens, so stock iced widgets
  follow your palette; DEC-02 amended to permit theme *emission*.
- [0.24 → 0.25](migration-0.24-to-0.25.md) — measurement methodology and
  documentation cleanup (size-probe crates; build-cost clean fix).
- [0.23 → 0.24](migration-0.23-to-0.24.md) — architect-review cleanup;
  **one breaking change** (chip selected state).
- [0.22 → 0.23](migration-0.22-to-0.23.md) — recipes and governance. No
  code changes.
- [0.21 → 0.22](migration-0.21-to-0.22.md) — code-quality and
  documentation audit. No breaking changes.
- [0.20 → 0.21](migration-0.20-to-0.21.md) — notice, chip and progress
  primitives added to Snora Design.
- [0.19 → 0.20](migration-0.19-to-0.20.md) — Snora Design activation;
  `snora-design` published.
- [0.18 → 0.19](migration-0.18-to-0.19.md) — Snora Design foundation
  (opt-in `design` feature). Additive.
- [0.10 → 0.11](migration-0.10-to-0.11.md) — `AppLayout` is now
  `#[non_exhaustive]` (builder path is the stable construction contract);
  toast ordering fix (newest toast now correctly closest to the anchor
  edge).
- [0.6 → 0.7](migration-0.6-to-0.7.md) — removal of the deprecated 0.6
  sheet aliases (`BottomSheet`, `SheetHeight`,
  `AppLayout::bottom_sheet`); two new navigation widgets (`Tab`,
  `Crumb`).
- [0.5 → 0.6](migration-0.5-to-0.6.md) — `BottomSheet` generalized to
  `Sheet` with four anchor edges; workspace split into three crates
  (`snora-core` / `snora-widgets` / `snora`).
- [0.4 → 0.5](migration-0.4-to-0.5.md) — `ToastPosition` introduced and
  defaulted to `TopEnd`; long-form documentation tree established.

If you are jumping more than one minor — for example 0.5 directly to
0.7 — apply each guide in order. The deprecation pattern means doing
them in sequence is generally less work than skipping any single
intermediate.

## What stays stable

These hold across the pre-1.0 line and are not expected to break
until 1.0:

- The shape of `AppLayout::new(body).header(...).side_bar(...)` and
  the rest of its builder chain.
- The "skeleton + injected slots" model — every slot is an
  `iced::Element`; no trait to implement.
- Logical edges (`Edge::Start`, `Edge::End`,
  `LayoutDirection::Ltr` / `Rtl`) and their direction-dependent
  resolution.
- The single-channel close-sink rules for overlays
  (`on_close_modals`, `on_close_menus`).

The full pre-1.0 versioning policy is summarized at the top of
[CHANGELOG.md](https://github.com/nabbisen/snora/blob/main/CHANGELOG.md).

## Writing a migration guide

Copy [`migration-template.md`](migration-template.md) to
`migration-X.Y-to-X.Z.md`, fill in the seven sections, and add a link
to the "Per-version guides" list above.

The [versioning policy](../contributing/versioning-policy.md) defines
when a migration guide is required and what changelog label to use.
