//! Cross-preset validation: token sanity (RFC-022/024) and mandatory automated
//! contrast tests (RFC-023).
//!
//! Threshold policy:
//! * normal/body text pairs: >= 4.5:1 (WCAG AA);
//! * non-text focus indicator pairs: >= 3.0:1;
//! * high-contrast presets are expected to exceed these comfortably.
//!
//! All colors used in mandatory pairs must be fully opaque (the assertions
//! check this); alpha roles would need compositing first.

use crate::contrast::contrast_ratio;
use crate::{Palette, Tokens};

const AA_TEXT: f32 = 4.5;
const FOCUS_MIN: f32 = 3.0;

fn all_presets() -> [(&'static str, Tokens); 4] {
    [
        ("light", Tokens::light()),
        ("dark", Tokens::dark()),
        ("high_contrast_light", Tokens::high_contrast_light()),
        ("high_contrast_dark", Tokens::high_contrast_dark()),
    ]
}

// ---- token sanity ----

#[test]
fn constructors_produce_valid_colors() {
    for (name, t) in all_presets() {
        for (i, c) in t.palette.roles().iter().enumerate() {
            assert!(
                c.is_valid(),
                "{name}: palette role #{i} out of range: {c:?}"
            );
        }
        assert!(
            t.focus.ring_color.is_valid(),
            "{name}: focus ring color invalid"
        );
    }
}

#[test]
fn spacing_radius_focus_are_finite_and_non_negative() {
    for (name, t) in all_presets() {
        let s = t.spacing;
        for v in [s.xs, s.sm, s.md, s.lg, s.xl, s.xxl] {
            assert!(v.is_finite() && v >= 0.0, "{name}: bad spacing {v}");
        }
        let r = t.radius;
        for v in [r.sm, r.md, r.lg, r.pill] {
            assert!(v.is_finite() && v >= 0.0, "{name}: bad radius {v}");
        }
        assert!(t.focus.ring_width.is_finite() && t.focus.ring_width >= 0.0);
        assert!(t.focus.ring_offset.is_finite() && t.focus.ring_offset >= 0.0);
    }
}

#[test]
fn line_heights_and_sizes_are_positive() {
    for (name, t) in all_presets() {
        let ty = t.typography;
        for role in [
            ty.body,
            ty.body_small,
            ty.label,
            ty.title,
            ty.heading,
            ty.display,
        ] {
            assert!(role.size > 0.0, "{name}: non-positive size {role:?}");
            assert!(
                role.line_height > 0.0,
                "{name}: non-positive line-height {role:?}"
            );
        }
    }
}

#[test]
fn density_default_is_comfortable() {
    for (name, t) in all_presets() {
        assert_eq!(t.density, crate::Density::Comfortable, "{name}");
    }
}

// ---- contrast ----

/// Asserts both colors are opaque and meet the minimum contrast ratio.
fn assert_pair(preset: &str, label: &str, fg: crate::Color, bg: crate::Color, min: f32) {
    assert!(fg.is_opaque(), "{preset}: {label} fg not opaque ({fg:?})");
    assert!(bg.is_opaque(), "{preset}: {label} bg not opaque ({bg:?})");
    let r = contrast_ratio(fg, bg);
    assert!(r >= min, "{preset}: {label} contrast {r:.2} < {min}");
}

fn mandatory_pairs(preset: &str, p: &Palette) {
    // Body text on surfaces.
    assert_pair(
        preset,
        "text_primary/background",
        p.text_primary,
        p.background,
        AA_TEXT,
    );
    assert_pair(
        preset,
        "text_primary/surface",
        p.text_primary,
        p.surface,
        AA_TEXT,
    );
    assert_pair(
        preset,
        "text_primary/surface_raised",
        p.text_primary,
        p.surface_raised,
        AA_TEXT,
    );
    assert_pair(
        preset,
        "text_secondary/background",
        p.text_secondary,
        p.background,
        AA_TEXT,
    );
    assert_pair(
        preset,
        "text_secondary/surface",
        p.text_secondary,
        p.surface,
        AA_TEXT,
    );
    // On-accent text.
    assert_pair(
        preset,
        "accent_text/accent",
        p.accent_text,
        p.accent,
        AA_TEXT,
    );
    // Danger button foreground (mandatory: danger button ships in v0.20).
    assert_pair(
        preset,
        "danger_text/danger",
        p.danger_text,
        p.danger,
        AA_TEXT,
    );
    // Other status foregrounds (required as their primitives ship; verified now).
    assert_pair(
        preset,
        "success_text/success",
        p.success_text,
        p.success,
        AA_TEXT,
    );
    assert_pair(
        preset,
        "warning_text/warning",
        p.warning_text,
        p.warning,
        AA_TEXT,
    );
    assert_pair(preset, "info_text/info", p.info_text, p.info, AA_TEXT);
    // Focus indicator (non-text target).
    assert_pair(preset, "focus/background", p.focus, p.background, FOCUS_MIN);
    assert_pair(preset, "focus/surface", p.focus, p.surface, FOCUS_MIN);
}

#[test]
fn all_presets_pass_mandatory_contrast() {
    for (name, t) in all_presets() {
        mandatory_pairs(name, &t.palette);
    }
}

#[test]
fn high_contrast_presets_exceed_aa_for_primary_text() {
    // High-contrast presets should clear a stronger bar (>= 7:1) for primary
    // body text on the background.
    for name in ["high_contrast_light", "high_contrast_dark"] {
        let t = match name {
            "high_contrast_light" => Tokens::high_contrast_light(),
            _ => Tokens::high_contrast_dark(),
        };
        let r = contrast_ratio(t.palette.text_primary, t.palette.background);
        assert!(r >= 7.0, "{name}: primary text contrast {r:.2} < 7.0");
    }
}

#[test]
fn customizing_a_token_does_not_affect_other_presets() {
    let mut a = Tokens::light();
    a.radius.md = 99.0;
    assert_eq!(a.radius.md, 99.0, "local mutation should take effect");
    assert_eq!(
        Tokens::light().radius.md,
        6.0,
        "presets must be independent"
    );
}
