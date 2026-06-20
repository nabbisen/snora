use super::*;

    #[test]
    fn all_roles_have_positive_size_and_line_height() {
        let t = Typography::default_roles();
        for role in [t.body, t.body_small, t.label, t.title, t.heading, t.display] {
            assert!(role.size > 0.0 && role.size.is_finite(), "size {role:?}");
            assert!(
                role.line_height > 0.0 && role.line_height.is_finite(),
                "lh {role:?}"
            );
        }
    }
