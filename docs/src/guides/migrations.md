# Migrating between versions

Snora is in pre-1.0 SemVer: minor version bumps may carry small
breaking changes when justified, with deprecation aliases bridging
two consecutive releases where possible. Each minor release ships a
focused migration guide describing exactly what to change and why.

This page is the index — pick the guide that matches your jump.

## Per-version guides

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
