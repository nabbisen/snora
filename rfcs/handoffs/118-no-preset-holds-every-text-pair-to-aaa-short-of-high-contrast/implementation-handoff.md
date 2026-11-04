# Implementation handoff — RFC-118: `light_aaa` and `dark_aaa`

**RFC.** `rfcs/accepted/118-no-preset-holds-every-text-pair-to-aaa-short-of-high-contrast.md`
**Release target.** 0.55.0.
**Depends on** RFC-117's R-5, the accent's text declaration. Without it, the sweep
skips the accent.

**Touch list.**
- `crates/snora-design/src/presets/` (two new files) and `presets.rs`;
- `crates/snora-design/src/tests.rs`;
- `crates/snora-style` / `snora-widgets`, only if a preset list is enumerated there;
- `CHANGELOG.md`.

**Not:** `light`, `dark` or the high-contrast presets, nor any `Palette` role. The
architect writes the docs table and Q-7's guidance.

## Already established (architect, 2026-11-04)

The figures that miss 7:1 are in the RFC's table:
- `text_muted`, at 4.53–5.44 in both presets;
- in Light, `success`, `warning`, `danger`, `info` and the accent;
- each fill's own text in Light, and accent, danger and info in Dark.

The high-contrast presets clear 7:1 on every pair. Their lowest is 8.38.

## Unit 1 — failing-first

- **Add `Tokens::light_aaa()` and `dark_aaa()`,** temporarily returning `light()` and
  `dark()`.
- **Write R-2's test:** every `Palette::usages` entry whose class is `Text` must be
  ≥ 7:1 on every declared surface, for the two AAA presets.
- **Record the failures.** They must list the pairs in the RFC's table.

## Unit 2 — the palettes

**Propose values** for each role that misses. Each preset keeps its parent's
background and surfaces, and darkens (Light) or lightens (Dark) the text roles and
fills only as far as 7:1 requires.

**Assert R-2's two postconditions:**
- **hierarchy order:** `text_primary` ≥ `text_secondary` ≥ `text_muted` against each
  surface;
- **hue bound:** each changed role's hue stays within *N*° of its parent's, in OKLCH.
  **Propose *N*** from what your values need, with the per-role figures. The owner
  rules on it.

**Every existing preset test runs on the two new presets** (`all_presets()`): non-text
3:1, surfaces, focus, and the dialog card against the dim.

## Report

- **A side-by-side table:** each changed role, its parent value, its AAA value, the
  hue change, and the ratios on each surface.
- **Any role where 7:1 could not be reached** inside the parent's surfaces without
  moving a surface. If one exists, stop and report it. Do not move surfaces on your
  own.

## Acceptance criteria

1. Unit 1 shown failing, then Unit 2 passing.
2. Both postconditions asserted, and each shown failing:
   - swap `text_secondary` and `text_muted` in `light_aaa`: the order assertion fails;
   - push one hue past *N*: the hue assertion fails.
3. All existing preset tests passing for both new presets.
4. CHANGELOG under Added.
