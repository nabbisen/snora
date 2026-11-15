use super::*;

#[test]
fn black_on_white_is_max_ratio() {
    let r = contrast_ratio(Color::rgb(0.0, 0.0, 0.0), Color::rgb(1.0, 1.0, 1.0));
    assert!((r - 21.0).abs() < 0.01, "got {r}");
}

#[test]
fn ratio_is_symmetric() {
    let a = Color::rgb(0.1, 0.3, 0.85);
    let b = Color::rgb(1.0, 1.0, 1.0);
    assert!((contrast_ratio(a, b) - contrast_ratio(b, a)).abs() < 1e-6);
}

#[test]
fn identical_colors_have_ratio_one() {
    let c = Color::rgb(0.4, 0.4, 0.4);
    assert!((contrast_ratio(c, c) - 1.0).abs() < 1e-6);
}

#[test]
fn composite_of_opaque_is_unchanged() {
    let fg = Color::rgb(0.2, 0.4, 0.6);
    assert_eq!(composite_over(fg, Color::rgb(1.0, 1.0, 1.0)), fg);
}

#[test]
fn composite_half_alpha_is_midpoint() {
    let out = composite_over(Color::rgba(0.0, 0.0, 0.0, 0.5), Color::rgb(1.0, 1.0, 1.0));
    assert!((out.r - 0.5).abs() < 1e-6 && out.is_opaque());
}

/// **Known ratios that are not black on white (0.63 audit, V1-073).** Black on white is 21 whatever the
/// luminance coefficients are, so a swapped `0.2126 / 0.7152 / 0.0722` (or a wrong gamma) passed the
/// whole palette suite. The pure primaries pin each coefficient on its own: against black the ratio is
/// `(L + 0.05) / 0.05`, and `L` of a pure primary is its coefficient. A mid-grey pins the transfer
/// curve (sRGB 0.5 linearises to 0.21404).
#[test]
fn the_luminance_coefficients_and_the_curve_are_pinned_by_known_ratios() {
    let black = Color::rgb(0.0, 0.0, 0.0);
    for (name, c, l) in [
        ("red", Color::rgb(1.0, 0.0, 0.0), 0.2126),
        ("green", Color::rgb(0.0, 1.0, 0.0), 0.7152),
        ("blue", Color::rgb(0.0, 0.0, 1.0), 0.0722),
    ] {
        let want = (l + 0.05) / 0.05;
        let got = contrast_ratio(c, black);
        assert!(
            (got - want).abs() < 0.001,
            "{name} on black: {got}, expected {want}"
        );
    }
    let grey = Color::rgb(0.5, 0.5, 0.5);
    let on_black = contrast_ratio(grey, black);
    let on_white = contrast_ratio(grey, Color::rgb(1.0, 1.0, 1.0));
    assert!(
        (on_black - 5.2808).abs() < 0.001,
        "mid-grey on black: {on_black}"
    );
    assert!(
        (on_white - 3.9773).abs() < 0.001,
        "mid-grey on white: {on_white}"
    );
}
