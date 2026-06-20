# High contrast

Snora Design ships two high-contrast presets that exceed WCAG AA for all
mandatory text pairs — `high_contrast_light` and `high_contrast_dark`.

## When to offer high contrast

Offer a high-contrast toggle when your application targets users who rely on
strong visual separation: users with low vision, users who work in bright
environments, or users who explicitly request higher contrast from OS settings.

Snora Design does not detect OS contrast preferences automatically in v0.20.
Preference detection is application responsibility.

## What the presets guarantee

Every built-in high-contrast palette passes all of the following pairs at
`>= 7:1` for primary text and `>= 4.5:1` for all mandatory status pairs:

| Pair | Light HC | Dark HC |
|---|---|---|
| `text_primary` on `background` | 21:1 | 21:1 |
| `text_secondary` on `background` | 17:1 | 17:1 |
| `accent_text` on `accent` | 10:1 | 12:1 |
| `danger_text` on `danger` | 9:1 | 10:1 |
| `focus` on `background` | 10:1 | 21:1 |

These ratios are verified by the automated contrast test suite
(`cargo test -p snora-design`) and must not regress.

## Characteristic differences from light/dark

- **Borders are solid black / solid white.** The `border` role is `#000000`
  (light HC) or `#FFFFFF` (dark HC), making every card, panel, and input
  clearly delineated.
- **Surfaces collapse to a single background.** `background`, `surface`,
  and `surface_raised` are the same color; separation comes from borders
  alone, not shading.
- **Focus ring is thicker.** `FocusTokens.ring_width` is `3.0` in the
  high-contrast presets versus `2.0` in light/dark.

## Customizing within high contrast

If you replace palette roles in a high-contrast preset, run the contrast
verification in your own tests:

```rust,ignore
#[test]
fn custom_high_contrast_passes_mandatory_pairs() {
    use snora_design::contrast::contrast_ratio;
    use snora::design::Tokens;

    let mut t = Tokens::high_contrast_light();
    t.palette.accent = my_brand_color;
    t.palette.accent_text = my_brand_text;

    let r = contrast_ratio(t.palette.accent_text, t.palette.accent);
    assert!(r >= 4.5, "accent_text on accent: {r:.2}:1");
}
```

## Visual fit checklist (high contrast)

When testing under a high-contrast preset in the workbench:

- All card borders must be clearly visible.
- All swatch backgrounds must be visually distinct.
- Button text must be readable in all four states (active, hover, pressed,
  disabled).
- The focus ring at `3.0` width must be visible when tabbing through controls.
  (In iced 0.14 the focus ring is not rendered through `button::Style` — see
  [Semantic accessibility](../contributing/semantic-accessibility.md).)
