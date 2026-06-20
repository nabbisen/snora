//! Chip primitives for Snora Design (RFC-032).
//!
//! Two variants:
//!
//! * [`filter`] — a toggle chip for filtering or categorizing. Tinted when
//!   selected, neutral at rest.
//! * [`removable`] — a chip with a separate remove (×) button.
//!
//! Both are backed by `iced::widget::button` and are keyboard-reachable.
//! The application owns selection/filter state.
//!
//! # iced 0.14 focus limitation
//!
//! No custom focus ring — `button::Status` has no `Focused` variant.
//! Documented limitation, not a regression (RFC-025, RFC-027).
//!
//! # Usage
//!
//! ```rust,ignore
//! use snora_design::Tokens;
//! use snora_widgets::design::chip;
//!
//! let tokens = Tokens::light();
//!
//! chip::filter(&tokens, "Draft", self.show_drafts, Message::ToggleDrafts)
//! chip::removable(&tokens, "Tag: Rust", true, Message::ToggleTag, Message::RemoveTag)
//! ```

use iced::{Border, Color, Element, widget::{button, row, text}};
use snora_design::Tokens;

use super::style;

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Blends a color toward black by `amount`. Used for hover/press states.
fn darken(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r - amount).max(0.0),
        g: (color.g - amount).max(0.0),
        b: (color.b - amount).max(0.0),
        a: color.a,
    }
}

fn chip_style_selected(tokens: &Tokens, status: button::Status) -> button::Style {
    let accent = style::color::to_iced_color(tokens.palette.accent);
    let bg = match status {
        button::Status::Active   => Color { a: 0.15, ..accent },
        button::Status::Hovered  => Color { a: 0.22, ..accent },
        button::Status::Pressed  => Color { a: 0.30, ..accent },
        button::Status::Disabled => Color { a: 0.06, ..accent },
    };
    button::Style {
        background: Some(bg.into()),
        text_color: accent,
        border: Border::default()
            .rounded(tokens.radius.pill)
            .color(accent)
            .width(1.0),
        shadow: iced::Shadow::default(),
        snap: true,
    }
}

fn chip_style_unselected(tokens: &Tokens, status: button::Status) -> button::Style {
    let border_col = style::color::to_iced_color(tokens.palette.border);
    let text_col   = style::color::to_iced_color(tokens.palette.text_secondary);
    let surface    = style::color::to_iced_color(tokens.palette.surface);
    let bg = match status {
        button::Status::Active   => surface,
        button::Status::Hovered  => darken(surface, 0.04),
        button::Status::Pressed  => darken(surface, 0.08),
        button::Status::Disabled => Color { a: 0.5, ..surface },
    };
    button::Style {
        background: Some(bg.into()),
        text_color: text_col,
        border: Border::default()
            .rounded(tokens.radius.pill)
            .color(border_col)
            .width(1.0),
        shadow: iced::Shadow::default(),
        snap: true,
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// A toggle chip for filtering or categorizing content.
///
/// Shows a tinted accent background and accent text when `selected`.
/// Emits `on_toggle` when pressed. Pass `None` to disable.
#[must_use]
pub fn filter<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    selected: bool,
    on_toggle: impl Into<Option<Message>>,
) -> Element<'a, Message> {
    let t = tokens.clone();
    let style_fn = if selected { chip_style_selected } else { chip_style_unselected };
    button(text(label.into()).size(style::text::label_size(tokens)))
        .on_press_maybe(on_toggle.into())
        .padding([tokens.spacing.xs, tokens.spacing.sm])
        .style(move |_theme, status| style_fn(&t, status))
        .into()
}

/// A chip with a separate remove (×) button.
///
/// The chip label toggles via `on_toggle`; the × button emits `on_remove`.
/// Both controls are `iced::widget::button` and are keyboard-reachable.
#[must_use]
pub fn removable<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    selected: bool,
    on_toggle: impl Into<Option<Message>>,
    on_remove: impl Into<Option<Message>>,
) -> Element<'a, Message> {
    let t_label  = tokens.clone();
    let t_remove = tokens.clone();
    let style_fn = if selected { chip_style_selected } else { chip_style_unselected };

    let label_btn: Element<'a, Message> = button(
        text(label.into()).size(style::text::label_size(tokens)),
    )
    .on_press_maybe(on_toggle.into())
    .padding([tokens.spacing.xs, tokens.spacing.sm])
    .style(move |_theme, status| style_fn(&t_label, status))
    .into();

    let remove_btn: Element<'a, Message> = button(
        text("×").size(style::text::label_size(tokens)),
    )
    .on_press_maybe(on_remove.into())
    .padding([tokens.spacing.xs, tokens.spacing.xs])
    .style(move |_theme, status| style_fn(&t_remove, status))
    .into();

    row![label_btn, remove_btn].spacing(0).into()
}

#[cfg(test)]
mod tests;
