use super::*;
use crate::Tokens;

#[test]
fn fields_round_trip() {
    assert_eq!(TargetTokens::new(44.0).min, 44.0);
}

/// WCAG 2.5.8's floor, restated here rather than imported: the point is
/// that a preset which fell under it would be caught by a number written
/// down independently of the one it was built from.
const WCAG_2_5_8_MIN: f32 = 24.0;

/// Every preset's minimum is exactly 24, so that choosing a preset moves
/// no size: RFC-115's rule is free at the default text size, and a preset
/// that set 32 would silently enlarge every control in an application
/// that changed only its theme.
#[test]
fn every_preset_sets_the_wcag_minimum_exactly() {
    for (name, tokens) in Tokens::presets() {
        assert_eq!(
            tokens.targets.min, WCAG_2_5_8_MIN,
            "{name}: the preset's minimum target is not 24"
        );
    }
}
