// Token types (enumerated, not glob).
pub use snora_design::{
    Color, Density, Emphasis, FocusTokens, Palette, Radius, Size, Spacing, TextRole, Tokens,
    Tone, Typography,
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
