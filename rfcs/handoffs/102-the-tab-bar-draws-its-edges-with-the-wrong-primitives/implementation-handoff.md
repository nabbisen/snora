# Implementation handoff — RFC-102: the tab bar draws its edges with the wrong primitives

**RFC.** `rfcs/accepted/102-the-tab-bar-draws-its-edges-with-the-wrong-primitives.md`
— read its *Rulings*, including the `bar_border_radius` consequence found after
acceptance.
**Release target.** 0.51.0, with RFC-100 and RFC-101.
**Rulings.** Q-1 (a) square hover fill within the tab · Q-2 (a) each variant's
own tooltip idiom · Q-3 0.51.0.
**Touch list.** `crates/snora-widgets/src/tab.rs`, `sidebar.rs`, `style.rs` (only
if a shared style helps), `design/widget.rs` and its `tests.rs`,
`contrast_tests.rs`, new rendered tests under `crates/snora/tests/`,
`CHANGELOG.md`. **Not** `snora-design`, `snora-style` or `docs/`; R-6, correcting
the 0.41 guide, is the architect's.
**No file overlap with RFC-100 or RFC-101**, so this can proceed in parallel. Do
not stage their files.

---

## Already established, so you do not redo it

All measured by the architect in the six themes of `contrast_tests.rs`'s
`theme_contexts()`:

| Theme | `primary.base` vs page | `primary.strong` vs page | `background.base.text` vs page |
|---|---|---|---|
| stock Light | 4.61 | **3.73** | 21.00 |
| stock Dark | **2.99** | **3.70** | 11.00 |
| design light / dark | 6.70 / 6.78 | 12.64 / 10.00 | 17.81 / 16.89 |
| design hc_light / hc_dark | 10.25 / 11.75 | 15.03 / 17.70 | 21.00 / 21.00 |

- Inactive labels are `mix(background.base.text, background.base.color, 0.3)`.
  They always have been, contrary to the 0.41 guide. **Do not change the label
  colours.** They are not in scope.
- iced 0.14 has **no per-side border**. Both edges must become elements.
  `iced::widget::rule::horizontal(px)` exists, and its `rule::Style` has `color`,
  `radius`, `fill_mode` and `snap`. A `container` with a background at a fixed
  height also works. Choose one, and say why.
- `bar_border_radius` (unstyled `0.0`, styled `Radius::sm`) is only ever visible
  through the border. It is **retired** by this RFC.
- **The tooltip overlay is invisible to `Simulator::find`** (RFC-100 §4). Use
  RFC-100's rendered frame-hash comparisons, within a single run, for anything
  about drawing, and style-function assertions for contrast.
- Every file in all four items is unchanged since at least 0.10.0.

## Unit 1 — failing-first

Record each against the current code:

- **(a) Indicator contrast.** In `contrast_tests.rs`, assert the **underline's**
  colour against `background.base.color` at `NON_TEXT_MIN`, in all six contexts,
  and **remove the "skipped by `assert_border_contrast`" comment**. **Today this
  fails on stock Dark (2.99).** Show it.
- **(b) The underline is a straight element.** By rendered bounds: an element
  under the active tab that is exactly 2 px tall and spans the tab's width. Its
  style has zero radius and the button's style has **no shadow**. Today: fails,
  because no such element exists.
- **(c) The bar has only a bottom rule.** The bar container's border width is 0.
  A 1 px rule spans the bar's width at its bottom. Today: fails.
- **(d) Tooltip body contrast.** Assert the tooltip text colour against the
  tooltip body's **own** background at 4.5:1 in all six contexts, for both the
  unstyled and styled bodies, and assert that the body's border meets
  `NON_TEXT_MIN` against the page. Today: fails to compile, because there is no
  body. Say so, as RFC-100 did for its absent API.
- **(e) Tooltip drawn with a body.** Frame hashes, hovered: with a tooltip versus
  the same frame hovered on an empty tooltip string. Or use whatever comparison
  isolates the body from the text, and say which.

## Unit 2 — the two edges

- **Underline:** `render_tab` returns the button above a 2 px rule. The active
  tab's rule is `primary.strong`. **Inactive tabs get a 2 px transparent spacer**,
  so labels do not shift vertically when the active tab changes. Assert that in a
  rendered test: every tab's label sits at the same y.
- **Bar:** the row of tabs above a 1 px rule in `background.base.text`, which
  keeps RFC-085 F-15's value. `tab_bar_container_style` loses its border.
  `tab_bar_container_style_border_meets_non_text_floor` becomes an assertion on
  **the rule's colour**, so F-15's guarantee moves rather than disappears.
- **Where the underline sits relative to the bar's rule.** Choose adjacent or
  overlapping, so the active underline "breaks" the bottom rule as the old comment
  intended. Show the rendered result in your report; this is a visual decision
  you are best placed to make with a screenshot.
- **Retire `bar_border_radius`.** Remove the field. Update the unstyled literal
  test (`design/widget/tests.rs` ~l.70), the mapping test (~l.193) and whatever
  reads it at ~l.240. **Read that last use before removing it** and say what it
  was checking.

## Unit 3 — hover (Q-1 a)

`tab_button_style`'s `(false, Hovered)` arm keeps `background.weak`, with
**`radius: 0`**. The active arm needs no radius either, since nothing it draws
depends on one. `tab_button_style_text_meets_aa` stays as it is, because the
label-on-`background.weak` pairing it already asserts is unchanged.

## Unit 4 — sidebar tooltip body (Q-2 a)

- `build_side_bar` is shared, and only the styled caller has tokens. Carry the
  body choice through `SideBarGeometry`, or equivalent, with **one
  implementation** and **no new public API**. Say which mechanism you chose.
- **Unstyled body:** `background.base.color` fill, a 1 px `background.base.text`
  border, `background.base.text` text, and padding you state and justify from the
  sidebar's existing literals.
- **Styled body:** `snora_style::container::card_raised(tokens)`, padding
  `[xs, sm]`, label-size text and gap `xs`, **matching RFC-100's
  `with_close_tooltip`**. The two design tooltips then look alike. You cannot call
  RFC-100's `pub(super)` helper from `widget.rs`; reproduce the four values and
  add a comment at each site pointing at the other. If RFC-100 has landed by then
  and a shared `pub(crate)` helper is cleaner, you may extract one; say so.
- **Position is unchanged** (direction-aware `Right`/`Left`).

## Covenant

Quote `git diff <base> -- crates/snora-design crates/snora-style` and show it is
empty.

## Appearance — measure and report, for the migration guide

For each of: unstyled tab bar, styled tab bar, sidebar tooltip.

- Give a one-line description of the before and after. Name the styled bar's
  change separately: a rounded outline becomes a bottom rule.
- Confirm the bar's **height** change, if any. The new 1 px rule and 2 px
  underline row may change it by a few pixels, so measure it.
- Stock Light's underline goes from 4.61 to 3.73. Measure the visible difference
  and state it as a fact.

## CHANGELOG

Under **Fixed**, crediting orbok for items 1–4:

- straight underline; bottom rule only; hover within the tab; tooltip body with
  asserted contrast;
- **the active-tab indicator was 2.99:1 on stock `Theme::Dark`**, under the
  3.0:1 non-text floor, and all four design presets were unaffected. It is now
  `primary.strong`, at 3.70:1 or better in every theme. The widget contrast suite
  had skipped it because it was drawn as a shadow;
- `bar_border_radius` retired: the styled tab bar is no longer a rounded outline.

## Acceptance criteria

1. Indicator contrast asserted in all six themes, **shown failing on stock Dark
   today**; the skip comment removed.
2. Straight 2 px underline element, no shadow; labels at constant y across
   active/inactive.
3. Bar border removed; a 1 px bottom rule exists, and F-15's test asserts its
   colour.
4. `bar_border_radius` removed and its tests updated, with the ~l.240 use
   explained.
5. Square hover within the tab.
6. Tooltip body on both variants; text contrast against the body's own
   background, and border contrast, asserted in all six themes; body drawing shown
   by frame hash.
7. Empty covenant diff quoted; appearance measured and reported; CHANGELOG as
   above; nothing under `docs/`.
