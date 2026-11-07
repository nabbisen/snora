//! Pointer-target tokens ([`TargetTokens`]).

/// The smallest size, in logical pixels, a snora control's pointer target
/// may be (WCAG 2.5.8, *Target Size (Minimum)*; RFC-115).
///
/// A control's target is the **largest** of three things: this minimum,
/// 24 px, and 1.5 times the text it carries. Of the three, only this one
/// is yours to move:
///
/// - **Raise it** to give every snora control a bigger target than the
///   WCAG floor, for touch, or for users who need one. `44.0` is the
///   figure the platform guidelines use.
/// - **Lowering it below 24 does nothing.** 24 px is WCAG's own floor and
///   is applied regardless, so the value is a floor on the floor and
///   cannot be used to take targets under it.
///
/// Every preset sets it to `24.0`, so choosing a preset changes no size.
///
/// ```
/// use snora_design::Tokens;
///
/// let mut tokens = Tokens::light();
/// assert_eq!(tokens.targets.min, 24.0);
///
/// // Bigger targets for a touch-first application.
/// tokens.targets.min = 44.0;
/// ```
///
/// `#[non_exhaustive]`: further target tokens can be added without a
/// breaking change. Read the fields; build one with [`TargetTokens::new`].
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TargetTokens {
    /// The minimum pointer-target size, in logical pixels, for both
    /// width and height. See the type's documentation for how it
    /// combines with the 24 px floor and the text-relative rule.
    pub min: f32,
}

impl TargetTokens {
    /// Constructs target tokens from the minimum target size.
    #[must_use]
    pub const fn new(min: f32) -> Self {
        Self { min }
    }
}

#[cfg(test)]
mod tests;
