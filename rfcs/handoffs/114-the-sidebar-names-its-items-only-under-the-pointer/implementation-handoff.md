# Implementation handoff — RFC-114: the sidebar names its items only under the pointer

**RFC.** `rfcs/accepted/114-the-sidebar-names-its-items-only-under-the-pointer.md`.
Read its **"Correction, 2026-11-04"** first: keyboard users cannot reach the rail in
either form, because iced 0.14's buttons take no keyboard input. Do not write any
test, doc or comment that assumes otherwise.
**Release target.** 0.54.0.
**Rulings (2026-11-04):**
- three forms, `SideBarLabels::{Beneath, Beside, Tooltip}`, chosen by the
  application, with `Beneath` the default. See the RFC's "Amendment, 2026-11-04";
- the rail is content-sized, capped at 240 px, with the cap confirmed by your
  measurement;
- `app_side_bar` keeps its signature, and the `_with` variants take `SideBarLabels`.

**Order.** After RFC-113 is committed: both touch `CHANGELOG.md`. RFC-115's Unit 0
measures after this lands.

**Touch list.**
- `crates/snora-widgets/src/sidebar.rs`, `crates/snora-widgets/src/design/widget.rs`,
  `crates/snora-widgets/src/lib.rs`;
- `crates/snora/src/widget.rs`, `crates/snora/src/design.rs` (re-exports);
- `crates/snora-core/src/sidebar.rs` (R-3, the rustdoc only);
- `crates/snora/tests/`: `side_bar_fit.rs`, `sidebar_tooltip_body.rs`,
  `drawn_contrast.rs`, `caller_text.rs`, plus a new rendered test;
- `CHANGELOG.md` (`[Unreleased]`).

**Not:** `SideBar` or `SideBarItem` fields, and icon sizes (RFC-115). The architect
writes the migration guide, `accessibility.md` and `reference/widgets.md`.

## Already established

- **Today:** a 64 px rail of 48 × 48 icon buttons (`sidebar.rs:23,25`). Each name
  appears only in a hover tooltip, and iced opens tooltips only under the cursor
  (`iced_widget-0.14.2/src/tooltip.rs:224`).
- **The tooltip form's tests call `app_side_bar(...)` directly:**
  `sidebar_tooltip_body.rs:187,222,246,252`, `side_bar_fit.rs:125,141,259`,
  `drawn_contrast.rs:303,667` and `caller_text.rs:330`. Once `Beneath` is the
  default, those that test the **tooltip** form must call
  `app_side_bar_with(..., SideBarLabels::Tooltip)`. **That is the only change they
  may get** (acceptance criterion 3). Decide per test which form it is about, and
  list your decision for each.

## Unit 1 — failing-first

A rendered test on both paths: the default rail shows each item's name **at rest,
without hover**, found by exact text. It fails today.

## Unit 2 — R-1 and R-2

- **`SideBarLabels::{Beneath, Beside, Tooltip}`** in `snora-widgets`, re-exported beside
  `app_side_bar` from `snora::widget` and `snora::design::widget`.
- **`app_side_bar_with(side_bar, direction, labels)`** and
  `design::app_side_bar_with(tokens, side_bar, direction, labels)`. The existing two
  functions call these with `Beneath`.
- **The labelled items:**
  - `Beneath`: the icon, with the name centred beneath it;
  - `Beside`: icon at the start edge, then the name, mirrored under RTL.

  For both:
  - Each item is at least 48 px tall.
  - The active highlight spans the whole item row.
  - The name wraps (`Wrapping::WordOrGlyph`, RFC-105) and never clips.
  - There is no tooltip.
- **Text size:** on the unstyled path, inherit the host's `default_text_size`; on the
  design path, use `Typography::label`. This follows RFC-104, the same as the other
  chrome labels.
- **Width:** content-sized to the longest name (`Beneath`: the wider of the button
  and the name; `Beside`: icon + name), capped. One cap for both, proposed at 240 px.
  - **Measure first:** the rendered width of every rail in `examples/` at the default
    text size, at a host text size of 24 px, and on the design path with
    `Typography` scaled ×2.0, which is orbok's setting (letter of 2026-11-04).
  - **If 240 cuts names that a reader would expect on one line, report the figures
    and stop.** The cap is the RFC's proposal, not a measurement.

## Unit 3 — the label is text now, so it carries the text floor

The tooltip form never put caller text on the rail. The labelled form does, so:
- **contrast:** each label against the **resting** rail and against the **active
  highlight**, on both paths, all four presets and `Theme::ALL`, at ≥ 4.5:1. Read
  it from drawn pixels through RFC-108's `frame_pixels` route, and add it as a
  `drawn_contrast.rs` row.
- **If any pair misses, stop and report.** Do not tune colours inside this RFC.
- **caller text:** extend `caller_text.rs` so that the labelled rail's names are
  verbatim (tekstide's fixture) and wrap at the cap.

## Unit 4 — R-3 and R-4 docs (rustdoc)

- **`SideBarItem::tooltip`:** "The item's name. The labelled rail shows it beside the
  icon. The icon rail shows it only in a tooltip under the pointer. Keyboard users
  cannot reach the rail in either form, because iced 0.14's buttons take no keyboard
  input. No screen reader reads it." Keep the meaning; the wording is yours.
- **`SideBarLabels::Tooltip`'s doc:** it names items only under the pointer, so touch
  users get none.

## Check each assertion can fail

Back up to named files with `cp`, break, run, restore, then `sha256sum -c`. **Never
`git checkout`/`restore`.**

| Assertion | Breakage |
|---|---|
| names visible at rest (each labelled form) | make the default `Tooltip` |
| `Beneath` puts the name below the icon | swap it to `Beside`'s row: the name's top must be at or below the icon's bottom |
| label contrast | draw the label in the rail's own background colour |
| wraps, never clips | set the label to `Wrapping::None` |
| RTL mirrors (`Beside`) | ignore `direction` in the labelled row |
| tooltip form unchanged | the existing tooltip tests, unmodified apart from the `_with` call |

**If a breakage cannot make its row fail, replace the assertion and say so.**

## Acceptance criteria

1. Every row above shown failing, then passing.
2. Unit 2's width figures reported, and the cap confirmed or a stop reported.
3. The tooltip-form tests pass with only the `_with` call changed; the list is in your
   report.
4. CHANGELOG:
   - Changed: the default rail shows names, and is wider;
   - Added: `SideBarLabels`, the `_with` functions;
   - Fixed: the `SideBarItem::tooltip` doc claim.
