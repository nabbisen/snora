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

### Added

- **`Palette::text_contrast` and `TextContrast` (RFC-129).** The level of contrast a palette holds
  its text to: `TextContrast::Aa` (4.5:1) or `TextContrast::Aaa` (7:1), with `TextContrast::ratio()`.
  `light` and `dark` are `Aa`; `light_aaa`, `dark_aaa` and both high-contrast presets are `Aaa`. It is
  the level snora aims every hovered and pressed label at, **not a certificate of the resting
  colours**: changing `accent` on `light_aaa` leaves `Aaa`, and lowering it is an explicit act
  (`palette.text_contrast = TextContrast::Aa`). It lives on the palette so that
  `tokens.palette = Tokens::light_aaa().palette` brings the level with the colours. There is no
  `Default`: a palette starts from a preset. `Palette` is `#[non_exhaustive]`, so reading and
  assigning the field is all that changes for you.
- **`snora_style::chip::{selected, unselected}`, also at `snora::design::style::chip` (RFC-129).**
  The chips' two style functions, `fn(&Tokens, button::Status) -> button::Style`, beside
  `snora_style::button::*`. They were private to `snora-widgets` and moved to the crate that holds
  the style functions (RFC-055); `design::chip::filter` and `removable` draw with them, so nothing
  about a chip changes but what the Fixed entry below lists. Use them to style your own button as a
  chip.

### Changed

- **A notice's action is drawn as a ghost action, in the accent colour, for every tone
  (RFC-128).** Its label used to take the notice's tone colour, which is not a text colour: on
  `Neutral` it read 3.12:1 on `light` and `light_aaa` and 3.50:1 on `dark` and `dark_aaa`, under the
  4.5:1 that text needs, and on `light_aaa` the `Success`, `Warning` and `Danger` labels read 6.51:1
  and `dark_aaa`'s `Danger` 6.54:1, under the 7:1 those presets promise for text. The tone still
  shows in the notice's bar and border. **If you keep screenshot baselines, retake those of notices
  with an action:** the label is now the accent in every tone. No API changes. A new test draws
  every notice action, button and chip label at rest on every neutral surface, in every preset, and
  fails under the preset's floor (the hovered and pressed states are RFC-129's).

### Fixed

- **A hovered or pressed label could fall under its preset's contrast level, and one chip state
  drew nothing (RFC-129).** Every variant shifted its fill or label the same way on every preset
  (lighten when hovered, darken when pressed), so on a preset where that moves toward the other
  colour the label lost contrast, and only the resting pair was ever checked. Measured on each
  neutral surface (`background`, `surface`, `surface_raised`), **17 groups of a preset, a variant and
  a state** are fixed. A state now keeps the shift it has always made where that holds the
  preset's level (nothing that held it moves), and takes the first of a fixed list of other shifts
  where it does not, and always differs from rest by a visible step (ΔE76 of at least 3). The
  cells that change:
  - `light`: `secondary` hovered (4.47:1 against 4.5, on `surface`): the label is now darker, 6.71.
  - `dark`: `secondary` and `ghost` pressed (3.73): the label is now lighter, 5.24.
  - `light_aaa`: hovered `primary`, `secondary`, `ghost` and `danger` (6.23, 5.07, 6.21, 5.90 against
    7); pressed `secondary` and `ghost` (6.84); pressed unselected chip (6.75).
  - `dark_aaa`: pressed `secondary`, `ghost` and `danger` (4.53, 4.53, 5.93); hovered `ghost`
    (6.09); pressed selected chip (5.96).
  - `high_contrast_dark`: the unselected chip, hovered and pressed. It drew **exactly** like rest
    (its `surface` is black and the chip darkened it), so nothing showed it was hovered or pressed.
    Its contrast stays above 7:1 (14.71 to 15.33).
  `button::link` is drawn with `ghost`'s colours, so its label and underline move with `ghost`
  (hovered on `light_aaa`: 6.21 → 7.08).
  A `secondary` button draws its border in the label's colour, so its border moves with the label
  and stays above 3:1 in every state. **If you keep screenshot baselines, retake those with a
  hovered or pressed button or chip on the presets above.** A new test draws every button, chip and
  notice action in every state on every neutral surface and fails under the preset's level or under
  the visible step. The `high_contrast_*` presets' other states were already at 7:1.
- **A sheet on the right had the spacer's width, not the width you asked for.** A sheet whose edge
  is on the right (`SheetEdge::End` under left-to-right, `SheetEdge::Start` under right-to-left)
  has been laid out with the width of the empty space beside it since 0.4.0. So `OneThird` drew
  two thirds of the window and `TwoThirds` drew one third; `Half` hid it, and `Pixels(…)`, the
  top and the bottom were right. It is fixed: **a right-side sheet now has the width you asked
  for.** If you chose `TwoThirds` to get a third of the window on the right, or `OneThird` to get
  two thirds, you compensated for the bug; change the size back to the one you meant. A sheet on
  the left, at the top or at the bottom is unchanged. (Found by the 0.62 audit.)
- **F6 after a click counted from no zone.** With nothing focused, F6 and Shift+F6 went to the
  first (or last) zone with a control, even right after a click on a control in another zone,
  though Tab carries on from that click (RFC-121). F6 now counts from the zone of the control
  last clicked, as Tab does: click in the sidebar, press F6, and focus goes to the body. A press
  anywhere that is not a control, or a move of focus by F6, ends the starting point as it does
  for Tab. (Found by the 0.62 audit.)

## [0.61.0] — 2026-11-12

**Upgrading from 0.60:** read the [0.60 → 0.61 migration guide](docs/src/guides/migration-0.60-to-0.61.md).

> **In short:** F6 and Shift+F6 now move focus between the frame's zones (header, sidebar,
> body, footer), from wherever focus is. Before 0.61 the documented pattern moved no focus at
> all. A snora control can be given an id, so an application can choose where focus starts,
> for example on a dialog's "Cancel".
>
> **Code:** **nothing breaks.** `next_zone`, `FocusZone` and `ZonePresence` are deprecated (they
> warn, still work, and are removed in 0.64.0). Replace them with `snora::keyboard::focus_zone`.
>
> **Appearance:** unchanged.
>
> **Security:** none. No dependency changed since 0.60.0; the advisory gate is clean, with the
> same accepted advisories.

### Added

- **`.id()` on `snora::control` and `snora::design::control` (RFC-127).** An application can
  now choose where focus starts: give a control a `widget::Id` and return iced's own
  `iced::widget::operation::focus(id)` (`is_focused(id)` reports it). The book had said to do
  this, and a snora control could not take an id. A control focused this way behaves as one
  reached by Tab: its ring is drawn, Enter and Space press it, and Tab carries on from it. The
  task can come from the same `update` that opens a dialog. An id that no reachable control has,
  and so a control under a dialog, a sheet or a menu's backdrop, focuses nothing and leaves
  nothing focused. No snora-specific focus API is added.
- **`snora::keyboard::focus_zone(cycle)` (RFC-126).** F6 and Shift+F6 now move focus. Route the
  key as you route Tab, in two lines and with no state of your own:
  `if let Some(cycle) = snora::keyboard::cycle_zones(key, modifiers) { return
  snora::keyboard::focus_zone(cycle); }`. Focus goes to the first control of the next (or
  previous) zone of the frame (header, sidebar, body, footer, wrapping, in logical order under
  RTL too), counted from the zone that holds focus when the key is pressed. A zone with no
  control that can take focus is skipped; with focus in no zone (nothing focused, a toast, an
  inline context menu) F6 goes to the first zone and Shift+F6 to the last. **F6 lands only on a
  zone's own controls, never inside an open menu**; an open header dropdown belongs to its
  trigger's zone, so F6 from one of its items moves on. Under a dialog or sheet, a menu with its
  backdrop, or an open header dropdown with a close message, F6 does nothing. It never presses
  anything; held, it keeps moving, one zone per repeat, as Tab does.

### Deprecated

- **`snora::focus::next_zone`, `FocusZone` and `ZonePresence` (RFC-126), removed in 0.64.0.**
  Use `snora::keyboard::focus_zone`. `next_zone` only returned a zone: it could not move focus,
  it counted from a zone your application held (so it was wrong after a Tab or a click), it
  ignored an open menu, and it stopped on a zone with nothing to focus. `Cycle` and
  `cycle_zones` stay: they are the binding.

### Fixed

- **F6 zone navigation moved no focus (RFC-126).** Following the book, F6 changed a variable and
  focus stayed where it was; an application could not write the move, because a snora control
  had no id to name. It also counted from a stale zone, ignored a covering menu, and stopped on
  zones with nothing to focus. All four are fixed by `focus_zone`.

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

[Unreleased]: https://github.com/nabbisen/snora/compare/0.61.0...HEAD
[0.61.0]: https://github.com/nabbisen/snora/compare/0.60.0...0.61.0
[0.60.0]: https://github.com/nabbisen/snora/compare/0.59.0...0.60.0
