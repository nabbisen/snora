/// Smoke test: `snora::design::contrast` re-export resolves correctly
/// and the well-known black/white ratio returns the WCAG 2.1 value of
/// 21.0 (±epsilon). Guards the re-export path against accidental removal.
///
/// The underlying math is covered exhaustively by `snora-design`'s own
/// test suite; this test only verifies the facade path.
#[test]
fn contrast_ratio_black_white_via_facade() {
    use crate::design::contrast::{composite_over, contrast_ratio, relative_luminance};
    use crate::design::{Color, Tokens};

    let t = Tokens::light();
    // Well-known WCAG 2.1 value: black/white = 21:1 (±epsilon).
    let ratio = contrast_ratio(Color::rgb(0.0, 0.0, 0.0), Color::rgb(1.0, 1.0, 1.0));
    assert!(
        (ratio - 21.0).abs() < 0.01,
        "black/white contrast ratio should be ~21.0, got {ratio}"
    );
    // Confirm relative_luminance and composite_over also resolve via facade.
    let _ = relative_luminance(t.palette.text_primary);
    let _ = composite_over(
        Color { a: 0.5, ..t.palette.accent },
        t.palette.surface,
    );
}
