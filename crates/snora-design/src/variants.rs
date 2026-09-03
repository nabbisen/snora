//! Shared semantic variant vocabulary.
//!
//! Intentionally generic, and must stay small.
//!
//! # Which of these anything actually reads
//!
//! Checked 2026-09-02, not inherited — the previous version of this comment
//! said these enums are *"reused across buttons, chips, notices, and
//! progress"*, and that was true of none of those four surfaces in the way it
//! implied:
//!
//! | Enum | Read by |
//! |---|---|
//! | [`Tone`] | `snora_widgets::design::notice`, and `progress` via `snora_style::progress::toned`. **Not** buttons or chips — neither takes it. |
//! | [`Density`] | [`crate::Tokens`] carries it as a field; the presets set it. |
//! | [`Emphasis`] | **Nothing.** Published and re-exported through `snora::design`; no widget or style function varies anything by it. |
//! | [`Size`] | **Nothing.** Same. Note it is unrelated to `iced::Size`, a different type used widely in the engine — not linked here, because this crate is iced-free by CI gate (RFC-083) and an intra-doc link would not resolve. |
//!
//! `Emphasis` and `Size` have been reserved vocabulary since they shipped in
//! v0.19 (RFC-020…RFC-030). They are frozen public surface under RFC-036's
//! additive-only covenant, so they cannot simply be dropped; whether they get
//! consumers or get removed is a 1.0 question, tracked against the API freeze
//! review rather than left to be rediscovered.

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
