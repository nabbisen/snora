# AAA presets

`Tokens::light_aaa()` and `Tokens::dark_aaa()` hold **every declared text pair at
7:1** (WCAG 1.4.6, AAA), while keeping Light's and Dark's character (RFC-118). They
are for applications that hold their text to AAA, for instance for older readers.

## Which preset

| Preset | Text | Non-text | For |
|---|---|---|---|
| `light`, `dark` | AA, 4.5:1 | 3:1 | general use |
| `light_aaa`, `dark_aaa` | **AAA, 7:1** | 3:1, as their parents | Light's or Dark's character at AAA text |
| `high_contrast_light`, `high_contrast_dark` | ≥ 8.38:1 | borders at 21:1 | maximum contrast ([High contrast](high-contrast.md)) |

`Tokens::presets()` lists every preset, in this order. Each declares its level as
[`Palette::text_contrast`](tokens.md#the-palettes-text-level-062): `Aa` for `light` and
`dark`, `Aaa` for the other four (0.62).

## In every state (0.62)

**The 7:1 holds while the reader points at a control or presses it,** not only at rest:
- for every `design::button` variant, both chips, `button::link` and a notice's action;
- hovered and pressed;
- on every neutral surface (`background`, `surface`, `surface_raised`);
- measured on the pixels drawn.

Before 0.62 it held only at rest. 17 combinations of a preset, a variant and a state
fell under their level, most of them on these two presets (RFC-129). Every state also
differs visibly from rest. The high-contrast presets' states were at 7:1 already, except
one unselected chip on `high_contrast_dark` that drew exactly like rest.

## How they were made

Each keeps its parent's background, surfaces and border. A role moves **only if one
of its declared text pairs is under 7:1**, and then by the smallest step that reaches
it:
- a lightness-only shift in OKLCh, with chroma and hue held;
- bisected to the floor.

A fill moves only when its *own text* needed it, never because its boundary against a
surface was low: that pair is non-text, at 3:1. Two properties are asserted, besides
the 7:1 sweep:
- **the hierarchy keeps its order:** `text_primary` ≥ `text_secondary` ≥
  `text_muted`, on every surface;
- **no role's hue moves more than 6°.**

### `light_aaa` against `light`

| Role | Parent | AAA | Hue shift | Lowest declared text pair |
|---|---|---|---|---|
| `text_muted` | `#6a717e` | `#4d5461` | 0.0° | 7.05:1 |
| `accent, info` | `#1d4ed8` | `#1544ce` | 0.0° | 7.05:1 |
| `success` | `#15803d` | `#006725` | 2.3° | 7.05:1 |
| `warning` | `#9a5b00` | `#874900` | 5.1° | 7.05:1 |
| `danger` | `#b3261e` | `#ad1f18` | 0.0° | 7.05:1 |

`text_primary` and `text_secondary` are unchanged, at 16.44 and 8.12.

### `dark_aaa` against `dark`

| Role | Parent | AAA | Hue shift | Lowest declared text pair |
|---|---|---|---|---|
| `text_muted` | `#828b97` | `#a6afbc` | 0.0° | 7.05:1 |
| `accent, info` | `#5b9bff` | `#70b2ff` | 5.4° | 7.05:1 |
| `danger` | `#ff6b6b` | `#ff7372` | 0.3° | 7.05:1 |

`text_primary`, `text_secondary`, `success` and `warning` are unchanged.

## The hierarchy is quieter at AAA

Muted text at 7:1 sits close to secondary text: 7.05 against 8.12 in `light_aaa`,
and 7.05 against 7.62 in `dark_aaa`. Colour then carries less of the hierarchy. **In
an AAA application, let size and weight carry it** ([Typography](typography.md)),
not colour alone. That is better for older readers anyway.

Weight is not part of `Typography`: its roles carry size and line height only. Set it
on the `Font` you draw with, and **prefer `Weight::Bold` over `Semibold`**. Many CJK sans
families ship no SemiBold. Noto Sans CJK JP has Medium and Bold but no SemiBold, while
Noto Serif CJK JP has one. So a Semibold request can switch Japanese text to a serif
face beside sans body text, while Latin text looks right. **Check the typeface of strong
text in every script you ship.** *(Reported by knotra, 2026-11-09.)*
