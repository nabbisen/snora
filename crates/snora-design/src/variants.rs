//! Shared semantic variant vocabulary.
//!
//! These small enums are reused across buttons, chips, notices, and progress.
//! They are intentionally generic and must stay small.

/// Semantic intent of a styled element.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tone {
    /// Neutral / default.
    Neutral,
    /// Accent / primary emphasis.
    Accent,
    /// Success state.
    Success,
    /// Warning state.
    Warning,
    /// Danger / destructive state.
    Danger,
    /// Informational state.
    Info,
}

/// Visual weight of a styled element.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Emphasis {
    /// Solid fill.
    Solid,
    /// Soft / tinted fill.
    Soft,
    /// Outline only.
    Outline,
    /// Ghost (no fill or border until interaction).
    Ghost,
}

/// Control size step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Size {
    /// Small.
    Small,
    /// Medium (default).
    Medium,
    /// Large.
    Large,
}

/// UI density. In v0.20 the field exists and all presets are
/// [`Density::Comfortable`]; compact resolution is deferred.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Density {
    /// Comfortable (default) spacing.
    Comfortable,
    /// Compact spacing (reserved; not resolved in v0.20).
    Compact,
}

#[cfg(test)]
mod tests;
