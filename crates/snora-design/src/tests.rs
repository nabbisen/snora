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

/// WCAG 2.1 SC 1.4.3 normal-text minimum. Three copies of this name
/// exist — this one, `snora-widgets/src/contrast_tests.rs`, and
/// `snora/src/toast/contrast_tests.rs` — none shareable across the
/// crate boundaries involved. **Enforced by
/// `scripts/check-wcag-floors.sh`**, added after audit 2026-09-12 (C-1)
/// found the hand-maintained sibling comments in the other three
/// *files* carrying these constants had drifted out of sync with each
/// other about which constant has how many siblings.
const AA_TEXT: f32 = 4.5;
/// WCAG 2.1 SC 1.4.11 focus-indicator minimum. `3.0` today, same value
/// as [`NON_TEXT_MIN`] but a deliberately separate constant (see module
/// doc) — the only copy of this name under `crates/`, so it is not part
/// of the duplication `NON_TEXT_MIN` and `AA_TEXT` have; still checked
/// by `scripts/check-wcag-floors.sh` for symmetry, where it will only
/// ever report one copy.
const FOCUS_MIN: f32 = 3.0;
/// WCAG 2.1 SC 1.4.11 non-text minimum. **Four** copies of this name
/// exist — this one, `snora-widgets/src/contrast_tests.rs`,
/// `snora/src/toast/contrast_tests.rs`, and
/// `snora/src/design/render/tests.rs` (that fourth one drifted once
/// already — `1.3` instead of `3.0`, RFC-071 review round 1 — see its
/// own doc comment). Not the same sibling set as [`AA_TEXT`] above,
/// which has three. **Enforced by `scripts/check-wcag-floors.sh`.**
const NON_TEXT_MIN: f32 = 3.0;

/// Every shipped preset, from the one public source.
///
/// **This used to be a hand-kept list, and so were eight others** across
/// the workspace; four of the nine missed the AAA presets when they
/// landed (RFC-118's review). Now every sweep iterates
/// [`Tokens::presets`], and `presets_is_complete_*` below fails if a
/// constructor exists that it does not list — so a seventh preset
/// reaches every test with no list to forget.
fn all_presets() -> Vec<(&'static str, Tokens)> {
    Tokens::presets()
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
/// there), and in `high_contrast_light` the two signals **cross**, so
/// the worst point is carried by neither alone — border and fill are
/// equal there at 4.58:1.
///
/// **Corrected 2026-08-21.** This previously said `high_contrast_light`
/// "passes on border alone (its fill equals `background` … so it cannot
/// signal on its own)". That conflates two different pairs. Its fill
/// equals `background`, so it cannot signal against **the background** —
/// but this assertion measures against **the dim**, and there its worst
/// `fill ǀ dim` is **3.24:1**, clearing the bar on its own. Measured
/// worst-point figures, all four presets: `light` fill 3.24 / border
/// 1.04; `dark` fill 3.16 / border 1.00; `high_contrast_light` 4.58 /
/// 4.58; `high_contrast_dark` 4.45 / 4.45. Splitting this into two assertions would fail two correct
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
    let family: Vec<_> = Tokens::presets()
        .into_iter()
        .filter(|(name, _)| name.starts_with("high_contrast"))
        .collect();
    assert_eq!(
        family.len(),
        2,
        "expected the two high-contrast presets, found {:?}",
        family.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
    );
    for (name, t) in family {
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
///
/// The only copy of this name under `crates/`, so it has none of the
/// duplication `AA_TEXT` and `NON_TEXT_MIN` carry — still pinned at one
/// copy by `scripts/check-wcag-floors.sh`, for the same reason
/// [`FOCUS_MIN`] is: a second copy appearing is a fact somebody should
/// have decided.
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

// ---------------------------------------------------------------------
// The AAA presets (RFC-118)
// ---------------------------------------------------------------------

/// WCAG 1.4.6 AAA for body text.
///
/// Only the AAA presets are held to this. Light, Dark and the two
/// high-contrast presets keep their own floors — AA for the first two,
/// which is snora's stated floor, and AA for the high-contrast pair as
/// a *requirement* even though they happen to clear 7:1 everywhere.
const AAA_TEXT: f32 = 7.0;

/// The AAA family, by the suffix the names carry (RFC-118 Q-5).
///
/// Derived from [`Tokens::presets`] rather than listed, so an AAA preset
/// added later is held to AAA without anyone remembering to say so. The
/// count is asserted so that a rename which drops the suffix fails here
/// instead of quietly shrinking the sweep to nothing.
fn aaa_presets() -> Vec<(&'static str, Tokens)> {
    let family: Vec<_> = Tokens::presets()
        .into_iter()
        .filter(|(name, _)| name.ends_with("_aaa"))
        .collect();
    assert!(
        family.len() >= 2,
        "found {} AAA preset(s) by the `_aaa` suffix; expected at least light_aaa and dark_aaa",
        family.len(),
    );
    family
}

/// R-2: every text pair the palette *declares* is at AAA, derived from
/// [`Palette::usages`] rather than from a hand-written list.
///
/// **This is why RFC-118 depends on RFC-117's R-5.** Before R-5,
/// `usages` declared the accent as non-text only, so this sweep would
/// have skipped the very pair orbok asked about and reported a pass.
///
/// Non-text and focus pairs keep their own floors; they are covered by
/// `all_presets_pass_mandatory_contrast`, which sweeps these two
/// presets as well.
///
/// Every failure is collected and reported together, rather than
/// stopping at the first: the question "which pairs miss AAA" has a
/// list for an answer, and a hard assert would show one name at a time.
#[test]
fn aaa_presets_hold_every_declared_text_pair_to_aaa() {
    let mut failures = Vec::new();

    for (name, t) in aaa_presets() {
        for usage in t.palette.usages() {
            if usage.class != Some(ThresholdClass::Text) {
                continue;
            }
            for (surface_label, surface_color) in usage.surfaces {
                let r = contrast_ratio(usage.fg, surface_color);
                if r < AAA_TEXT {
                    failures.push(format!(
                        "{name}: {}/{surface_label} {r:.2} < {AAA_TEXT}",
                        usage.label
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} declared text pair(s) under AAA:\n  {}",
        failures.len(),
        failures.join("\n  "),
    );
}

/// The maximum hue shift an AAA preset may make to any role, in degrees,
/// measured in OKLCh against its parent preset.
///
/// **Proposed from what the values needed, not picked first.** The
/// largest shift the derivation produced is 5.442° — Dark's accent,
/// which has to lighten by 0.0706 in OKLCh L and whose chroma is
/// clipped slightly by the sRGB gamut on the way. The next largest are
/// Light's warning at 5.095° and Light's success at 2.251°; every other
/// role moves by under 0.7°, and most by nothing.
///
/// 6 leaves the derivation a little room without admitting a shift a
/// reader would call a different colour. **The owner rules on it**; this
/// is the figure the values support.
const AAA_MAX_HUE_SHIFT_DEG: f32 = 6.0;

/// Each AAA preset beside the parent it is derived from.
///
/// The parent is found by name — `light_aaa` is derived from `light` —
/// so the pairing is stated once, in the suffix convention, and not
/// again here.
fn aaa_parents() -> Vec<(&'static str, Tokens, Tokens)> {
    let all = Tokens::presets();
    aaa_presets()
        .into_iter()
        .map(|(name, tokens)| {
            let parent_name = name.trim_end_matches("_aaa");
            let parent = all
                .iter()
                .find(|(n, _)| *n == parent_name)
                .unwrap_or_else(|| panic!("{name} has no parent preset named {parent_name}"))
                .1
                .clone();
            (name, tokens, parent)
        })
        .collect()
}

/// OKLCh hue of a colour, in degrees, or `None` for a colour with no
/// meaningful hue.
///
/// A near-neutral has a hue that is numerically defined but visually
/// arbitrary: at chroma 0.001 the angle swings freely while the colour
/// stays grey, so comparing it to a parent's would fail on noise rather
/// than on anything a reader could see. Those roles are skipped, and
/// the test says how many it skipped so the number cannot quietly grow.
fn oklch_hue(c: crate::Color) -> Option<f32> {
    /// Below this chroma, hue is not a visible property.
    const NEUTRAL_CHROMA: f32 = 0.01;

    fn to_linear(v: f32) -> f32 {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    let (r, g, b) = (to_linear(c.r), to_linear(c.g), to_linear(c.b));
    let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;
    let (l_, m_, s_) = (l.cbrt(), m.cbrt(), s.cbrt());

    let a = 1.977_998_5 * l_ - 2.428_592_2 * m_ + 0.450_593_7 * s_;
    let bb = 0.025_904_037 * l_ + 0.782_771_77 * m_ - 0.808_675_77 * s_;

    let chroma = a.hypot(bb);
    if chroma < NEUTRAL_CHROMA {
        return None;
    }
    Some(bb.atan2(a).to_degrees().rem_euclid(360.0))
}

/// **Postcondition 1:** the text hierarchy keeps its order.
///
/// `text_primary` ≥ `text_secondary` ≥ `text_muted`, by contrast
/// against each surface. An AAA preset that made "muted" text stronger
/// than "secondary" would invert what the role names mean — and
/// compressing the hierarchy, which raising `text_muted` to 7:1 does,
/// is exactly the move that could overshoot into inverting it.
#[test]
fn aaa_presets_keep_the_text_hierarchy_in_order() {
    let mut failures = Vec::new();

    for (name, t, _parent) in aaa_parents() {
        let p = &t.palette;
        for (surface_label, surface) in [
            ("background", p.background),
            ("surface", p.surface),
            ("surface_raised", p.surface_raised),
        ] {
            let primary = contrast_ratio(p.text_primary, surface);
            let secondary = contrast_ratio(p.text_secondary, surface);
            let muted = contrast_ratio(p.text_muted, surface);

            if primary < secondary {
                failures.push(format!(
                    "{name}/{surface_label}: text_primary {primary:.2} < text_secondary \
                     {secondary:.2}",
                ));
            }
            if secondary < muted {
                failures.push(format!(
                    "{name}/{surface_label}: text_secondary {secondary:.2} < text_muted \
                     {muted:.2}",
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} hierarchy inversion(s):\n  {}",
        failures.len(),
        failures.join("\n  "),
    );
}

/// **Postcondition 2:** each changed role keeps its parent's hue.
///
/// So `light_aaa` still reads as Light, rather than being a different
/// theme wearing Light's name. Measured in OKLCh, where a hue angle
/// means roughly the same thing across lightnesses — which is the whole
/// reason the derivation moved lightness in that space rather than in
/// sRGB.
#[test]
fn aaa_presets_keep_their_parents_hues() {
    let mut failures = Vec::new();
    let mut compared = 0usize;
    let mut skipped_neutral = 0usize;

    for (name, t, parent) in aaa_parents() {
        let (a, b) = (&t.palette, &parent.palette);
        for (role, now, before) in [
            ("text_primary", a.text_primary, b.text_primary),
            ("text_secondary", a.text_secondary, b.text_secondary),
            ("text_muted", a.text_muted, b.text_muted),
            ("border", a.border, b.border),
            ("accent", a.accent, b.accent),
            ("accent_text", a.accent_text, b.accent_text),
            ("success", a.success, b.success),
            ("success_text", a.success_text, b.success_text),
            ("warning", a.warning, b.warning),
            ("warning_text", a.warning_text, b.warning_text),
            ("danger", a.danger, b.danger),
            ("danger_text", a.danger_text, b.danger_text),
            ("info", a.info, b.info),
            ("info_text", a.info_text, b.info_text),
            ("focus", a.focus, b.focus),
        ] {
            let (Some(h_now), Some(h_before)) = (oklch_hue(now), oklch_hue(before)) else {
                skipped_neutral += 1;
                continue;
            };
            compared += 1;
            let delta = (h_now - h_before).abs();
            let delta = delta.min(360.0 - delta);
            if delta > AAA_MAX_HUE_SHIFT_DEG {
                failures.push(format!(
                    "{name}: {role} moved {delta:.3}° in hue, past {AAA_MAX_HUE_SHIFT_DEG}° \
                     ({before:?} -> {now:?})",
                ));
            }
        }
    }

    assert!(
        compared >= 12,
        "only {compared} role(s) had a comparable hue; this test is not measuring what it \
         claims (skipped {skipped_neutral} as near-neutral)",
    );
    assert!(
        failures.is_empty(),
        "{} hue shift(s) past {AAA_MAX_HUE_SHIFT_DEG}°:\n  {}",
        failures.len(),
        failures.join("\n  "),
    );
}

// ---------------------------------------------------------------------
// `Tokens::presets()` is the one list, and it is complete (RFC-118)
// ---------------------------------------------------------------------

/// The names, in the order the function promises.
const PRESET_ORDER: [&str; 6] = [
    "light",
    "dark",
    "light_aaa",
    "dark_aaa",
    "high_contrast_light",
    "high_contrast_dark",
];

/// Zero-argument constructors in `impl Tokens` that are deliberately
/// **not** presets.
///
/// Empty, and meant to stay so. If a constructor genuinely is not a
/// named preset, it is listed here with the reason — the same shape as
/// `check-literal-sizes.sh`'s allow-list, and for the same purpose:
/// someone has to write down why.
const NOT_PRESETS: &[&str] = &[];

/// Zero-argument `pub fn name() -> Self` constructors in `tokens.rs`,
/// read from the file's own source.
///
/// **This is the mechanism.** A function cannot be enumerated from
/// inside Rust, and an exhaustive `match` only catches a new *variant*,
/// not a new *function*. So the source is the data, as
/// `state_register.rs` does for the tests it cross-checks: the pattern is
/// narrow enough (`pub fn <ident>() -> Self`) that a constructor taking
/// arguments, or returning anything else — `presets()` itself returns a
/// `Vec` — is not mistaken for a preset.
fn constructors_in_source() -> Vec<String> {
    const SOURCE: &str = include_str!("tokens.rs");
    SOURCE
        .lines()
        .filter_map(|line| {
            let rest = line.trim_start().strip_prefix("pub fn ")?;
            let (name, tail) = rest.split_once("()")?;
            let valid_ident =
                !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            (valid_ident && tail.trim_start().starts_with("-> Self")).then(|| name.to_owned())
        })
        .collect()
}

/// A constructor added without being added to `presets()` fails here.
#[test]
fn presets_is_complete_against_the_constructors_in_its_own_source() {
    let found = constructors_in_source();

    // Control: the scan must be finding the six known constructors, or
    // it is scanning nothing and every assertion below passes for free.
    for known in PRESET_ORDER {
        assert!(
            found.iter().any(|f| f == known),
            "the source scan did not find `{known}`, so it is not reading the constructors \
             it claims to (found {found:?})",
        );
    }

    let listed: Vec<&str> = Tokens::presets().iter().map(|(n, _)| *n).collect();
    let missing: Vec<&String> = found
        .iter()
        .filter(|f| !listed.contains(&f.as_str()) && !NOT_PRESETS.contains(&f.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "`Tokens` has constructor(s) {missing:?} that `Tokens::presets()` does not list. Add \
         each to `presets()` and to `PRESET_ORDER` here — then every sweep in the workspace \
         covers it — or, if it is not a named preset, list it in `NOT_PRESETS` with the reason.",
    );
}

/// Every listed name is a real constructor, and the order is the one
/// promised.
#[test]
fn presets_lists_exactly_the_promised_names_in_order() {
    let listed: Vec<&str> = Tokens::presets().iter().map(|(n, _)| *n).collect();
    assert_eq!(
        listed, PRESET_ORDER,
        "the documented order is part of the contract"
    );
}

/// Each name is paired with **its own** constructor's tokens.
///
/// `presets_lists_exactly_the_promised_names_in_order` would pass for
/// `("light", Tokens::dark())`: the names are right and the contents are
/// swapped. This is the check that a label means what it says.
#[test]
fn each_preset_name_builds_its_own_constructor() {
    for (name, tokens) in Tokens::presets() {
        let built = match name {
            "light" => Tokens::light(),
            "dark" => Tokens::dark(),
            "light_aaa" => Tokens::light_aaa(),
            "dark_aaa" => Tokens::dark_aaa(),
            "high_contrast_light" => Tokens::high_contrast_light(),
            "high_contrast_dark" => Tokens::high_contrast_dark(),
            other => {
                panic!("`presets()` lists `{other}`, which this test does not know how to build")
            }
        };
        // `assert!`, not `assert_eq!`: a failing `assert_eq!` prints both
        // `Tokens` in full, three kilobytes each, and the name is the
        // whole diagnosis.
        assert!(
            tokens == built,
            "`presets()` pairs `{name}` with another preset's tokens"
        );
    }
}

// ---------------------------------------------------------------------
// `light_aaa`'s focus is its accent, as Light's is (RFC-118 ruling)
// ---------------------------------------------------------------------

/// Light's focus is its accent, and `light_aaa`'s is its own new accent.
///
/// The identity is Light's, so it is asserted for the pair that has it
/// rather than as a general property of presets — Dark's focus was never
/// its accent, and `dark_aaa` is unchanged.
#[test]
fn light_aaa_focus_is_its_own_accent() {
    let light = Tokens::light();
    assert_eq!(
        light.palette.focus, light.palette.accent,
        "the premise: in Light the focus ring and the accent are one colour"
    );

    let aaa = Tokens::light_aaa();
    assert_eq!(
        aaa.palette.focus, aaa.palette.accent,
        "light_aaa's focus is not its accent, so it dropped the identity that makes it Light"
    );
    assert_ne!(
        aaa.palette.focus, light.palette.focus,
        "light_aaa's focus is Light's own, but its accent darkened: they have diverged"
    );
    assert_eq!(
        aaa.focus.ring_color, aaa.palette.focus,
        "the focus-ring token does not follow the palette's focus"
    );
}

/// `dark_aaa` is untouched: its focus is Dark's, and was never its accent.
#[test]
fn dark_aaa_keeps_darks_focus() {
    let (dark, aaa) = (Tokens::dark(), Tokens::dark_aaa());
    assert_eq!(aaa.palette.focus, dark.palette.focus);
    assert_ne!(
        dark.palette.focus, dark.palette.accent,
        "the premise: Dark's focus was never its accent"
    );
}

/// The darker focus still clears focus's own floor, on every neutral
/// surface.
///
/// Also covered by `all_presets_pass_mandatory_contrast`, which sweeps
/// `usages()`; asserted here by name because the ruling made a claim
/// about exactly this pair.
#[test]
fn light_aaa_focus_clears_its_floor() {
    let p = Tokens::light_aaa().palette;
    for (surface, colour) in [
        ("background", p.background),
        ("surface", p.surface),
        ("surface_raised", p.surface_raised),
    ] {
        let r = contrast_ratio(p.focus, colour);
        assert!(
            r >= FOCUS_MIN,
            "light_aaa focus on {surface}: {r:.2} < {FOCUS_MIN}"
        );
    }
}
