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
//!
//! **`Emphasis` and `Size` were removed in 0.45.0 (RFC-095).** Both shipped
//! in v0.19 (RFC-020…RFC-030) and were read by nothing for 24 minors —
//! confirmed unread by all six adopting teams, not inferred from silence
//! (RFC-078's error was trusting silence; this asked and got six positive
//! checks). `Size` additionally shadowed `iced::Size`, a type the engine
//! uses heavily, with no compiler error to warn a consumer who reached for
//! the wrong one. Removing a frozen-surface item is forbidden under
//! RFC-036's additive-only covenant except through its own reopening
//! condition; this RFC paid that price explicitly — see
//! `docs/src/contributing/api-freeze-review.md`'s D-3/D-4 rows.

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
