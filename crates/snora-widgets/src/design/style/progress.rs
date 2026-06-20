//! Progress bar style functions for Snora Design tokens.

use iced::{Border, widget::progress_bar};
use snora_design::{Tokens, Tone};

use super::color::to_iced_color;

/// Returns a [`progress_bar::Style`] colored for the given [`Tone`].
///
/// Uses the tone's background role as the filled bar color and `surface` as
/// the track background. This keeps the progress bar visually consistent
/// with other toned surfaces (notices, chips) using the same token palette.
#[must_use]
pub fn toned(tokens: &Tokens, tone: Tone) -> progress_bar::Style {
    let p = &tokens.palette;
    let bar = to_iced_color(match tone {
        Tone::Accent   => p.accent,
        Tone::Success  => p.success,
        Tone::Warning  => p.warning,
        Tone::Danger   => p.danger,
        Tone::Info     => p.info,
        Tone::Neutral  => p.border,
    });
    progress_bar::Style {
        background: to_iced_color(p.surface).into(),
        bar: bar.into(),
        border: Border::default().rounded(tokens.radius.sm),
    }
}

#[cfg(test)]
mod tests;
