// Token types (enumerated, not glob).
pub use snora_design::{
    Color, Density, FocusTokens, Palette, Radius, Spacing, TextRole, Tokens, Tone, Typography,
};

/// Pure-Rust WCAG contrast utilities, re-exported from `snora-design`.
///
/// Available through the `snora::design` facade so that applications do
/// not need a direct `snora-design` dependency to use the contrast
/// utilities alongside the rest of the design system.
///
/// Functions:
///
/// * [`contrast_ratio`] — WCAG 2.1 contrast ratio between two opaque
///   colors (range 1.0–21.0; AA body text threshold is 4.5).
/// * [`relative_luminance`] — WCAG 2.1 relative luminance of a color
///   (range 0.0–1.0).
/// * [`composite_over`] — alpha-composite a translucent foreground over
///   an opaque background before contrast checking.
///
/// [`contrast_ratio`]: snora_design::contrast::contrast_ratio
/// [`relative_luminance`]: snora_design::contrast::relative_luminance
/// [`composite_over`]: snora_design::contrast::composite_over
pub mod contrast {
    pub use snora_design::contrast::{composite_over, contrast_ratio, relative_luminance};
}

/// iced style functions derived from Snora Design tokens.
///
/// Reaches [`snora_style`] directly (RFC-055) — available whenever
/// `design` is enabled, independent of `widgets`. Not gated: unlike the
/// modules below, this one does not depend on `snora-widgets` at all.
pub mod style {
    pub use snora_style::button;
    pub use snora_style::color;
    pub use snora_style::container;
    pub use snora_style::progress;
    pub use snora_style::text;
}

/// Token-derived `iced::Theme` emission (RFC-038).
///
/// [`theme`] derives a complete `iced::Theme` from a [`Tokens`] bundle so
/// stock iced widgets and the window background follow the same palette as
/// snora's design primitives — without letting iced substitute its own
/// heuristically-corrected colors for the contrast-tested ones the preset
/// guarantees. See [`theme`] for the full mapping and usage example.
///
/// Reaches [`snora_style`] directly (RFC-055 round 2) — available
/// whenever `design` is enabled, independent of `widgets`, matching
/// [`style`] above. Not `widgets`-gated: `theme.rs` imports nothing
/// from the widget layer either (it styles *stock* iced widgets —
/// `text_input`, `pick_list`, `scrollable` — not snora's own prefab
/// ones), and the one known consumer of it has zero
/// `snora::widget::*` call sites, so gating it behind `widgets` would
/// have made design-without-widgets incomplete for exactly the
/// consumer it exists to serve.
pub use snora_style::theme::theme;

/// Token-derived engine surface styling: the dialog card and the modal
/// dim (RFC-039).
///
/// [`render()`] is a sibling to [`crate::render::render`] (`snora::render`),
/// not a replacement — see [`render()`]'s own documentation for the
/// derivation and the gating-invariant guarantee.
///
/// [`responsive_render`] is the `design`-path pair to
/// [`crate::responsive::responsive_render`] (`snora::responsive_render`)
/// — width-aware layout that keeps the styled dialog card and derived
/// modal dim, where the engine-path function would silently drop them
/// (RFC-053).
pub mod render;
pub use render::{render, responsive_render};

/// Token-derived chrome geometry: styled variants of the header, sidebar,
/// footer, tab bar, and breadcrumb (RFC-040).
///
/// Each function takes `&Tokens` first, then the same parameters as its
/// unstyled `snora::widget::*` counterpart. Colors already follow the
/// theme (RFC-038); these additionally map padding, gaps, and corner
/// radii to `tokens.spacing`/`tokens.radius`. See
/// `snora_widgets::design::widget`'s module documentation for the full
/// mapping table and rationale.
///
/// `widgets`-gated (RFC-055): this is a widget, not a style function —
/// it returns a composed `Element`, and depends on `snora-widgets`'s own
/// prefab building blocks.
#[cfg(feature = "widgets")]
pub mod widget {
    pub use snora_widgets::design::widget::{
        app_breadcrumb, app_footer, app_header, app_side_bar, app_tab_bar,
    };
}

/// Pilot button helpers (RFC-028).
///
/// Each function wraps `iced::widget::button` with Snora Design token
/// styling. Token ownership is handled internally via `Clone`; callers
/// do not need to annotate lifetimes.
///
/// `widgets`-gated (RFC-055): see [`style::button`] for the underlying
/// style functions these wrap, which work without `widgets`.
#[cfg(feature = "widgets")]
pub mod button {
    pub use snora_widgets::design::button::{
        danger, danger_maybe, ghost, ghost_maybe, primary, primary_maybe, secondary,
        secondary_maybe,
    };
}

/// Pilot card helpers (RFC-029).
///
/// Each function wraps `iced::widget::container` with Snora Design token
/// styling. Cards are non-interactive visual grouping surfaces;
/// application behaviour lives outside the card.
///
/// `widgets`-gated (RFC-055): see [`style::container`] for the
/// underlying style functions these wrap, which work without `widgets`
/// — the same ones [`crate::design::render()`]'s dialog card uses.
#[cfg(feature = "widgets")]
pub mod card {
    pub use snora_widgets::design::card::{raised, selected, surface};
}

/// Notice banner primitive (RFC-032).
///
/// Builder: `Notice::new(tokens, tone, body).title(…).action(…).dismiss(…).render()`.
///
/// `widgets`-gated (RFC-055).
#[cfg(feature = "widgets")]
pub mod notice {
    pub use snora_widgets::design::notice::Notice;
}

/// Filter and removable chip primitives (RFC-032).
///
/// `widgets`-gated (RFC-055).
#[cfg(feature = "widgets")]
pub mod chip {
    pub use snora_widgets::design::chip::{filter, removable};
}

/// Progress row and card primitives (RFC-032).
///
/// `widgets`-gated (RFC-055): see [`style::progress`] for the underlying
/// style functions, which work without `widgets`.
#[cfg(feature = "widgets")]
pub mod progress {
    pub use snora_widgets::design::progress::{card, row};
}

#[cfg(test)]
mod tests;
