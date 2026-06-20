//! Progress primitives for Snora Design (RFC-032).
//!
//! Two layout variants:
//!
//! * [`row`] — compact inline label + bar, for use inside existing surfaces.
//! * [`card`] — the same content wrapped in a `card::surface` for prominence.
//!
//! Both accept `Option<f32>` for the value:
//! * `Some(v)` — determinate progress, `v` clamped to `0.0..=1.0`.
//! * `None` — indeterminate; renders the bar at 0% with a "…" label suffix
//!   (iced 0.14 has no native indeterminate animation; documented limitation).
//!
//! The application owns the task, progress value, and cancellation semantics.
//!
//! # Semantic construction (RFC-027)
//!
//! Uses `iced::widget::progress_bar`. Display-only; emits no events.
//!
//! # Usage
//!
//! ```rust,ignore
//! use snora_design::{Tokens, Tone};
//! use snora_widgets::design::progress;
//!
//! let tokens = Tokens::light();
//!
//! progress::row(&tokens, "Indexing files", Some(0.6), Tone::Accent)
//! progress::card(&tokens, "Syncing", None, Tone::Info)
//! ```

use iced::{Element, Length, widget::{column, container, progress_bar, text}};
use snora_design::{Tone, Tokens};

use super::{card, style};

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn progress_content<'a, Message: 'a>(
    tokens: &'a Tokens,
    label: &'a str,
    value: Option<f32>,
    tone: Tone,
) -> Element<'a, Message> {
    let t = tokens;
    let (display_value, label_suffix) = match value {
        Some(v) => (v.clamp(0.0, 1.0), format!("{:.0}%", v.clamp(0.0, 1.0) * 100.0)),
        // Indeterminate: iced 0.14 has no native indeterminate animation.
        // Show 0% bar with "…" suffix to signal in-progress state.
        // See docs/src/design/v021-primitives.md for the documented limitation.
        None    => (0.0_f32, "…".into()),
    };

    let label_line = format!("{label}  {label_suffix}");
    let tok = t.clone();

    column![
        text(label_line)
            .size(style::text::body_size(t))
            .color(style::color::to_iced_color(t.palette.text_primary)),
        progress_bar(0.0..=1.0, display_value)
            .girth(8.0)
            .length(Length::Fill)
            .style(move |_theme| style::progress::toned(&tok, tone)),
    ]
    .spacing(t.spacing.xs)
    .into()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compact progress indicator: label + bar, no outer card.
///
/// Use inside an existing card or panel. The progress value is clamped to
/// `0.0..=1.0`; pass `None` for indeterminate state.
#[must_use]
pub fn row<'a, Message: 'a>(
    tokens: &'a Tokens,
    label: &'a str,
    value: Option<f32>,
    tone: Tone,
) -> Element<'a, Message> {
    container(progress_content(tokens, label, value, tone))
        .width(Length::Fill)
        .into()
}

/// Progress indicator wrapped in a `card::surface` for visual prominence.
///
/// Use when the progress item is a primary focus of a region. The progress
/// value is clamped to `0.0..=1.0`; pass `None` for indeterminate state.
#[must_use]
pub fn card<'a, Message: 'a>(
    tokens: &'a Tokens,
    label: &'a str,
    value: Option<f32>,
    tone: Tone,
) -> Element<'a, Message> {
    card::surface(tokens, progress_content(tokens, label, value, tone))
}
