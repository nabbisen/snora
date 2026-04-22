//! Shared styling for snora's built-in widgets.
//!
//! These are small, opinionated defaults. Applications that need a
//! distinct visual identity should not try to override these — they
//! should instead skip the built-in widgets ([`crate::widget`]) and
//! build their own elements for the relevant [`crate::AppLayout`] slots.
//! The framework does **not** gate that path: AppLayout slots are raw
//! `Element`s, so your own widgets compose into the skeleton without
//! touching snora's style surface.

use iced::{
    Border, Color, Shadow, Theme,
    widget::{button, container},
};

/// Neutral text-only button used for menu entries. Colors are pulled from
/// the theme's primary palette so the widget follows light / dark mode.
pub fn menu_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let ep = theme.extended_palette();
    let text_color = match status {
        button::Status::Hovered => ep.primary.strong.color,
        button::Status::Pressed => ep.primary.base.color,
        _ => ep.primary.weak.color,
    };
    button::Style {
        background: None,
        text_color,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Thin-bordered chrome container used for the app header and footer.
pub fn chrome_container_style(theme: &Theme) -> container::Style {
    let ep = theme.extended_palette();
    container::Style {
        text_color: Some(ep.background.base.text),
        background: None,
        border: Border {
            radius: 0.0.into(),
            width: 1.0,
            color: ep.background.weak.color,
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Subtle highlight for the currently-active sidebar item. Returns a
/// background color appropriate for the theme.
pub fn sidebar_active_color(theme: &Theme) -> Color {
    theme.extended_palette().primary.weak.color
}
