use super::*;
use crate::Tokens;

#[test]
fn dim_alpha_is_always_dim_alpha_constant() {
    for (name, t) in [
        ("light", Tokens::light()),
        ("dark", Tokens::dark()),
        ("high_contrast_light", Tokens::high_contrast_light()),
        ("high_contrast_dark", Tokens::high_contrast_dark()),
    ] {
        assert_eq!(
            modal_dim(&t).a,
            DIM_ALPHA,
            "{name}: modal_dim's alpha must stay fixed at DIM_ALPHA regardless of preset"
        );
    }
}

/// Direct unit test of the pole choice, isolated from any specific
/// preset — the property that makes the derivation safe at both
/// luminance extremes without needing a clamp fallback: the two poles
/// can never both describe the same input.
#[test]
fn modal_dim_picks_the_opposite_pole() {
    let mut white_bg = Tokens::light();
    white_bg.palette.background = Color::rgb(1.0, 1.0, 1.0);
    let dim = modal_dim(&white_bg);
    assert_eq!(
        (dim.r, dim.g, dim.b),
        (0.0, 0.0, 0.0),
        "a light (white) background must pick the black pole"
    );

    let mut black_bg = Tokens::high_contrast_dark();
    black_bg.palette.background = Color::rgb(0.0, 0.0, 0.0);
    let dim = modal_dim(&black_bg);
    assert_eq!(
        (dim.r, dim.g, dim.b),
        (1.0, 1.0, 1.0),
        "a dark (black) background must pick the white pole"
    );
}

#[test]
fn is_dark_classifies_pure_black_and_white() {
    assert!(is_dark(Color::rgb(0.0, 0.0, 0.0)));
    assert!(!is_dark(Color::rgb(1.0, 1.0, 1.0)));
}
