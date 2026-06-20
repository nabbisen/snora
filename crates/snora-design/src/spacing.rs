//! The [`Spacing`] scale.

/// A small, fixed spacing scale (logical pixels).
///
/// Semantic guidance: `xs` tiny inline gap, `sm` compact internal gap, `md`
/// ordinary component padding/gap, `lg` section spacing, `xl` page-region
/// spacing, `xxl` major layout separation.
///
/// ```
/// use snora_design::Spacing;
/// let s = Spacing::comfortable();
/// assert!(s.xs < s.md && s.md < s.xxl);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spacing {
    /// Tiny inline gap.
    pub xs: f32,
    /// Compact internal gap.
    pub sm: f32,
    /// Ordinary component padding / gap.
    pub md: f32,
    /// Section spacing.
    pub lg: f32,
    /// Page-region spacing.
    pub xl: f32,
    /// Major layout separation.
    pub xxl: f32,
}

impl Spacing {
    /// The default ("comfortable") spacing scale shared by all v0.20 presets.
    #[must_use]
    pub const fn comfortable() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 16.0,
            xl: 24.0,
            xxl: 32.0,
        }
    }
}

#[cfg(test)]
mod tests;
