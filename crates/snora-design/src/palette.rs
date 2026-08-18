//! The semantic color [`Palette`].

use crate::Color;

/// Semantic color roles for a theme.
///
/// Roles are named by *meaning*, not by a color scale. Status background roles
/// (`success`/`warning`/`danger`/`info`) each have a paired on-status
/// foreground (`*_text`) so status surfaces — starting with the v0.20 danger
/// button — have a contrast-tested foreground rather than borrowing
/// `accent_text`.
///
/// `Palette` is `#[non_exhaustive]`: new roles (the documented future roles
/// such as `surface_sunken`, `overlay`, `selection`, `separator`) can be added
/// without a breaking change. Construct one through a [`crate::Tokens`] preset
/// rather than a struct literal; you may still mutate individual fields
/// (`tokens.palette.accent = ...`).
///
/// `text_muted` is the lowest-contrast text role, for non-essential text,
/// and — like every other text role — meets WCAG AA body-text contrast
/// (4.5:1) against all three surfaces, asserted in the contrast suite
/// (RFC-058).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    /// Window / app background.
    pub background: Color,
    /// Primary surface (cards, panels).
    pub surface: Color,
    /// Raised surface (elevated cards, popovers).
    pub surface_raised: Color,

    /// Primary body text.
    pub text_primary: Color,
    /// Secondary text. Meets WCAG AA body-text contrast (4.5:1) against all
    /// three surfaces.
    pub text_secondary: Color,
    /// Muted text for non-essential content. Meets WCAG AA body-text
    /// contrast (4.5:1) against all three surfaces (RFC-058), same as every
    /// other text role.
    pub text_muted: Color,

    /// Borders and separators.
    pub border: Color,
    /// Accent / primary action color.
    pub accent: Color,
    /// Foreground used on top of `accent`.
    pub accent_text: Color,

    /// Success background.
    pub success: Color,
    /// Foreground used on top of `success`.
    pub success_text: Color,
    /// Warning background.
    pub warning: Color,
    /// Foreground used on top of `warning`.
    pub warning_text: Color,
    /// Danger / destructive background.
    pub danger: Color,
    /// Foreground used on top of `danger`.
    pub danger_text: Color,
    /// Informational background.
    pub info: Color,
    /// Foreground used on top of `info`.
    pub info_text: Color,

    /// Focus-ring color.
    pub focus: Color,
}

impl Palette {
    /// Returns every role color in a stable order, for crate-internal
    /// validation and tests. Crate-private: the fixed-size return type would
    /// become a breaking change if new roles are added to `#[non_exhaustive]`
    /// `Palette`. External code should access fields directly.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn roles(&self) -> [Color; 18] {
        [
            self.background,
            self.surface,
            self.surface_raised,
            self.text_primary,
            self.text_secondary,
            self.text_muted,
            self.border,
            self.accent,
            self.accent_text,
            self.success,
            self.success_text,
            self.warning,
            self.warning_text,
            self.danger,
            self.danger_text,
            self.info,
            self.info_text,
            self.focus,
        ]
    }
}
