# Migration 0.51 → 0.52

> **No breaking API change, but rendered appearance changes on the default
> path.** Chrome and toast labels now take their size from your application,
> long unbroken text wraps instead of being cut off, and disabled chips dim.
> One published accessibility claim is narrowed (§5).

## Who is affected

| You render… | What changes | Re-check |
|---|---|---|
| Tab bar, breadcrumb, menus (unstyled) | Labels follow `default_text_size` (13/14 → 16 at iced's default) | Baselines, and layouts sized around those rows |
| The same (`design`) | Labels follow `Typography` | Baselines |
| Toasts | Message follows the host; Warning fill from the theme; on `design`, every intent from the tokens | Baselines with a toast |
| Long file names, paths, URLs in toasts, notices, tooltips | They wrap instead of being cut off | Surfaces that showed such text |
| Disabled chips | Label and border dim | Baselines with a disabled chip |

**For code: nobody needs to change anything.**

## 1. Label sizes follow your application (RFC-104)

Every chrome label was a literal size: tab and crumb 13, menu 14, header 16,
toast title and message 16 and 14. **An application's text-size setting did not
reach them.** Now:

| Label | Before | Unstyled (at iced's default 16) | `design` |
|---|---|---|---|
| Header title | 16 | 16 — unchanged | `title` (18 in the shipped presets) |
| Tab, crumb | 13 | 16 | `label` (14) |
| Menu | 14 | 16 | `label` (14) |
| Toast title | 16 | 16 — unchanged | `title` (18) |
| Toast message | 14 | 16 | `body` (16) |

**Heights that follow** (unstyled, default 16): tab bar 33.9 → 37.8, breadcrumb
row 28.9 → 32.8, an open menu 56.4 → 61.6, and the toast 67.0 → 69.6 (72.2 on
`design`). The header bar does not move.

**Restoring the old look:** `default_text_size` applies to all text that
inherits it, so it **cannot** reproduce the old mix of four different literals.
If you need per-surface sizes, use the `design` path and set its `Typography`
roles.

**Why it matters:** the literal sizes ignored your application's text-size
setting (orbok, for example, offers Default / Large / Larger). They now follow
it. A new CI scan, `scripts/check-literal-sizes.sh`, fails snora's build on any
literal text size, so this cannot quietly return.

## 2. Toast colours come from the theme (RFC-104)

- **Warning** was a private constant. It is now the theme's warning pair: stock
  Light `rgb(0.718, 0.494, 0.200)` and stock Dark `rgb(1.000, 0.757, 0.306)`,
  each with the theme's paired text.
- **On `design`, toasts now follow the tokens** for every intent. Before, the
  design path did not reach toasts at all. `Debug`, which has no status colour,
  uses the neutral raised surface.
- Contrast is asserted for every intent on both paths.

## 3. Long unbroken text is shown whole (RFC-105)

A string that cannot break at a word, such as a file name, path, URL or hash, was
**silently cut off**. In a toast, about the first 48 of 400 characters showed,
and nothing marked the cut. It now wraps inside the word when it must, in toast
titles and messages, notice titles and bodies, and **tooltips**. Tooltips now
also have a 320 px maximum width, so a long one wraps instead of spanning the
window.

**Ordinary text is unchanged.** Prose fixtures measure the same, and short
tooltips render byte-identically to 0.51. Surfaces showing long unbroken text
grow taller.

## 4. Disabled chips dim (RFC-106)

A disabled **unselected** chip looked exactly like an enabled one (1.00–1.05:1).
Both chip states now dim their label and border to 45 % when disabled, the factor
disabled buttons already use. That gives an unselected disabled chip ≥ 3.0:1
against an enabled one. The fill is unchanged.

## 5. For conformance records — one claim narrowed

**snora's assistive-technology position said ABDD includes "non-colour status
encoding".** That overstated what snora provides:

- **Variants** (toast intent, notice tone, progress tone): snora contributes
  colour, and **you** supply the words. This was already ruled (RFC-093); the
  phrase implied otherwise.
- **States snora draws itself** now each carry an **asserted** cue: the tab
  underline, the sidebar fill, chip selected and disabled, disabled buttons, the
  open menu, and the breadcrumb leaf. See the table in
  [the accessibility guide](accessibility.md#what-snora-does-not-provide).
- **Qualification:** a selected chip's cue is luminance (≥ 3.0:1), not a word or
  shape. If your record needs a word or shape, put a mark in the chip's label.

**What to re-check:** any record statement that cites snora for non-colour status
encoding. Also any that cites the disabled chip, which was indistinguishable
before this release.

## Also new, for contributors and evaluators

- **Verbatim text is asserted.** A fixture with a bidi override, a newline and a
  control character is found by exact content in every caller-text surface.
- **Bounded work is asserted.** Ten times the toasts must cost under 30 times the
  layout time. It is measured at 656 ms for 100,000 toasts on the reference
  machine, and recorded in [the performance envelope](../reference/performance-envelope.md).

## What did not change

- No public API removed or renamed. No MSRV change (still `1.88`).
- `Typography`, `Palette`, `Spacing` and `snora_style` values are unchanged:
  they are read, never modified, so the frozen design surface is untouched.

## If you are jumping more than one minor

**0.51** redrew the tab bar's edges, gave tooltips a body, and fixed the toast
close target (appearance changes, with conformance findings split by renderer).
**0.50** fixed the sidebar's button geometry. The
[migration index](migrations.md) lists them all.
