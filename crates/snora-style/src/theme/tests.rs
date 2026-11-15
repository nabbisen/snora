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

// RFC-118: every preset, from the one public source — see
// `Tokens::presets()`. This was a hand-kept list of four that missed the
// two AAA presets when they landed.
fn named_presets() -> Vec<(&'static str, Tokens)> {
    Tokens::presets()
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
    // and `background_strong_is_the_stated_colour_in_every_preset`.
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

/// **`background.strong`, stated (0.63 audit).** This used to compare `theme()`'s value with
/// `background_strong_color(seed)`, the function that produced it: it could not fail. The expected colours are
/// written here, six decimals as printed, one per preset (recorded from the code at the 0.63 audit, so they are
/// goldens and not an independent derivation: a change to the search, its start amount or its floor fails here
/// and points at `theme.md`). `.text` stays `text_primary`.
#[test]
fn background_strong_is_the_stated_colour_in_every_preset() {
    let want: [(&str, [f32; 3]); 6] = [
        ("light", [0.806045, 0.806046, 0.806045]),
        ("dark", [0.176565, 0.213017, 0.261495]),
        ("light_aaa", [0.806045, 0.806046, 0.806045]),
        ("dark_aaa", [0.176565, 0.213017, 0.261495]),
        ("high_contrast_light", [0.806045, 0.806046, 0.806045]),
        ("high_contrast_dark", [0.179236, 0.179236, 0.179236]),
    ];
    for (name, t) in named_presets() {
        let (_, rgb) = want.iter().find(|(n, _)| *n == name).unwrap_or_else(|| {
            panic!("`{name}` is a new preset: state its background.strong here")
        });
        let th = theme(&t);
        let ext = th.extended_palette();
        let got = ext.background.strong.color;
        for (c, w) in [got.r, got.g, got.b].into_iter().zip(rgb) {
            assert!(
                (c - w).abs() < 2e-6,
                "{name}: background.strong {got:?}, stated {rgb:?}"
            );
        }
        assert_eq!(
            ext.background.strong.text,
            iced::Color::from_rgba(
                t.palette.text_primary.r,
                t.palette.text_primary.g,
                t.palette.text_primary.b,
                t.palette.text_primary.a
            ),
            "{name}: background.strong.text must stay fixed at text_primary"
        );
    }
}

/// **The derived semantic tiers, stated (0.63 audit).** The derivation tests above compute their expectation with
/// the production `shift_away_from`; these state it. One preset per family (an AA preset, an AAA preset, and
/// `high_contrast_dark`, whose pure-black `secondary` is the clamp case the fallback exists for): `weak` and
/// `strong` of the five sets, as recorded at the audit, to 2e-6. A change to an amount, the direction rule or the
/// fallback fails here, and `theme.md` is where it must be said.
#[test]
fn derived_semantic_tiers_are_the_stated_colours() {
    // (preset, set, weak, strong)
    #[rustfmt::skip]
    let want: [(&str, &str, [f32; 3], [f32; 3]); 15] = [
        ("light", "primary",   [0.036467, 0.208922, 0.790470], [0.016230, 0.000000, 0.704639]),
        ("light", "secondary", [0.992540, 1.000000, 1.000000], [0.991306, 1.000000, 1.000000]),
        ("light", "success",   [0.000000, 0.437000, 0.149448], [0.000000, 0.340529, 0.000000]),
        ("light", "warning",   [0.545381, 0.278047, 0.000000], [0.456202, 0.153869, 0.000000]),
        ("light", "danger",    [0.636445, 0.000000, 0.000000], [0.534904, 0.000000, 0.000000]),
        ("dark_aaa", "primary",   [0.467072, 0.772207, 1.000000], [0.504255, 0.887261, 1.000000]),
        ("dark_aaa", "secondary", [0.026949, 0.053209, 0.094829], [0.000000, 0.001765, 0.040562]),
        ("dark_aaa", "success",   [0.244467, 0.966329, 0.532819], [0.052958, 1.000000, 0.578191]),
        ("dark_aaa", "warning",   [1.000000, 0.816714, 0.254435], [1.000000, 0.919885, 0.147675]),
        ("dark_aaa", "danger",    [1.000000, 0.478975, 0.482108], [1.000000, 0.518012, 0.532945]),
        ("high_contrast_dark", "primary",   [0.661879, 0.845189, 1.000000], [0.738087, 0.962330, 1.000000]),
        ("high_contrast_dark", "secondary", [0.002791, 0.002791, 0.002791], [0.043484, 0.043484, 0.043484]),
        ("high_contrast_dark", "success",   [0.499090, 0.986672, 0.648553], [0.519454, 1.000000, 0.716036]),
        ("high_contrast_dark", "warning",   [1.000000, 0.900057, 0.491002], [1.000000, 0.961956, 0.454469]),
        ("high_contrast_dark", "danger",    [1.000000, 0.654920, 0.656817], [1.000000, 0.732076, 0.737491]),
    ];
    let close = |a: iced::Color, b: [f32; 3]| {
        [a.r, a.g, a.b]
            .into_iter()
            .zip(b)
            .all(|(x, y)| (x - y).abs() < 2e-6)
    };
    for (preset, set, weak, strong) in want {
        let (_, t) = named_presets()
            .into_iter()
            .find(|(n, _)| *n == preset)
            .unwrap();
        let th = theme(&t);
        let e = th.extended_palette();
        let (w, st) = match set {
            "primary" => (e.primary.weak.color, e.primary.strong.color),
            "secondary" => (e.secondary.weak.color, e.secondary.strong.color),
            "success" => (e.success.weak.color, e.success.strong.color),
            "warning" => (e.warning.weak.color, e.warning.strong.color),
            _ => (e.danger.weak.color, e.danger.strong.color),
        };
        assert!(
            close(w, weak),
            "{preset} {set}.weak is {w:?}, stated {weak:?}"
        );
        assert!(
            close(st, strong),
            "{preset} {set}.strong is {st:?}, stated {strong:?}"
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
        // The floor is this test's own 1.5, not `BORDER_CONTRAST_FLOOR`: a guard must not read the constant it
        // guards (0.63 audit), or lowering the constant lowers the bar with it.
        assert!(
            r >= 1.5,
            "{name}: background.strong vs background.base contrast {r:.2} < 1.5 \
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
    let family: Vec<_> = Tokens::presets()
        .into_iter()
        .filter(|(name, _)| name.starts_with("high_contrast"))
        .collect();
    assert_eq!(family.len(), 2, "expected the two high-contrast presets");
    for (name, t) in family {
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

/// Whether a preset is meant to be dark, from its name.
///
/// The convention the names already carry — `dark`, `dark_aaa`,
/// `high_contrast_dark` are dark; `light`, `light_aaa`,
/// `high_contrast_light` are not. Stated once here, so a new preset has
/// to follow it or say why it does not.
fn intended_dark(name: &str) -> bool {
    name.contains("dark")
}

#[test]
fn is_dark_matches_preset_intent() {
    // RFC-118: every preset, from the one public source. This was four
    // hand-written assertions that missed `light_aaa` and `dark_aaa`.
    let presets = Tokens::presets();

    // Control: both polarities are present, or the loop below could pass
    // by only ever checking one of them.
    assert!(presets.iter().any(|(n, _)| intended_dark(n)));
    assert!(presets.iter().any(|(n, _)| !intended_dark(n)));

    for (name, tokens) in presets {
        let is_dark = theme(&tokens).extended_palette().is_dark;
        assert_eq!(
            is_dark,
            intended_dark(name),
            "{name} preset should {}be dark",
            if intended_dark(name) { "" } else { "not " }
        );
    }
}

/// RFC-065 review R-1: `snora_design::surfaces::modal_dim` reimplements
/// `iced::theme::palette::is_dark`'s OKLCH-lightness classification
/// iced-free, so `snora-design` can classify a background without
/// depending on iced to do it. Nothing previously asserted the two
/// classifications agree beyond the four built-in presets — an
/// unasserted equivalence claim, the exact shape RFC-058/061/064 each
/// closed elsewhere. `snora-style` is the only crate with both `iced`
/// and `snora-design` in scope, so the check belongs here.
///
/// `modal_dim`'s pole choice is driven entirely by that classification
/// (white pole iff the background classifies dark) — a change of pole
/// as `background` sweeps a color grid is a direct, cheap probe of
/// agreement with `iced::theme::palette::is_dark`, without needing
/// `snora-design` to expose the classifier itself as public API.
#[test]
fn modal_dim_pole_choice_agrees_with_iced_is_dark_across_a_color_grid() {
    let mut tokens = Tokens::light();
    let steps = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

    for &r in &steps {
        for &g in &steps {
            for &b in &steps {
                let background = SnColor::rgb(r, g, b);
                tokens.palette.background = background;

                let dim = snora_design::surfaces::modal_dim(&tokens);
                let snora_design_dark = dim.r > 0.5; // white pole chosen iff classified dark
                let iced_dark = is_dark(to_iced_color(background));

                assert_eq!(
                    snora_design_dark,
                    iced_dark,
                    "background rgb({r:.1}, {g:.1}, {b:.1}): snora_design::surfaces \
                     classified {}, iced::theme::palette::is_dark classified {} — the two \
                     implementations disagree",
                    if snora_design_dark { "dark" } else { "light" },
                    if iced_dark { "dark" } else { "light" }
                );
            }
        }
    }
}

/// **`engine-surfaces.md`, "The sheet panel": the table, asserted (0.63 audit).** The sheet's border is
/// `background.weak` and its fill `background.base`, under `theme(&tokens)`. The page prints, per preset, the
/// minimum of `border | dim` and of `dim | fill` over every grey the content could be, and the border against its
/// own fill (1.02-1.35:1, well under 3). These are the page's numbers, stated here to two decimals, so a change to
/// the theme derivation (or to the dim) that moves them fails and points at the page.
#[test]
fn engine_surfaces_md_sheet_table() {
    #[rustfmt::skip]
    let rows: [(&str, f32, f32, f32); 4] = [
        // preset, border|dim min, dim|fill min, own border|own fill
        ("light",               2.41, 3.24, 1.35),
        ("dark",                2.95, 3.80, 1.29),
        ("high_contrast_light", 2.41, 3.24, 1.35),
        ("high_contrast_dark",  4.17, 4.25, 1.02),
    ];
    for (name, bd_min, df_min, own) in rows {
        let (_, t) = named_presets()
            .into_iter()
            .find(|(n, _)| *n == name)
            .unwrap();
        let th = theme(&t);
        let ext = th.extended_palette();
        let border = to_sn(ext.background.weak.color);
        let fill = to_sn(ext.background.base.color);
        let dim = snora_design::surfaces::modal_dim(&t);
        let (mut bd, mut df) = (f32::INFINITY, f32::INFINITY);
        for step in 0..=1000 {
            let c = step as f32 / 1000.0;
            let over = snora_design::contrast::composite_over(dim, SnColor::rgb(c, c, c));
            bd = bd.min(contrast_ratio(border, over));
            df = df.min(contrast_ratio(fill, over));
        }
        let near = |what: &str, got: f32, want: f32| {
            assert!(
                (got - want).abs() <= 0.01,
                "{name}: {what} is {got:.3}, the page prints {want}"
            );
        };
        near("border | dim, minimum", bd, bd_min);
        near("dim | fill, minimum", df, df_min);
        near("own border | own fill", contrast_ratio(border, fill), own);
    }
    // The AAA children keep their parents' background and so these rows hold for them.
    for (aaa, parent) in [("light_aaa", "light"), ("dark_aaa", "dark")] {
        let get = |n: &str| {
            named_presets()
                .into_iter()
                .find(|(x, _)| *x == n)
                .unwrap()
                .1
        };
        let (a, b) = (theme(&get(aaa)), theme(&get(parent)));
        assert_eq!(
            a.extended_palette().background.weak.color,
            b.extended_palette().background.weak.color,
            "{aaa} against {parent}"
        );
    }
}
