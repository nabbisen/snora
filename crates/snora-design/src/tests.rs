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
use crate::typography::{TextRole, Typography};
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

// ---- the modal dim, a composited/derived surface (RFC-065, swept RFC-066) ----

/// Content-range sweep resolution for [`worst_case_over_content_sweep`]
/// (RFC-066 Q-3). Convergence was measured directly: 10, 100, 1000, and
/// 10 000 steps give `hc_dark` minima of 4.9091, 4.4565, 4.4515, 4.4497
/// and `hc_light` minima of 4.7939, 4.6012, 4.5827, 4.5827 — **1000
/// lands within 0.002 of the 10 000-step answer**, at a cost (a
/// thousand float ops per preset) that is free in a unit test. The two
/// presets nearer the 3:1 floor (`light`, `dark`) are unaffected by
/// resolution at all: their true minimum sits at an *endpoint*
/// (content = white or black), exact at any step count — resolution
/// only changes the reported margin on the two presets already nowhere
/// near the bar. A bare `1000` invites someone to change it on taste;
/// this is why it is not one.
const DIM_SWEEP_STEPS: usize = 1000;

/// Worst-case contrast of the dialog card against the modal dim, swept
/// over the full achievable content range (RFC-066) rather than checked
/// at discrete surfaces — the dim is painted over whatever the
/// application actually rendered, which is a continuum, and for two of
/// the four built-in presets the true worst case is an **interior**
/// minimum a discrete check cannot see (RFC-065 recorded `high_contrast_light`
/// at 7.37 and `high_contrast_dark` at 5.25, checking only three
/// surfaces; the true minima are 4.58 and 4.45).
///
/// Returns `(worst_best, worst_content, worst_border, worst_fill)`: the
/// worst either-signal contrast found, the greyscale content value it
/// occurred at, and both channel contrasts there.
///
/// **Greyscale is the whole sweep, not a simplification of one.** The
/// dim composites channelwise in sRGB:
/// `dim_over_i = α·base_i + (1−α)·content_i`. Contrast depends only on
/// relative luminance, which is monotonic in each channel — so for
/// *any* content color in the RGB cube, `relative_luminance(dim_over)`
/// is bounded by its values at `content = black` and `content = white`,
/// and a greyscale sweep traverses that interval continuously. A 3D
/// sweep would add roughly 10⁶ points and **no** additional coverage —
/// if this is later "improved" into a color-cube sweep, that improvement
/// costs CI time for nothing.
///
/// **Sweep, not the analytic crossing, on purpose (Q-2).** The minimum
/// of `max(f, g)` lies either at an endpoint or where `f == g`, and
/// that crossing is solvable directly — three evaluations instead of
/// `DIM_SWEEP_STEPS`. The sweep is used anyway: it is exact enough (see
/// [`DIM_SWEEP_STEPS`]'s own convergence numbers) and is far less code
/// to get subtly wrong, in a test whose entire purpose is to be
/// trusted. The analytic form was declined for robustness, not
/// overlooked.
fn worst_case_over_content_sweep(
    dim: crate::Color,
    border: crate::Color,
    fill: crate::Color,
) -> (f32, f32, f32, f32) {
    let mut worst_best = f32::INFINITY;
    let mut worst_content = 0.0;
    let mut worst_border = 0.0;
    let mut worst_fill = 0.0;

    for step in 0..=DIM_SWEEP_STEPS {
        let c = step as f32 / DIM_SWEEP_STEPS as f32;
        let dim_over_content = composite_over(dim, crate::Color::rgb(c, c, c));
        let border_contrast = contrast_ratio(border, dim_over_content);
        let fill_contrast = contrast_ratio(fill, dim_over_content);
        let best = border_contrast.max(fill_contrast);

        if best < worst_best {
            worst_best = best;
            worst_content = c;
            worst_border = border_contrast;
            worst_fill = fill_contrast;
        }
    }

    (worst_best, worst_content, worst_border, worst_fill)
}

/// Failure-message context: the three named surfaces RFC-065 originally
/// checked, reported alongside the sweep's own worst case (RFC-066 §6)
/// so a failure names a location a maintainer can go look at, not just
/// a bare content fraction — `content 0.822` alone tells nobody which
/// part of their palette to inspect.
fn named_surface_report(t: &Tokens, dim: crate::Color) -> String {
    [
        ("background", t.palette.background),
        ("surface", t.palette.surface),
        ("surface_raised", t.palette.surface_raised),
    ]
    .into_iter()
    .map(|(label, backdrop)| {
        let dim_over_backdrop = composite_over(dim, backdrop);
        let border = contrast_ratio(t.palette.border, dim_over_backdrop);
        let fill = contrast_ratio(t.palette.surface_raised, dim_over_backdrop);
        format!("{label} (border {border:.2}, fill {fill:.2})")
    })
    .collect::<Vec<_>>()
    .join("; ")
}

/// The dialog card ([`crate::surfaces`] doc; styled by the `snora` crate's
/// `design::render` module) sits over the modal dim, not directly over
/// one fixed surface — a fourth surface `Palette::usages` cannot express,
/// because it is not a `Palette` role. Asserted here instead, in the one
/// contrast suite this crate has, rather than split into a second suite
/// in `snora` (Q-1, RFC-065).
///
/// **Swept over the full content range, not checked at three discrete
/// surfaces (RFC-066).** The sweep subsumes the three-surface check —
/// `background`, `surface`, and `surface_raised` are three specific
/// points inside the range it covers — so this replaces that check
/// rather than keeping both (Q-1, RFC-066): a strictly stronger
/// assertion alongside a strictly weaker one is only cost, and the
/// weaker one had just demonstrated what it costs by reporting 7.37
/// where the truth was 4.58. See [`worst_case_over_content_sweep`] for
/// the sweep itself and why greyscale suffices.
///
/// **Either-signal, not both.** Under WCAG 2.1 SC 1.4.11 the card is
/// identifiable if *either* its border *or* its fill clears the bar —
/// `max(border ǀ dim, fill ǀ dim)`, not two separate assertions.
/// Asserting both individually would fail two presets that are
/// genuinely fine: `dark` passes on fill alone (its border measures
/// ~1:1 against the dim — border and dim land on the same luminance
/// there), and `high_contrast_light` passes on border alone (its fill
/// equals `background`, by token design, so it cannot signal on its
/// own). Splitting this into two assertions would fail two correct
/// presets — read `max(...)` as intentional, not as a mistake to
/// tighten. `max` is also *why the interior minimum this sweep exists
/// to catch exists at all*: it is the point where the border and fill
/// contrast cross and neither channel alone is carrying the boundary.
#[test]
fn dialog_card_distinguishable_from_modal_dim_all_presets() {
    for (name, t) in all_presets() {
        let dim = modal_dim(&t);
        let (worst_best, worst_content, worst_border, worst_fill) =
            worst_case_over_content_sweep(dim, t.palette.border, t.palette.surface_raised);

        assert!(
            worst_best >= NON_TEXT_MIN,
            "{name}: dialog card vs modal dim worst-case contrast {worst_best:.2} < \
             {NON_TEXT_MIN} at content {worst_content:.3} (border {worst_border:.2}, fill \
             {worst_fill:.2}) — swept over the full achievable content range (RFC-066), not \
             just three named surfaces. Named-surface context: {}",
            named_surface_report(&t, dim)
        );
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
        let sp = t.spacing;
        for (role_name, role) in every_text_role(t.typography) {
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

// ---- 12px text floor (RFC-081) ----

/// Snora's own minimum text size, logical pixels — **not a WCAG number.**
/// SC 1.4.4 is about *resize*, not a minimum size; this floor has no
/// standards citation and the assertion below must not invent one. See
/// `docs/src/guides/readability.md` for the floor's own rationale
/// (including the remediation cost an earlier, less precise wording of
/// it already caused one consumer).
const TEXT_SIZE_MIN: f32 = 12.0;

/// Every `TextRole` in a bundle, paired with its name, **derived by
/// exhaustive destructuring rather than listed by hand**.
///
/// A seventh role added to [`Typography`] makes the pattern below fail to
/// compile (`E0027: pattern does not mention field ...`) until it is named
/// here — and every caller then covers it automatically. **Do not add `..`
/// to the pattern.** The compiler suggests it on a missing-field error, and
/// that suggestion silently returns this to the hand-maintained list it
/// replaced: two accessibility floors (the 12px text size and the 24px
/// pointer target) both iterate this function, and a role missing from a
/// literal array would escape both without failing anything.
///
/// Same mechanism, same reason, as [`crate::palette::Palette::usages`]
/// (RFC-063) one file over.
fn every_text_role(ty: Typography) -> [(&'static str, TextRole); 6] {
    let Typography {
        body,
        body_small,
        label,
        title,
        heading,
        display,
    } = ty;
    [
        ("body", body),
        ("body_small", body_small),
        ("label", label),
        ("title", title),
        ("heading", heading),
        ("display", display),
    ]
}

/// Asserts every `TextRole`'s `size` in all four **built-in** presets
/// clears [`TEXT_SIZE_MIN`]. **What this does and does not prove:** it
/// proves our four shipped presets comply, and would catch a future
/// preset edit that dropped a role below the floor. **It cannot
/// constrain a consumer's own `Tokens`** — `Typography`'s fields are
/// public and RFC-036's covenant freezes that surface, so
/// `tokens.typography.body.size = 8.0` in an application is
/// unreachable by any test this crate ships. Enforcing the floor at
/// construction would require private fields, a breaking change to a
/// frozen surface — out of scope (RFC-081 Q-1: presets only, no public
/// validator; nobody has asked for one, and a helper nobody calls is a
/// third thing to keep true).
#[test]
fn text_size_meets_12px_floor_for_every_role() {
    for (name, t) in all_presets() {
        for (role_name, role) in every_text_role(t.typography) {
            assert!(
                role.size >= TEXT_SIZE_MIN,
                "{name}: {role_name} size {:.1} < {TEXT_SIZE_MIN} — see \
                 docs/src/guides/readability.md for snora's own text-size floor",
                role.size
            );
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
