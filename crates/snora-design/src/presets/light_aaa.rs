//! Light AAA preset (RFC-118).
//!
//! [`crate::Tokens::light`]'s character with every **declared text
//! pair** at WCAG AAA (7:1, criterion 1.4.6). Light itself is unchanged
//! and remains an AA preset, which is its stated floor.
//!
//! # How these values were derived
//!
//! Not by eye. Each changed role moved **only in lightness**, in OKLCh,
//! with chroma and hue held, by the smallest step that brings its worst
//! declared pair to 7.05:1 — a bisection to 1e-4. The floor is 7.05
//! rather than 7.0 so that a value rounded to six decimals in this file
//! cannot land under 7.0.
//!
//! - **The background and the surfaces are Light's own**, untouched.
//!   That is what makes this Light at AAA rather than a new theme.
//! - **`text_primary` and `text_secondary` did not move**: they already
//!   clear 7:1 (16.44 and 8.12 at their worst). Only `text_muted` did,
//!   and the hierarchy keeps its order — 16.44 ≥ 8.12 ≥ 7.05 — though
//!   compressed, which is the cost the RFC accepts.
//! - **A fill moved only when its own text pair required it**, never
//!   because the fill-on-surface pair was low: that pair is declared
//!   non-text at 3:1, and holding it to 7:1 would be the category error
//!   RFC-117's R-5 corrected for the accent.
//! - **`info` is set to the solved `accent`.** The two are the same
//!   colour in Light, as are `info_text` and `accent_text`, and solving
//!   them separately broke that visible identity for no reason.
//! - **`focus` equals the accent, as Light's does** (ruled 2026-11-07).
//!   In Light the focus ring and the accent are one colour, and that is
//!   part of what makes the preset recognisably Light. The first
//!   version of this preset left `focus` at Light's own value while the
//!   accent darkened, which dropped the identity; focus has its own 3:1
//!   floor and the darker accent clears it, so it follows the accent.
//!   Dark's focus was never its accent, so `dark_aaa` is unchanged here.
//!
//! The largest hue shift across both AAA presets is 5.442°, asserted in
//! `tests.rs` against a bound of 6°.

use crate::{Color, Palette, Tokens};

pub(crate) fn palette() -> Palette {
    let parent = super::light::palette();
    // The AAA accent, which `info` follows.
    let accent = Color::rgb(0.083622, 0.266716, 0.807263);

    Palette {
        // Light's own, untouched.
        background: parent.background,
        surface: parent.surface,
        surface_raised: parent.surface_raised,
        border: parent.border,
        // Light's focus is its accent, so this follows the AAA accent.
        focus: accent,
        // Already above 7:1 against every surface.
        text_primary: parent.text_primary,
        text_secondary: parent.text_secondary,
        // 4.55 at its worst in Light; darkened to 7.05.
        text_muted: Color::rgb(0.302764, 0.328408, 0.379857),
        // 6.19 at its worst as text, and 6.70 under `accent_text`.
        accent,
        accent_text: parent.accent_text,
        info: accent,
        info_text: parent.info_text,
        // Each moved by what its own white text needs: 5.02, 5.43, 6.54.
        success: Color::rgb(0.000000, 0.405061, 0.146109),
        success_text: parent.success_text,
        warning: Color::rgb(0.527727, 0.286668, 0.000000),
        warning_text: parent.warning_text,
        danger: Color::rgb(0.678062, 0.119634, 0.093896),
        danger_text: parent.danger_text,
    }
}

pub(crate) fn tokens() -> Tokens {
    super::assemble(palette(), 2.0)
}
