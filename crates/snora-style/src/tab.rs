//! Tab style functions (RFC-131).
//!
//! * [`active`]: the label in `text_primary`, no fill, in every state;
//! * [`inactive`]: the label **muted** (a mix of `text_primary` and the page, held at the palette's
//!   [`text_contrast`](snora_design::Palette::text_contrast) on every neutral surface), and the
//!   `background` tier `weak` as a fill when hovered, with the label then in `text_primary`.
//!
//! These were the design path's use of `snora-widgets`' private `tab_button_style`, which takes an
//! `iced::Theme` and so cannot know a palette's level. They moved here as the chips' did (RFC-129
//! Q-5): pure functions of [`Tokens`] and a [`button::Status`], and the widget layer consumes them.
//! The unstyled path keeps its theme-based function and its 30% mix, because an `iced::Theme` has no
//! level to hold.
//!
//! # What changed, and for whom
//!
//! An inactive tab's label was `mix(text_primary, background, 0.3)`: a colour no palette declares.
//! Against the page it reads 6.58:1 on `light` and `light_aaa`, which clears `light`'s 4.5 and not
//! `light_aaa`'s 7; and on `surface`, which a tab bar can sit on, 6.10:1. It is now the first mix,
//! from 30% toward the page down to the text itself in steps of 2%, that holds the palette's level on
//! all three neutral surfaces (`state::muted`). **A mix that held the level does not
//! move**: five of the six presets draw exactly what they drew; `light_aaa` takes 26% (a little
//! nearer the text, 7.01:1 on `surface`, its lowest).
//!
//! **The underline carries the state.** The active tab is told apart by its underline, held at 3:1;
//! the muted label is a secondary cue, kept only as far as the level allows.
//!
//! ```rust,no_run
//! use snora_design::Tokens;
//! use snora_style::tab;
//!
//! let tokens = Tokens::light_aaa();
//! let tab_button = iced::widget::button("Settings")
//!     .style(move |_theme, status| tab::inactive(&tokens, status));
//! # let _: iced::widget::Button<'_, (), iced::Theme, iced::Renderer> = tab_button;
//! ```

use iced::{Background, Border, Shadow, widget::button};
use snora_design::Tokens;

use super::color::to_iced_color;
use super::state::{MUTED_TODAY, muted};
use super::theme::background_weak;

/// The active tab's style: its label in `text_primary` and no fill, in every state. Its underline is
/// its own element (RFC-102), not part of this style.
#[must_use]
pub fn active(tokens: &Tokens, _status: button::Status) -> button::Style {
    style(None, to_iced_color(tokens.palette.text_primary))
}

/// An inactive tab's style.
///
/// * **At rest and pressed:** no fill, and the label **muted**: a mix of `text_primary` and the
///   `background`, the 30% it has always been where that holds the palette's text level on every
///   neutral surface, otherwise the nearest mix toward the text that does (see the [module](self)).
/// * **Hovered:** the `background` tier `weak` as a fill, and the label in `text_primary`.
/// * **Disabled** is drawn as at rest (a tab is not disabled by snora).
#[must_use]
pub fn inactive(tokens: &Tokens, status: button::Status) -> button::Style {
    let text = to_iced_color(tokens.palette.text_primary);
    match status {
        button::Status::Hovered => style(Some(Background::Color(background_weak(tokens))), text),
        _ => style(
            None,
            muted(
                &tokens.palette,
                text,
                to_iced_color(tokens.palette.background),
                MUTED_TODAY,
            ),
        ),
    }
}

fn style(background: Option<Background>, text_color: iced::Color) -> button::Style {
    button::Style {
        background,
        text_color,
        border: Border::default(),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

#[cfg(test)]
mod tests;
