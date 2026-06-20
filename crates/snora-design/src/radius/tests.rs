use super::*;

    #[test]
    fn radii_are_non_negative_and_ordered() {
        let r = Radius::default_roles();
        assert!(
            [r.sm, r.md, r.lg, r.pill]
                .iter()
                .all(|x| x.is_finite() && *x >= 0.0)
        );
        assert!(r.sm <= r.md && r.md <= r.lg && r.lg <= r.pill);
    }
