//! Progress bar style functions for Snora Design tokens.

use iced::{Border, widget::progress_bar};
use snora_design::{Tokens, Tone};

use super::color::to_iced_color;

/// Returns a [`progress_bar::Style`] colored for the given [`Tone`].
///
/// Uses the tone's background role as the filled bar color and `surface` as
/// the track background. This keeps the progress bar visually consistent
/// with other toned surfaces (notices, chips) using the same token palette.
///
/// **The track has an outline** (RFC-135): the palette's `border` role, 1 px. The track is
/// `surface`, which is next to the page's colour (1.00 to 1.11:1 measured, and exactly 1.00 on
/// both high-contrast presets, where `surface` *is* the page), so without an outline the
/// unfilled part of a bar was invisible. The outline holds 3:1 against every neutral surface, and
/// leaves the bar's own contrast against the track as it was (a darker track would have taken
/// it under 3:1 on both ordinary presets, for every tone: see RFC-135's Unit 1).
#[must_use]
pub fn toned(tokens: &Tokens, tone: Tone) -> progress_bar::Style {
    let p = &tokens.palette;
    let bar = to_iced_color(match tone {
        Tone::Accent => p.accent,
        Tone::Success => p.success,
        Tone::Warning => p.warning,
        Tone::Danger => p.danger,
        Tone::Info => p.info,
        Tone::Neutral => p.border,
    });
    progress_bar::Style {
        background: to_iced_color(p.surface).into(),
        bar: bar.into(),
        border: Border::default()
            .rounded(tokens.radius.sm)
            .color(to_iced_color(p.border))
            .width(1.0),
    }
}

#[cfg(test)]
mod tests;
