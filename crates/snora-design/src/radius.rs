//! The corner [`Radius`] scale.

/// A small corner-radius scale (logical pixels).
///
/// `sm` small controls, `md` buttons/chips/notices, `lg` cards/panels,
/// `pill` fully rounded indicators.
///
/// ```
/// use snora_design::Radius;
/// let r = Radius::default_roles();
/// assert!(r.sm <= r.lg && r.pill >= r.lg);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radius {
    /// Small controls.
    pub sm: f32,
    /// Buttons, chips, notices.
    pub md: f32,
    /// Cards, panels.
    pub lg: f32,
    /// Fully rounded (chips, pills).
    pub pill: f32,
}

impl Radius {
    /// The default radius scale shared by all v0.20 presets.
    #[must_use]
    pub const fn default_roles() -> Self {
        Self {
            sm: 4.0,
            md: 6.0,
            lg: 10.0,
            pill: 999.0,
        }
    }
}

#[cfg(test)]
mod tests;
