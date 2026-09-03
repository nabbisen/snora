//! RFC-093's channel register for `snora-widgets`' prefab surfaces.
//!
//! **What this proves and what it does not.** WCAG 1.4.1 (Use of
//! Colour) is a property of everything rendered on a surface, not of
//! any one colour pair — no test that only inspects style structs can
//! observe it. What *is* mechanically checkable is narrower and is all
//! this module claims: for each surface below, [`Tone`] changes the
//! style output **only** through colour-bearing fields, never through
//! layout, border width, radius, or any other shape. That is the
//! register RFC-093 asks for — a claim about what `snora-widgets`
//! contributes, pinned against the code, not a conformance check.
//!
//! # Surfaces covered
//!
//! | Surface | Function under test |
//! |---|---|
//! | `notice`'s left accent bar | [`notice_accent_bar_style`] |
//! | `notice`'s outer container | [`notice_outer_style`] |
//! | `progress`'s bar | [`snora_style::progress::toned`] |
//!
//! `snora::toast` carries the equivalent register for the engine crate,
//! in `crates/snora/src/toast/channel_register.rs` — kept separate
//! because the engine has no dependency on this crate (RFC-021/022).
//!
//! # Exhaustiveness
//!
//! [`ALL_TONES`] is a hand-listed array — Rust has no reflection over
//! enum variants. What *is* exhaustive: [`tone_label`] matches on
//! [`Tone`] with no wildcard arm, so a seventh variant fails to compile
//! here until it is given a label (RFC-063's pattern, `Palette::usages`).
//!
//! # Canonicalization is unconditional, on purpose
//!
//! **Reviewer-required (R-1, 2026-09-02).** The first version of this
//! module canonicalized only whichever colour-bearing field happened to
//! vary *today* per function — `canonical_outer_style` reset
//! `border.color` because that's the field `notice_outer_style` varies,
//! but left `background`/`text_color`/`shadow.color` untouched, and the
//! other four canonicalizers each normalized a different, disjoint
//! subset. The reviewer perturbed `toast_style` to vary **border
//! colour** by intent (width, radius, everything else held constant —
//! a colour-only change) and the register reported it as a non-colour
//! channel: the un-normalized `border.color` field differed and nothing
//! else did, but `border.color` wasn't in that function's canonicalized
//! set. `PLACEHOLDER`'s doc comment claimed universal normalization
//! while the code normalized four different partial sets — the exact
//! defect class this RFC exists to close, reproduced inside its own
//! fix.
//!
//! [`canonicalize_container`] and [`canonicalize_progress`] below
//! normalize **every** colour-bearing field on their struct,
//! unconditionally, whether or not it varies today — `background`,
//! `text_color`, `border.color`, `shadow.color` for `container::Style`;
//! `background`, `bar`, `border.color` for `progress_bar::Style`. Every
//! canonicalizer in this module goes through one of these two, so they
//! cannot drift apart again.

use iced::widget::{container, progress_bar};
use iced::{Background, Color};
use snora_design::{Tokens, Tone};

use super::notice::{notice_accent_bar_style, notice_accent_color, notice_outer_style};

/// All six [`Tone`] variants. Hand-listed — see module doc.
const ALL_TONES: [Tone; 6] = [
    Tone::Neutral,
    Tone::Accent,
    Tone::Success,
    Tone::Warning,
    Tone::Danger,
    Tone::Info,
];

/// Exhaustive on `tone` — see module doc.
fn tone_label(tone: Tone) -> &'static str {
    match tone {
        Tone::Neutral => "Neutral",
        Tone::Accent => "Accent",
        Tone::Success => "Success",
        Tone::Warning => "Warning",
        Tone::Danger => "Danger",
        Tone::Info => "Info",
    }
}

/// A placeholder colour distinct from any real palette colour, swapped
/// in for every colour-bearing field before comparing styles. What
/// matters is that every tone's style canonicalizes to the *same*
/// struct once colour is normalized away — not what the placeholder is.
const PLACEHOLDER: Color = Color::WHITE;

/// A `Background::Color` becomes the placeholder; any other variant
/// (e.g. a future `Background::Gradient`) passes through unchanged —
/// that shape difference is exactly the kind of "more than colour"
/// finding this register exists to catch, not something to normalize
/// away.
fn canonicalize_background(bg: Background) -> Background {
    match bg {
        Background::Color(_) => Background::Color(PLACEHOLDER),
        other => other,
    }
}

/// Normalizes every colour-bearing field of a [`container::Style`]:
/// `background`, `text_color`, `border.color`, `shadow.color`. See the
/// module's "Canonicalization is unconditional" section for why partial
/// normalization is exactly the defect this exists to close.
fn canonicalize_container(style: container::Style) -> container::Style {
    container::Style {
        background: style.background.map(canonicalize_background),
        text_color: style.text_color.map(|_| PLACEHOLDER),
        border: iced::Border {
            color: PLACEHOLDER,
            ..style.border
        },
        shadow: iced::Shadow {
            color: PLACEHOLDER,
            ..style.shadow
        },
        ..style
    }
}

/// Normalizes every colour-bearing field of a [`progress_bar::Style`]:
/// `background`, `bar`, `border.color`.
fn canonicalize_progress(style: progress_bar::Style) -> progress_bar::Style {
    progress_bar::Style {
        background: canonicalize_background(style.background),
        bar: canonicalize_background(style.bar),
        border: iced::Border {
            color: PLACEHOLDER,
            ..style.border
        },
    }
}

fn canonical_bar_style(tokens: &Tokens, tone: Tone) -> container::Style {
    canonicalize_container(notice_accent_bar_style(notice_accent_color(tokens, tone)))
}

fn canonical_outer_style(tokens: &Tokens, tone: Tone) -> container::Style {
    let surface = snora_style::color::to_iced_color(tokens.palette.surface);
    let radius = tokens.radius.md;
    canonicalize_container(notice_outer_style(
        surface,
        notice_accent_color(tokens, tone),
        radius,
    ))
}

fn canonical_progress_style(tokens: &Tokens, tone: Tone) -> progress_bar::Style {
    canonicalize_progress(snora_style::progress::toned(tokens, tone))
}

/// Asserts that `canonicalize(surface_tokens, tone)` produces the same
/// struct for every [`Tone`], collecting every mismatch before failing
/// so a regression states every affected variant at once — mirrors
/// `toast/contrast_tests.rs`'s failure-collection style.
fn assert_colour_only<T: PartialEq + std::fmt::Debug>(
    surface: &str,
    canonicalize: impl Fn(&Tokens, Tone) -> T,
) {
    let tokens = Tokens::light();
    let reference = canonicalize(&tokens, ALL_TONES[0]);
    let mut failures = Vec::new();
    for tone in ALL_TONES {
        let actual = canonicalize(&tokens, tone);
        if actual != reference {
            failures.push(format!(
                "{surface} / {}: varies by more than colour\n  {actual:?}\n  != reference {reference:?}",
                tone_label(tone),
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} channel-register failure(s):\n{}",
        failures.len(),
        failures.join("\n"),
    );
}

#[test]
fn notice_accent_bar_varies_by_colour_only() {
    assert_colour_only("notice accent bar", canonical_bar_style);
}

#[test]
fn notice_outer_container_varies_by_colour_only() {
    assert_colour_only("notice outer container", canonical_outer_style);
}

#[test]
fn progress_bar_varies_by_colour_only() {
    assert_colour_only("progress bar", canonical_progress_style);
}
