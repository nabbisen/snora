use super::*;

    #[test]
    fn fields_round_trip() {
        let c = Color::rgb(0.0, 0.2, 0.7);
        let f = FocusTokens::new(3.0, 1.0, c);
        assert_eq!((f.ring_width, f.ring_offset, f.ring_color), (3.0, 1.0, c));
    }
