//! Token-derived engine surface styling for Snora Design (RFC-039).
//!
//! [`render`] is a sibling to [`crate::render::render`], not a
//! replacement: with `design` inactive, [`crate::render::render`]'s
//! output is byte-for-byte unchanged (RFC-037's gating invariant).
//! Applications opt in by calling the other function; nothing here runs
//! unless they do.
//!
//! # Why a sibling entry point, not a field on `AppLayout`
//!
//! `AppLayout` lives in `snora-core`, which has **no dependencies at
//! all** — adding a `Tokens` field would pull `snora-design` into every
//! engine-only build, defeating the opt-in size discipline (DEC-11) and
//! inverting the documented crate-dependency direction. `snora` already
//! depends on both `snora-core` and (behind `design`) `snora-design`, so
//! the sibling function lives here instead.
//!
//! # Two surfaces, both within the frozen token surface
//!
//! RFC-036's additive-only covenant freezes `snora-design`'s `Palette`
//! (18 roles) and `Tokens` (no shadow/elevation scale). Neither surface
//! this module styles has a purpose-built token — deriving from existing
//! roles, not extending the frozen surface, is the owner-confirmed
//! approach (RFC-039 §"The covenant bites here").
//!
//! ## The dialog card
//!
//! Fill `surface_raised`, edge `border`, radius `radius.lg`, padding
//! `spacing.lg` — reusing [`snora_widgets::design::style::container::card_raised`]
//! (RFC-029) directly rather than recomputing the same color/border
//! mapping, with its drop shadow zeroed out. **Border-defined, not
//! shadow-defined**, deliberately: shadows are close to meaningless in
//! the high-contrast presets (`high_contrast_light`'s shadow color and
//! its background are both near-white; `high_contrast_dark`'s near-black
//! against near-black), and a border already renders correctly there.
//!
//! ## The modal dim
//!
//! `iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4)` — opaque black at 40%
//! alpha — is the unstyled default. Composited over a **dark** page
//! background, black-on-black is close to a no-op: the exact class of
//! defect RFC-038's `shift_away_from` was built to prevent for derived
//! theme tiers, here for a fixed constant instead of a derived one.
//!
//! [`dim_color`] instead picks the dim's base color from
//! **`background`'s own darkness**, not a fixed pole:
//! [`iced::theme::palette::is_dark`] on the emitted `background` color —
//! `Color::WHITE` if dark, `Color::BLACK` if light — at the same 40%
//! alpha as before. This has no clamping edge case (unlike
//! `shift_away_from`'s OKLCH-lightness shift): alpha-compositing a color
//! chosen to be the *opposite* pole from the background's own category
//! can never degenerate to a no-op, because the two poles cannot both
//! describe the same background. Verified against all four presets in
//! `render/tests.rs`, including the two clamping cases that broke
//! RFC-038's first attempt — `light`'s pure-white background and
//! `high_contrast_dark`'s pure-black one — neither of which is a
//! clamping case *here* precisely because the derivation never tries to
//! move a color away from its own tone; it only ever chooses between two
//! fixed, maximally-distinct poles.

use iced::Element;
use iced::theme::palette::is_dark;
use snora_core::AppLayout;
use snora_design::Tokens;
use snora_widgets::design::style::color::to_iced_color;

use crate::overlay::dialog::DialogCardStyle;
use crate::render::{ChromeStyle, render_with_style};

/// Alpha applied to the derived dim color. Matches the unstyled path's
/// literal (`Color::from_rgba(0.0, 0.0, 0.0, 0.4)`) — only the base color
/// becomes token-derived, not the strength of the dim.
const DIM_ALPHA: f32 = 0.4;

/// Derives a complete, token-styled render from an [`AppLayout`]: the
/// dialog gets a real card (fill, border, radius, padding), and the
/// modal dim is derived from the token bundle instead of a fixed
/// constant. See the module documentation for the derivation and its
/// rationale.
///
/// Snora does not call this on the application's behalf; it is a sibling
/// to [`crate::render::render`], not a replacement. Applications opt in
/// explicitly:
///
/// ```rust,ignore
/// use snora::design::{Tokens, render};
///
/// let tokens = Tokens::light();
/// let element = render(layout, &tokens);
/// ```
#[must_use]
pub fn render<'a, Message>(
    layout: AppLayout<Element<'a, Message>, Message>,
    tokens: &Tokens,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    render_with_style(layout, &chrome_style(tokens))
}

fn chrome_style(tokens: &Tokens) -> ChromeStyle {
    ChromeStyle {
        dim_color: dim_color(tokens),
        dialog_card: Some(dialog_card_style(tokens)),
    }
}

/// See the module documentation's "The modal dim" section for the full
/// derivation rationale.
fn dim_color(tokens: &Tokens) -> iced::Color {
    let background = to_iced_color(tokens.palette.background);
    let base = if is_dark(background) {
        iced::Color::WHITE
    } else {
        iced::Color::BLACK
    };
    iced::Color {
        a: DIM_ALPHA,
        ..base
    }
}

/// See the module documentation's "The dialog card" section.
fn dialog_card_style(tokens: &Tokens) -> DialogCardStyle {
    let mut style = snora_widgets::design::style::container::card_raised(tokens);
    // Border-defined, not shadow-defined (RFC-039) — card_raised's shadow
    // is meant for popovers/floating panels, not this surface.
    style.shadow = iced::Shadow::default();

    DialogCardStyle {
        padding: tokens.spacing.lg,
        style,
    }
}

#[cfg(test)]
mod tests;
