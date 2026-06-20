//! Pilot card helper for Snora Design (RFC-029).
//!
//! Provides token-driven wrappers around `iced::widget::container` for the
//! three semantic card variants: [`surface`], [`raised`], and [`selected`].
//!
//! # Non-interactive cards only
//!
//! Cards in v0.20 are **non-interactive visual grouping surfaces**. They do
//! not own application behaviour and do not emit messages. Interactive card
//! semantics (selection, navigation) require a separate semantic construction
//! review before they can be safely added; see RFC-027.
//!
//! # Token cloning
//!
//! The returned `Element<'a, Message>` must own its style closure, so each
//! helper clones the `Tokens` once. The clone cost is acceptable in a
//! retained-mode `view()` function.
//!
//! # iced 0.14 limitation
//!
//! `iced::widget::container` has no interaction status in iced 0.14; the
//! style closure receives only `&Theme`. Focus rings and hover effects are not
//! expressible on container-backed cards through the standard styling path.
//!
//! # Usage
//!
//! ```rust,ignore
//! use snora_design::Tokens;
//! use snora_widgets::design::card;
//! use iced::widget::text;
//!
//! let tokens = Tokens::light();
//!
//! let my_card = card::surface(&tokens, text("Card content"));
//! let highlighted = card::selected(&tokens, text("Active item"));
//! ```

use iced::{Element, widget::container};
use snora_design::Tokens;

use super::style;

// ---- internal helper --------------------------------------------------------

fn make_card<'a, Message: 'a>(
    tokens: &Tokens,
    content: impl Into<Element<'a, Message>>,
    style_fn: fn(&Tokens) -> container::Style,
) -> Element<'a, Message> {
    let padding = tokens.spacing.md;
    let t = tokens.clone();
    container(content)
        .padding(padding)
        .style(move |_theme| style_fn(&t))
        .into()
}

// ---- public API -------------------------------------------------------------

/// Standard card on a `surface` background.
///
/// The default card for grouping related content — form sections, result
/// summaries, status panels. Sits on the application background.
#[must_use]
pub fn surface<'a, Message: 'a>(
    tokens: &Tokens,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    make_card(tokens, content, style::container::card_surface)
}

/// Elevated card on a `surface_raised` background with a soft drop shadow.
///
/// Use for content that visually floats above surrounding material — floating
/// panels, highlighted sections, or primary feature cards.
#[must_use]
pub fn raised<'a, Message: 'a>(
    tokens: &Tokens,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    make_card(tokens, content, style::container::card_raised)
}

/// Card with an `accent`-coloured border indicating a selected state.
///
/// Use when multiple peer cards can be selected and the active one must be
/// visually distinguished.
///
/// **Non-interactive in v0.20.** The visual selection state is controlled by
/// the caller (pass `card::selected` when the card's id matches the active
/// selection, `card::surface` otherwise). Interactive selection semantics
/// require a separate review; see RFC-027.
#[must_use]
pub fn selected<'a, Message: 'a>(
    tokens: &Tokens,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    make_card(tokens, content, style::container::card_selected)
}
