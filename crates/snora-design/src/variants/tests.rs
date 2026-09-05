use super::*;

#[test]
fn variants_are_comparable() {
    assert_eq!(Tone::Danger, Tone::Danger);
    assert_eq!(Density::Comfortable, Density::Comfortable);
}
