//! Chip style functions (RFC-032, RFC-129).
//!
//! * [`selected`]: a solid `accent` fill with `accent_text` on it;
//! * [`unselected`]: the `surface` fill, a `border` outline and a `text_secondary` label.
//!
//! These were `snora-widgets`' private `chip_style_selected` and `chip_style_unselected`, and
//! moved here (RFC-129 Q-5) with the rest of the style functions (RFC-055): they are pure
//! functions of [`Tokens`] and a [`button::Status`], and the widget layer consumes them.
//!
//! # Hovered and pressed (RFC-129)
//!
//! A chip's hovered and pressed fills used to be a fixed darkening (selected: 0.06 and 0.12;
//! unselected: 0.04 and 0.08 of `surface`), which took the label under its preset's level on
//! `light_aaa` and `dark_aaa` and drew **nothing at all** for an unselected chip on
//! `high_contrast_dark` (its `surface` is black). They are now held to the palette's
//! [`text_contrast`](snora_design::Palette::text_contrast) on every neutral surface and stay
//! visibly different from rest, by the rule the buttons use: the fixed darkening where it holds,
//! another shift where it does not. Disabled is exempt.
//!
//! ```rust,no_run
//! use snora_design::Tokens;
//! use snora_style::chip;
//!
//! let tokens = Tokens::light();
//! let chip_button = iced::widget::button("Filter")
//!     .style(move |_theme, status| chip::selected(&tokens, status));
//! # let _: iced::widget::Button<'_, (), iced::Theme, iced::Renderer> = chip_button;
//! ```

use iced::{Border, Color, Shadow, widget::button};
use snora_design::Tokens;

use super::button::disabled_alpha;
use super::color::to_iced_color;
use super::state::{Knob, Pair, shifted};

/// A chip's hovered or pressed fill (RFC-129): `fill` shifted by the first candidate that keeps
/// `label` at the palette's text level on every neutral surface, `today` (a darkening, always,
/// before RFC-129) first.
fn fill_in_state(tokens: &Tokens, label: Color, fill: Color, today: f32) -> Color {
    let rest = Pair { label, fill };
    shifted(&tokens.palette, Knob::Fill, today, rest, rest).fill
}

/// Selected chip style: solid accent background + accent_text foreground.
///
/// This replaces the previous semi-transparent tint approach, which failed
/// WCAG AA (4.5:1) at hovered (α=0.22) and pressed (α=0.30) states after
/// compositing over the surface. Solid background + paired foreground role
/// yields ≥6.7:1 across all four built-in presets.
pub fn selected(tokens: &Tokens, status: button::Status) -> button::Style {
    let accent = to_iced_color(tokens.palette.accent);
    let accent_text = to_iced_color(tokens.palette.accent_text);
    let disabled = status == button::Status::Disabled;
    let bg = match status {
        button::Status::Active => accent,
        button::Status::Hovered => fill_in_state(tokens, accent_text, accent, -0.06),
        button::Status::Pressed => fill_in_state(tokens, accent_text, accent, -0.12),
        button::Status::Disabled => Color { a: 0.5, ..accent },
    };
    // The label and border dim with the fill (RFC-106 Q-2), so the two
    // chip states' disabled treatment reads alike.
    let (text_color, border_color) = if disabled {
        (disabled_alpha(accent_text), disabled_alpha(accent))
    } else {
        (accent_text, accent)
    };
    button::Style {
        background: Some(bg.into()),
        text_color,
        border: Border::default()
            .rounded(tokens.radius.pill)
            .color(border_color)
            .width(1.0),
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Unselected chip style: `surface` fill, a `border` outline and a `text_secondary` label.
///
/// Hovered and pressed darken the fill; see the [module](self) for how far.
pub fn unselected(tokens: &Tokens, status: button::Status) -> button::Style {
    let border_col = to_iced_color(tokens.palette.border);
    let text_col = to_iced_color(tokens.palette.text_secondary);
    let surface = to_iced_color(tokens.palette.surface);
    let disabled = status == button::Status::Disabled;
    let bg = match status {
        button::Status::Active => surface,
        button::Status::Hovered => fill_in_state(tokens, text_col, surface, -0.04),
        button::Status::Pressed => fill_in_state(tokens, text_col, surface, -0.08),
        button::Status::Disabled => Color { a: 0.5, ..surface },
    };
    // **This chip's fill cannot carry its disabled cue** (RFC-106): it is
    // `surface` at half alpha over a page that is nearly `surface`, which
    // measured 1.00-1.05:1 against the enabled fill — a disabled
    // unselected chip was indistinguishable from an enabled one. The
    // label and border carry it instead.
    let (text_color, border_color) = if disabled {
        (disabled_alpha(text_col), disabled_alpha(border_col))
    } else {
        (text_col, border_col)
    };
    button::Style {
        background: Some(bg.into()),
        text_color,
        border: Border::default()
            .rounded(tokens.radius.pill)
            .color(border_color)
            .width(1.0),
        shadow: Shadow::default(),
        snap: true,
    }
}

#[cfg(test)]
mod tests;
