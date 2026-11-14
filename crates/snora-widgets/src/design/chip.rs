//! Chip primitives for Snora Design (RFC-032).
//!
//! Two variants:
//!
//! * `filter` — a toggle chip for filtering or categorizing. Solid accent
//!   background + `accent_text` foreground when selected; neutral border at
//!   rest.
//! * `removable` — a chip with a separate remove (×) button.
//!
//! Every control on a chip is **focusable** (RFC-116): they are `control`s,
//! not iced buttons, and iced 0.14's `button` registers no focus target.
//! Tab reaches the label and, on a removable chip, the remove button, once
//! the application routes the key with `snora::keyboard::traverse`; Enter
//! and Space press the focused one and publish the message a click
//! publishes; a ring shows which has focus. A control given `None` is
//! disabled and is skipped.
//! The application owns selection/filter state.
//!
//! # Contrast design
//!
//! A selected chip is a solid `accent` fill with `accent_text` on it. **Its label, and an
//! unselected chip's, are held at the palette's
//! [`text_contrast`](snora_design::Palette::text_contrast) in every state but disabled, on every
//! neutral surface.** The tests that hold it: `snora-style`'s `state/tests.rs` (the level and the
//! visible step from rest, exactly, for both chips and every preset) and `snora`'s
//! `tests/text_floor.rs` (the same, read from the pixels the widget draws).
//!
//! # Hovered and pressed (RFC-129)
//!
//! A chip's hovered and pressed fills are held to the palette's text level on every neutral
//! surface and stay visibly different from rest; see [`snora_style::chip`], where the chips'
//! style functions live (they moved there from this module, RFC-129 Q-5).
//!
//! # iced 0.14 focus limitation
//!
//! No custom focus ring — `button::Status` has no `Focused` variant.
//! Documented limitation, not a regression (RFC-025, RFC-027).
//!
//! # The remove glyph, and what a tooltip is not
//!
//! The remove button shows lucide `X` when the `lucide-icons` feature is
//! enabled, and the text glyph `"×"` otherwise (RFC-100). The notice's
//! dismiss button shares the same glyph.
//!
//! [`removable_with_tooltip`](crate::design::chip::removable_with_tooltip)
//! adds a short text shown when the pointer hovers the remove button, such
//! as `"Remove tag"`. **It is a visual tooltip only. It is not exposed to
//! assistive technology:** iced 0.14 has no accessible-name API for buttons,
//! and snora has no accessibility tree. Do not rely on it as the remove
//! control's accessible name.
//!
//! # Usage
//!
//! ```rust,no_run
//! use snora_design::Tokens;
//! use snora_widgets::design::chip;
//!
//! #[derive(Clone)]
//! enum Message { ToggleDrafts, ToggleTag, RemoveTag }
//!
//! let tokens = Tokens::light();
//! let show_drafts = true; // stands in for e.g. `self.show_drafts`
//!
//! let _draft_chip: iced::Element<'_, Message> =
//!     chip::filter(&tokens, "Draft", show_drafts, Message::ToggleDrafts);
//! let _tag_chip: iced::Element<'_, Message> = chip::removable(
//!     &tokens,
//!     "Tag: Rust",
//!     true,
//!     Message::ToggleTag,
//!     Message::RemoveTag,
//! );
//! ```

use iced::widget::text::Wrapping;
use iced::{
    Color, Element, Theme,
    widget::{container, row, text, tooltip},
};
use snora_design::Tokens;

use snora_style as style;

use crate::style::design_control;

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// The `remove` button's square pointer-target size (RFC-061): computed
/// the same way its height already resolves — `line_box + 2 × spacing.xs`
/// — so the width fix tracks any future token change automatically
/// rather than hard-coding a second "24".
fn remove_btn_target_size(tokens: &Tokens) -> f32 {
    tokens.typography.label.size * tokens.typography.label.line_height + 2.0 * tokens.spacing.xs
}

/// The close glyph shared by the removable chip's remove control and the
/// notice's dismiss control (RFC-100), sized to the label text.
///
/// One function, so the two controls cannot drift apart and the
/// `lucide-icons` `cfg` lives in one place. Lucide `X` under that feature;
/// text `"×"` otherwise, exactly as before.
///
/// `color` of `None` inherits the enclosing button's text colour. Under
/// `lucide-icons` the glyph comes from [`crate::icon::icon_element_sized`],
/// which returns an [`Element`] that cannot be recoloured afterwards, so the
/// colour is applied through a container's default text colour instead —
/// otherwise the notice's glyph would silently take its ghost button's
/// `accent` colour in place of `text_primary`.
pub(super) fn close_glyph<'a, Message: 'a>(
    tokens: &Tokens,
    color: Option<Color>,
) -> Element<'a, Message> {
    let size = style::text::label_size(tokens);

    #[cfg(feature = "lucide-icons")]
    {
        let glyph = crate::icon::icon_element_sized(
            &snora_core::Icon::Lucide(lucide_icons::Icon::X),
            size.0,
        );
        container(glyph)
            .style(move |_| container::Style {
                text_color: color,
                ..container::Style::default()
            })
            .into()
    }

    #[cfg(not(feature = "lucide-icons"))]
    {
        let glyph = text("×").size(size);
        match color {
            Some(color) => glyph.color(color).into(),
            None => glyph.into(),
        }
    }
}

/// Wraps a close control in a visual tooltip, or returns it unchanged when
/// there is none (RFC-100).
///
/// **Above the control** (`Position::Top`). Neither primitive knows the
/// layout direction, so `Left`/`Right` would be wrong under one direction or
/// the other. Of the two vertical positions, `Top` is iced's default, and it
/// keeps the text clear of the mouse pointer, whose arrow extends downward
/// from the hotspot and would cover part of a tooltip placed below.
///
/// The body is [`style::container::card_raised`], the popover style: its
/// `text_primary` on `surface_raised` is an existing, contrast-tested
/// palette pairing.
///
/// **Not an accessible name.** iced 0.14 has no accessible-name API for
/// buttons and snora has no accessibility tree; assistive technology does
/// not see this text.
pub(super) fn with_close_tooltip<'a, Message: 'a>(
    tokens: &Tokens,
    control: Element<'a, Message>,
    tip: Option<String>,
) -> Element<'a, Message> {
    let Some(tip) = tip else {
        return control;
    };
    let t = tokens.clone();
    tooltip(
        control,
        // Bounded and wrapped (RFC-105 R-1), the same treatment the
        // sidebar's tooltips get and through the same constant: a
        // content-sized tooltip has nothing to wrap against but the
        // viewport, so long caller text was cut off inside the overlay.
        container(
            text(tip)
                .size(style::text::label_size(tokens))
                .wrapping(Wrapping::WordOrGlyph),
        )
        .max_width(crate::style::TOOLTIP_MAX_WIDTH)
        .padding([tokens.spacing.xs, tokens.spacing.sm])
        .style(move |_| style::container::card_raised(&t)),
        tooltip::Position::Top,
    )
    .gap(tokens.spacing.xs)
    .into()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// A toggle chip for filtering or categorizing content.
///
/// Shows a solid accent background and `accent_text` foreground when
/// `selected`; the label is held at the palette's `text_contrast` in every
/// state but disabled (see the module's *Contrast design*). Emits
/// `on_toggle` when pressed. Pass `None` to disable.
#[must_use]
pub fn filter<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    selected: bool,
    on_toggle: impl Into<Option<Message>>,
) -> Element<'a, Message> {
    let t = tokens.clone();
    let style_fn = if selected {
        style::chip::selected
    } else {
        style::chip::unselected
    };
    design_control(
        tokens,
        text(label.into()).size(style::text::label_size(tokens)),
        move |_theme: &Theme, status| style_fn(&t, status),
        tokens.palette.background,
    )
    .on_press_maybe(on_toggle.into())
    .padding([tokens.spacing.xs, tokens.spacing.sm])
    .into()
}

/// A chip with a separate remove button.
///
/// The chip label toggles via `on_toggle`; the remove button emits
/// `on_remove`. Both controls are focusable, in that order, and Enter and
/// Space press the focused one (RFC-116). The remove button shows lucide
/// `X` under the `lucide-icons` feature and `"×"` otherwise.
///
/// **Where focus goes when the chip is removed (RFC-120):** if the × had focus
/// and the application removes the chip, focus moves to the **next chip's
/// label**, or else the **previous chip's ×**, within the row the chip sat in.
/// (The label goes with the chip, which is why the next chip's label and not its
/// ×: a second Enter toggles rather than removes.)
///
/// If your container is left with no control in it, snora cannot tell that it is still
/// there, and treats it as gone: focus moves to the nearest control in the container
/// around it.
///
/// To show a tooltip on the remove button, use [`removable_with_tooltip`].
#[must_use]
pub fn removable<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    selected: bool,
    on_toggle: impl Into<Option<Message>>,
    on_remove: impl Into<Option<Message>>,
) -> Element<'a, Message> {
    build_removable(tokens, label, selected, on_toggle, on_remove, None)
}

/// [`removable`], with a tooltip on the remove button.
///
/// `tooltip` is shown above the remove button while the pointer hovers it
/// — for example `"Remove tag"`.
///
/// **This is a visual tooltip, not an accessible name.** It is not exposed
/// to assistive technology: iced 0.14 has no accessible-name API for
/// buttons, and snora has no accessibility tree.
#[must_use]
pub fn removable_with_tooltip<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    selected: bool,
    on_toggle: impl Into<Option<Message>>,
    on_remove: impl Into<Option<Message>>,
    tooltip: impl Into<String>,
) -> Element<'a, Message> {
    build_removable(
        tokens,
        label,
        selected,
        on_toggle,
        on_remove,
        Some(tooltip.into()),
    )
}

/// The one implementation behind [`removable`] and
/// [`removable_with_tooltip`].
fn build_removable<'a, Message: Clone + 'a>(
    tokens: &Tokens,
    label: impl Into<String>,
    selected: bool,
    on_toggle: impl Into<Option<Message>>,
    on_remove: impl Into<Option<Message>>,
    tip: Option<String>,
) -> Element<'a, Message> {
    let t_label = tokens.clone();
    let t_remove = tokens.clone();
    let style_fn = if selected {
        style::chip::selected
    } else {
        style::chip::unselected
    };

    let label_btn: Element<'a, Message> = design_control(
        tokens,
        text(label.into()).size(style::text::label_size(tokens)),
        move |_theme: &Theme, status| style_fn(&t_label, status),
        tokens.palette.background,
    )
    .on_press_maybe(on_toggle.into())
    .padding([tokens.spacing.xs, tokens.spacing.sm])
    .into();

    // Pointer-target size (RFC-061): the glyph's own advance width is
    // not token-derivable (font/shaping-dependent — measured at 15.0px
    // total for the shipped fallback font at the current tokens, well
    // under the 24px WCAG 2.5.8 floor; lucide `X` is 1em wide, also short).
    // Padding alone cannot fix this
    // reliably: even bumping to `spacing.sm` only reaches 23.0px on that
    // same font, still short. Instead, the *content* box inside the
    // button is forced to a computed width and its text centered within
    // it — `iced::widget::button`'s own layout does not re-center a
    // child when the button is simply widened (see `layout::padded`),
    // so the centering has to happen one level in.
    let target_size = remove_btn_target_size(tokens);
    let content_width = target_size - 2.0 * tokens.spacing.xs;
    let remove_btn: Element<'a, Message> = design_control(
        tokens,
        container(close_glyph(tokens, None)).center_x(content_width),
        move |_theme: &Theme, status| style_fn(&t_remove, status),
        tokens.palette.background,
    )
    .on_press_maybe(on_remove.into())
    // Removing the chip takes its label control with it (RFC-120).
    .removes(1, 0)
    .padding([tokens.spacing.xs, tokens.spacing.xs])
    .into();
    let remove_btn = with_close_tooltip(tokens, remove_btn, tip);

    row![label_btn, remove_btn].spacing(0).into()
}

#[cfg(test)]
mod tests;
