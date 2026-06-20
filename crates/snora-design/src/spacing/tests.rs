use super::*;

    #[test]
    fn scale_is_monotonic_and_positive() {
        let s = Spacing::comfortable();
        let v = [s.xs, s.sm, s.md, s.lg, s.xl, s.xxl];
        assert!(v.iter().all(|x| x.is_finite() && *x >= 0.0));
        assert!(v.windows(2).all(|w| w[0] < w[1]));
    }
