# RFC-102 — The tab bar draws its edges with the wrong primitives, and its state indicator is under the floor

**Status.** Accepted 2026-09-25. Q-1 = (a), Q-2 = (a), Q-3 = 0.51.0.
**Raised.** 2026-09-25, architect, on a report from **orbok** (four visual
defects, with screenshots). **Item 5 below was found by the architect while
verifying them.**
**Release target.** 0.51.0, alongside RFC-100 and RFC-101 (Q-3).
**Priority.** Medium. Four cosmetic defects, plus one conformance defect on one
theme.

---

## The report, verified in source

orbok's owner noticed four things in 0.50.0. All four are confirmed in
`crates/snora-widgets/src/tab.rs` and `sidebar.rs`. **All four date back to at
least 0.10.0**; the relevant lines are identical at the 0.10.0, 0.30.0 and 0.50.0
tags.

1. **The active tab's underline curls up at both ends.** It is drawn as a
   `Shadow` (offset `(0, 1.5)`, blur 0) under a button with `radius: 4.0`, so the
   shadow takes the rounded corners and ends in two upward ticks. The code's own
   comment says it is *"visually indistinguishable from a border-bottom in normal
   use"*. It is not, and orbok's 4.5× crop shows it.
2. **The whole bar has a 1 px outline.** `tab_bar_container_style`'s comment says
   *"Drop the top/left/right borders; keep only a thin bottom edge"*, but it sets a
   single `Border { width: 1.0, .. }`, and **iced 0.14 borders are all-sided**. The
   comment describes intent the code cannot express.
3. **Hover paints a box over the bar's edges.** An inactive tab hovers to a
   `background.weak` fill on the same `radius: 4.0` button, which covers the
   outline from item 2 (attachment 2).
4. **The sidebar's tooltip has no background.** `tooltip(btn, text(item.tooltip),
   ..)` draws bare text over whatever page content sits beside the rail
   (attachment 3: "Settings" over the page's "Theme" heading). **This is a
   legibility defect, not only a cosmetic one.** The tooltip's text contrast is
   whatever the page happens to have underneath, so nothing can guarantee it.

**Items 1–3 share one root cause:** the bar's two edges are faked with the wrong
primitives. Item 1 uses a shadow where a rule was meant, and item 2 uses an
all-sided border where one side was meant. Item 3 follows from item 2, because a
fill cannot paint over an edge that is not inside it.

## Item 5 — found while verifying: the active-tab indicator is under the non-text floor

Measured in all six themes the widget contrast suite covers:

| Theme | Underline (`primary.base`) vs page | Active label vs inactive label |
|---|---|---|
| stock Light | 4.61:1 | 2.46:1 |
| **stock Dark** | **2.99:1** | 1.77:1 |
| design light | 6.70:1 | 2.71:1 |
| design dark | 6.78:1 | 1.97:1 |
| design high_contrast_light | 10.25:1 | 2.46:1 |
| design high_contrast_dark | 11.75:1 | 2.11:1 |

**The underline is the active tab's state indicator, and on stock `Theme::Dark` it
is 0.01 under snora's 3.0:1 non-text floor** (WCAG 1.4.11: *"visual information
required to identify … states"*). The label difference cannot stand in for it: it
is below 3:1 everywhere, and it is a colour difference anyway.

### How it got there

- **0.41.0 (RFC-085) moved the state distinction onto the underline on purpose.**
  The label had been `primary.base.color`, measured at 2.99:1 on stock Dark as
  text, under 4.5. RFC-085 moved the label to `background.base.text`. The 0.40 →
  0.41 migration guide then told consumers that *"the underline (unchanged, still
  the theme's primary color) is the state indicator."*
- **So the indicator stayed on the colour just measured at 2.99:1**, now judged
  against the 3.0 non-text floor instead of 4.5. It misses by 0.01, and nobody
  measured it that way.
- **The widget suite cannot see it.** `tab_button_style_text_meets_aa` states that
  the underline is *"drawn via `shadow` instead — skipped by
  `assert_border_contrast`"*. The docstring calls it *"a decorative accent line
  [that] carries no text-contrast requirement"*. It indeed carries no **text**
  requirement. It carries a **non-text** one, because it is not decorative: the
  same sentence says it is what distinguishes the active tab.

**Scope of the defect:** applications rendering the tab bar with stock
`Theme::Dark`. All four design presets clear with wide margins. **No published
claim states the underline's contrast.** The 0.41 guide designated it the state
indicator without measuring it. orbok's record cites us for 1.4.11 and should
hear this directly, stated precisely: stock Dark only, not the design presets.

### A second, harmless falsehood in the same guide

The 0.40 → 0.41 guide says *"the active tab's label is now the same color as an
inactive one"*, and `tab_button_style`'s docstring says `background.base.text` is
*"the same value inactive tabs already use"*. **Both are false and were false when
published.** Inactive labels have used `mix(background.base.text,
background.base.color, 0.3)` since at least 0.10.0. The error is in the harmless
direction: a small colour distinction was never removed. But it is a false
statement in a published guide, in the one section written to explain a visual
trade-off.

## Proposal

**R-1 — draw the underline as its own element.** A 2 px rule beneath the active
label, spanning the tab, with square corners. This fixes item 1 structurally,
because an element has no inherited radius to curl. It also makes the indicator a
real element whose colour the widget suite can assert.

**R-2 — draw the bar's bottom edge as its own element.** A 1 px rule under the
bar, kept at `background.base.text` (RFC-085 F-15's measured value). Drop the
container's border. This fixes item 2, and makes item 3 impossible by
construction, since no tab's fill can extend over a rule outside the tab.
`tab_bar_container_style_border_meets_non_text_floor` moves to the rule's colour,
so F-15's guarantee is kept rather than dropped.

**R-3 — the indicator's colour is `primary.strong`.** Measured at **3.70:1 on
stock Dark and 3.73:1 on stock Light** (the worst cases), and 10.00–17.70:1 on the
design presets. It clears the floor everywhere. It is also the colour the
sidebar's active highlight already moved to in RFC-085, so the two navigation
widgets would share one indicator colour. **Assert it**: the suite checks the
indicator against the page background in all six themes, and the skip comment is
removed. Stock Light drops from 4.61 to 3.73, which still clears the floor, and
the migration guide must state that change.

**R-4 — give the sidebar tooltip a body.** A container with a background, a
border, padding, and text that meets 4.5:1 **against that background**, asserted
by the widget suite. After this, tooltip legibility is a property of snora's
style, not of whatever the application renders beside the rail. Shape: Q-2.

**R-5 — hover within the tab's own box.** Shape: Q-1.

**R-6 — correct the two false statements (architect's).** Annotate the 0.40 →
0.41 guide's table row and trade-off paragraph rather than rewriting them, and
state the correction in the 0.51 guide with what to re-check. That follows
RFC-067's rule for withdrawn claims.

## What this is not

- **Not a covenant change.** `tab.rs`, `sidebar.rs` and `style.rs` are in
  `snora-widgets`, outside RFC-036's frozen surface. `Spacing`, `Palette` and
  `snora_style` do not change; the proposal only chooses different existing theme
  colours. D-3/D-4 stand. Quote the empty diff.
- **Not a tab-bar redesign.** Geometry, gaps and label sizes are unchanged.

## Appearance

Every application rendering `app_tab_bar` sees the bar lose its top, left and
right outline and gain a straight underline, and on stock themes a different
underline colour. Every application with a sidebar sees tooltips gain a body.
**Visual baselines with either are invalidated.** Q-3 exists because of this.

## Open questions

**Q-1 — how does an inactive tab show hover?**

| Option | Shape | Consequence |
|---|---|---|
| **(a) Square fill within the tab** | `background.weak`, radius 0, above the bar's rule | Keeps today's hover affordance. With R-2 it can no longer cover an edge |
| (b) Label-colour only | Muted label → full text colour | Reads almost exactly like the active state minus its underline, blurring the one distinction R-3 makes reliable |
| (c) Keep today's rounded fill | Unchanged | After R-2 it no longer overlaps an edge, but a rounded pill above a straight rule is the look orbok reported |

**Suggest (a).** It changes the least and it cannot collide with R-2's rule.

**Q-2 — what does the sidebar tooltip body look like?**
`build_side_bar` is shared by the unstyled and styled variants, and only the
styled one has tokens.

| Option | Unstyled | Styled |
|---|---|---|
| **(a) Match each variant's existing idiom** | `background.base.color` fill, the chrome border (`background.base.text`, 1 px), text `background.base.text`, which is iced's own guaranteed pairing | `style::container::card_raised(tokens)`, the body RFC-100 chose for notice/chip tooltips |
| (b) One theme-derived style for both | As (a)'s unstyled column | Styled tooltips then differ from RFC-100's design tooltips |

**Suggest (a)**, so that a design application's tooltips all look alike.

**Q-3 — which release?** **Suggest 0.51.0**, bundled with RFC-100 and RFC-101.
All three invalidate visual baselines, and one release means consumers
re-baseline once instead of three times. No files overlap with either RFC. The
cost is a larger 0.51.0 for review.

## Acceptance criteria

1. The underline is its own element with square corners, spanning the active tab,
   by rendered bounds. There is no shadow in `tab_button_style`.
2. The bar has no container border, and a 1 px bottom rule exists by rendered
   bounds. F-15's contrast test now asserts the rule's colour.
3. **The indicator's contrast is asserted in all six themes, and shown failing
   against today's `primary.base` on stock Dark (2.99:1).** The "skipped"
   comment is removed.
4. The tooltip has a body, and its text contrast against its own background is
   asserted in all six themes. Because the tooltip overlay is invisible to
   `Simulator::find` (RFC-100), that is asserted on the style function, plus a
   rendered frame-hash check that the body is drawn.
5. Hover as ruled in Q-1, contained within the tab.
6. Empty covenant diff quoted; CHANGELOG under Fixed, crediting orbok, naming
   stock Dark precisely; nothing under `docs/` (R-6 is the architect's).

---

## Rulings, 2026-09-25

**Q-1 — (a).** Square `background.weak` fill within the tab, above the bar's rule.
**Q-2 — (a).** Each variant keeps its own idiom: unstyled uses a theme background,
the chrome border and `background.base.text`; styled uses `card_raised(tokens)`,
matching RFC-100's tooltips.
**Q-3 — 0.51.0**, bundled with RFC-100 and RFC-101, so consumers re-baseline once.
**orbok's reply** goes in the 0.51.0 release note, with the RFC-101 note it
already owes them, stating the stock-Dark finding precisely.

### A consequence found after acceptance

**The styled tab bar is a rounded outline**, not a plain box. `tab_geometry` maps
`bar_border_radius` to `Radius::sm`, and the bar has `background: None`, so that
radius was only ever visible **through the border**. Once R-2 removes the border,
`bar_border_radius` paints nothing. The styled bar's change is therefore larger
than the unstyled one's: a rounded outline becomes a bottom rule. **The field is
retired rather than left in place**, because a geometry field that configures
nothing is a dead setting that its mapping test would keep "verifying". The
migration guide states the styled variant's change separately.

---

## Post-review, 2026-09-26: the defect depended on the renderer

Implementation found that the active tab was **a filled `primary.base` block**,
with the label drawn on it, which put the label under AA in five of six themes.
orbok's real-application screenshot of the same 0.50.0 code shows only a thin
underline. **Both are right, on different renderers**, and the architect and the
dev team each confirmed this in the iced 0.14 sources:

- **wgpu** (`iced_wgpu-0.14.0/src/shader/quad/solid.wgsl:98`):
  `mix(quad_color, shadow_color, (1.0 - quad_alpha) * shadow_alpha)`, where
  `quad_alpha` is geometric coverage. The shadow never shows inside the quad's
  shape, so the result was a curled 1.5 px underline with the label on the page.
- **tiny-skia** (`iced_tiny_skia-0.14.0/src/engine.rs:90–124`): the shadow
  shape is filled before the quad and nothing excludes the interior, so behind
  a transparent button it filled the whole tab.

| Renderer | Reached by | Active tab | Label |
|---|---|---|---|
| wgpu | default with a working GPU | curled underline | on the page |
| tiny-skia | iced's fallback when wgpu cannot start, or `ICED_BACKEND` | solid `primary.base` | **under AA in 5 of 6 themes** |

**The indicator finding (item 5) holds on both renderers**, since it is a colour
pairing and 2.99:1 on stock Dark whether drawn as a band or a block. **The label
finding holds on tiny-skia only.** The root cause lies one level below both:
snora relied on a primitive the two renderers draw differently. The fix removes
the shadow, and `tab::tests::tab_button_style_draws_no_shadow` keeps it removed.

**Consequence for this project's evidence:** the test harness pins tiny-skia
(RFC-099), so pixel evidence describes the fallback path. `guides/testing.md`
now says so.

