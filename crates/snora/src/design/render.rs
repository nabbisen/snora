//! Token-derived engine surface styling for Snora Design (RFC-039).
//!
//! [`render()`] is a sibling to [`crate::render::render`], not a
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
//! `spacing.lg` — reusing [`snora_style::container::card_raised`]
//! (RFC-029, relocated from `snora-widgets` by RFC-055) directly rather
//! than recomputing the same color/border
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
//! `dim_color` instead picks the dim's base color from
//! **`background`'s own darkness**, not a fixed pole — `Color::WHITE` if
//! dark, `Color::BLACK` if light. This has no clamping edge case (unlike
//! `shift_away_from`'s OKLCH-lightness shift): alpha-compositing a color
//! chosen to be the *opposite* pole from the background's own category
//! can never degenerate to a no-op, because the two poles cannot both
//! describe the same background. Safe at both luminance extremes —
//! `light`'s pure-white background and `high_contrast_dark`'s pure-black
//! one, the two cases that broke RFC-038's first attempt — precisely
//! because the derivation never tries to move a color away from its own
//! tone; it only ever chooses between two fixed, maximally-distinct
//! poles.
//!
//! **The derivation itself lives in [`snora_design::surfaces::modal_dim`]
//! (RFC-065), not here.** `dim_color` is a thin adapter: it calls that
//! function and converts the result to [`iced::Color`]. This is not the
//! same alpha as the unstyled path's `0.4` any more —
//! [`snora_design::surfaces::DIM_ALPHA`] is `0.44`, repaired because the
//! `light` preset's dialog card was measured at 2.85:1 against its own
//! dimmed backdrop, below SC 1.4.11's 3:1 floor. **That measurement
//! describes the pre-repair state at alpha `0.40`**, where neither signal
//! cleared: fill|dim 2.85:1 and border|dim 1.00:1. It is not the claim
//! the 0.38→0.39 guide withdrew — that one said the border is what
//! identifies the card against the dim, and it is not; the card is
//! fill-defined against the dim and border-defined against its own
//! surface. Reported as a stale withdrawn claim twice (aaai, 2026-08 and
//! 2026-09) and re-measured both times; the sentence is accurate and the
//! wording, not the fact, is what was fixed here. The two paths were symmetric at `0.40` by coincidence, not
//! design; RFC-065 lets them diverge on purpose. See
//! `crates/snora-design/src/tests.rs` for the either-signal assertion
//! this repair answers, and `render/tests.rs` for the alpha and pole
//! tests on this crate's own adapter.

use iced::widget::Responsive;
use iced::{Element, Size};
use snora_core::AppLayout;
use snora_design::Tokens;
// RFC-055: relocated from snora_widgets::design::style::color — the
// engine surface reaches snora-style directly, without depending on
// snora-widgets.
use snora_style::color::to_iced_color;

use crate::overlay::dialog::DialogCardStyle;
use crate::render::{ChromeStyle, render_with_style};

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
/// ```rust,no_run
/// use iced::{Element, widget::text};
/// use snora::{AppLayout, design::{Tokens, render}};
///
/// #[derive(Debug, Clone)]
/// enum Message {}
///
/// let tokens = Tokens::light();
/// let body: Element<'_, Message> = text("Hello, snora!").into();
/// let layout = AppLayout::new(body);
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

/// Renders an [`AppLayout`] that may depend on the available width,
/// through the `design` path — the styled dialog card and the
/// token-derived modal dim survive, unlike [`crate::responsive::responsive_render`]
/// (RFC-053).
///
/// `build` receives the width available to the layout (in logical
/// pixels) and returns the `AppLayout` to render at that width. It may
/// be called again whenever the available size changes — see
/// [`iced::widget::Responsive`]'s own documentation for the underlying
/// mechanism. Mirrors [`crate::responsive::responsive_render`] exactly,
/// with [`render`] in place of [`crate::render::render`]; it is a
/// wrapper around the existing composition path, not a second one — see
/// this module's documentation for why that duplication is exactly what
/// RFC-039 built [`render`] to avoid.
///
/// `&'a Tokens`, matching [`render`]'s own `(layout, &tokens)` shape:
/// the borrow is natural in the usual `fn view(&self) -> Element<'_,
/// Message>`, where the returned element already borrows `&self`.
///
/// ```rust,no_run
/// use iced::{Element, widget::text};
/// use snora::{AppLayout, design::{Tokens, responsive_render}};
///
/// struct State;
/// #[derive(Debug, Clone)]
/// enum Message {}
///
/// fn body(_state: &State) -> Element<'_, Message> {
///     text("body").into()
/// }
/// fn sidebar(_state: &State) -> Element<'_, Message> {
///     text("sidebar").into()
/// }
///
/// fn view<'a>(state: &'a State, tokens: &'a Tokens) -> Element<'a, Message> {
///     responsive_render(
///         move |width| {
///             let layout = AppLayout::new(body(state));
///             if width < 600.0 {
///                 layout
///             } else {
///                 layout.side_bar(sidebar(state))
///             }
///         },
///         tokens,
///     )
/// }
/// ```
#[must_use]
pub fn responsive_render<'a, Message, F>(build: F, tokens: &'a Tokens) -> Element<'a, Message>
where
    F: Fn(f32) -> AppLayout<Element<'a, Message>, Message> + 'a,
    Message: Clone + 'a,
{
    Responsive::new(move |size: Size| render(build(size.width), tokens)).into()
}

fn chrome_style(tokens: &Tokens) -> ChromeStyle {
    ChromeStyle {
        dim_color: dim_color(tokens),
        dialog_card: Some(dialog_card_style(tokens)),
    }
}

/// See the module documentation's "The modal dim" section for the full
/// derivation rationale. A thin adapter over
/// [`snora_design::surfaces::modal_dim`] (RFC-065) — the derivation
/// itself, and [`snora_design::surfaces::DIM_ALPHA`], live there as the
/// single source; this function only converts the result to
/// [`iced::Color`].
fn dim_color(tokens: &Tokens) -> iced::Color {
    to_iced_color(snora_design::surfaces::modal_dim(tokens))
}

/// See the module documentation's "The dialog card" section.
fn dialog_card_style(tokens: &Tokens) -> DialogCardStyle {
    let mut style = snora_style::container::card_raised(tokens);
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
