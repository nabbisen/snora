use super::*;

    #[test]
    fn variants_are_comparable() {
        assert_eq!(Tone::Danger, Tone::Danger);
        assert_ne!(Emphasis::Solid, Emphasis::Ghost);
        assert_eq!(Size::Medium, Size::Medium);
        assert_eq!(Density::Comfortable, Density::Comfortable);
    }
