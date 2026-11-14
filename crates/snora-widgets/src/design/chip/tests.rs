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
/// WCAG AA (≥4.5:1) for every built-in preset.
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

/// Verifies that the **hovered and pressed fills the chip draws** still yield passing contrast
/// against `accent_text`.
///
/// RFC-129: this test used to replicate the fixed darkening (`accent - 0.06` and `- 0.12`) and
/// check the replica, which is a model of the code and no longer what it draws (the shift is now
/// chosen per preset so the label holds its level). It reads the real style function instead.
/// The fill is opaque, so no compositing is needed. The floor here is AA for every preset; the
/// preset's own level (AAA on four of them) is held by `snora/tests/text_floor.rs`.
///
/// Verifies: M-4 (chip selected contrast — hover/pressed states).
#[test]
fn chip_selected_text_hover_pressed_meets_aa_all_presets() {
    const AA: f32 = 4.5;
    for (name, t) in named_presets() {
        let accent_text = t.palette.accent_text;
        for (state, status) in [("hover", Status::Hovered), ("pressed", Status::Pressed)] {
            let style = chip_style_selected(&t, status);
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

// RFC-118: every preset, from the one public source. A preset that
// passed `snora-design`'s own suite and then failed a widget-level
// style assertion is the obvious way for a new preset to go wrong, and
// a hand-kept list is how one gets missed.
fn all_presets() -> Vec<Tokens> {
    Tokens::presets().into_iter().map(|(_, t)| t).collect()
}

fn named_presets() -> Vec<(&'static str, Tokens)> {
    Tokens::presets()
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

// ---------------------------------------------------------------------------
// RFC-129: a hovered or pressed chip holds its preset's level
// ---------------------------------------------------------------------------

/// **Both chips, hovered and pressed, at the preset's own level, exactly.** The chips paint their
/// own opaque fill, so the surface is not in the sum. The level is this suite's own table (the
/// book's), never the palette's token: a guard does not read what it guards (RFC-129 Q-4).
///
/// Fails on the fixed darkening the chips had before RFC-129: `light_aaa`'s unselected chip
/// pressed (6.75) and `dark_aaa`'s selected chip pressed (5.96).
#[test]
fn a_hovered_or_pressed_chip_label_holds_its_presets_level() {
    let level = |preset: &str| match preset {
        "light" | "dark" => 4.5,
        "light_aaa" | "dark_aaa" | "high_contrast_light" | "high_contrast_dark" => 7.0,
        other => panic!("`{other}` is a new preset: say its text level here"),
    };
    let mut failures = Vec::new();
    for (name, t) in named_presets() {
        for (chip, style) in [
            (
                "selected",
                chip_style_selected as fn(&Tokens, Status) -> button::Style,
            ),
            ("unselected", chip_style_unselected),
        ] {
            for status in [Status::Hovered, Status::Pressed] {
                let s = style(&t, status);
                let Some(iced::Background::Color(fill)) = s.background else {
                    panic!("{name} {chip} {status:?}: the fill is not a colour");
                };
                let d = |c: iced::Color| snora_design::Color::rgb(c.r, c.g, c.b);
                let r = contrast::contrast_ratio(d(s.text_color), d(fill));
                if r < level(name) {
                    failures.push(format!(
                        "{name} {chip} {status:?}: {r:.2}, level {}",
                        level(name)
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} cells:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
