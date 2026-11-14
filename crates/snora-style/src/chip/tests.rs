//! Tests for the chip style functions (RFC-032, M-4 contrast fix; moved here from `snora-widgets`,
//! RFC-129 Q-5).
//!
//! Contrast requirement: selected chip text over its composited background must meet WCAG AA
//! (≥4.5:1) for every status that is not `Disabled`; `Disabled` is exempt per WCAG 2.1 §1.4.3. The
//! exact sweep of both chips at each preset's own level, hovered and pressed, and their visible
//! step from rest, is `state/tests.rs`, which holds every button and chip to one table.

use super::*;
use iced::widget::button::Status;
use snora_design::{Tokens, contrast};

fn all_statuses() -> [Status; 4] {
    [
        Status::Active,
        Status::Hovered,
        Status::Pressed,
        Status::Disabled,
    ]
}

#[test]
fn selected_all_statuses_all_presets() {
    for (_, t) in Tokens::presets() {
        for s in all_statuses() {
            let style = selected(&t, s);
            assert!(
                style.background.is_some(),
                "selected {s:?}: background must be set"
            );
        }
    }
}

#[test]
fn unselected_all_statuses_all_presets() {
    for (_, t) in Tokens::presets() {
        for s in all_statuses() {
            let style = unselected(&t, s);
            assert!(
                style.background.is_some(),
                "unselected {s:?}: background must be set"
            );
        }
    }
}

#[test]
fn selected_active_uses_accent_text_foreground() {
    // M-4: solid accent bg → accent_text foreground (not accent itself).
    for (_, t) in Tokens::presets() {
        let style = selected(&t, Status::Active);
        let expected = to_iced_color(t.palette.accent_text);
        assert_eq!(
            style.text_color, expected,
            "selected active must use accent_text for contrast"
        );
    }
}

/// Verifies that accent_text over the active (solid accent) background meets WCAG AA (≥4.5:1) for
/// every built-in preset.
///
/// Verifies: M-4 (chip selected contrast fix).
#[test]
fn chip_selected_text_over_accent_background_meets_aa_all_presets() {
    const AA: f32 = 4.5;
    for (name, t) in Tokens::presets() {
        let ratio = contrast::contrast_ratio(t.palette.accent_text, t.palette.accent);
        assert!(
            ratio >= AA,
            "{name}: accent_text/accent contrast {ratio:.2} < {AA} (WCAG AA)"
        );
    }
}

/// Verifies that the **hovered and pressed fills the chip draws** still yield passing contrast
/// against `accent_text`.
///
/// RFC-129: this test used to replicate the fixed darkening (`accent - 0.06` and `- 0.12`) and
/// check the replica, which is a model of the code and no longer what it draws (the shift is now
/// chosen per preset so the label holds its level). It reads the real style function instead. The
/// fill is opaque, so no compositing is needed. The floor here is AA for every preset; the
/// preset's own level (AAA on four of them) is held by `state/tests.rs`.
///
/// Verifies: M-4 (chip selected contrast — hover/pressed states).
#[test]
fn chip_selected_text_hover_pressed_meets_aa_all_presets() {
    const AA: f32 = 4.5;
    for (name, t) in Tokens::presets() {
        let accent_text = t.palette.accent_text;
        for (state, status) in [("hover", Status::Hovered), ("pressed", Status::Pressed)] {
            let style = selected(&t, status);
            let Some(iced::Background::Color(fill)) = style.background else {
                panic!("{name}: the selected chip's {state} fill is not a colour");
            };
            let fill = snora_design::Color::rgb(fill.r, fill.g, fill.b);
            let ratio = contrast::contrast_ratio(accent_text, fill);
            assert!(
                ratio >= AA,
                "{name}: accent_text/{state} fill {ratio:.2} < {AA} (WCAG AA)"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Disabled treatment (RFC-106 Q-2)
// ---------------------------------------------------------------------------

/// Both chip states dim their label and border when disabled, by exactly the factor a disabled
/// button's colours take (`button::disabled_alpha`, 0.45): RFC-129 Q-5 made it one function, where
/// the chips had reproduced it for want of a shared home.
///
/// **Why this is structural and not a contrast floor.** The register asserts each state's *cue*:
/// the unselected chip's is its label, which measures 3.18-6.03 disabled against enabled, and the
/// selected chip's is its fill at 2.64-3.59. The selected chip's label moves only 1.56-1.72,
/// because its fill lightens at the same time — too little to be a floor, and not what tells a
/// user the chip is disabled.
///
/// It is still ruled (Q-2) that both states dim alike, so that the two read as one treatment.
/// Nothing else asserts that: with the selected chip's dim removed, every cue assertion still
/// passes. This is what would notice.
#[test]
fn both_disabled_arms_dim_their_label_and_border() {
    // The factor, stated here: 0.45 (a disabled button's, RFC-106 Q-2).
    const FACTOR: f32 = 0.45;
    for (name, t) in Tokens::presets() {
        for (state, enabled, disabled) in [
            (
                "selected",
                selected(&t, Status::Active),
                selected(&t, Status::Disabled),
            ),
            (
                "unselected",
                unselected(&t, Status::Active),
                unselected(&t, Status::Disabled),
            ),
        ] {
            let expected = enabled.text_color.a * FACTOR;
            assert!(
                (disabled.text_color.a - expected).abs() < 1e-6,
                "{name}/{state}: the disabled label's alpha is {} against the expected {expected} \
                 — both chip states must dim alike (RFC-106 Q-2)",
                disabled.text_color.a,
            );
            let expected = enabled.border.color.a * FACTOR;
            assert!(
                (disabled.border.color.a - expected).abs() < 1e-6,
                "{name}/{state}: the disabled border's alpha is {} against the expected \
                 {expected}",
                disabled.border.color.a,
            );
            // The fill is not part of this: it is the selected chip's own cue and keeps its half
            // alpha.
            assert_eq!(
                match disabled.background {
                    Some(iced::Background::Color(c)) => c.a,
                    other => panic!("{name}/{state}: background changed shape: {other:?}"),
                },
                0.5,
                "{name}/{state}: the disabled fill's alpha moved; RFC-106 left it at 0.5",
            );
        }
    }
}
