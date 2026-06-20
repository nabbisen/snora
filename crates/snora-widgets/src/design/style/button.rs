//! Semantic button style functions for Snora Design tokens.
//!
//! Four variants are provided:
//!
//! * [`primary`] — filled accent button; the strongest call to action.
//! * [`secondary`] — outlined accent button; secondary action.
//! * [`ghost`] — no fill or border at rest; low-emphasis tertiary action.
//! * [`danger`] — filled danger/destructive button; uses `danger` / `danger_text`.
//!
//! # iced 0.14 focus limitation
//!
//! `iced::widget::button::Status` has `Active | Hovered | Pressed | Disabled`.
//! There is no `Focused` variant in iced 0.14. These functions map every
//! status iced exposes; they do **not** attempt to render a focus ring.
//! That is a documented limitation, not a bug.
//!
//! # Usage
//!
//! ```rust,ignore
//! use snora_design::Tokens;
//! use snora_widgets::design::style::button;
//!
//! let tokens = Tokens::light();
//! let btn = iced::widget::button("Save")
//!     .style(move |_theme, status| button::primary(&tokens, status));
//! ```

use iced::{
    Border, Color, Shadow,
    widget::button,
};
use snora_design::Tokens;

use super::color::to_iced_color;

// ---- internal helpers ----------------------------------------------------

/// Blends `color` toward white by `amount` (0.0 = unchanged, 1.0 = white).
fn lighten(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r + amount).min(1.0),
        g: (color.g + amount).min(1.0),
        b: (color.b + amount).min(1.0),
        a: color.a,
    }
}

/// Blends `color` toward black by `amount` (0.0 = unchanged, 1.0 = black).
fn darken(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r - amount).max(0.0),
        g: (color.g - amount).max(0.0),
        b: (color.b - amount).max(0.0),
        a: color.a,
    }
}

/// Applies a standard disabled alpha reduction.
fn disabled_alpha(color: Color) -> Color {
    Color { a: color.a * 0.45, ..color }
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
        button::Status::Hovered => (lighten(accent, 0.06), text),
        button::Status::Pressed => (darken(accent, 0.06), text),
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
        button::Status::Hovered => (
            Color { a: 0.08, ..accent },
            lighten(accent, 0.06),
            lighten(accent, 0.06),
        ),
        button::Status::Pressed => (
            Color { a: 0.14, ..accent },
            darken(accent, 0.06),
            darken(accent, 0.06),
        ),
        button::Status::Disabled => (
            Color::TRANSPARENT,
            disabled_alpha(accent),
            disabled_alpha(accent),
        ),
    };

    button::Style {
        background: Some(bg.into()),
        text_color,
        border: Border::default().rounded(radius).color(border_color).width(1.0),
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
        button::Status::Hovered => (Color { a: 0.08, ..accent }, accent),
        button::Status::Pressed => (Color { a: 0.14, ..accent }, darken(accent, 0.06)),
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
        button::Status::Hovered => (lighten(bg_color, 0.06), text),
        button::Status::Pressed => (darken(bg_color, 0.06), text),
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
            assert!(style.background.is_some(), "primary {s:?}: background must be set");
        }
    }

    #[test]
    fn secondary_active_has_transparent_background() {
        let t = Tokens::dark();
        let style = secondary(&t, button::Status::Active);
        // Secondary at rest is outline-only; background fully transparent.
        assert!(
            matches!(style.background, Some(iced::Background::Color(c)) if c.a < 0.01),
            "secondary active should be transparent, got {:?}", style.background
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
            assert!(style.background.is_some(), "danger {s:?}: background must be set");
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
            disabled.text_color.a, active.text_color.a
        );
    }

    #[test]
    fn all_variants_use_all_presets() {
        for tokens in [
            Tokens::light(),
            Tokens::dark(),
            Tokens::high_contrast_light(),
            Tokens::high_contrast_dark(),
        ] {
            for status in all_statuses() {
                let _ = primary(&tokens, status);
                let _ = secondary(&tokens, status);
                let _ = ghost(&tokens, status);
                let _ = danger(&tokens, status);
            }
        }
    }
}
