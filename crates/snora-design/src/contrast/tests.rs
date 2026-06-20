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
