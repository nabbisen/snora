//! Token-derived `iced::Theme` emission for Snora Design (RFC-038).
//!
//! [`theme`] builds a complete `iced::Theme` from a Snora Design token
//! bundle without letting iced substitute its own colors. Every [`Pair`] in
//! the emitted `Extended` palette is a struct literal — never [`Pair::new`],
//! which "corrects" a foreground color by lightening or darkening it until
//! it clears a heuristic `relative_contrast >= 6.0` bar. Routing
//! `snora-design`'s contrast-tested roles through that would silently
//! discard the WCAG AA guarantee `snora-design`'s own contrast tests already
//! establish.
//!
//! # Base tiers vs. derived tiers (round-1/round-2 corrections)
//!
//! An earlier revision made every tier of every semantic set (`base`,
//! `weak`, `strong`) identical, reasoning that `snora-design` has no
//! weak/strong token variants so exact fidelity could only be satisfied by
//! collapsing them. That was wrong: iced's stock widgets key interaction
//! state off these tiers — `button::primary` reads `primary.base` at rest
//! and `primary.strong` on hover (`iced_widget-0.14.2/src/button.rs:600,605`)
//! — so a collapsed theme silently removes all hover/pressed feedback from
//! every stock button, which is worse than iced's own default theme.
//!
//! The corrected rule: **base** tiers equal their source token role
//! *exactly* (proving iced's heuristic never ran). **Derived** tiers
//! (`weak`, `strong`, and the eight `Background` sub-tiers) are computed by
//! a deterministic transform, `shift_away_from` — built as `Pair` struct
//! literals so the *text* color is never routed through [`Pair::new`]'s
//! `readable()` heuristic; it stays fixed at the token's own paired
//! foreground.
//!
//! `shift_away_from` uses iced's own public [`iced::theme::palette::lighten`]
//! / [`iced::theme::palette::darken`], but its direction depends on *which*
//! call site is asking — this is [`shift_away_from`]'s `reference`
//! parameter, and the two derivations below use it differently on purpose.
//!
//! For semantic sets ([`derive_tiers`] — `weak`/`strong` of `primary`,
//! `secondary`, `success`, `warning`, `danger`), the direction comes from
//! the *fixed paired text*'s darkness, not the base color's own (unlike
//! iced's own [`iced::theme::palette::deviate`]). This guarantees each
//! derived tier's contrast against that text is no worse than `base`'s:
//! moving the color further from the text's own tone can only widen the
//! luminance gap. Two unsafe alternatives were tried and rejected
//! empirically during development, not by inspection: mixing `weak`
//! toward `background` (iced's own `Primary::generate` shape) degenerates
//! to `base` unmodified whenever `background` happens to equal the seed
//! color — both high-contrast presets deliberately do
//! (`surface == background`) — and even where it doesn't collapse, a 0.4
//! wash toward a white/black background dropped one preset's contrast to
//! 2.04:1; plain `deviate` (direction from the *base color's* own
//! darkness) held for most cases but still dropped one preset's `success`
//! tier to 4.40:1, just under the 4.5 AA floor. Text-relative direction
//! removed both failures without per-preset tuning — see `theme/tests.rs`.
//!
//! For `Background` ([`derive_background`]), the direction instead comes
//! from the seed color's *own* darkness — matching `deviate` exactly. An
//! earlier revision (round 2) used the text-relative rule here too, which
//! was wrong (R-3, round-2 review): in the `dark` preset, a dark-but-not-
//! extreme background paired with light text picked "darken" as the
//! contrast-increasing direction, so every background tier darkened from
//! an already near-black background — invisible borders, since iced reads
//! `background.strong` as a border color
//! (`iced_widget-0.14.2/src/button.rs:700,721`,
//! `checkbox.rs:574,581`). `Background` tiers are compared against each
//! other (borders vs. adjacent surfaces), not against text, so the
//! text-relative guarantee that helps semantic sets does not apply and
//! only picked the wrong direction. Two independent signals confirmed the
//! fix: `dark`'s own `border` token is lighter than its `background`
//! (the token set's own intent, contradicted by the old direction), and
//! `high_contrast_dark` only ever looked correct by accident, via its
//! pure-black background forcing the clamp fallback.
//!
//! These two derivations are deliberately inconsistent with each other —
//! see [`derive_tiers`]'s doc comment for why that's a flagged choice, not
//! an oversight.
//!
//! # The 18 → 6 mapping
//!
//! Full table and rationale: `docs/src/design/theme.md`. Summary:
//!
//! - The six-slot base [`iced::theme::Palette`] (what `Theme::palette()`
//!   reports) maps directly: `background`, `text` ← `text_primary`,
//!   `primary` ← `accent`, `success`, `warning`, `danger`.
//! - The full `Extended` palette (what widgets actually read via
//!   `extended_palette()`) is constructed exhaustively from all 18 roles;
//!   `Extended` is not `#[non_exhaustive]`, so a future iced field addition
//!   is a compile error here, not a silent default.
//! - `secondary` has no corresponding token role (round-1 correction:
//!   originally mapped to `info`/`info_text`, which is wrong — iced derives
//!   `Secondary` from `background`+`text` as a **neutral** set, not a
//!   semantic accent (`iced_core-0.14.0/src/theme/palette.rs:403`); mapping
//!   it to `info` rendered every stock secondary control in the info hue).
//!   Now derived from `surface`/`text_primary`, the neutral family, matching
//!   iced's own derivation shape.
//! - `text_secondary`, `text_muted`, `border`, and `focus` are not
//!   represented in the emitted theme at all. `border` was tried as an
//!   eighth background-fill tier and reverted: in `high_contrast_light` it
//!   is bitwise identical to `text_primary` (both pure black), so pairing
//!   it as a background emitted invisible black-on-black text — caught by
//!   the contrast tests exactly as intended, not shipped. Applications
//!   needing these four roles read `tokens.palette` directly.
//!
//! # `is_dark`
//!
//! `Tokens` carries no preset-identity field (there is no `Tokens::is_dark()`
//! or equivalent), and this function's signature is fixed to `&Tokens` only
//! per RFC-038 — so `is_dark` cannot be threaded through as a separate
//! argument. It is derived from the token data via iced's own public
//! [`iced::theme::palette::is_dark`] on `palette.background`, which is
//! exactly the function `Extended::generate` itself uses internally. This
//! reuses iced's canonical light/dark determination rather than inventing a
//! second one.

use iced::Theme;
use iced::theme::palette::{
    Background, Danger, Extended, Pair, Palette as IcedPalette, Primary, Secondary, Success,
    Warning, darken, is_dark, lighten,
};
use snora_design::Tokens;
use snora_design::contrast::contrast_ratio;

use super::color::to_iced_color;

/// Shift amount for a semantic set's `weak` tier.
const WEAK_SHIFT_AMOUNT: f32 = 0.06;

/// Shift amount for a semantic set's `strong` tier. Larger than
/// [`WEAK_SHIFT_AMOUNT`] so the two tiers, and `base`, stay pairwise
/// distinct.
const STRONG_SHIFT_AMOUNT: f32 = 0.15;

/// Fixed shift amounts for `Background`'s six non-`base`, non-`strong`
/// tiers, in tier order (`weakest, weaker, weak, neutral, stronger,
/// strongest` — `strong` is excluded; see [`background_strong_color`]).
/// Same magnitudes `iced::theme::palette::Background::new` uses
/// internally for those slots.
const BACKGROUND_FIXED_SHIFT_AMOUNTS: [f32; 6] = [0.03, 0.07, 0.10, 0.125, 0.175, 0.20];

/// Minimum WCAG contrast [`background_strong_color`] must reach against
/// its own seed. iced's stock widgets read `Background.strong` as a
/// border/separator color compared against the adjacent surface, not as
/// a text background (`iced_widget-0.14.2/src/button.rs:700,721`,
/// `checkbox.rs:574,581`, `rule.rs:308`, `slider.rs:687`, and others) — a
/// gap the fidelity/text-contrast test suite couldn't catch, since it
/// only checks each tier's color against its own paired text (R-3,
/// round-2 review).
///
/// `1.5` is a modest, clearly-nonzero visibility bar — deliberately below
/// WCAG SC 1.4.11's full `3.0` non-text-contrast threshold for arbitrary
/// UI component boundaries. Reaching `3.0` uniformly would require
/// growing the shift amount for presets that already clear a reasonable
/// bar (`light`, `dark`, `high_contrast_light` all sit at `1.5`–`1.6` at
/// the original, unmodified amount), widening this change's blast radius
/// well past the defect actually found — a single extreme preset
/// (`high_contrast_dark`'s pure-black background) failing outright.
const BORDER_CONTRAST_FLOOR: f32 = 1.5;

/// Step size for [`background_strong_color`]'s search.
const BORDER_SEARCH_STEP: f32 = 0.01;

/// Starting shift amount for `Background.strong` — identical to the fixed
/// amount used before the R-3 fix, and to [`STRONG_SHIFT_AMOUNT`] (same
/// magnitude, unrelated purpose — kept as a separate constant so a future
/// change to one doesn't silently change the other).
const BACKGROUND_STRONG_START_AMOUNT: f32 = 0.15;

/// Shifts `color` by `amount`, choosing lighten-vs-darken from
/// `reference`'s darkness.
///
/// `reference` is a separate parameter, not always `color` itself, because
/// the two call sites need different notions of "away from": [`derive_tiers`]
/// passes the tier's *fixed paired text* as `reference` (round-1/R-1 fix —
/// this is the property that makes a derived tier's contrast against that
/// text provably no worse than `base`'s, since moving `color` further from
/// `reference`'s tone can only widen the luminance gap, never narrow it).
/// [`derive_background`] instead passes `color` as its own `reference`
/// (round-2/R-3 fix), matching [`iced::theme::palette::deviate`]'s
/// direction exactly — see that function's call site for why a
/// text-relative direction was wrong there.
///
/// A color already at a luminance extreme (pure black or white) cannot
/// move further in the `reference`-relative direction — `darken`/`lighten`
/// clamp, so the shift would be a no-op and the tier would collapse back
/// onto `base` (found in `high_contrast_dark`, where `secondary`'s base
/// color is pure black). When that happens, shift the *other* way instead:
/// a color already at that extreme has the most contrast headroom to
/// spare, so a small step in the "wrong" direction is safe.
fn shift_away_from(color: iced::Color, reference: iced::Color, amount: f32) -> iced::Color {
    let toward_contrast = if is_dark(reference) { lighten } else { darken };
    let shifted = toward_contrast(color, amount);
    if shifted != color {
        shifted
    } else {
        let fallback = if is_dark(reference) { darken } else { lighten };
        fallback(color, amount)
    }
}

/// Derives a semantic tier set (`Primary`, `Secondary`, `Success`, `Warning`,
/// or `Danger` all share this `{ base, weak, strong }` shape) from a single
/// base color and a fixed foreground.
///
/// `text` is held constant across all three tiers and is never passed
/// through [`Pair::new`] — this is the property that keeps the emitted
/// foreground exactly the token's verified paired role.
///
/// **Flagged choice (R-3, round-2 review):** unlike [`derive_background`],
/// this keys `shift_away_from`'s direction off `text`, not off
/// `base_color` itself — i.e. it deliberately does *not* match
/// [`iced::theme::palette::deviate`]'s direction here, even though
/// `derive_background` now does. Kept because every semantic tier's job
/// *is* legibility of `text` painted on top of it — unlike `Background`,
/// whose tiers are read as border/fill colors compared against each
/// other, not against text. A `deviate`-style, color's-own-darkness
/// direction was tried for these tiers during the R-1 fix and measured a
/// real AA failure (`light`'s `success.weak` at 4.40:1, see `theme/
/// tests.rs`); text-relative direction is what removed it, and nothing
/// about the R-3 background finding changes that — the two derivations
/// solve different problems and inconsistency between them is intentional,
/// not an oversight.
fn derive_tiers(base_color: iced::Color, text: iced::Color) -> (Pair, Pair, Pair) {
    let base = Pair {
        color: base_color,
        text,
    };
    let weak = Pair {
        color: shift_away_from(base_color, text, WEAK_SHIFT_AMOUNT),
        text,
    };
    let strong = Pair {
        color: shift_away_from(base_color, text, STRONG_SHIFT_AMOUNT),
        text,
    };
    (base, weak, strong)
}

/// Derives the eight-tier `Background` gradient from a single seed color,
/// built as `Pair` struct literals (never [`Pair::new`]) so `text` stays
/// fixed rather than being corrected per tier.
///
/// The shift direction is keyed off `seed` itself (`reference == color` in
/// [`shift_away_from`]), matching [`iced::theme::palette::deviate`] exactly
/// — **not** off `text`, unlike [`derive_tiers`]. R-3 (round-2 review):
/// keying off `text` picked the wrong direction whenever a preset's
/// background is dark-but-not-extreme with light text (`dark`: background
/// `rgb(0.0588, 0.0706, 0.0863)`, `is_dark(text) == false` → every tier
/// *darkened from an already near-black background*, and the clamp
/// fallback never fired because `darken` still succeeded — it just kept
/// succeeding in the wrong direction). Two independent signals confirmed
/// this was wrong, not merely different: `dark`'s own `border` token
/// (`rgb(0.169, 0.192, 0.227)`) is *lighter* than `background`, so the
/// token set's own intent is "borders lighten in dark mode," the opposite
/// of what the text-relative rule produced; and `high_contrast_dark`
/// (pure-black background) only ever looked correct because its extremity
/// forced the clamp fallback to fire — the right result via an accidental
/// route, not a general one. Background tiers pair with `text_primary` for
/// legibility, but their *primary* visual job (per
/// `iced_widget-0.14.2/src/button.rs:700,721` and
/// `checkbox.rs:574,581`, which read `background.strong` as a border
/// color) is contrast against the *adjacent surface*, which is what
/// keying off the color's own darkness — not the paired text's — actually
/// serves.
///
/// `strong` — the one tier iced actually reads as a border/separator
/// color — is computed separately, by [`background_strong_color`], rather
/// than from [`BACKGROUND_FIXED_SHIFT_AMOUNTS`] like the other six tiers:
/// a *fixed* shift amount cannot guarantee a real border is visible
/// against every possible seed, because the same OKLCH-lightness delta
/// produces wildly different WCAG contrast depending on how close the
/// seed already is to a luminance extreme (`high_contrast_dark`'s pure
/// black needs roughly double the shift `dark`'s near-black background
/// does, to clear the same contrast floor). [`background_strong_color`]
/// grows the amount until the floor is met instead of assuming one fixed
/// amount works everywhere.
fn derive_background(seed: iced::Color, text: iced::Color) -> Background {
    let pair_at = |amount: f32| Pair {
        color: shift_away_from(seed, seed, amount),
        text,
    };
    let [weakest, weaker, weak, neutral, stronger, strongest] =
        BACKGROUND_FIXED_SHIFT_AMOUNTS.map(pair_at);
    let strong = Pair {
        color: background_strong_color(seed),
        text,
    };
    Background {
        base: Pair { color: seed, text },
        weakest,
        weaker,
        weak,
        neutral,
        strong,
        stronger,
        strongest,
    }
}

/// Computes `Background.strong`'s fill color: [`shift_away_from`] the
/// seed away from its own tone (see [`derive_background`]'s doc comment
/// for why `strong` keys off the seed rather than text), growing the
/// shift amount past [`BACKGROUND_STRONG_START_AMOUNT`] in fixed
/// [`BORDER_SEARCH_STEP`] increments until the result clears
/// [`BORDER_CONTRAST_FLOOR`] against `seed`, or the amount reaches `1.0`
/// (iced's own valid lightness-delta ceiling) — whichever comes first.
///
/// Deterministic and preset-agnostic: the same function runs for every
/// preset, with no per-preset constant. For three of the four built-in
/// presets (`light`, `dark`, `high_contrast_light`), the starting amount
/// already clears the floor and this returns on the first iteration —
/// identical output to the fixed-amount approach it replaces. Only a
/// preset whose background sits at or very near a luminance extreme
/// (`high_contrast_dark`'s pure black, in practice) needs the amount
/// grown further.
fn background_strong_color(seed: iced::Color) -> iced::Color {
    let mut amount = BACKGROUND_STRONG_START_AMOUNT;
    loop {
        let candidate = shift_away_from(seed, seed, amount);
        let contrast = contrast_ratio(to_sn_color(candidate), to_sn_color(seed));
        if contrast >= BORDER_CONTRAST_FLOOR || amount >= 1.0 {
            return candidate;
        }
        amount += BORDER_SEARCH_STEP;
    }
}

/// Converts an iced color to `snora-design`'s iced-free [`snora_design::Color`]
/// so [`contrast_ratio`] — which only operates on that type — can check
/// [`background_strong_color`]'s result against its own seed.
///
/// Local and intentionally not exported: `snora-widgets`' one committed
/// style-bridge converter, `to_iced_color`, only goes the other direction
/// (a round-1 decision — no reverse converter existed in production). This
/// is purely an internal derivation detail, not a second public bridge.
fn to_sn_color(color: iced::Color) -> snora_design::Color {
    snora_design::Color::rgba(color.r, color.g, color.b, color.a)
}

/// Derives a complete `iced::Theme` from a Snora Design token bundle.
///
/// The returned theme's `extended_palette()` carries every one of
/// `snora-design`'s verified color roles as its `base` tiers, with `weak`
/// and `strong` tiers computed deterministically — see the module
/// documentation for the full mapping and the base/derived distinction.
/// `Theme::palette()` (the six-slot base) is a lossy view; widgets read
/// `extended_palette()`, which this function constructs in full.
///
/// Snora does not call this function on the application's behalf. The
/// application stores the returned value and passes it to iced's
/// `.theme()` hook:
///
/// `ignore` (RFC-064): calls `.run()`, a real event loop with no
/// headless mode, and references an undefined `App` type standing in for
/// the reader's own application — neither is satisfiable without the
/// padding this policy explicitly rejects.
///
/// ```rust,ignore
/// use snora::design::{Tokens, theme};
///
/// let tokens = Tokens::high_contrast_dark();
/// let iced_theme = theme(&tokens);
///
/// iced::application(App::default, App::update, App::view)
///     .theme(move |_state| iced_theme.clone())
///     .run()
/// ```
#[must_use]
pub fn theme(tokens: &Tokens) -> Theme {
    let p = &tokens.palette;

    let base = IcedPalette {
        background: to_iced_color(p.background),
        text: to_iced_color(p.text_primary),
        primary: to_iced_color(p.accent),
        success: to_iced_color(p.success),
        warning: to_iced_color(p.warning),
        danger: to_iced_color(p.danger),
    };

    let name = preset_name(tokens);
    let dark = is_dark(base.background);

    Theme::custom_with_fn(name, base, move |_generated_base| {
        let text = to_iced_color(p.text_primary);
        let bg_ref = to_iced_color(p.background);

        let background = derive_background(bg_ref, text);

        let (primary_base, primary_weak, primary_strong) =
            derive_tiers(to_iced_color(p.accent), to_iced_color(p.accent_text));
        let primary = Primary {
            base: primary_base,
            weak: primary_weak,
            strong: primary_strong,
        };

        // Secondary has no corresponding token role. Derived from the
        // neutral surface/text_primary family — matching iced's own
        // Secondary::generate(background, text) shape, which treats
        // secondary as neutral chrome, not a semantic accent.
        let (secondary_base, secondary_weak, secondary_strong) =
            derive_tiers(to_iced_color(p.surface), text);
        let secondary = Secondary {
            base: secondary_base,
            weak: secondary_weak,
            strong: secondary_strong,
        };

        let (success_base, success_weak, success_strong) =
            derive_tiers(to_iced_color(p.success), to_iced_color(p.success_text));
        let success = Success {
            base: success_base,
            weak: success_weak,
            strong: success_strong,
        };

        let (warning_base, warning_weak, warning_strong) =
            derive_tiers(to_iced_color(p.warning), to_iced_color(p.warning_text));
        let warning = Warning {
            base: warning_base,
            weak: warning_weak,
            strong: warning_strong,
        };

        let (danger_base, danger_weak, danger_strong) =
            derive_tiers(to_iced_color(p.danger), to_iced_color(p.danger_text));
        let danger = Danger {
            base: danger_base,
            weak: danger_weak,
            strong: danger_strong,
        };

        Extended {
            background,
            primary,
            secondary,
            success,
            warning,
            danger,
            is_dark: dark,
        }
    })
}

/// Names the theme so `Theme::to_string()` is meaningful. Not a public API;
/// purely cosmetic.
///
/// `Tokens` carries no preset-identity field, so this can only reflect
/// what is derivable from the data: light vs. dark. It cannot distinguish
/// a high-contrast preset from its standard counterpart without guessing —
/// deliberately not attempted, rather than inventing a second fragile
/// heuristic alongside `is_dark`'s.
fn preset_name(tokens: &Tokens) -> String {
    if is_dark(to_iced_color(tokens.palette.background)) {
        "Snora Design (dark)".to_string()
    } else {
        "Snora Design (light)".to_string()
    }
}

#[cfg(test)]
mod tests;
