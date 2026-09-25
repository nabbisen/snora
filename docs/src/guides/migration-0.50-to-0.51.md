# Migration 0.50 → 0.51

> **No breaking API change, but rendered appearance changes in three places,
> and two accessibility defects are fixed that a conformance record may cite.**
> One addition is new API (two tooltip functions). Read §4 if you keep a WCAG
> record that cites snora.

## Who is affected

| You render… | What changes | Re-check |
|---|---|---|
| `app_tab_bar` (either variant) | Edges redrawn; indicator colour on stock themes | Visual baselines with a tab bar |
| `app_side_bar` | Tooltips gain a body | Baselines that capture a sidebar tooltip |
| Toasts | The close button's target grows to 24 × 24 | Baselines with a toast |
| `design` + `lucide-icons` | Notice dismiss and chip remove draw lucide `X` | Baselines with either control |

**For code: nobody needs to change anything.** If you compile without
`snora-widgets` (apimokka), only the toast row applies.

## 1. The tab bar (RFC-102)

Reported by orbok. Every item has been present since at least 0.10.0.

- **The active tab's underline was a shadow**, and it curled up at both ends.
  It is now a straight 2 px element spanning the tab.
- **The whole bar had a 1 px outline** where the code's comment promised only a
  bottom edge; iced 0.14 borders are all-sided. The bar now has **no outline and
  a 1 px bottom rule**.
  - **The styled (`design`) bar changes more:** it was a **rounded outline**
    (`Radius::sm`), and it is now the same bottom rule. `bar_border_radius` is
    retired, since it no longer draws anything.
- **Hover** is a square fill within the tab; it can no longer paint over the
  bar's edge.
- **Height: +1 px**, 32.90 → 33.90, from the new rule. Labels do not move.
- **The indicator colour is now `primary.strong`** (§4). On stock Light that
  means a slightly lighter blue (4.61:1 → 3.73:1, still above the 3.0 floor).

## 2. The sidebar tooltip has a body (RFC-102)

It used to be bare text drawn over whatever sat beside the rail. It now has a
background, a border and padding: the page background with the chrome border
(unstyled), or `card_raised` (styled, matching the notice and chip tooltips).
**Its text contrast is now asserted against its own background** in every
theme, instead of depending on your page.

## 3. Close controls (RFC-100, RFC-101)

- **Toast close button** (every application with toasts): the pointer target
  was **25.00 × 23.40**, under the 24 × 24 floor. It is now **exactly
  24.00 × 24.00**. The toast is unchanged at 340 × 67, and the message column is
  288 px (was 287), so no message wraps earlier. Idle frames change; hovered
  frames look like idle ones, as before.
- **Under `lucide-icons`**, the toast close button, notice dismiss and chip
  remove all draw lucide `X`. Without the feature, all three keep text "×",
  unchanged. Measured targets: notice 27 × 28.2 (text) / 34 × 28.2 (lucide);
  chip 24.8 × 26.2.
- **New API:** `Notice::dismiss_tooltip(..)` and
  `chip::removable_with_tooltip(..)`; `removable` is unchanged. **These are
  visual tooltips, not accessible names.** iced 0.14 has no accessible-name API
  for buttons, and snora has no accessibility tree. Do not cite them as
  satisfying a naming criterion.

## 4. For conformance records — what was wrong, on which renderer

**Two findings depend on which of iced's two renderers draws your application.**
iced uses **wgpu** by default when a GPU backend starts. It falls back to
**tiny-skia**, a software renderer, when wgpu cannot start (virtual machines,
remote desktops, missing GPU drivers), or when `ICED_BACKEND` selects it.

| Defect, fixed in 0.51 | wgpu (default) | tiny-skia (fallback) | Criterion |
|---|---|---|---|
| Active-tab indicator colour | **2.99:1 on stock Dark** (under 3.0); all four design presets clear | same | 1.4.11 |
| Active-tab label | on the page (fine) | **on a filled `primary.base` block: under 4.5:1 in five of six themes** — 3.67 stock Dark, 2.66 / 2.49 / 2.05 / 1.79 on the four design presets | 1.4.3 |
| Toast close target | 25.00 × 23.40, under snora's 24 × 24 | same | 2.5.8 |
| Sidebar tooltip text | contrast depended on your page | same | 1.4.3 |

**Why the label differed:** the underline was a shadow behind a transparent
button. wgpu draws a shadow only outside the button's shape, while tiny-skia
fills the whole shape, so under tiny-skia the active tab was a solid block with
the label on it. snora no longer uses a shadow for this, and a test keeps it that
way.

**On the toast target and WCAG 2.5.8:** 2.5.8 excuses a small target when a
24 px circle centred on it overlaps no other target. Whether your application
qualified depends on your layout. snora's own standard had no such exception,
and the fix removes the question.

**What to re-check:** any record statement about the tab bar's active state
(1.4.11), its label contrast (1.4.3) if your users can reach the software
renderer, the toast close button's target (2.5.8), or tooltip legibility.

### Two corrections to earlier documentation

- **The 0.40 → 0.41 guide** said the underline "is the state indicator", without
  measuring it as one (it was 2.99:1 on stock Dark). It also said the active label
  became "the same color as an inactive one", which was never true, since
  inactive labels were always muted. Both claims are annotated in place.
- **The accessibility checklist** said pointer-target height is "mechanically
  asserted". That was true only for token-sized controls; the toast close button
  was literal-sized and outside the assertion. The checklist now says so.

## What did not change

- No public API removed or renamed; `removable` is unchanged.
- `Spacing`, `Palette` and `snora_style` are unchanged (the frozen design surface
  is untouched).
- No MSRV change (still `1.88`). No third-party dependency moved.

## If you are jumping more than one minor

**0.50** fixed the sidebar's 32 × 48 buttons (a rendered-appearance change) and
corrected the claim that `iced_test` is CPU-only. **0.49** repaired an advisory
gate that could not see `unsound` advisories. **0.47** enforced
`#![forbid(unsafe_code)]`. The [migration index](migrations.md) lists them all.
