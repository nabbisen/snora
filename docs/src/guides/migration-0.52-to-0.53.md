# Migration 0.52 → 0.53

> **No API change. Rendered appearance changes on the default path in three
> places, each a defect fix.** One published guarantee is corrected (§4). Read §4
> if your project relies on "snora's rendered output is unchanged without
> `design`".

## Who is affected

| You render… | What changes | Re-check |
|---|---|---|
| Dialogs or sheets, unstyled path, dark theme | The modal dim is now a light veil; before, it barely changed the page | Baselines with a modal open |
| The same, light theme | The dim is slightly darker (alpha 0.40 → 0.44) | Baselines with a modal open |
| Breadcrumbs | Ancestors are underlined; the row is 1 px taller (32.80 → 33.80 at the default text size) | Baselines and layouts around the breadcrumb row |
| Toasts on a non-stock iced theme | On six built-in themes, one intent's text colour changes to clear AA | Baselines on those themes |
| Toasts on stock Light or Dark | **Nothing**: all ten text colours are unchanged, and pinned by a test | — |

**For code: nobody needs to change anything.**

## 1. The modal dim follows the theme (RFC-110)

The unstyled dim was a literal, black at 40%. Over a dark page that changes almost
nothing: measured over iced's 22 built-in themes, the dimmed page differed from the
undimmed one by **1.02–1.33:1 on all 16 dark themes**, and by 2.70–2.85:1 on the 6
light ones.

It now takes the pole opposite the page, white over dark and black over light, at
alpha 0.44. That is the rule the `design` path has used since RFC-065, so both paths
now agree. Measured after: **3.74–4.36 on the dark themes and 3.05–3.24 on the light
ones**, every theme clearing 3.0:1.

That 3.0 is snora's own floor for "this page is inert", borrowed from WCAG 1.4.11's
figure. WCAG does not require it of dims as such.

## 2. Ancestor breadcrumbs are underlined (RFC-109)

At rest, an ancestor crumb and the current page were drawn in the same colour, by
construction. Hover gave a fill, but nothing marked an ancestor as clickable for a
reader who was not already pointing at it, and nothing at all for a keyboard user.

Ancestors now carry a 1 px rule under the label, inside the button. The leaf has none,
and that absence is its cue. It is a shape, so it does not depend on colour. On hover
the rule keeps its resting colour; measured against the hover fill, it is ≥ 4.51:1 on
all 22 themes and the 4 presets.

**There is no opt-out.** It is a cue snora draws for a state snora draws (RFC-103's
rule). If you have a reason to need one, tell us.

## 3. Toast text clears AA on every built-in theme (RFC-111)

The toast's contrast suite checked iced's two stock themes. Over all 22, six text
pairs missed 4.5:1:

| Theme | Intent | Before | After |
|---|---|---|---|
| Kanagawa Dragon | Info | 2.42 | 8.69 |
| Nord | Error | 4.25 | 5.13 |
| Oxocarbon | Error | 4.36 | 5.30 |
| Kanagawa Lotus | Success | 4.40 | 5.36 |
| Solarized Light | Error | 4.44 | 4.63 |
| Solarized Dark | Error | 4.44 | 4.63 |

**If you call `snora::render` under a theme from `snora::design::theme(&tokens)`**,
Info measured 1.40–1.66:1; it is now 12.64–15.03.

The rule: keep the theme's own text colour wherever it clears AA, and otherwise use
black or white, whichever contrasts more. That fallback cannot fail. For any opaque
fill, the better of black and white reaches at least √(1.05 / 0.05) ≈ 4.58:1. So
toasts clear AA on themes that don't exist yet, too.

## 4. For compatibility records: a guarantee corrected

snora's docs said that with `design` inactive, its rendered output "is unchanged from
v0.25 — a guarantee, not an aspiration". **That has not been true since at least
0.42.0**, when the toast colours changed on the default path. Several releases since
have changed default-path appearance (0.50's sidebar buttons, 0.51's tab bar, 0.52's
label sizes, and this release). Each was named in its migration guide, but none
amended the guarantee.

**What does hold, and is asserted:**
- Without `design`, snora's **composition** is unchanged: layer order, which
  surfaces appear, how they dismiss, and how they mirror. `render_semantics.rs`
  checks this, and must pass unmodified.
- Default-path **appearance** changes only through an accepted RFC that the minor's
  migration guide names.

The corrected wording is in [the design overview](../design/overview.md). If a
compatibility record of yours cites the old sentence, it should cite this one.

## 5. Three new advisories, accepted (no vulnerability)

`RUSTSEC-2026-0370`, `-0371` and `-0372` mark `zvariant`, `zbus_names` and
`zvariant_derive` as unmaintained. They were merged into `zbus` 6. They reach the
graph **on Linux only**, through iced's default `linux-theme-detection` feature,
which reads the desktop colour scheme over D-Bus. They are not vulnerabilities, and
no upgrade clears them while iced uses `zbus` 5. snora accepts them in `deny.toml`
with the condition that retires them.

**If your own `cargo deny` or `cargo audit` denies unmaintained crates**, it will
report these three. Accept them by ID, or turn off iced's `linux-theme-detection`
if you don't need the system colour scheme.

## Also new, for contributors and evaluators

- **Contrast is read from drawn pixels (RFC-108).** Eight element groups are measured
  from the rendered frame, not only from style functions, against the same floors.
  `iced_test` writes each frame as a lossless PNG named after the renderer that drew
  it, so each test also asserts which renderer that was.
- **CI tests both renderers.** The rendered suite runs on wgpu (on lavapipe, a
  software Vulkan driver) as well as tiny-skia. A third job compares what the two
  drew across 92 elements: core colours and bounds, with antialiased edge pixels
  excluded.
- **The release history is filed by series of ten minors** (`changelog/0.Nx.md`), and
  each version from 0.53.0 gets a GitHub Release built from its changelog entry
  (RFC-107).

## What did not change

- No public API removed, renamed or added. No MSRV change (still `1.88`).
- `snora-design` and `snora-style` are unchanged. The design path's dim and toast
  colours are unchanged.

## If you are jumping more than one minor

**0.52** made chrome labels follow your text size, wrapped long unbroken text, and
dimmed disabled chips. **0.51** redrew the tab bar's edges. The
[migration index](migrations.md) lists them all.
