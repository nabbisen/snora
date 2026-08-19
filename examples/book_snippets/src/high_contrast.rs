//! Anchored source for `docs/src/design/high-contrast.md`.

use snora::design::Color;

// Prelude the reader doesn't need to see: `my_brand_color`/`my_brand_text`
// stand in for real values a reader would substitute their own brand
// colors for. The anchored test below references them as ordinary
// module items, exactly as it would in a reader's own test module.
#[allow(non_upper_case_globals)]
const my_brand_color: Color = Color::rgb(0.1, 0.3, 0.85);
#[allow(non_upper_case_globals)]
const my_brand_text: Color = Color::rgb(1.0, 1.0, 1.0);

// ANCHOR: high_contrast_custom_verification
#[test]
fn custom_high_contrast_passes_mandatory_pairs() {
    use snora::design::Tokens;
    use snora_design::contrast::contrast_ratio;

    let mut t = Tokens::high_contrast_light();
    t.palette.accent = my_brand_color;
    t.palette.accent_text = my_brand_text;

    let r = contrast_ratio(t.palette.accent_text, t.palette.accent);
    assert!(r >= 4.5, "accent_text on accent: {r:.2}:1");
}
// ANCHOR_END: high_contrast_custom_verification
