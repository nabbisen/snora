use super::*;

    #[test]
    fn rgb_is_opaque() {
        assert!(Color::rgb(0.2, 0.4, 0.6).is_opaque());
    }

    #[test]
    fn rgba_keeps_alpha() {
        let c = Color::rgba(0.1, 0.2, 0.3, 0.25);
        assert_eq!(c.a, 0.25);
        assert!(!c.is_opaque());
    }

    #[test]
    fn validity_checks_range_and_finiteness() {
        assert!(Color::rgb(0.0, 1.0, 0.5).is_valid());
        assert!(!Color::rgb(1.5, 0.0, 0.0).is_valid());
        assert!(!Color::rgb(-0.1, 0.0, 0.0).is_valid());
        assert!(!Color::rgba(0.0, 0.0, 0.0, f32::NAN).is_valid());
    }
