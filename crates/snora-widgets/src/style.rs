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

/// Neutral text-only button used for menu entries.
///
/// **Corrected (RFC-085 F-13).** Every status previously used a shade
/// from `ep.primary` — a **background-tier** family — as `text_color`,
/// on `background: None`, so it painted over the dropdown surface's own
/// background. No status reached WCAG AA (measured: 1.89:1 light /
/// 2.20:1 dark at rest, 3.73:1 / 3.70:1 hovered). `background.base.text`
/// is the tier iced itself guarantees is readable against
/// `background.base.color`, which is what this button actually paints
/// over — used uniformly across all statuses, since the background never
/// changes. This trades away the old (illegible) hover/press color
/// differentiation; a future change may reintroduce it via a background
/// highlight rather than a foreground shade, following the same pattern
/// `crate::sidebar::sidebar_button_style` and
/// `crate::crumb::crumb_button_style` already use for their own hover
/// states — not done here, to keep this fix a re-pairing rather than an
/// added visual element.
pub fn menu_button_style(theme: &Theme, _status: button::Status) -> button::Style {
    let ep = theme.extended_palette();
    button::Style {
        background: None,
        text_color: ep.background.base.text,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Thin-bordered chrome container used for the app header and footer.
///
/// Radius fixed at `0.0` — today's literal, unchanged. See
/// `chrome_container_style_with_radius` for the `design`-gated styled
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
            // Corrected (RFC-085 F-15): `background.weak.color` measured
            // 1.02–1.48:1 against the page background in every preset and
            // both stock themes, well under the 3.0:1 non-text floor.
            // `background.strong` was tried next and measured better but
            // still short (1.54–1.58:1) — iced's `strong` tier is tuned
            // for a subtle surface fill, not a guaranteed-visible border.
            // `background.base.text` is the only value derivable from
            // `Theme::extended_palette()` alone that iced itself
            // guarantees sufficient contrast against `background.base`,
            // which is what this border is actually drawn over — bolder
            // than a typical hairline border, but reliably visible in
            // every preset and both stock themes.
            color: ep.background.base.text,
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

/// Highlight for the currently-active sidebar item. Returns a background
/// color appropriate for the theme.
///
/// **Corrected (RFC-085 F-14).** `primary.weak.color` measured 1.89:1
/// (stock light) / 2.20:1 (stock dark) against the rail's own
/// background — under the 3.0:1 non-text floor, and this was the
/// *only* cue an active item had (also a WCAG 1.4.1 use-of-colour
/// concern, addressed in `crate::sidebar::sidebar_button_style`'s own
/// doc comment). `primary.base` came within 0.01 of the floor on stock
/// Dark (2.99:1) — not a margin to ship on. `primary.strong` clears it
/// with real margin in every preset and both stock themes.
pub fn sidebar_active_color(theme: &Theme) -> Color {
    theme.extended_palette().primary.strong.color
}
