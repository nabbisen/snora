//! Semantic button style functions for Snora Design tokens.
//!
//! Four variants are provided:
//!
//! * `primary` — filled accent button; the strongest call to action.
//! * `secondary` — outlined accent button; secondary action.
//! * `ghost` — no fill or border at rest; low-emphasis tertiary action.
//! * `danger` — filled danger/destructive button; uses `danger` / `danger_text`.
//!
//! # iced 0.14 focus limitation
//!
//! `iced::widget::button::Status` has `Active | Hovered | Pressed | Disabled`.
//! There is no `Focused` variant in iced 0.14. These functions map every
//! status iced exposes; they do **not** attempt to render a focus ring.
//! That is a documented limitation, not a bug.
//!
//! # Hovered and pressed (RFC-129)
//!
//! A hovered or pressed label is held to the palette's text level
//! ([`snora_design::Palette::text_contrast`]: 4.5:1 or 7:1) on every neutral surface, and the state
//! stays visibly different from rest. Each variant makes the shift it has always made
//! (`primary` and `danger` lighten when hovered and darken when pressed; `secondary` shifts its
//! label and border; `ghost` darkens its label when pressed), **keeps it where it holds that**, and
//! takes another from a fixed list where it does not (a hovered `primary` on `light_aaa`, for
//! example, darkens instead). The rule is [`crate::state::shifted`], shared with the chips.
//! Disabled is exempt.
//!
//! # Usage
//!
//! ```rust,no_run
//! use snora_design::Tokens;
//! use snora_style::button;
//!
//! let tokens = Tokens::light();
//! let btn = iced::widget::button("Save")
//!     .style(move |_theme, status| button::primary(&tokens, status));
//! # let _: iced::widget::Button<'_, (), iced::Theme, iced::Renderer> = btn;
//! ```

use iced::{Border, Color, Shadow, widget::button};
use snora_design::Tokens;

use super::color::to_iced_color;
use super::state::{Knob, Pair, shifted};

// ---- internal helpers ----------------------------------------------------

/// A filled variant's hovered or pressed fill (RFC-129): `fill` shifted by the first candidate
/// that keeps `label` at the palette's text level on every neutral surface, `today` first.
fn fill_in_state(tokens: &Tokens, label: Color, fill: Color, today: f32) -> Color {
    let rest = Pair { label, fill };
    shifted(&tokens.palette, Knob::Fill, today, rest, rest).fill
}

/// A tinted variant's hovered or pressed look (RFC-129): `accent` at `alpha` behind the label,
/// and the label (which `secondary` also draws its border in) shifted the same way.
fn tint_in_state(tokens: &Tokens, accent: Color, alpha: f32, today: f32) -> Pair {
    let rest = Pair {
        label: accent,
        fill: Color::TRANSPARENT,
    };
    let state = Pair {
        label: accent,
        fill: Color { a: alpha, ..accent },
    };
    shifted(&tokens.palette, Knob::Label, today, rest, state)
}

/// Applies a standard disabled alpha reduction.
fn disabled_alpha(color: Color) -> Color {
    Color {
        a: color.a * 0.45,
        ..color
    }
}

fn border_radius(tokens: &Tokens) -> iced::border::Radius {
    tokens.radius.md.into()
}

// ---- public style functions -----------------------------------------------

/// Filled accent button (solid `accent` background, `accent_text` foreground).
///
/// The strongest call to action on a surface; use sparingly.
#[must_use]
pub fn primary(tokens: &Tokens, status: button::Status) -> button::Style {
    let accent = to_iced_color(tokens.palette.accent);
    let text = to_iced_color(tokens.palette.accent_text);
    let radius = border_radius(tokens);

    let (bg, text_color) = match status {
        button::Status::Active => (accent, text),
        button::Status::Hovered => (fill_in_state(tokens, text, accent, 0.06), text),
        button::Status::Pressed => (fill_in_state(tokens, text, accent, -0.06), text),
        button::Status::Disabled => (disabled_alpha(accent), disabled_alpha(text)),
    };

    button::Style {
        background: Some(bg.into()),
        text_color,
        border: Border::default().rounded(radius),
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Outlined accent button (transparent background, `accent`-colored border and text).
///
/// Use for secondary actions on a surface where a `primary` button is present.
#[must_use]
pub fn secondary(tokens: &Tokens, status: button::Status) -> button::Style {
    let accent = to_iced_color(tokens.palette.accent);
    let radius = border_radius(tokens);

    let (bg, text_color, border_color) = match status {
        button::Status::Active => (Color::TRANSPARENT, accent, accent),
        button::Status::Hovered => {
            let p = tint_in_state(tokens, accent, 0.08, 0.06);
            (p.fill, p.label, p.label)
        }
        button::Status::Pressed => {
            let p = tint_in_state(tokens, accent, 0.14, -0.06);
            (p.fill, p.label, p.label)
        }
        button::Status::Disabled => (
            Color::TRANSPARENT,
            disabled_alpha(accent),
            disabled_alpha(accent),
        ),
    };

    button::Style {
        background: Some(bg.into()),
        text_color,
        border: Border::default()
            .rounded(radius)
            .color(border_color)
            .width(1.0),
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Ghost button (no fill or border at rest; subtle tint on hover/press).
///
/// Use for tertiary or low-emphasis actions.
#[must_use]
pub fn ghost(tokens: &Tokens, status: button::Status) -> button::Style {
    let accent = to_iced_color(tokens.palette.accent);
    let radius = border_radius(tokens);

    let (bg, text_color) = match status {
        button::Status::Active => (Color::TRANSPARENT, accent),
        button::Status::Hovered => {
            let p = tint_in_state(tokens, accent, 0.08, 0.0);
            (p.fill, p.label)
        }
        button::Status::Pressed => {
            let p = tint_in_state(tokens, accent, 0.14, -0.06);
            (p.fill, p.label)
        }
        button::Status::Disabled => (Color::TRANSPARENT, disabled_alpha(accent)),
    };

    button::Style {
        background: Some(bg.into()),
        text_color,
        border: Border::default().rounded(radius),
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Filled danger/destructive button (`danger` background, `danger_text` foreground).
///
/// The `danger_text on danger` contrast pair is mandatory in v0.20 and is
/// verified by the `snora-design` automated contrast tests.
///
/// Reserve for irreversible actions (delete, revoke, reset).
#[must_use]
pub fn danger(tokens: &Tokens, status: button::Status) -> button::Style {
    let bg_color = to_iced_color(tokens.palette.danger);
    let text = to_iced_color(tokens.palette.danger_text);
    let radius = border_radius(tokens);

    let (bg, text_color) = match status {
        button::Status::Active => (bg_color, text),
        button::Status::Hovered => (fill_in_state(tokens, text, bg_color, 0.06), text),
        button::Status::Pressed => (fill_in_state(tokens, text, bg_color, -0.06), text),
        button::Status::Disabled => (disabled_alpha(bg_color), disabled_alpha(text)),
    };

    button::Style {
        background: Some(bg.into()),
        text_color,
        border: Border::default().rounded(radius),
        shadow: Shadow::default(),
        snap: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snora_design::Tokens;

    fn all_statuses() -> [button::Status; 4] {
        [
            button::Status::Active,
            button::Status::Hovered,
            button::Status::Pressed,
            button::Status::Disabled,
        ]
    }

    #[test]
    fn primary_returns_valid_style_for_all_statuses() {
        let t = Tokens::light();
        for s in all_statuses() {
            let style = primary(&t, s);
            assert!(
                style.background.is_some(),
                "primary {s:?}: background must be set"
            );
        }
    }

    #[test]
    fn secondary_active_has_transparent_background() {
        let t = Tokens::dark();
        let style = secondary(&t, button::Status::Active);
        // Secondary at rest is outline-only; background fully transparent.
        assert!(
            matches!(style.background, Some(iced::Background::Color(c)) if c.a < 0.01),
            "secondary active should be transparent, got {:?}",
            style.background
        );
    }

    #[test]
    fn ghost_active_has_transparent_background() {
        let t = Tokens::high_contrast_light();
        let style = ghost(&t, button::Status::Active);
        assert!(
            matches!(style.background, Some(iced::Background::Color(c)) if c.a < 0.01),
            "ghost active should be transparent"
        );
    }

    #[test]
    fn danger_all_statuses_compile_and_produce_background() {
        let t = Tokens::high_contrast_dark();
        for s in all_statuses() {
            let style = danger(&t, s);
            assert!(
                style.background.is_some(),
                "danger {s:?}: background must be set"
            );
        }
    }

    #[test]
    fn disabled_reduces_alpha() {
        let t = Tokens::light();
        let active = primary(&t, button::Status::Active);
        let disabled = primary(&t, button::Status::Disabled);
        // Disabled text should be more transparent than active text.
        assert!(
            disabled.text_color.a < active.text_color.a,
            "disabled alpha {:.2} should be < active alpha {:.2}",
            disabled.text_color.a,
            active.text_color.a
        );
    }

    #[test]
    fn all_variants_use_all_presets() {
        // RFC-118: every preset, from the one public source.
        for (_name, tokens) in Tokens::presets() {
            for status in all_statuses() {
                let _ = primary(&tokens, status);
                let _ = secondary(&tokens, status);
                let _ = ghost(&tokens, status);
                let _ = danger(&tokens, status);
            }
        }
    }
}
