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
    Element, Padding, Theme,
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
/// **A ghost button is an action: it does something where the reader already
/// is** (Cancel, Skip, Save). If pressing it takes the reader *somewhere* — a
/// view, a document, a chooser — use [`link`], which is underlined so that
/// "this goes somewhere" is not carried by colour alone. It has no underline
/// and never will: underlining it would make every tertiary action read as a
/// link.
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

/// A link: the ghost button's colours, **underlined**, for something that
/// takes the reader somewhere.
///
/// # Link or action
///
/// > A **link** takes the reader somewhere: another view, a document, a
/// > chooser. An **action** does something where the reader already is:
/// > Cancel, Skip, Save. Use `link` for the first and [`ghost`] for the second.
///
/// "Choose another folder" opens a chooser, so it is a link.
///
/// # Why underlined
///
/// Older readers do not read coloured text as clickable, and colour is the
/// cue that fails first as eyesight changes (WCAG 1.4.1, Use of Color). The
/// underline is drawn **in the label's colour, in every state** (rest, hover,
/// pressed), and **under every line** if the label wraps (RFC-105: caller text
/// wraps and is never clipped).
///
/// **The rule.** An underline is 1/16 em thick, and its top edge is 1/16 em
/// below its line's box. Each of the two is rounded to the nearest whole pixel,
/// halves up, and is never under 1 px. Every wrapped line has its own underline,
/// and snapping to the pixel grid never moves an underline up toward its text.
/// At the default label size that is 1 px thick, 1 px below the box.
///
/// **A link's lines are spaced further apart than plain text's:** each line's
/// box is four of those units taller (1/16 em each, rounded, never under 1 px),
/// with the text centred in it, because an underline sits in the space the next
/// line's accents and tall ascenders occupy and would otherwise touch them. The
/// link is nevertheless as tall as a [`ghost`] button with the same label, and
/// its text sits where the ghost's does: the padding gives the extra height
/// back. **Two exceptions, both taller:**
///
/// - **a label of 40 px or more**, where half of the extra height exceeds the
///   button's default 5 px of padding, so there is none left to give back;
/// - **a preset whose focus ring is wider than its offset** (the two
///   high-contrast presets: 3 px wide, 2 px out), at a label of 24 px or more:
///   the ring reaches inside the control, so the bottom padding keeps the
///   underline clear of it, and the link is 1 to 2 px taller than a ghost. The
///   text still sits where the ghost's does.
///
/// The gap is measured from the line's box, which is all iced reports: a font
/// whose descenders reach past its own line box would come closer to the
/// underline than the default font's do.
///
/// # There is no disabled link
///
/// There is no `link_maybe`, deliberately: a dimmed, underlined phrase still
/// says "you can go here" and then does nothing, which is the misunderstanding
/// this control exists to remove. When the destination is unavailable, show
/// **plain text**, or nothing.
///
/// # Keyboard
///
/// A link is a `control` like every other: Tab reaches it (once the
/// application routes the key with `snora::keyboard::traverse`), Enter and
/// Space press it, and the focus ring shows where it is.
#[must_use]
pub fn link<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    on_press: Message,
) -> Element<'a, Message> {
    let t = tokens.clone();
    let size = style::text::label_size(tokens);
    // The label is `reach` taller than plain text (its lines are spaced
    // further apart, the text centred in each, the underline inside), so the
    // vertical padding gives half of it back on each side: the control is then
    // exactly as tall as a ghost button, and its text sits where the ghost's
    // does. Where the text is so large that `reach / 2` exceeds the default
    // padding, there is no padding left to give and the link is taller.
    let vertical =
        (BUTTON_DEFAULT_PADDING.top - snora_control::Underlined::reach(size.0) / 2.0).max(0.0);
    // The underline ends at the bottom of the label, and a focus ring wider
    // than its offset reaches that far *into* the control (the high-contrast
    // presets: 3 px wide, 2 px out). So the bottom padding keeps the underline
    // clear of the ring, plus a pixel for snapping the underline down. Only
    // the bottom: the text stays where a ghost button's is.
    let ring_inside = (tokens.focus.ring_width - tokens.focus.ring_offset).max(0.0);
    let bottom = vertical.max(ring_inside + 1.0);
    design_control(
        tokens,
        snora_control::Underlined::new(label.into()).size(size),
        move |_theme: &Theme, status| style::button::ghost(&t, status),
        tokens.palette.background,
    )
    .padding(Padding {
        top: vertical,
        bottom,
        ..BUTTON_DEFAULT_PADDING
    })
    .on_press(on_press)
    .into()
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
