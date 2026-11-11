# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

While the crate version is below 1.0, breaking changes are signaled by a
**minor** bump (e.g. `0.7.0` → `0.8.0`); patch releases (`0.7.0` → `0.7.1`)
are bug fixes and additive changes only.

**This file holds the current series, 0.60 onward.** Each closed series of
ten minors has its own file, which does not change (RFC-107):

| Series | File |
|---|---|
| 0.50 – 0.59 | [`changelog/0.5x.md`](changelog/0.5x.md) |
| 0.40 – 0.49 | [`changelog/0.4x.md`](changelog/0.4x.md) |
| 0.30 – 0.39 | [`changelog/0.3x.md`](changelog/0.3x.md) |
| 0.20 – 0.29 | [`changelog/0.2x.md`](changelog/0.2x.md) |
| 0.10 – 0.19 | [`changelog/0.1x.md`](changelog/0.1x.md) |
| 0.7 – 0.9 | [`changelog/0.0x.md`](changelog/0.0x.md) |

History begins at 0.7.0. Earlier release notes are recorded in the
per-version migration guides under [`docs/guides/`](docs/src/guides/).

## [Unreleased]

Nothing yet.

## [0.60.0] — 2026-11-11

**Upgrading from 0.59:** read the [0.59 → 0.60 migration guide](docs/src/guides/migration-0.59-to-0.60.md).

> **In short:** ↓ ↑ Home and End work inside snora's menus, and a covered page gets no
> pointer. Under a dialog, a sheet, or a menu with its backdrop, the page shows no hover
> and does not scroll, and **a wheel over a dialog's dim no longer closes the dialog**,
> which it did once per wheel event.
>
> **Code:** **one breaking change:** `snora::keyboard::dismiss_on_escape` (deprecated since
> 0.57) is removed. Use `AppLayout::escape`. No `#[deprecated]` item remains.
>
> **Appearance:** changes on the default path: a page under a dialog, a sheet, or a menu
> with its backdrop shows no hover and no hand cursor. Colours and presets are unchanged.
>
> **Security:** none. No dependency changed since 0.59.0; the advisory gate is clean, with
> the same accepted advisories.

### Added

- **Arrow keys, Home and End in snora's menus (RFC-123).** In an open header menu or context
  menu, ↓ and ↑ move focus between the items and wrap at both ends, and Home and End go to the
  first and the last. With nothing focused, ↓ starts at the first item and ↑ at the last (opening
  a menu still moves no focus). They are answered only while focus is on an item, on the menu's
  trigger or nowhere, so a text field, a page control or a toast's close control keeps its keys;
  an answered key is captured; Shift, Ctrl and Alt with an arrow are left alone; a menu under a
  dialog stays silent. No wiring: an application that renders through `snora::render` has them,
  as it has Escape. `render_menu` placed outside `snora::render` still gets Tab, Enter and Space
  only.

### Changed

- **A page under a dialog, a sheet, or a menu with its backdrop shows no hover, no pointer
  shape and no tooltip (RFC-125).** One rule for every cover: *a covered layer gets no
  pointer.* Whatever the keyboard cannot reach (RFC-119), the pointer cannot reach either.
  It was already so under an open header dropdown (0.59); it is now so under a dialog, a
  sheet, a context menu and an application's `header_menu` with its backdrop, where the page
  showed hover and the hand cursor, promising a click the cover would not let through. A
  slider dragged when a dialog opens no longer moves while covered, and the release still
  ends the drag. Toasts are never covered, and window events and redraws still reach a
  covered layer, so an animation beneath keeps running.

### Removed

The one item deprecated in 0.57.0 (RFC-116 R-2), whose note said "removed in 0.60.0".
**The rule:** a deprecated item is removed in the third minor after the one that
deprecated it.

- **`snora::keyboard::dismiss_on_escape`** (deprecated since 0.57.0). Use
  `AppLayout::escape(Escape { modal, menu })`, which answers Escape on the overlay's own
  layer with named fields. An overlay of your own, outside `AppLayout`, needs a three-line
  `match` on `Key::Named(Named::Escape)`. No `#[deprecated]` item remains in snora.

### Fixed

- **A wheel scroll over a dialog's or a sheet's dim closed it, once per wheel event
  (RFC-125).** The dim published `on_close_modals` on a wheel, so a two-finger swipe that
  strayed off a long dialog closed it, with whatever the reader had typed, and a swipe's many
  events published many close messages. A wheel over the dim, or a menu's backdrop, now
  publishes nothing, scrolls nothing beneath and is captured. A wheel over the dialog's own
  card still scrolls the card.
- **A wheel over a context menu's or an application `header_menu`'s backdrop scrolled the
  page beneath it.** It no longer does.

[Unreleased]: https://github.com/nabbisen/snora/compare/0.60.0...HEAD
[0.60.0]: https://github.com/nabbisen/snora/compare/0.59.0...0.60.0
