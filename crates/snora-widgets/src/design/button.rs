//! Pilot button helper for Snora Design (RFC-028).
//!
//! Provides ergonomic, token-driven wrappers around `iced::widget::button`
//! for the four semantic variants: [`primary`], [`secondary`], [`ghost`], and
//! [`danger`].
//!
//! # Token cloning
//!
//! The returned `Element<'a, Message>` owns its iced style closure, which must
//! be `'a`. To avoid binding a `&'a Tokens` lifetime into the caller's `view`
//! signature, each helper clones the `Tokens` once into the closure. `Tokens`
//! is `Clone` and small; the clone cost is acceptable in a retained-mode
//! `view()` function.
//!
//! # iced 0.14 focus limitation
//!
//! `iced::widget::button::Status` has `Active | Hovered | Pressed | Disabled`
//! — no `Focused` variant. These helpers do not render a custom focus ring.
//! That is a known, documented limitation for iced 0.14 (see RFC-025 and
//! `docs/src/contributing/semantic-accessibility.md`), not a regression.
//!
//! # Disabled state
//!
//! Pass `on_press: None` to disable the button. Iced automatically applies the
//! `Disabled` status when no press handler is set.
//!
//! # Usage
//!
//! ```rust,ignore
//! use snora_design::Tokens;
//! use snora_widgets::design::button;
//!
//! let tokens = Tokens::light();
//!
//! // Enabled
//! let save = button::primary(&tokens, "Save", Message::Save);
//!
//! // Disabled
//! let save = button::primary_maybe(&tokens, "Save", None::<Message>);
//! ```

use iced::{Element, widget::{button, text}};
use snora_design::Tokens;

use super::style;

// ---- helpers ----------------------------------------------------------------

fn make_button<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    on_press: Option<Message>,
    style_fn: fn(&Tokens, button::Status) -> button::Style,
) -> Element<'a, Message> {
    let t = tokens.clone();
    button(text(label.into()).size(super::style::text::label_size(tokens)))
        .on_press_maybe(on_press)
        .style(move |_theme, status| style_fn(&t, status))
        .into()
}

// ---- public API -------------------------------------------------------------

/// Filled accent button (primary call to action).
///
/// Uses `palette.accent` as the background and `palette.accent_text` as the
/// foreground. Reserve for the single strongest action on a surface.
///
/// To disable, use [`primary_maybe`] with `None`.
#[must_use]
pub fn primary<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    on_press: Message,
) -> Element<'a, Message> {
    make_button(tokens, label, Some(on_press), style::button::primary)
}

/// Filled accent button with optional press handler.
///
/// `on_press: None` produces a disabled button styled with the `Disabled`
/// status automatically.
#[must_use]
pub fn primary_maybe<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    make_button(tokens, label, on_press, style::button::primary)
}

/// Outlined accent button (secondary action).
///
/// Uses a transparent background with an `accent`-coloured border and text.
/// Use alongside a [`primary`] button for secondary actions.
///
/// To disable, use [`secondary_maybe`] with `None`.
#[must_use]
pub fn secondary<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    on_press: Message,
) -> Element<'a, Message> {
    make_button(tokens, label, Some(on_press), style::button::secondary)
}

/// Outlined accent button with optional press handler.
#[must_use]
pub fn secondary_maybe<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    make_button(tokens, label, on_press, style::button::secondary)
}

/// Ghost button (no fill or border at rest; tertiary / low-emphasis action).
///
/// To disable, use [`ghost_maybe`] with `None`.
#[must_use]
pub fn ghost<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    on_press: Message,
) -> Element<'a, Message> {
    make_button(tokens, label, Some(on_press), style::button::ghost)
}

/// Ghost button with optional press handler.
#[must_use]
pub fn ghost_maybe<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    make_button(tokens, label, on_press, style::button::ghost)
}

/// Danger / destructive button (irreversible actions: delete, revoke, reset).
///
/// Uses `palette.danger` as the background and `palette.danger_text` as the
/// foreground. The `danger_text on danger` contrast pair is mandatory (RFC-023)
/// and is verified by the `snora-design` automated contrast tests.
///
/// To disable, use [`danger_maybe`] with `None`.
#[must_use]
pub fn danger<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    on_press: Message,
) -> Element<'a, Message> {
    make_button(tokens, label, Some(on_press), style::button::danger)
}

/// Danger button with optional press handler.
#[must_use]
pub fn danger_maybe<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    make_button(tokens, label, on_press, style::button::danger)
}
