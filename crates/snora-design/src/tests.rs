//! Cross-preset validation: token sanity (RFC-022/024) and mandatory automated
//! contrast tests (RFC-023).
//!
//! Threshold policy:
//! * normal/body text pairs: >= 4.5:1 (WCAG AA);
//! * non-text boundary pairs (focus indicators, borders that identify a
//!   component, e.g. the RFC-039 dialog card): >= 3.0:1 (WCAG 2.1 SC 1.4.11);
//! * high-contrast presets are expected to exceed these comfortably.
//!
//! `NON_TEXT_MIN` and `FOCUS_MIN` are both `3.0` today (RFC-058); they are
//! kept as separate constants rather than one shared name so focus and
//! border contrast can diverge later without one silently following the
//! other.
//!
//! All colors used in mandatory pairs must be fully opaque (the assertions
//! check this); alpha roles would need compositing first.

use crate::contrast::{composite_over, contrast_ratio};
use crate::palette::ThresholdClass;
use crate::surfaces::modal_dim;
use crate::{Palette, Tokens};

const AA_TEXT: f32 = 4.5;
const FOCUS_MIN: f32 = 3.0;
const NON_TEXT_MIN: f32 = 3.0;

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
        for usage in t.palette.usages() {
            assert!(
                usage.fg.is_valid(),
                "{name}: palette role {} out of range: {:?}",
                usage.label,
                usage.fg
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

/// Derives every mandatory contrast pair from [`Palette::usages`]
/// (RFC-063) — no hand-written pair list. A role's surfaces and
/// threshold class are declared once, in `palette.rs`, under exhaustive-
/// destructuring compiler enforcement; adding a `Palette` field without
/// declaring where it renders fails to compile there, before this
/// function ever runs. `class: None` (the fill/surface roles) is
/// skipped — they were declared as measuring nothing, deliberately.
fn mandatory_pairs(preset: &str, p: &Palette) {
    for usage in p.usages() {
        let Some(class) = usage.class else {
            continue;
        };
        let min = match class {
            ThresholdClass::Text => AA_TEXT,
            ThresholdClass::Focus => FOCUS_MIN,
            ThresholdClass::NonText => NON_TEXT_MIN,
        };
        for (surface_label, surface_color) in usage.surfaces {
            assert_pair(
                preset,
                &format!("{}/{surface_label}", usage.label),
                usage.fg,
                surface_color,
                min,
            );
        }
    }
}

#[test]
fn all_presets_pass_mandatory_contrast() {
    for (name, t) in all_presets() {
        mandatory_pairs(name, &t.palette);
    }
}

// ---- the modal dim, a composited/derived surface (RFC-065) ----

/// The dialog card ([`crate::surfaces`] doc; styled by the `snora` crate's
/// `design::render` module) sits over the modal dim, not directly over
/// one fixed surface — a fourth surface `Palette::usages` cannot express,
/// because it is not a `Palette` role. Asserted here instead, in the one
/// contrast suite this crate has, rather than split into a second suite
/// in `snora` (Q-1, RFC-065).
///
/// **Worst case across all three neutral surfaces the dim can sit
/// over** — `background`, `surface`, and `surface_raised` — since an
/// application can open a dialog over any of them. This is not a
/// simplification for symmetry: `high_contrast_dark`'s worst backdrop is
/// `surface_raised` (5.25:1), not `background` (5.74:1) — checking only
/// `background` would silently under-measure that preset.
///
/// **Either-signal, not both.** Under WCAG 2.1 SC 1.4.11 the card is
/// identifiable if *either* its border *or* its fill clears the bar —
/// `max(contrast(border, dim), contrast(surface_raised, dim))`, not two
/// separate assertions. Asserting both individually would fail two
/// presets that are genuinely fine: `dark` passes on fill alone (its
/// border measures ~1:1 against the dim — border and dim land on the same
/// luminance there), and `high_contrast_light` passes on border alone
/// (its fill equals `background`, by token design, so it cannot signal on
/// its own). Splitting this into two assertions would fail two correct
/// presets — read `max(...)` as intentional, not as a mistake to tighten.
#[test]
fn dialog_card_distinguishable_from_modal_dim_all_presets() {
    for (name, t) in all_presets() {
        let dim = modal_dim(&t);
        for (backdrop_label, backdrop) in [
            ("background", t.palette.background),
            ("surface", t.palette.surface),
            ("surface_raised", t.palette.surface_raised),
        ] {
            let dim_over_backdrop = composite_over(dim, backdrop);
            let border_contrast = contrast_ratio(t.palette.border, dim_over_backdrop);
            let fill_contrast = contrast_ratio(t.palette.surface_raised, dim_over_backdrop);
            let best = border_contrast.max(fill_contrast);
            assert!(
                best >= NON_TEXT_MIN,
                "{name}/{backdrop_label}: dialog card vs modal dim contrast {best:.2} < \
                 {NON_TEXT_MIN} (border {border_contrast:.2}, fill {fill_contrast:.2}) — the \
                 card would not be distinguishable from its own dimmed backdrop by either signal"
            );
        }
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

// ---- pointer target size (RFC-061) ----

/// WCAG 2.5.8 mandatory pointer-target minimum, logical pixels.
const POINTER_TARGET_MIN_HEIGHT: f32 = 24.0;

#[test]
fn pointer_target_height_meets_24px_for_every_role_and_padding_step() {
    // Height is `line_box + 2 × vertical_padding`, and both terms are
    // token values — the same property that lets the contrast suite
    // run without a renderer. Width (`content_advance +
    // 2 × horizontal_padding`) depends on the rendered string, the
    // font, and the shaping engine; snora cannot compute it and does
    // not assert it here — see accessibility-checklist.md for why, and
    // for the 44×44 preferred bar's per-combination status.
    //
    // Q-2 (RFC-061): asserts every `TextRole` × `Spacing` step
    // combination, not only the ones a prefab control actually uses.
    // Enumerating "the ones a control uses" would re-derive a
    // hand-maintained list of call sites — the same failure mode that
    // made three prior handoff scopes short this cycle. The full
    // matrix costs nothing to compute and is a stronger ratchet.
    for (name, t) in all_presets() {
        let ty = t.typography;
        let sp = t.spacing;
        for (role_name, role) in [
            ("body", ty.body),
            ("body_small", ty.body_small),
            ("label", ty.label),
            ("title", ty.title),
            ("heading", ty.heading),
            ("display", ty.display),
        ] {
            for (step_name, step) in [
                ("xs", sp.xs),
                ("sm", sp.sm),
                ("md", sp.md),
                ("lg", sp.lg),
                ("xl", sp.xl),
                ("xxl", sp.xxl),
            ] {
                let line_box = role.size * role.line_height;
                let height = line_box + 2.0 * step;
                assert!(
                    height >= POINTER_TARGET_MIN_HEIGHT,
                    "{name}: {role_name}/{step_name} height {height:.1} \
                     < {POINTER_TARGET_MIN_HEIGHT} (line_box {line_box:.1} + 2×{step})"
                );
            }
        }
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
