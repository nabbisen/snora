# Implementation handoff — RFC-104: chrome label sizes and toast colours ignore the host

**RFC.** `rfcs/accepted/104-chrome-label-sizes-and-toast-colours-ignore-the-host.md`
— read its Rulings: Q-3 was settled by measurement, and the `ChromeStyle` gap
was found before this handoff.
**Release target.** 0.52.0. **RFC-105 starts after this is committed** (both
change `toast.rs` and `CHANGELOG.md`). RFC-103 may run in parallel; do not stage
its files.
**Rulings.** Q-1 (a): unstyled labels drop their literals and inherit
`default_text_size`. Q-2: styled labels map to `Typography` roles. Q-3: the
theme's warning replaces the literal. Q-4: the scan allow-lists named constants
only.
**Touch list.** `crates/snora-widgets/src/{tab,crumb,menu,header}.rs`,
`design/widget.rs` and its `tests.rs`, `crates/snora/src/{toast,render}.rs`,
`crates/snora/src/design/render.rs`, a new `scripts/check-literal-sizes.sh`,
`.github/workflows/ci.yaml` (one step), new tests, `CHANGELOG.md`. **Not**
`snora-design` / `snora-style`: `Typography` is read, never changed. **Not**
`docs/`.

## Already established

- **Literal sites:** `tab.rs` (13), `crumb.rs` (13 ×3), `menu.rs` (14 ×2),
  `header.rs` (16, bold), `toast.rs` (16 / 14). Re-grep to be sure.
- `Typography` (in `snora-design`) is a struct of `TextRole`s: `body`,
  `body_small`, `label`, `title`, `heading`, `display`, each with `size` and
  `line_height`. `snora_style::text` has `*_size` / `*_line_height` helpers for
  each.
- iced 0.14's `default_text_size` defaults to **16**. So unstyled tabs and crumbs
  go 13 → 16, and menus 14 → 16, unless the application sets it.
- **The design path cannot reach toasts today.** `design::render` calls
  `render_with_style(layout, &chrome_style(tokens))`, `ChromeStyle` holds only
  `dim_color` and `dialog_card`, and `render_toasts` takes no style.
- **Warning contrast, measured:** stock `palette.warning` against black is
  6.05:1 (Light) and 12.99:1 (Dark); against `extended_palette().warning.base.text`
  it is 6.05 / 8.12:1. The tokens' `warning` against `warning_text` is
  5.43–14.85:1.

## Unit 1 — failing-first

- **(a) Labels follow the host, unstyled.** Render each chrome widget under
  `iced_test::Simulator::with_settings`, at two different `default_text_size`
  values, and assert that the label's rendered height changes between them.
  **Today: fails**, because the literals ignore the setting.
- **(b) Labels follow the tokens, styled.** Render with two `Tokens` whose
  `typography.label` (and whichever role each label maps to) differ, and assert
  that the rendered label size follows. **Today: fails.**
- **(c) Toast on both paths.** Title and message follow `default_text_size`
  (default path) and the mapped roles (design path).
- **(d) Warning colour.** The toast's Warning fill equals the theme's warning
  (default path) and the tokens' warning (design path), and the text on it clears
  AA. **Today: the equality fails** against the literal.

**Measure text, not boxes.** A fixed-size parent can hand a text widget a box
bigger than its glyphs (RFC-099). Compare natural sizes, or rendered heights of
text-only elements, and say which you used.

## Unit 2 — sizes

- **Unstyled:** remove `.size(..)` from those labels. Keep the header's bold
  weight, which is a style and not a size.
- **Styled:** carry a label size through each widget's geometry, as spacing
  already is, from a single mapping table. Suggested roles: tab, crumb and menu
  → `label`; header title → `title`. The table records any role that does not fit
  cleanly, with the reason.
- **Toast:** extend `ChromeStyle` with the toast's title and message sizes
  (`None` = inherit the default on the default path; the tokens' `title` and
  `body` roles on the design path), and pass it into `render_toasts`.

## Unit 3 — toast colours

- **Default path:** Warning uses `extended_palette().warning.base.color` with its
  `.text`. Leave the other intents' existing pairings alone unless your
  measurement says otherwise.
- **Design path:** every intent uses the tokens' status pair (`success` /
  `warning` / `danger` / `info` with their `*_text`), carried through
  `ChromeStyle`.
- **Contrast asserted** for every intent on both paths, in the toast's contrast
  suite. **RFC-093's toast channel register must still pass**: intents still
  vary by colour only. **RFC-101's colour-inheritance check** for the close glyph
  must still hold.

## Unit 4 — the scan (R-4, Q-4)

`scripts/check-literal-sizes.sh`, in the `check-*.sh` family (same shape as
`check-wcag-floors.sh`, including `|| true` on any grep that may match nothing,
and why).

- **Fails** on `.size(` whose argument begins with a digit, in non-test
  `crates/snora-widgets/src` and `crates/snora/src`.
- **Fails** on `.size(` with an **upper-case constant argument not in the
  script's allow-list**, so that `const X: f32 = 13.0` cannot bypass the rule.
- **Allow-list:** named constants only, each with a one-line reason in the
  script, e.g. `CLOSE_GLYPH_SIZE` (a glyph metric). Token helper calls
  (`snora_style::text::…`) are allowed structurally.
- Wired into CI's `rust-quality` beside `check-wcag-floors.sh`, and
  `check-workflows.sh` run on the edit.
- **Shown failing** three ways: a scratch `.size(13)`, a scratch
  `const NEW_SIZE: f32 = 13.0` used in `.size(NEW_SIZE)`, and a scratch
  allow-list entry removed while its constant is still used.

## Appearance — measure and report, for the migration guide

For each widget: label size before and after, on the unstyled path at iced's
default (16) and on the styled path in all four presets. Report any change in
bar, row or toast **height** that follows, and the toast's Warning fill before
and after on both stock themes.

## CHANGELOG

- **Fixed:** chrome labels ignored the host's text size, on both paths (REQ-002,
  stated by tekstide; also a defect for current consumers). The toast's Warning
  fill is from the theme.
- **Changed:** unstyled label sizes now follow `default_text_size`. Say
  **exactly how to restore the old look**, e.g. set `default_text_size` to 13.
  Check what that does to other text, and say so.

## Acceptance criteria

1. (a)–(d) exist and are **shown failing** against today's code.
2. No literal text size remains outside the allow-list, and the scan enforces
   it in CI, **shown failing** three ways.
3. Toast colours come from the theme or tokens, with contrast asserted for every
   intent on both paths; RFC-093's register and RFC-101's colour check still pass.
4. `ChromeStyle` carries the toast's sizes and colours; no new public API unless
   stated and justified.
5. Appearance measured and reported; the covenant diff is quoted and empty;
   nothing under `docs/`.
