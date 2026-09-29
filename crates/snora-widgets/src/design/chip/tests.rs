//! Tests for chip style functions (RFC-032, M-4 contrast fix).
//!
//! Contrast requirement: selected chip text over composited background must
//! meet WCAG AA (≥4.5:1) for all statuses that are not `Disabled`.
//! `Disabled` is exempt per WCAG 2.1 §1.4.3 exception.

use super::*;
use iced::widget::button::Status;
use snora_design::{Tokens, contrast};

// ---------------------------------------------------------------------------
// Style function structural tests
// ---------------------------------------------------------------------------

#[test]
fn chip_style_selected_all_statuses_all_presets() {
    for t in all_presets() {
        for s in all_statuses() {
            let style = chip_style_selected(&t, s);
            assert!(
                style.background.is_some(),
                "selected {s:?}: background must be set"
            );
        }
    }
}

#[test]
fn chip_style_unselected_all_statuses_all_presets() {
    for t in all_presets() {
        for s in all_statuses() {
            let style = chip_style_unselected(&t, s);
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
    for t in all_presets() {
        let style = chip_style_selected(&t, Status::Active);
        let expected = style::color::to_iced_color(t.palette.accent_text);
        assert_eq!(
            style.text_color, expected,
            "selected active must use accent_text for contrast"
        );
    }
}

// ---------------------------------------------------------------------------
// Contrast tests: selected chip (M-4)
// ---------------------------------------------------------------------------

/// Verifies that accent_text over the active (solid accent) background meets
/// WCAG AA (≥4.5:1) for all four built-in presets.
///
/// Verifies: M-4 (chip selected contrast fix).
#[test]
fn chip_selected_text_over_accent_background_meets_aa_all_presets() {
    const AA: f32 = 4.5;
    for (name, t) in named_presets() {
        let ratio = contrast::contrast_ratio(t.palette.accent_text, t.palette.accent);
        assert!(
            ratio >= AA,
            "{name}: accent_text/accent contrast {ratio:.2} < {AA} (WCAG AA)"
        );
    }
}

/// Verifies that the hovered/pressed backgrounds (darken(accent, amount))
/// still yield passing contrast against accent_text.
///
/// We composite darken(accent, 0.06) (hover) and darken(accent, 0.12) (pressed)
/// and check against accent_text. Because both background and foreground colors
/// are opaque, no alpha compositing is needed.
///
/// Verifies: M-4 (chip selected contrast — hover/pressed states).
#[test]
fn chip_selected_text_hover_pressed_meets_aa_all_presets() {
    const AA: f32 = 4.5;
    for (name, t) in named_presets() {
        let accent = t.palette.accent;
        let accent_text = t.palette.accent_text;

        let hover_bg = snora_design::Color {
            r: (accent.r - 0.06).max(0.0),
            g: (accent.g - 0.06).max(0.0),
            b: (accent.b - 0.06).max(0.0),
            a: accent.a,
        };
        let pressed_bg = snora_design::Color {
            r: (accent.r - 0.12).max(0.0),
            g: (accent.g - 0.12).max(0.0),
            b: (accent.b - 0.12).max(0.0),
            a: accent.a,
        };

        let hover_ratio = contrast::contrast_ratio(accent_text, hover_bg);
        let pressed_ratio = contrast::contrast_ratio(accent_text, pressed_bg);

        assert!(
            hover_ratio >= AA,
            "{name}: accent_text/hover_bg {hover_ratio:.2} < {AA} (WCAG AA)"
        );
        assert!(
            pressed_ratio >= AA,
            "{name}: accent_text/pressed_bg {pressed_ratio:.2} < {AA} (WCAG AA)"
        );
    }
}

// ---------------------------------------------------------------------------
// Private helper
// ---------------------------------------------------------------------------

#[test]
fn darken_clamps_to_zero() {
    let black = darken(iced::Color::BLACK, 1.0);
    assert!(black.r >= 0.0 && black.g >= 0.0 && black.b >= 0.0);
}

// ---------------------------------------------------------------------------
// Pointer target size (RFC-061)
// ---------------------------------------------------------------------------

#[test]
fn remove_btn_target_size_clears_24px_floor_for_all_presets() {
    for t in all_presets() {
        let size = remove_btn_target_size(&t);
        assert!(
            size >= 24.0,
            "remove_btn_target_size {size} < 24.0 pointer-target floor"
        );
    }
}

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn all_presets() -> [Tokens; 4] {
    [
        Tokens::light(),
        Tokens::dark(),
        Tokens::high_contrast_light(),
        Tokens::high_contrast_dark(),
    ]
}

fn named_presets() -> [(&'static str, Tokens); 4] {
    [
        ("light", Tokens::light()),
        ("dark", Tokens::dark()),
        ("high_contrast_light", Tokens::high_contrast_light()),
        ("high_contrast_dark", Tokens::high_contrast_dark()),
    ]
}

fn all_statuses() -> [Status; 4] {
    [
        Status::Active,
        Status::Hovered,
        Status::Pressed,
        Status::Disabled,
    ]
}

// ---------------------------------------------------------------------------
// Disabled treatment (RFC-106 Q-2)
// ---------------------------------------------------------------------------

/// Both chip states dim their label and border when disabled, by
/// exactly [`CHIP_DISABLED_ALPHA`].
///
/// **Why this is structural and not a contrast floor.** The register
/// asserts each state's *cue*: the unselected chip's is its label, which
/// measures 3.18-6.03 disabled against enabled, and the selected chip's
/// is its fill at 2.64-3.59. The selected chip's label moves only
/// 1.56-1.72, because its fill lightens at the same time — too little to
/// be a floor, and not what tells a user the chip is disabled.
///
/// It is still ruled (Q-2) that both states dim alike, so that the two
/// read as one treatment. Nothing else asserts that: with the selected
/// chip's dim removed, every cue assertion still passes. This is what
/// would notice.
#[test]
fn both_disabled_arms_dim_their_label_and_border() {
    for (name, t) in named_presets() {
        for (state, enabled, disabled) in [
            (
                "selected",
                chip_style_selected(&t, Status::Active),
                chip_style_selected(&t, Status::Disabled),
            ),
            (
                "unselected",
                chip_style_unselected(&t, Status::Active),
                chip_style_unselected(&t, Status::Disabled),
            ),
        ] {
            let expected = enabled.text_color.a * super::CHIP_DISABLED_ALPHA;
            assert!(
                (disabled.text_color.a - expected).abs() < 1e-6,
                "{name}/{state}: the disabled label's alpha is {} against the expected {expected} \
                 — both chip states must dim alike (RFC-106 Q-2)",
                disabled.text_color.a,
            );
            let expected = enabled.border.color.a * super::CHIP_DISABLED_ALPHA;
            assert!(
                (disabled.border.color.a - expected).abs() < 1e-6,
                "{name}/{state}: the disabled border's alpha is {} against the expected \
                 {expected}",
                disabled.border.color.a,
            );
            // The fill is not part of this: it is the selected chip's own
            // cue and keeps its half alpha.
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
