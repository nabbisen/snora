//! Progress primitives for Snora Design (RFC-032).
//!
//! Two layout variants:
//!
//! * `row` — compact inline label + bar, for use inside existing surfaces.
//! * [`card`] — the same content wrapped in a `card::surface` for prominence.
//!
//! Both accept `Option<f32>` for the value:
//! * `Some(v)` — determinate progress, `v` clamped to `0.0..=1.0`.
//! * `None` — indeterminate; renders the bar at 0% with a "…" label suffix
//!   (iced 0.14 has no native indeterminate animation; documented limitation).
//!
//! The application owns the task, progress value, and cancellation semantics.
//!
//! # Direction (RFC-133)
//!
//! A progress bar has a reading order: it **fills from the start edge**, the left under
//! [`LayoutDirection::Ltr`](snora_core::LayoutDirection::Ltr) and the right under
//! [`LayoutDirection::Rtl`](snora_core::LayoutDirection::Rtl), and its label sits at
//! the start edge too. Both functions take the direction as their last argument.
//!
//! # Semantic construction (RFC-027)
//!
//! Display-only; emits no events. The track and the fill are drawn here, not by
//! `iced::widget::progress_bar`: iced's has no direction and always fills from the left, and a
//! reflection of the whole bar cannot move a fill that is one child. The fill is a
//! [`LogicalRow`](crate::direction::LogicalRow) of two cells in proportion to the value, so RFC-132's one mechanism places it,
//! and under left-to-right it is pixel-identical to iced's bar (the test
//! `the_fill_is_the_bar_iced_draws_under_ltr` draws both, for every preset, tone and value).
//!
//! # Usage
//!
//! ```rust,no_run
//! use snora_core::LayoutDirection;
//! use snora_design::{Tokens, Tone};
//! use snora_widgets::design::progress;
//!
//! enum Message {}
//!
//! let tokens = Tokens::light();
//!
//! let _row: iced::Element<'_, Message> =
//!     progress::row(&tokens, "Indexing files", Some(0.6), Tone::Accent, LayoutDirection::Ltr);
//! let _card: iced::Element<'_, Message> =
//!     progress::card(&tokens, "Syncing", None, Tone::Info, LayoutDirection::Rtl);
//! ```

use iced::{
    Background, Element, Length,
    widget::{column, container, space, text},
};
use snora_core::LayoutDirection;
use snora_design::{Tokens, Tone};

use super::card;
use crate::direction::{LogicalRow, start_aligned};
use snora_style as style;

/// The fill is a ratio of two integers, `filled : STEPS - filled`, because iced's
/// `FillPortion` is a `u16`. The value is rounded to the nearest `1 / STEPS`, so the fill is at
/// most half a step off: **under 0.1 px at any width up to 10,000 px** (at the 760 px of an
/// 800 px window it is 0.04 px).
const STEPS: u16 = 10_000;

/// The track and the fill, from the start edge of `direction`.
///
/// **An empty end is left out of the row.** iced treats `FillPortion(0)` as `Fill`, not as no
/// width, so a cell asked for zero would take the room that is left, and a 0% bar would draw
/// full. At 0% there is no fill cell, and at 100% no rest cell.
fn bar<'a, Message: 'a>(
    tokens: &Tokens,
    value: f32,
    tone: Tone,
    direction: LayoutDirection,
) -> Element<'a, Message> {
    let look = style::progress::toned(tokens, tone);
    let filled = (value.clamp(0.0, 1.0) * f32::from(STEPS)).round() as u16;
    // **The fill keeps the track's border width and radius, with a transparent colour**, as iced's
    // bar did. A renderer that draws a border as a rim over the fill (wgpu) shows the track's own
    // border through it, one pixel in; a fill with no border would paint over that rim, and the
    // two would differ on the GPU and not on the CPU.
    let bar = look.bar;
    let fill_border = iced::Border {
        color: iced::Color::TRANSPARENT,
        ..look.border
    };

    let mut cells = LogicalRow::new(direction)
        .width(Length::Fill)
        .height(Length::Fill);
    if filled > 0 {
        cells = cells.push(
            container(space())
                .width(Length::FillPortion(filled))
                .height(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(bar),
                    border: fill_border,
                    ..container::Style::default()
                }),
        );
    }
    if filled < STEPS {
        cells = cells.push(
            container(space())
                .width(Length::FillPortion(STEPS - filled))
                .height(Length::Fill),
        );
    }

    let (track, border): (Background, iced::Border) = (look.background, look.border);
    container(cells)
        .width(Length::Fill)
        .height(Length::Fixed(8.0))
        .style(move |_| container::Style {
            background: Some(track),
            border,
            ..container::Style::default()
        })
        .into()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn progress_content<'a, Message: 'a>(
    tokens: &'a Tokens,
    label: &'a str,
    value: Option<f32>,
    tone: Tone,
    direction: LayoutDirection,
) -> Element<'a, Message> {
    let t = tokens;
    let (display_value, label_suffix) = match value {
        Some(v) => (
            v.clamp(0.0, 1.0),
            format!("{:.0}%", v.clamp(0.0, 1.0) * 100.0),
        ),
        // Indeterminate: iced 0.14 has no native indeterminate animation.
        // Show 0% bar with "…" suffix to signal in-progress state.
        // See docs/src/design/v021-primitives.md for the documented limitation.
        None => (0.0_f32, "…".into()),
    };

    let label_line = format!("{label}  {label_suffix}");

    column![
        // The label is as wide as the bar and aligned to the start edge (RFC-133 R-4).
        text(label_line)
            .size(style::text::body_size(t))
            .color(style::color::to_iced_color(t.palette.text_primary))
            .width(Length::Fill)
            .align_x(start_aligned(direction)),
        bar(t, display_value, tone, direction),
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
/// `0.0..=1.0`; pass `None` for indeterminate state. The bar fills from the start edge of
/// `direction`.
#[must_use]
pub fn row<'a, Message: 'a>(
    tokens: &'a Tokens,
    label: &'a str,
    value: Option<f32>,
    tone: Tone,
    direction: LayoutDirection,
) -> Element<'a, Message> {
    container(progress_content(tokens, label, value, tone, direction))
        .width(Length::Fill)
        .into()
}

/// Progress indicator wrapped in a `card::surface` for visual prominence.
///
/// Use when the progress item is a primary focus of a region. The progress
/// value is clamped to `0.0..=1.0`; pass `None` for indeterminate state. The bar fills from the
/// start edge of `direction`.
#[must_use]
pub fn card<'a, Message: 'a>(
    tokens: &'a Tokens,
    label: &'a str,
    value: Option<f32>,
    tone: Tone,
    direction: LayoutDirection,
) -> Element<'a, Message> {
    card::surface(
        tokens,
        progress_content(tokens, label, value, tone, direction),
    )
}

#[cfg(test)]
mod tests;
