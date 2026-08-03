//! Shared styling for snora's built-in widgets.
//!
//! These are small, opinionated defaults. Applications that need a
//! distinct visual identity should not try to override these — they
//! should instead skip the built-in widgets in this crate and build
//! their own elements for the relevant
//! [`snora_core::AppLayout`] slots. The framework does **not** gate that
//! path: AppLayout slots are raw `Element`s, so your own widgets compose
//! into the skeleton without touching snora's style surface.

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
///
/// Radius fixed at `0.0` — today's literal, unchanged. See
/// [`chrome_container_style_with_radius`] for the `design`-gated styled
/// chrome widgets' (RFC-040) token-derived radius.
pub fn chrome_container_style(theme: &Theme) -> container::Style {
    chrome_container_style_with_radius(theme, 0.0)
}

/// Geometry-parameterized variant of [`chrome_container_style`] (RFC-040):
/// identical color logic, but the corner radius is a parameter instead of
/// a hardcoded `0.0`. [`chrome_container_style`] is a thin wrapper over
/// this with `radius = 0.0` — the one-body-two-geometry-sources shape
/// applied to the shared style function itself, not only the widgets that
/// call it.
///
/// Not part of the public style surface `chrome_container_style` is;
/// `pub(crate)` since only the `design::widget` styled variants need the
/// radius parameter.
pub(crate) fn chrome_container_style_with_radius(theme: &Theme, radius: f32) -> container::Style {
    let ep = theme.extended_palette();
    container::Style {
        text_color: Some(ep.background.base.text),
        background: None,
        border: Border {
            radius: radius.into(),
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
