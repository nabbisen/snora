//! Style function modules for the Snora Design iced bridge.
//!
//! Re-exported from `snora-style` (RFC-055) — every path below resolves
//! exactly as before the extraction; nothing here changed for a
//! consumer. See [`snora_style`]'s own module documentation for the
//! layering rationale.

/// Color conversion between `snora_design::Color` and `iced::Color`.
pub use snora_style::color;

/// Semantic button style functions.
pub use snora_style::button;

/// Card and container style functions.
pub use snora_style::container;

/// Text style helpers.
pub use snora_style::text;

/// Progress bar style functions.
pub use snora_style::progress;
