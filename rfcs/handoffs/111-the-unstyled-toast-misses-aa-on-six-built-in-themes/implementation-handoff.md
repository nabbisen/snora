# Implementation handoff — RFC-111: the unstyled toast misses AA on six built-in themes

**RFC.** `rfcs/accepted/111-the-unstyled-toast-misses-aa-on-six-built-in-themes.md`
**Release target.** 0.53.0.
**Rulings.**
- Q-1: Info takes the better of its pair and the stronger pole.
- Q-2: the design path is unchanged, but swept.
- Q-3: 0.53.0.

**Touch list.**
- `crates/snora/src/toast.rs`;
- `crates/snora/src/toast/contrast_tests.rs`;
- `crates/snora/tests/drawn_contrast.rs` (one drawn row);
- `CHANGELOG.md` (`[Unreleased]`).

**Not** any fill, `snora-design`, `snora-style`, or the design path's colours.

**Order.** Start after RFC-108, RFC-110 and RFC-109 are committed: this touches
RFC-108's `drawn_contrast.rs` and the shared CHANGELOG. Commit it as a fourth commit.

## Already established (architect's scratch probe, since deleted)

- **The colours.** `intent_colors` (`toast.rs:~355`) is the single source of each
  toast's text colour, and it serves both `toast_style` and `close_button_style`
  (`toast.rs:453`). Fixing it fixes the message text and the dismiss mark together.
- **The design path** arrives through `overrides` and is left as is (Q-2).
- **`theme_contexts()`** (`toast/contrast_tests.rs:137`) is stock Light and Dark only.
- **Measured over `Theme::ALL`, today**, with pairs under 4.5:1:

  | Theme | Intent | Ratio |
  |---|---|---|
  | Kanagawa Dragon | Info (literal `Color::BLACK`) | 2.42 |
  | Nord | Error | 4.25 |
  | Oxocarbon | Error | 4.36 |
  | Kanagawa Lotus | Success | 4.40 |
  | Solarized Light | Error | 4.44 |
  | Solarized Dark | Error | 4.44 |

- **The stock themes today:**
  - Light: Debug 13.28, Info 5.63, Success 6.61, Warning 6.05, Error 4.83;
  - Dark: Debug 6.02, Info 5.63, Success 6.91, Warning 8.12, Error 4.83.

  **These must not change** (acceptance criterion 2).
- **The math R-1 rests on.** For any opaque colour, the better of black and white
  contrasts at ≥ √(1.05 / 0.05) ≈ 4.583:1, so the fallback always clears 4.5.
- **The dev team's configuration:** `snora::render` under
  `snora::design::theme(tokens)` draws Info at 1.40–1.66:1.

## Unit 1 — failing-first

1. Grow `theme_contexts()` to cover all of `Theme::ALL`, **plus** the four presets'
   `snora::design::theme(&tokens)` rendered on the **unstyled** path.
   - It needs `design` for the presets: gate that half behind the feature, as the
     existing design tests are.
   - Rename it if the name now misleads.
2. The existing two assertions now run over that list:
   - `toast_body_text_meets_aa_all_intents_both_themes`. Rename it, since it is no
     longer "both themes".
   - `toast_dismiss_mark_meets_non_text_floor_all_intents_both_themes`.
3. **Today they fail.** Record every failing pair. Expect the six above, plus the
   token-theme Info cases.

## Unit 2 — the fix

In `intent_colors`, for the unstyled branch only (no `overrides`):

- `stronger_pole(fill) -> Color` is `Color::BLACK` or `Color::WHITE`, whichever
  contrasts more with `fill`.
- **Debug, Success, Warning and Error:** keep the theme's pair text when it measures
  ≥ 4.5:1 against the fill, and otherwise use `stronger_pole(fill)`.
- **Info (Q-1):** use the higher-contrast of `primary.strong.text` and
  `stronger_pole(fill)`. On stock themes that is black (5.63), as today.

The contrast helper that `intent_colors` uses must be the same WCAG computation the
tests use. If one does not exist outside the tests, add a small private `fn` in
`toast.rs`, named and commented as the runtime twin of the test's
`contrast_ratio`.

**Comments.**
- Fix `toast_style`'s doc ("a hand-picked warning color (iced's extended palette has
  no `warning` pair of its own)"), which has been stale since RFC-104.
- Rewrite the long comment above the Info arm so its stock-only measurements say
  they are stock-only.

## Unit 3 — one drawn check

In `drawn_contrast.rs`, render a toast on **Kanagawa Dragon / Info** and read text
against fill through RFC-108's toast-row route (`peak_colour`). Assert ≥ 4.5:1.

## Check each assertion can fail

Back up to named files with `cp`, break, run, restore, then `sha256sum -c`. Never
`git checkout`/`restore`.

| Assertion | Breakage |
|---|---|
| the sweep, text ≥ 4.5 | restore today's `intent_colors` (Unit 1 shows this) |
| the sweep: the fallback is what fixes it | make `stronger_pole` always return black: Kanagawa Dragon must fail |
| stock themes unchanged | a dedicated test pinning the ten stock values above; break it by forcing Info to `primary.strong.text` (4.57 on stock) |
| drawn row | force Info's text to the literal black again |

**If a breakage cannot make its row fail, replace the assertion and say so.**

## Acceptance criteria

1. Units 1–3, with every breakage shown failing.
2. The ten stock-theme values unchanged, quoted before and after, and pinned by the
   test in the table.
3. The design path's toast tests pass unmodified, and its colours are unchanged.
4. CHANGELOG under Fixed:
   - the six themes, named, with their before and after ratios;
   - the token-theme configuration;
   - "no change on stock Light and Dark".

   The architect adds the 0.52.0 `*Corrected*` line separately.
