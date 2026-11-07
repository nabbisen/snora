//! Pilot button helper for Snora Design (RFC-028).
//!
//! Provides ergonomic, token-driven buttons for the four semantic
//! variants: `primary`, `secondary`, `ghost`, and `danger`. Each is a
//! `control` (RFC-116), styled from the tokens.
//!
//! # Token cloning
//!
//! The returned `Element<'a, Message>` owns its iced style closure, which must
//! be `'a`. To avoid binding a `&'a Tokens` lifetime into the caller's `view`
//! signature, each helper clones the `Tokens` once into the closure. `Tokens`
//! is `Clone` and small; the clone cost is acceptable in a retained-mode
//! `view()` function.
//!
//! # Keyboard
//!
//! iced 0.14's `button` takes no keyboard input and registers no focus
//! target (its `Status` has no `Focused` variant), which is why these are
//! not iced buttons. Each is a focus target: Tab reaches it once the
//! application routes the key with `snora::keyboard::traverse`, Enter and
//! Space press it and publish the message a click publishes, and a ring
//! drawn from `Tokens::focus` shows which has focus. A disabled button
//! (`*_maybe(.., None)`) is not a focus target, so Tab skips it. The
//! button is at least `max(24, 1.5 x its label, Tokens::targets.min)`
//! pixels in each direction (RFC-115).
//!
//! # Disabled state
//!
//! Pass `on_press: None` to disable the button: it draws with the `Disabled`
//! status, ignores the pointer and the keyboard, and is skipped by Tab.
//!
//! # Usage
//!
//! ```rust,no_run
//! use snora_design::Tokens;
//! use snora_widgets::design::button;
//!
//! #[derive(Clone)]
//! enum Message { Save }
//!
//! let tokens = Tokens::light();
//!
//! // Enabled
//! let save = button::primary(&tokens, "Save", Message::Save);
//!
//! // Disabled
//! let save = button::primary_maybe(&tokens, "Save", None::<Message>);
//! ```

use iced::{
    Element, Theme,
    widget::{button, text},
};
use snora_design::Tokens;

use crate::style::{BUTTON_DEFAULT_PADDING, design_control};

use snora_style as style;

// ---- helpers ----------------------------------------------------------------

fn make_button<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    on_press: Option<Message>,
    style_fn: fn(&Tokens, button::Status) -> button::Style,
) -> Element<'a, Message> {
    let t = tokens.clone();
    // RFC-116: `control` in place of `button`, so every variant is a focus
    // target, Enter and Space press it, and a disabled one (`None`) is
    // skipped by Tab. Padding is `button`'s own default.
    design_control(
        tokens,
        text(label.into()).size(style::text::label_size(tokens)),
        move |_theme: &Theme, status| style_fn(&t, status),
        tokens.palette.background,
    )
    .padding(BUTTON_DEFAULT_PADDING)
    .on_press_maybe(on_press)
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
