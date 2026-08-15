//! Base-tier fidelity, derived-tier determinism/contrast, and tier
//! distinctness for token-derived theme emission (RFC-038, round-1/round-2
//! fixes).
//!
//! - **Fidelity** (base tiers): every emitted `base` color must equal its
//!   source token role *exactly* — proves iced's `Pair::new` heuristic
//!   never ran.
//! - **Determinism** (derived tiers): `weak`, `strong`, and the
//!   `Background` sub-tiers must equal the same transform production
//!   uses, computed independently here — `shift_away_from` for everything
//!   except `Background.strong`, which is `background_strong_color`'s
//!   search (R-3, round-2 review).
//! - **Contrast**: every emitted `Pair`, base or derived, must clear WCAG
//!   AA (AAA where the underlying tokens already do for HC presets)
//!   against its own paired text.
//! - **Adjacent-surface distinctness** (R-3): `background.strong` must
//!   also clear a contrast floor against `background.base` — the
//!   comparison that matters for its actual role as a border/separator
//!   color in iced's stock widgets, which the text-contrast checks above
//!   cannot catch.
//! - **Distinctness**: `base`, `weak`, and `strong` must be pairwise
//!   distinct for every semantic set — the property whose absence broke
//!   stock-widget hover/pressed feedback in round 1.

use super::*;
use iced::theme::palette::Pair;
use snora_design::{Color as SnColor, Tokens, contrast::contrast_ratio};

const AA: f32 = 4.5;
const AAA: f32 = 7.0;

fn named_presets() -> [(&'static str, Tokens); 4] {
    [
        ("light", Tokens::light()),
        ("dark", Tokens::dark()),
        ("high_contrast_light", Tokens::high_contrast_light()),
        ("high_contrast_dark", Tokens::high_contrast_dark()),
    ]
}

fn to_sn(c: iced::Color) -> SnColor {
    SnColor::rgba(c.r, c.g, c.b, c.a)
}

fn assert_color_eq(preset: &str, label: &str, emitted: iced::Color, expected: SnColor) {
    assert_eq!(
        emitted,
        to_iced_color(expected),
        "{preset}: {label} — emitted color does not exactly match its source token role \
         (iced's Pair::new heuristic may have run)"
    );
}

fn assert_pair_aa(preset: &str, label: &str, pair: Pair) {
    let r = contrast_ratio(to_sn(pair.text), to_sn(pair.color));
    assert!(
        r >= AA,
        "{preset}: {label} contrast {r:.2} < {AA} (WCAG AA)"
    );
}

// ---------------------------------------------------------------------------
// Fidelity: base tiers equal their source token role exactly.
// ---------------------------------------------------------------------------

#[test]
fn base_palette_matches_token_roles_exactly() {
    for (name, t) in named_presets() {
        let th = theme(&t);
        let base = th.palette();
        let p = &t.palette;
        assert_color_eq(name, "background", base.background, p.background);
        assert_color_eq(name, "text", base.text, p.text_primary);
        assert_color_eq(name, "primary", base.primary, p.accent);
        assert_color_eq(name, "success", base.success, p.success);
        assert_color_eq(name, "warning", base.warning, p.warning);
        assert_color_eq(name, "danger", base.danger, p.danger);
    }
}

#[test]
fn base_tiers_match_token_roles_exactly() {
    for (name, t) in named_presets() {
        let th = theme(&t);
        let ext = th.extended_palette();
        let p = &t.palette;

        assert_color_eq(
            name,
            "background.base",
            ext.background.base.color,
            p.background,
        );
        assert_color_eq(name, "primary.base", ext.primary.base.color, p.accent);
        assert_color_eq(
            name,
            "primary.base.text",
            ext.primary.base.text,
            p.accent_text,
        );
        assert_color_eq(name, "secondary.base", ext.secondary.base.color, p.surface);
        assert_color_eq(
            name,
            "secondary.base.text",
            ext.secondary.base.text,
            p.text_primary,
        );
        assert_color_eq(name, "success.base", ext.success.base.color, p.success);
        assert_color_eq(
            name,
            "success.base.text",
            ext.success.base.text,
            p.success_text,
        );
        assert_color_eq(name, "warning.base", ext.warning.base.color, p.warning);
        assert_color_eq(
            name,
            "warning.base.text",
            ext.warning.base.text,
            p.warning_text,
        );
        assert_color_eq(name, "danger.base", ext.danger.base.color, p.danger);
        assert_color_eq(
            name,
            "danger.base.text",
            ext.danger.base.text,
            p.danger_text,
        );
    }
}

// ---------------------------------------------------------------------------
// Determinism: derived tiers equal the documented mix/deviate transform.
// ---------------------------------------------------------------------------

#[test]
fn derived_semantic_tiers_match_expected_transform() {
    const WEAK_SHIFT: f32 = 0.06;
    const STRONG_SHIFT: f32 = 0.15;

    for (name, t) in named_presets() {
        let th = theme(&t);
        let ext = th.extended_palette();
        let p = &t.palette;

        let cases: [(&str, iced::Color, iced::Color, Pair, Pair); 5] = [
            (
                "primary",
                to_iced_color(p.accent),
                to_iced_color(p.accent_text),
                ext.primary.weak,
                ext.primary.strong,
            ),
            (
                "secondary",
                to_iced_color(p.surface),
                to_iced_color(p.text_primary),
                ext.secondary.weak,
                ext.secondary.strong,
            ),
            (
                "success",
                to_iced_color(p.success),
                to_iced_color(p.success_text),
                ext.success.weak,
                ext.success.strong,
            ),
            (
                "warning",
                to_iced_color(p.warning),
                to_iced_color(p.warning_text),
                ext.warning.weak,
                ext.warning.strong,
            ),
            (
                "danger",
                to_iced_color(p.danger),
                to_iced_color(p.danger_text),
                ext.danger.weak,
                ext.danger.strong,
            ),
        ];

        for (label, base_color, text, weak, strong) in cases {
            let expected_weak = shift_away_from(base_color, text, WEAK_SHIFT);
            let expected_strong = shift_away_from(base_color, text, STRONG_SHIFT);
            assert_eq!(
                weak.color, expected_weak,
                "{name}: {label}.weak does not match shift_away_from(base, text, {WEAK_SHIFT})"
            );
            assert_eq!(
                weak.text, text,
                "{name}: {label}.weak.text must stay fixed at the token's paired text"
            );
            assert_eq!(
                strong.color, expected_strong,
                "{name}: {label}.strong does not match shift_away_from(base, text, {STRONG_SHIFT})"
            );
            assert_eq!(
                strong.text, text,
                "{name}: {label}.strong.text must stay fixed at the token's paired text"
            );
        }
    }
}

#[test]
fn background_derived_tiers_match_expected_transform() {
    // strong is excluded — computed by `background_strong_color`'s search,
    // not a fixed amount; see `background_strong_tier_clears_border_floor`
    // and `background_strong_matches_expected_search`.
    const FIXED_AMOUNTS: [f32; 6] = [0.03, 0.07, 0.10, 0.125, 0.175, 0.20];

    for (name, t) in named_presets() {
        let th = theme(&t);
        let ext = th.extended_palette();
        let seed = to_iced_color(t.palette.background);
        let text = to_iced_color(t.palette.text_primary);

        let tiers = [
            ("weakest", ext.background.weakest),
            ("weaker", ext.background.weaker),
            ("weak", ext.background.weak),
            ("neutral", ext.background.neutral),
            ("stronger", ext.background.stronger),
            ("strongest", ext.background.strongest),
        ];

        for ((label, pair), amount) in tiers.into_iter().zip(FIXED_AMOUNTS) {
            // R-3 (round-2 review): direction keys off the seed color
            // itself, not off text — background tiers are compared
            // against each other (borders vs. adjacent surfaces), not
            // against text. `reference == seed` here, matching
            // `derive_background`'s call site exactly.
            let expected = shift_away_from(seed, seed, amount);
            assert_eq!(
                pair.color, expected,
                "{name}: background.{label} does not match shift_away_from(background, background, {amount})"
            );
            assert_eq!(
                pair.text, text,
                "{name}: background.{label}.text must stay fixed at text_primary"
            );
        }
    }
}

#[test]
fn background_strong_matches_expected_search() {
    for (name, t) in named_presets() {
        let th = theme(&t);
        let ext = th.extended_palette();
        let seed = to_iced_color(t.palette.background);
        let text = to_iced_color(t.palette.text_primary);

        assert_eq!(
            ext.background.strong.color,
            background_strong_color(seed),
            "{name}: background.strong does not match background_strong_color(background)"
        );
        assert_eq!(
            ext.background.strong.text, text,
            "{name}: background.strong.text must stay fixed at text_primary"
        );
    }
}

// ---------------------------------------------------------------------------
// Adjacent-surface distinctness: background.strong must be visible as a
// border against background.base, not just legible-against-text (R-3).
// ---------------------------------------------------------------------------

#[test]
fn background_strong_clears_border_floor_all_presets() {
    for (name, t) in named_presets() {
        let th = theme(&t);
        let ext = th.extended_palette();
        let r = contrast_ratio(
            to_sn(ext.background.strong.color),
            to_sn(ext.background.base.color),
        );
        assert!(
            r >= BORDER_CONTRAST_FLOOR,
            "{name}: background.strong vs background.base contrast {r:.2} < {BORDER_CONTRAST_FLOOR} \
             — a border this close to its surface is effectively invisible"
        );
    }
}

/// Direct unit test for [`background_strong_color`]'s search, isolated
/// from any specific preset. Pure black is the degenerate case that
/// motivated the search in the first place (`high_contrast_dark`'s
/// background) — a fixed amount starting point cannot clear the floor
/// there, so the search must grow past it.
#[test]
fn background_strong_color_grows_past_start_for_pure_black() {
    let black = iced::Color::BLACK;
    let strong = background_strong_color(black);
    let r = contrast_ratio(to_sn(strong), to_sn(black));
    assert!(
        r >= BORDER_CONTRAST_FLOOR,
        "background_strong_color(black) contrast {r:.2} < {BORDER_CONTRAST_FLOOR}"
    );
    let at_start_amount = shift_away_from(black, black, BACKGROUND_STRONG_START_AMOUNT);
    assert_ne!(
        strong, at_start_amount,
        "pure black needs the search to grow past the starting amount to clear the floor"
    );
}

// ---------------------------------------------------------------------------
// Contrast: every emitted pair, base and derived, meets WCAG AA.
// ---------------------------------------------------------------------------

#[test]
fn every_emitted_pair_meets_aa_all_presets() {
    for (name, t) in named_presets() {
        let th = theme(&t);
        let ext = th.extended_palette();

        let pairs = [
            ("background.base", ext.background.base),
            ("background.weakest", ext.background.weakest),
            ("background.weaker", ext.background.weaker),
            ("background.weak", ext.background.weak),
            ("background.neutral", ext.background.neutral),
            ("background.strong", ext.background.strong),
            ("background.stronger", ext.background.stronger),
            ("background.strongest", ext.background.strongest),
            ("primary.base", ext.primary.base),
            ("primary.weak", ext.primary.weak),
            ("primary.strong", ext.primary.strong),
            ("secondary.base", ext.secondary.base),
            ("secondary.weak", ext.secondary.weak),
            ("secondary.strong", ext.secondary.strong),
            ("success.base", ext.success.base),
            ("success.weak", ext.success.weak),
            ("success.strong", ext.success.strong),
            ("warning.base", ext.warning.base),
            ("warning.weak", ext.warning.weak),
            ("warning.strong", ext.warning.strong),
            ("danger.base", ext.danger.base),
            ("danger.weak", ext.danger.weak),
            ("danger.strong", ext.danger.strong),
        ];

        for (label, pair) in pairs {
            assert_pair_aa(name, label, pair);
        }
    }
}

#[test]
fn high_contrast_presets_meet_aaa_where_tokens_already_do() {
    // Mirrors snora-design's own
    // `high_contrast_presets_exceed_aa_for_primary_text`: text_primary on
    // background already clears AAA for both HC presets at the token
    // level, so the corresponding emitted *base* pair (background.base,
    // which is exactly text_primary/background — an exact-fidelity tier,
    // not derived) must too.
    for name in ["high_contrast_light", "high_contrast_dark"] {
        let t = if name == "high_contrast_light" {
            Tokens::high_contrast_light()
        } else {
            Tokens::high_contrast_dark()
        };
        let th = theme(&t);
        let ext = th.extended_palette();
        let pair = ext.background.base;
        let r = contrast_ratio(to_sn(pair.text), to_sn(pair.color));
        assert!(
            r >= AAA,
            "{name}: background.base (text_primary/background) contrast {r:.2} < {AAA} (WCAG AAA)"
        );
    }
}

// ---------------------------------------------------------------------------
// Distinctness: base/weak/strong must not collapse to one value.
// ---------------------------------------------------------------------------

#[test]
fn semantic_tiers_are_pairwise_distinct_all_presets() {
    for (name, t) in named_presets() {
        let th = theme(&t);
        let ext = th.extended_palette();

        let sets = [
            (
                "primary",
                ext.primary.base,
                ext.primary.weak,
                ext.primary.strong,
            ),
            (
                "secondary",
                ext.secondary.base,
                ext.secondary.weak,
                ext.secondary.strong,
            ),
            (
                "success",
                ext.success.base,
                ext.success.weak,
                ext.success.strong,
            ),
            (
                "warning",
                ext.warning.base,
                ext.warning.weak,
                ext.warning.strong,
            ),
            (
                "danger",
                ext.danger.base,
                ext.danger.weak,
                ext.danger.strong,
            ),
        ];

        for (label, base, weak, strong) in sets {
            assert_ne!(
                base.color, weak.color,
                "{name}: {label}.base == {label}.weak — collapsed tier removes interaction feedback"
            );
            assert_ne!(
                base.color, strong.color,
                "{name}: {label}.base == {label}.strong — collapsed tier removes interaction feedback"
            );
            assert_ne!(
                weak.color, strong.color,
                "{name}: {label}.weak == {label}.strong — collapsed tier removes interaction feedback"
            );
        }
    }
}

/// Direct unit test for the extreme-value fallback in `shift_away_from`,
/// isolated from any specific preset. `high_contrast_dark`'s `secondary`
/// (pure-black `surface` against white `text_primary`) is what surfaced
/// this during development — this test pins the underlying behavior so a
/// future refactor of `shift_away_from` can't silently reintroduce the
/// collapse without a preset happening to exercise it.
#[test]
fn shift_away_from_does_not_collapse_at_a_clamped_extreme() {
    let black = iced::Color::BLACK;
    let white = iced::Color::WHITE;

    // text is light -> primary direction is darken; black can't darken
    // further, so the fallback (lighten) must still produce a change.
    assert_ne!(shift_away_from(black, white, 0.06), black);

    // text is dark -> primary direction is lighten; white can't lighten
    // further, so the fallback (darken) must still produce a change.
    assert_ne!(shift_away_from(white, black, 0.06), white);
}

// ---------------------------------------------------------------------------
// is_dark
// ---------------------------------------------------------------------------

#[test]
fn is_dark_matches_preset_intent() {
    assert!(
        !theme(&Tokens::light()).extended_palette().is_dark,
        "light preset should not be dark"
    );
    assert!(
        theme(&Tokens::dark()).extended_palette().is_dark,
        "dark preset should be dark"
    );
    assert!(
        !theme(&Tokens::high_contrast_light())
            .extended_palette()
            .is_dark,
        "high_contrast_light preset should not be dark"
    );
    assert!(
        theme(&Tokens::high_contrast_dark())
            .extended_palette()
            .is_dark,
        "high_contrast_dark preset should be dark"
    );
}
