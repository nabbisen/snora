# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

While the crate version is below 1.0, breaking changes are signaled by a
**minor** bump (e.g. `0.7.0` → `0.8.0`); patch releases (`0.7.0` → `0.7.1`)
are bug fixes and additive changes only.

**This file holds the current series, 0.50 onward.** Each closed series of
ten minors has its own file, which does not change (RFC-107):

| Series | File |
|---|---|
| 0.40 – 0.49 | [`changelog/0.4x.md`](changelog/0.4x.md) |
| 0.30 – 0.39 | [`changelog/0.3x.md`](changelog/0.3x.md) |
| 0.20 – 0.29 | [`changelog/0.2x.md`](changelog/0.2x.md) |
| 0.10 – 0.19 | [`changelog/0.1x.md`](changelog/0.1x.md) |
| 0.7 – 0.9 | [`changelog/0.0x.md`](changelog/0.0x.md) |

History begins at 0.7.0. Earlier release notes are recorded in the
per-version migration guides under [`docs/guides/`](docs/src/guides/).

## [Unreleased]

### Changed

- **Release history is filed by series of ten minors (RFC-107).** This file
  now holds 0.50 onward. Earlier entries moved, unchanged, to
  `changelog/0.0x.md` … `changelog/0.4x.md`, linked from the table at the top.
  Every minor from 0.39 links its migration guide, and the compare links run
  through 0.52.0 again (they had stopped at 0.46.0).
- **Three entries revised after their release now say so.** 0.19.1, 0.20.0
  and 0.25.2 each carry a `*Corrected*` line naming the commit that changed
  them. From now on a released entry changes only that way, and CI checks it.

- **An ancestor breadcrumb is underlined (RFC-109).** An ancestor and the
  current page were the same colour by construction — an ancestor's resting
  `text_color` is `background.base.text`, and the leaf's plain text inherits
  the same value — so the only thing separating "you can click this" from
  "you are here" was word order. Ancestors now carry a 1px rule under the
  label, inside the button and spanning the label's width; the leaf has
  none, and that absence is now the cue.

  **This changes appearance on the default path**, with no opt-out (Q-3).
  The breadcrumb row grows by the rule's height: measured **32.80 → 33.80**
  at `default_text_size` 16, with the label itself unchanged at 20.80.
  Visual baselines that include a breadcrumb are invalidated.

  The rule's layout offset is fractional (26.80 at the default size), because
  it sits below a label whose height follows `default_text_size`. It asks to
  be snapped to the pixel grid, and **that takes effect on wgpu, the default
  renderer**, where it removes the smear a thin element otherwise gets;
  `iced_tiny_skia` ignores the request in iced 0.14, so a tiny-skia host sees
  no difference. Measured on tiny-skia, the rule's strongest row carries the
  full colour in every context — 21.00:1 on stock Light, 11.05 on stock Dark,
  16.89–21.00 across the four design presets — which is the peak row any 1px
  rule at a fractional offset has, not an effect of snapping.

### Added

- **A GitHub Release for each version, from 0.53.0 (RFC-107).** Its notes are
  the changelog entry, with links pointing at the tagged files. Each entry
  from 0.53.0 opens with four lines: what changed, whether your code must
  change, whether appearance changes, and whether there is a security item.

- **Contrast is read from the pixels that were drawn (RFC-108).** Every
  contrast assertion before this one read a style function's return value,
  which answers "is the colour we intend sufficient?" and not "is the colour
  that reached the screen sufficient?". Eight element groups are now measured
  from the rendered frame instead — the tab bar's bottom rule and the active
  tab's underline, the sidebar's active fill against its rail, the chrome
  container's border, the tooltip body's border on both paths, every toast
  intent's fill on both paths, and the chip's selected and disabled fills —
  each against the same floor its style test already used.

  Elements under two logical pixels are asserted twice: on the region's
  modal colour and on its strongest row, because a 1px rule at a fractional
  offset is spread across three physical rows at partial coverage. The tab
  bar's rule is exactly that case, and reads 4.00:1 as a region against
  21.00:1 at its core.

  `common::frame_pixels` is the instrument: it decodes the lossless PNG
  `iced_test` writes and **asserts which renderer drew it**, so a silent
  fallback cannot be reported as a pass. This does not yet make REQ-005 met;
  the default renderer is still untested in CI, which is a separate question.

  **The tab bar's rule and the active tab's underline are now snapped to the
  pixel grid.** Both are thin elements at fractional offsets, and on the
  default renderer that cost them their edges: measured on stock Light, the
  1px rule drew as a grey-black-grey band over three physical rows. With
  `snap` it draws as two solid rows. **The change is visible on wgpu only** —
  `iced_tiny_skia` ignores the flag in iced 0.14 — so an application using the
  default renderer sees slightly crisper chrome, and one rendering with
  tiny-skia sees no change at all. No colour moves either way.

### Fixed

- **A toast's text missed AA on six built-in themes (RFC-111).** iced derives
  each intent's fill and paired text from whatever palette the application
  sets, and snora's contrast suite only ever checked two of its 22 themes. On
  the other twenty, six pairs sat under the 4.5:1 floor:

  | Theme | Intent | Before | After |
  |---|---|---|---|
  | Kanagawa Dragon | Info | 2.42 | **8.69** |
  | Nord | Error | 4.25 | **5.13** |
  | Oxocarbon | Error | 4.36 | **5.30** |
  | Kanagawa Lotus | Success | 4.40 | **5.36** |
  | Solarized Light | Error | 4.44 | **4.63** |
  | Solarized Dark | Error | 4.44 | **4.63** |

  Kanagawa Dragon was the worst because `Info`'s text was the literal
  `Color::BLACK`, chosen against the two stock themes and then applied to
  every other one — on that theme a mid-blue fill.

  **The same literal reached applications that derive their theme from
  tokens.** An application that passes `snora::design::theme(&tokens)` to the
  unstyled `snora::render` drew Info at **1.40–1.66:1**; it is now 12.64 and
  15.03 on the two affected presets.

  **The unstyled toast now keeps the theme's own paired text wherever it
  clears AA, and falls back to whichever of black and white contrasts the
  fill more where it does not.** That fallback cannot itself fail: for any
  opaque colour the better pole contrasts at ≥ 4.583:1. `Info` takes the
  better of its pair and that pole, which on the stock themes is still black.

  **No change on stock Light or stock Dark** — all ten of their ratios are
  unchanged and are now pinned by a test, since a floor alone would not have
  noticed them moving. **The `design` path is unchanged**, value and rule
  both; it supplies its own status pairs and is swept rather than altered.

- **The modal dim did almost nothing on a dark theme (RFC-110).** The
  unstyled dim was black at 40% alpha, a literal, under a comment claiming
  "the gating invariant depends on this value never changing" — and nothing
  asserted it. Black over a nearly-black page barely changes it: measured
  against the undimmed page, **1.02–1.33:1 across iced's 16 dark themes**,
  under the 3.0:1 non-text floor, so a modal did not read as modal at all.
  The 6 light themes were short too, at 2.70–2.85.

  The unstyled dim now follows the theme, picking the pole opposite the page
  — white over a dark background, black over a light one — at alpha 0.44.
  That is the rule the `design` path has used since RFC-065, and the two now
  agree. Measured after: **3.74–4.36 on the 16 dark themes and 3.05–3.24 on
  the 6 light ones**, every one of the 22 clearing the floor.

  **The design path is unchanged**, value and rule both. What binds the
  unstyled dim is composition — which layer it occupies, what it blocks,
  what dismisses it — and that is asserted by `render_semantics.rs`, which
  passes unmodified.

## [0.52.0] — 2026-09-29

**Upgrading from 0.51:** read the [0.51 → 0.52 migration guide](docs/src/guides/migration-0.51-to-0.52.md).

### Added

- **Verbatim and bounded-work assertions (RFC-105).** A fixture
  containing a bidi override, a newline and a control character is
  passed through every caller-text surface in scope and found by exact
  content, which shows snora hands the string to iced unmodified — it
  says nothing about which glyphs iced then draws. And an `#[ignore]`d
  timing test, run in release by one CI step, asserts that laying out
  ten times as many toasts takes less than thirty times as long;
  measured 11.6–12.0 on a Ryzen 9 9950X, against 92 with a deliberate
  quadratic in place. REQ-003 and REQ-006, stated by **tekstide**.

- **An open menu's dropdown is asserted (RFC-103).** Nothing covered it:
  the menu tests in `render_semantics.rs` exercise the engine's menu
  layers, which is a different code path from `snora-widgets`'
  `render_menu`. `crates/snora/tests/menu_dropdown.rs` now asserts that
  an open menu draws its items and a closed one does not.

### Changed

- **The channel register now covers states, not only variants
  (RFC-103).** A test and documentation change; no behaviour changes.
  RFC-093's register asserts that a `Tone` varies a style by colour
  alone. The new state register asserts the opposite property for the
  states a user must be able to identify: that something other than hue
  distinguishes them (WCAG 1.4.1). It covers the active tab, the active
  sidebar item, an open menu, the breadcrumb leaf, a selected chip, a
  disabled chip and a disabled prefab button, with a compile error if a
  state is added without deciding what tells a user it is on.

  Newly asserted, in all four presets: **a selected chip differs from an
  unselected one by ≥ 3.0:1** (measured 6.19–11.75), and **a disabled
  selected chip differs from an enabled one by ≥ 2.6:1** (measured
  2.64–3.59), as does a disabled prefab button on the channel its family
  uses — fill for `primary` and `danger`, label for `secondary` and
  `ghost` — at ≥ 2.9:1 (measured 2.92–4.05).

  **One gap is recorded rather than closed**, because closing it is a
  design change: **a disabled *unselected* chip is indistinguishable
  from an enabled one.** Measured 1.00–1.05:1, with its text colour and
  border unchanged. The register pins this, so closing it fails the test
  and forces the record to be updated.

  REQ-004 in the consumer requirements register, stated by **tekstide**.

- **Unstyled chrome and toast label sizes now follow
  `default_text_size` (RFC-104),** which is 16 unless the application
  sets it. Measured at that default: tab and breadcrumb labels go 13 →
  16, menu labels 14 → 16, the toast message 14 → 16; the header title
  and toast title were already 16 and do not move. The rows they sit in
  grow with them — the tab bar 33.9 → 37.8, the breadcrumb row 28.9 →
  32.8, an open menu 56.4 → 61.6, and the toast surface 67.0 → 69.6.
  **Visual baselines that include any chrome are invalidated.**

  **To restore the old sizes**, set `default_text_size` to the old
  value — but note what that does: it applies to **all** text that
  inherits, including the application's own, and it cannot reproduce
  the old mix, which was four different literals (13 tab/crumb, 14
  menu/toast-message, 16 header/toast-title). An application that wants
  per-surface sizes back should use the `design` path and set the
  `Typography` roles, which is what they are for. On the styled path the
  sizes now follow those roles: at the shipped presets, tab and crumb
  labels are 14, the header and toast titles 18, and the toast message
  16.

### Fixed

- **A disabled unselected chip looked enabled (RFC-106).** Both disabled
  chip states dimmed only their fill, and an unselected chip's fill is
  `surface` at half alpha over a page that is nearly `surface` —
  measured **1.00–1.05:1** against the enabled fill, with its label and
  border unchanged. There was nothing to tell the two apart.

  **A disabled chip's label and border are now dimmed to 45% opacity**,
  the same factor a disabled button uses, so the two read alike. The
  unselected chip's label measures **3.84 / 3.18 / 6.03 / 4.50** across
  the four presets against its enabled state, clearing the 3.0 non-text
  floor in every one. The selected chip's fill remains its cue
  (2.64–3.59, unchanged), and its label and border dim for consistency.
  Fills are untouched.

  A disabled chip's label sits below AA against its own fill by design:
  WCAG 1.4.3 exempts inactive components, and the cue is that it dimmed.

  **Found by RFC-103's state register**, which recorded the gap rather
  than closing it, with a test that failed the moment it was closed.

- **Long unbroken text was silently cut off (RFC-105).** A file name, a
  path, a URL or a hash — anything without spaces — was truncated at the
  column with nothing to say so. Measured: 400 unbroken characters in a
  toast rendered 288.0px wide against a natural 3481.6px, on one line,
  with about 48 characters visible. iced's default wrapping breaks at
  spaces only, so it could not help a string that has none.

  **Toast titles and messages, and notice titles and bodies, now wrap
  between glyphs when a word cannot fit.** Ordinary prose is unaffected
  — measured identical before and after — but **a surface carrying such
  a string grows taller**, which changes the layout around it.
  REQ-003, stated by **tekstide**, and a defect for current consumers
  too.

  **Tooltips were cut off in the same way, and are fixed too** — the
  sidebar's on both variants, and the notice's and chip's close
  controls. A tooltip is sized by its content, so long text had nothing
  to wrap against but the window; each tooltip body now has a maximum
  width of 320px and wraps inside it. Measured by frame comparison,
  since tooltip text cannot be inspected directly: at 400 characters an
  extra character at the end used to make no difference to what was
  drawn, and now does. Short tooltips render byte-identically to
  before.

- **Chrome labels ignored the host's text size, on both paths
  (RFC-104).** The tab bar and breadcrumb drew their labels at 13, menus
  at 14, the header title at 16, and toasts at 16/14 — literals, so an
  application that set iced's `default_text_size`, or that shipped its
  own `Typography`, got snora's numbers anyway. Every one of those now
  comes from the host: `default_text_size` on the unstyled path, a
  `Typography` role on the styled one (tab, breadcrumb and menu labels
  → `label`; the header title and toast title → `title`; the toast
  message → `body`). REQ-002, stated by **tekstide**, and a defect for
  current consumers too.

  A new `scripts/check-literal-sizes.sh`, wired into CI, fails on a
  numeric `.size(..)` in non-test source and on an upper-case constant
  that is not allow-listed — so a `const` cannot quietly put one back.

- **The toast's Warning fill was a private constant (RFC-104).** It is
  now the theme's `warning` pair, fill and text together. The comment
  justifying the constant said iced's extended palette had no `warning`
  semantic; iced 0.14's does. Measured, the fill moves from
  `rgb(0.851, 0.467, 0.024)` to `rgb(0.718, 0.494, 0.200)` on stock
  Light and `rgb(1.000, 0.757, 0.306)` on stock Dark, and every intent's
  text clears AA against its own fill on both paths.

- **The `design` path could not reach toasts at all (RFC-104).**
  `ChromeStyle` carried only the modal dim and the dialog card, and the
  toast renderer took no style, so a design application's toasts used
  literal sizes and colours derived from the `iced::Theme` rather than
  from its own `Tokens`. `ChromeStyle` now carries the toast's sizes and
  its per-intent colours, which come from the token palette's status
  pairs (`info`, `success`, `warning`, `danger` with their `*_text`;
  `Debug` has no status token and maps to the neutral
  `surface_raised`/`text_primary` pairing).

## [0.51.0] — 2026-09-26

**Upgrading from 0.50:** read the [0.50 → 0.51 migration guide](docs/src/guides/migration-0.50-to-0.51.md).

### Added

- **`Notice::dismiss_tooltip` and `chip::removable_with_tooltip`
  (RFC-100).** Each shows a short text, such as `"Dismiss"` or
  `"Remove tag"`, above the close control while the pointer hovers it.
  Above rather than beside, because neither primitive knows the layout
  direction.

  **These are visual tooltips, not accessible names.** The text is not
  exposed to assistive technology: iced 0.14 has no accessible-name API
  for buttons, and snora has no accessibility tree. Do not cite either
  as the control's accessible name in a WCAG record.

  `chip::removable` is unchanged and still attaches no tooltip;
  `removable_with_tooltip` is the additive form of it, and both share
  one implementation. Requested by **orbok**.

### Changed

- **Under `design` + `lucide-icons` only, the notice dismiss control and
  the removable chip's remove control render lucide `X` instead of the
  text glyph `"×"` (RFC-100).** Both controls share one glyph, sized to
  the label text; the notice's glyph keeps its `text_primary` colour.
  **Visual baselines that include either control are invalidated, for
  applications with `lucide-icons` enabled only.** Without
  `lucide-icons`, both controls render `"×"` exactly as before.

  Pointer targets still clear WCAG 2.5.8's 24 × 24 minimum. Measured:
  the notice dismiss button is 27 × 28.2 with `"×"` and 34 × 28.2 with
  lucide `X`; the chip remove button is 24.8 × 26.2 in both, through
  RFC-061's existing width mechanism. Raised by **orbok**.

### Fixed

- **The tab bar drew both of its edges with the wrong primitives
  (RFC-102).** Four defects, all present since at least 0.10.0, all
  **reported by orbok**:

  - **The active tab's "underline" was a `Shadow`**, offset 1.5px down
    under a button with `radius: 4.0`, so it took the button's rounded
    corners and curled upward at both ends. It is now a 2px element with
    square corners, spanning the tab.
  - **The bar's "bottom edge" was an all-sided `Border`.** iced 0.14 has
    no per-side border width, so the whole bar was outlined — the code's
    own comment claimed it kept "only a thin bottom edge". The bar now
    has no border, and its bottom edge is a 1px rule element spanning
    its width.
  - **An inactive tab's hover fill painted over that outline.** The fill
    is now square and within the tab, and the bar's rule is outside
    every tab's box, so no tab's fill can reach it.
  - **The sidebar's tooltip had no body**, so its text landed on
    whatever the application rendered beside the rail and its contrast
    was the page's business, not snora's. It now has a body: the page
    background with the chrome border unstyled, `card_raised` styled —
    the same body RFC-100 gave the notice and chip tooltips. Text
    contrast against that body, and the body's border against the page,
    are asserted in every theme.

- **The active-tab indicator was 2.99:1 on stock `Theme::Dark`
  (RFC-102),** under snora's 3.0:1 non-text floor for the visual
  information that identifies a state (WCAG 1.4.11). It was
  `primary.base`; it is now `primary.strong`, measured at **3.70:1 on
  stock Dark and 3.73:1 on stock Light**, its two worst cases, and
  10.00–17.70:1 across the four design presets. As a colour pairing, all
  four design presets were already clear of the floor (6.70–11.75:1);
  **stock Dark was the only one under it.** On stock Light the indicator
  goes from 4.61:1 to 3.73:1, which still clears the floor.

  The widget contrast suite could not see it: the underline was drawn as
  a shadow, and the suite skipped it as decorative. The indicator is now
  an element with its own style function, asserted in all six theme
  contexts.

- **On the software renderer, the active tab was a filled block and its
  label was under WCAG AA (RFC-102).** Found while implementing, and not
  part of the original report. **This one depends on which renderer the
  application runs**, because iced 0.14's two renderers draw a shadow
  behind a transparent background differently:

  - Under **tiny-skia** — the software renderer iced falls back to when
    its GPU renderer cannot start (virtual machines, remote desktops,
    missing GPU drivers), or when `ICED_BACKEND` selects it — the shadow
    filled the whole tab, because nothing excluded the quad's interior
    and the active tab's background is `None`. The label was therefore
    drawn on `primary.base`, and that pairing is **under AA (4.5:1) in
    five of the six themes**: 3.67:1 stock Dark, 2.66:1 design light,
    2.49:1 design dark, 2.05:1 design high-contrast light, 1.79:1 design
    high-contrast dark. Only stock Light cleared, at 4.56:1.
  - Under **wgpu**, the default whenever a GPU backend starts, the
    shadow was drawn only outside the button's shape, so the label sat
    on the page background at the ratios the contrast suite reported,
    and the visible defect was the curled underline orbok reported.

  The contrast suite read the active tab's background from the style
  struct, where it is `None`, and so measured the label against the page
  in both cases — right for wgpu, wrong for tiny-skia.

  **This is fixed by the same change**, which removes the shadow
  entirely, so neither renderer draws anything behind the label. snora no
  longer uses a shadow as a visual element, and a test now asserts that
  no tab style draws one in any state.

- **`TabGeometry::bar_border_radius` is retired (RFC-102).** The bar has
  no background, so its radius was only ever visible through the border
  that RFC-102 removed; the field configured nothing. **For the styled
  variant this is the larger visual change: a rounded outline becomes a
  bottom rule.** The unstyled bar was already square.

  **Visual baselines that include a tab bar or a sidebar tooltip are
  invalidated.** The bar is 1px taller — the height of its new rule —
  and tab labels do not move.

- **The toast close button's pointer target was 25.00 × 23.40, under the
  24 × 24 minimum snora mandates (RFC-101).** It is now **24.00 × 24.00**,
  reached by centring the glyph in a box at that floor rather than by
  padding tuned against iced's line height.

  The button was `button(text("×").size(18)).padding([0, 8])`, whose
  vertical padding is zero, so its height was the glyph's line box —
  18 × 1.3 = 23.4. **snora's height assertions covered token-derived
  controls, and this control derives from neither tokens nor a shared
  style**: it is the only interactive control the engine renders itself.

  **No published claim covered it.** Whether a given application failed
  WCAG 2.5.8 depends on its own layout, because the success criterion's
  spacing exception can be met by the space around a control rather than
  by the control itself — so this is stated as a measured fact about
  snora's own geometry, not as a conformance verdict on anyone's
  application.

  The toast's own size is unchanged at 340 × 67, and the message column
  is **288px against 0.50.0's 287**, so no message wraps earlier than it
  did and nothing reflows. Toast visual baselines are invalidated: the
  button is 1px narrower and 0.6px taller.

- **Under `lucide-icons`, the toast close button renders lucide `X`
  (RFC-101),** matching the notice and chip controls RFC-100 moved.
  Without the feature it renders `"×"` exactly as before. Before this,
  the engine drew `"×"` whichever features were on — measured: the two
  feature states produced identical frames.

  The engine builds the glyph itself, because it does not depend on
  `snora-widgets`; the two sites carry comments pointing at each other,
  and a test asserts they draw the same codepoint as a lucide glyph.

  Opened from RFC-100's review on the owner's instruction; the
  observation came from the dev team.

## [0.50.0] — 2026-09-17

**Upgrading from 0.49:** read the [0.49 → 0.50 migration guide](docs/src/guides/migration-0.49-to-0.50.md).

### Fixed

- **The sidebar rail was narrower than its buttons, and the icon was
  not centred in them (RFC-099).** The rail is 64px wide and its buttons
  are meant to be 48px square, but the rail padded all four sides by
  16px, leaving a 32px content box. iced resolved that by rendering each
  button **32 × 48 instead of 48 × 48.** Separately, the icon had no
  centring container, so the glyph was drawn at the top-left of the
  button's content area — measured about 10px left of and 10px above
  the button's centre.

  **Both the unstyled and styled variants, in every preset, since at
  least 0.10.0.** The unstyled rail used the literal `16`, and every
  built-in preset's `Spacing::lg` is `16`, so nothing fit anywhere.

  **Found by orbok** — their owner noticed the rail looked wrong, and
  they traced it to source.

  The rail's horizontal padding is now derived from the rail and button
  sizes (`(64 − 48) / 2 = 8`) rather than taken from spacing; vertical
  padding and the inter-button gap keep their sources (literals
  unstyled, `Spacing` tokens styled). The icon is centred on both axes
  in a container, and the button's own padding is zeroed so the
  container spans the full 48 × 48. `Spacing` is unchanged.

  - **This is a rendered-appearance change on the default path.** Visual
    baselines that include a sidebar are invalidated.
  - **Rail width is unchanged**, so body content does not reflow.
  - **The pointer target is now 48 × 48** (was 32 × 48). Both sizes
    clear WCAG 2.5.8's 24 × 24 minimum; no conformance claim moves.

  The existing `side_bar_geometry_matches_mapping_all_presets` test
  passed on every one of those releases: it asserted that each geometry
  field was wired to the right token, not that the result fit. The new
  tests in `crates/snora/tests/side_bar_fit.rs` measure rendered layout
  instead — clicking just inside both edges of the intended button, and
  comparing the icon's rendered box with its natural size and the
  button's centre — for the unstyled variant and all four styled
  presets.

### Changed

- **Two documentation claims you may have relied on were wrong, and are
  corrected.**
  - `guides/testing.md` described `iced_test` as *"a CPU-only headless
    renderer."* **It tries wgpu first.** With several simulators starting
    at once, as a parallel `cargo test` does, that was observed to crash
    the test binary with SIGSEGV inside the system Vulkan loader. Cargo
    reports that with the same exit code as a failing assertion. snora's
    own suite now sets `ICED_TEST_BACKEND = "tiny-skia"` in
    `.cargo/config.toml`, and the guide explains the line — and its two
    caveats — for anyone writing simulator tests of their own.
  - `reference/threat-model.md` now states that snora's advisory
    disclosures describe **snora's resolved graph, not yours.** Your graph
    is a superset: enabling more of iced's features, or declaring a crate
    directly that reaches snora only through an uncompiled path, puts
    advisories in your graph that no list of ours can contain. It also
    explains why a lockfile scanner (`cargo audit`) reports entries a graph
    scanner (`cargo-deny`) does not.
- `anyhow` 1.0.102 → 1.0.104 in snora's lockfile (`RUSTSEC-2026-0190`,
  unsound). It is unreachable in snora's graph, and a library's lockfile
  does not reach yours — check your own with `cargo tree -i anyhow
  --target all`.

[Unreleased]: https://github.com/nabbisen/snora/compare/0.52.0...HEAD
[0.52.0]: https://github.com/nabbisen/snora/compare/0.51.0...0.52.0
[0.51.0]: https://github.com/nabbisen/snora/compare/0.50.0...0.51.0
[0.50.0]: https://github.com/nabbisen/snora/compare/0.49.0...0.50.0
