// Token types (enumerated, not glob).
pub use snora_design::{
    Color, Density, Emphasis, FocusTokens, Palette, Radius, Size, Spacing, TextRole, Tokens, Tone,
    Typography,
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
pub mod style {
    pub use snora_widgets::design::style::button;
    pub use snora_widgets::design::style::color;
    pub use snora_widgets::design::style::container;
    pub use snora_widgets::design::style::progress;
    pub use snora_widgets::design::style::text;
}

/// Token-derived `iced::Theme` emission (RFC-038).
///
/// [`theme`] derives a complete `iced::Theme` from a [`Tokens`] bundle so
/// stock iced widgets and the window background follow the same palette as
/// snora's design primitives — without letting iced substitute its own
/// heuristically-corrected colors for the contrast-tested ones the preset
/// guarantees. See [`theme`] for the full mapping and usage example.
pub use snora_widgets::design::theme::theme;

/// Token-derived engine surface styling: the dialog card and the modal
/// dim (RFC-039).
///
/// [`render`] is a sibling to [`crate::render::render`] (`snora::render`),
/// not a replacement — see [`render`]'s own documentation for the
/// derivation and the gating-invariant guarantee.
pub mod render;
pub use render::render;

/// Token-derived chrome geometry: styled variants of the header, sidebar,
/// footer, tab bar, and breadcrumb (RFC-040).
///
/// Each function takes `&Tokens` first, then the same parameters as its
/// unstyled `snora::widget::*` counterpart. Colors already follow the
/// theme (RFC-038); these additionally map padding, gaps, and corner
/// radii to `tokens.spacing`/`tokens.radius`. See
/// `snora_widgets::design::widget`'s module documentation for the full
/// mapping table and rationale.
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
pub mod card {
    pub use snora_widgets::design::card::{raised, selected, surface};
}

/// Notice banner primitive (RFC-032).
///
/// Builder: `Notice::new(tokens, tone, body).title(…).action(…).dismiss(…).render()`.
pub mod notice {
    pub use snora_widgets::design::notice::Notice;
}

/// Filter and removable chip primitives (RFC-032).
pub mod chip {
    pub use snora_widgets::design::chip::{filter, removable};
}

/// Progress row and card primitives (RFC-032).
pub mod progress {
    pub use snora_widgets::design::progress::{card, row};
}

#[cfg(test)]
mod tests;
