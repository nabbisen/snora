//! Composited/derived surfaces (RFC-065): render-time colors that are not
//! a [`crate::Palette`] role, but still need to be measured for contrast.
//!
//! `Palette::usages` (RFC-063) declares where a *role* renders — but a
//! surface does not have to be a role to need measuring. The modal dim is
//! composited at render time from `DIM_ALPHA` and `background`'s own
//! darkness; it exists in no `Palette` field, so `Palette::usages` cannot
//! see it and the contrast suite silently had nothing to check it
//! against. This module makes it a pure function over [`Tokens`] instead,
//! so it can be asserted beside every other mandatory pair in this
//! crate's own test suite, and so the `snora` crate's renderer has a
//! single place to call rather than a second place to reimplement it.

use crate::color::Color;
use crate::contrast::linearize_srgb_channel;
use crate::tokens::Tokens;

/// Alpha applied to the derived modal dim (RFC-065).
///
/// Not the unstyled path's `0.4` (see `ChromeStyle`'s default in the
/// `snora` crate's `render` module) — measured against `light`'s SC
/// 1.4.11 floor and chosen to clear it with real margin (8%, 3.24:1)
/// rather than hug it the way `0.42` (1.3%, 3.04:1) would, matching the
/// precedent RFC-058 set for `border`'s repair. The two paths were
/// symmetric at `0.40` by coincidence, not by design; they diverge here
/// on purpose.
pub const DIM_ALPHA: f32 = 0.44;

/// Derives the modal dim color for a token bundle: opaque black or white
/// — whichever pole is opposite `background`'s own darkness — at
/// [`DIM_ALPHA`].
///
/// Picking the pole *opposite* `background`'s own category, rather than
/// shifting a fixed color toward or away from it, has no clamping edge
/// case: the two poles can never both describe the same background,
/// unlike an OKLCH-lightness *shift* (`snora-style`'s
/// `shift_away_from`, used for derived theme tiers). This only ever
/// chooses between two fixed, maximally-distinct poles, so a background
/// at either luminance extreme (`light`'s pure white, `high_contrast_dark`'s
/// pure black) is exactly where the derivation is safest, not where it
/// needs a fallback.
#[must_use]
pub fn modal_dim(tokens: &Tokens) -> Color {
    let base = if is_dark(tokens.palette.background) {
        Color::rgb(1.0, 1.0, 1.0)
    } else {
        Color::rgb(0.0, 0.0, 0.0)
    };
    Color {
        a: DIM_ALPHA,
        ..base
    }
}

/// Classifies a color as dark or light by OKLCH lightness (threshold
/// `0.6`), matching `iced::theme::palette::is_dark`'s own algorithm —
/// same sRGB→linear step ([`linearize_srgb_channel`]), same OKLab
/// matrices — reimplemented here so `snora-design` can classify a
/// background without depending on iced to do it. A background
/// classifies identically whichever crate asks.
fn is_dark(color: Color) -> bool {
    oklch_lightness(color) < 0.6
}

/// `sRGB → linear RGB → LMS → Oklab L` (the lightness channel only; `a`/`b`
/// are not needed for [`is_dark`]'s threshold). Coefficients match
/// `iced_core::theme::palette`'s `to_oklch`, itself following
/// <https://en.wikipedia.org/wiki/Oklab_color_space#Conversions_between_color_spaces>.
fn oklch_lightness(color: Color) -> f32 {
    let r = linearize_srgb_channel(color.r);
    let g = linearize_srgb_channel(color.g);
    let b = linearize_srgb_channel(color.b);

    let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    0.210_454_26 * l_ + 0.793_617_8 * m_ - 0.004_072_047 * s_
}

#[cfg(test)]
mod tests;
